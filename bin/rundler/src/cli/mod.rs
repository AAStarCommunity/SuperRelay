// This file is part of Rundler.
//
// Rundler is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later version.
//
// Rundler is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Rundler.
// If not, see https://www.gnu.org/licenses/.

use std::{sync::Arc, time::Duration};

use admin::AdminCliArgs;
use aggregator::AggregatorType;
use alloy_primitives::{Address, Bytes, TxHash, B256, U256};
use alloy_rpc_types_eth::{
    state::StateOverride, BlockId, BlockNumberOrTag, FeeHistory, Filter, Log,
};
use alloy_rpc_types_trace::geth::{
    GethDebugTracingCallOptions, GethDebugTracingOptions, GethTrace,
};
use anyhow::{bail, Context};
use clap::{
    builder::{PossibleValuesParser, ValueParser},
    Args, Parser, Subcommand,
};

mod admin;
mod aggregator;
mod builder;
mod chain_spec;
mod json;
mod metrics;
mod node;
mod pool;
mod proxy;
mod rpc;
mod signer;
mod tracing;

use builder::{BuilderCliArgs, EntryPointBuilderConfigs};
use json::get_json_config;
use node::NodeCliArgs;
use pool::PoolCliArgs;
use reth_tasks::TaskManager;
use rpc::RpcCliArgs;
use rundler_provider::{
    new_fee_estimator, AggregatorOut, AlloyEntryPointV0_6, AlloyEntryPointV0_7, AlloyEvmProvider,
    Block, BlockHashOrNumber, BundleHandler, DAGasOracle, DAGasOracleSync, DAGasProvider,
    DepositInfo, EntryPoint, EntryPointProvider, EvmCall, EvmProvider, ExecutionResult,
    FeeEstimator, GasUsedResult, HandleOpsOut, ProviderResult, Providers, RpcRecv, RpcSend,
    SignatureAggregator, SimulationProvider, Transaction, TransactionReceipt, TransactionRequest,
};
use rundler_sim::{
    EstimationSettings, MempoolConfigs, PrecheckSettings, SimulationSettings, MIN_CALL_GAS_LIMIT,
};
use rundler_types::{
    chain::{ChainSpec, TryFromWithSpec},
    da::{DAGasBlockData, DAGasData, DAGasOracleType},
    v0_6::UserOperation as UserOperationV0_6,
    v0_7::UserOperation as UserOperationV0_7,
    EntryPointVersion, ExpectedStorage, GasFees, PriorityFeeMode, UserOpsPerAggregator,
    ValidationOutput, ValidationRevert,
};
use secrecy::SecretString;

/// Main entry point for the CLI
///
/// Parses the CLI arguments and runs the appropriate subcommand.
/// Listens for a ctrl-c signal and shuts down all components when received.
pub async fn run() -> anyhow::Result<()> {
    let opt = Cli::parse();
    let _guard = tracing::configure_logging(&opt.common.network, &opt.logs)?;
    tracing::info!("Parsed CLI options: {:#?}", opt);
    tracing::info!("Initializing task manager...");
    let mut task_manager = TaskManager::current();
    let task_spawner = task_manager.executor();
    tracing::info!("Task manager initialized.");

    tracing::info!("Initializing metrics...");
    let metrics_addr = format!("{}:{}", opt.metrics.host, opt.metrics.port).parse()?;
    metrics::initialize(
        &task_spawner,
        opt.metrics.sample_interval_millis,
        metrics_addr,
        &opt.metrics.tags,
        &opt.metrics.buckets,
    )
    .context("metrics server should start")?;
    tracing::info!("Metrics initialized.");

    tracing::info!("Resolving chain spec...");
    let mut cs = chain_spec::resolve_chain_spec(&opt.common.network, &opt.common.chain_spec);
    tracing::info!("Chain spec resolved.");

    tracing::info!("Loading configs...");
    let (mempool_configs, entry_point_builders) = load_configs(&opt.common).await?;
    if let Some(entry_point_builders) = &entry_point_builders {
        entry_point_builders.set_proxies(&mut cs);
    }
    tracing::info!("Configs loaded.");

    tracing::info!("Constructing providers...");
    let providers = construct_providers(&opt.common, &cs)?;
    tracing::info!("Providers constructed.");

    tracing::info!("Starting task spawner...");
    let _handles: Vec<tokio::task::JoinHandle<()>> = match &opt.command {
        Command::Rpc(args) => {
            rpc::spawn_tasks(
                task_spawner.clone(),
                cs,
                args.clone(),
                opt.common.clone(),
                providers.clone(),
            )
            .await?;
            vec![]
        }
        Command::Builder(args) => {
            builder::spawn_tasks(
                task_spawner.clone(),
                cs,
                args.clone(),
                opt.common.clone(),
                providers.clone(),
            )
            .await?;
            vec![]
        }
        Command::Admin(args) => {
            admin::run(args.clone(), cs, providers.clone(), task_spawner.clone()).await?;
            return Ok(());
        }
        Command::Pool(args) => {
            pool::spawn_tasks(
                task_spawner.clone(),
                cs,
                args.clone(),
                opt.common.clone(),
                providers.clone(),
                mempool_configs,
            )
            .await?;
            vec![]
        }
        Command::Node(args) => {
            node::spawn_tasks(
                task_spawner.clone(),
                cs,
                *args.clone(),
                opt.common.clone(),
                providers.clone(),
                mempool_configs,
                entry_point_builders,
            )
            .await?;
            vec![]
        }
    };

    // wait for ctrl-c or the task manager to panic
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received ctrl-c, shutting down");
        },
        e = &mut task_manager => {
            tracing::error!("Task manager panicked, shutting down: {e:?}");
        },
    }

    // wait for the task manager to shutdown
    task_manager.graceful_shutdown_with_timeout(Duration::from_secs(10));

    tracing::info!("Shutdown, goodbye");
    Ok(())
}

/// CLI commands
#[derive(Debug, Subcommand)]
enum Command {
    /// Bundler command
    ///
    /// Runs the Pool, Builder, and RPC servers in a single process.
    #[command(name = "node")]
    Node(Box<NodeCliArgs>),

    /// Rpc command
    ///
    /// Runs the Rpc server
    #[command(name = "rpc")]
    Rpc(RpcCliArgs),

    /// Pool command
    ///
    /// Runs the Pool server
    #[command(name = "pool")]
    Pool(PoolCliArgs),

    /// Builder command
    ///
    /// Runs the Builder server
    #[command(name = "builder")]
    Builder(BuilderCliArgs),

    /// Admin command
    ///
    /// Runs the admin commands
    #[command(name = "admin")]
    Admin(AdminCliArgs),
}

/// CLI common options
#[derive(Debug, Args, Clone)]
#[command(next_help_heading = "Common")]
pub struct CommonArgs {
    /// Network flag
    #[arg(
        long = "network",
        name = "network",
        env = "NETWORK",
        value_parser = PossibleValuesParser::new(chain_spec::HARDCODED_CHAIN_SPECS),
        global = true)
    ]
    network: Option<String>,

    /// Chain spec file path
    #[arg(
        long = "chain_spec",
        name = "chain_spec",
        env = "CHAIN_SPEC",
        global = true
    )]
    chain_spec: Option<String>,

    /// ETH Node HTTP URL to connect to
    #[arg(
        long = "node_http",
        name = "node_http",
        env = "NODE_HTTP",
        global = true
    )]
    node_http: Option<String>,

    /// Flag for turning unsafe bundling mode on
    #[arg(long = "unsafe", env = "UNSAFE", global = true)]
    unsafe_mode: bool,

    #[arg(
        long = "max_verification_gas",
        name = "max_verification_gas",
        default_value = "5000000",
        env = "MAX_VERIFICATION_GAS",
        global = true
    )]
    max_verification_gas: u64,

    #[arg(
        long = "max_simulate_handle_op_gas",
        name = "max_simulate_handle_op_gas",
        default_value = "5000000",
        env = "MAX_SIMULATE_HANDLE_OP_GAS",
        global = true
    )]
    max_simulate_handle_op_gas: u64,

    #[arg(
        long = "max_aggregation_gas",
        name = "max_aggregation_gas",
        default_value = "1000000",
        env = "MAX_AGGREGATION_GAS",
        global = true
    )]
    max_aggregation_gas: u64,

    #[arg(
        long = "max_uo_cost",
        name = "max_uo_cost",
        env = "MAX_UO_COST",
        value_parser = alloy_primitives::utils::parse_ether,
        global = true
    )]
    max_uo_cost: Option<U256>,

    #[arg(
        long = "target_bundle_block_gas_limit_ratio",
        name = "target_bundle_block_gas_limit_ratio",
        default_value = "0.5",
        env = "TARGET_BUNDLE_BLOCK_GAS_LIMIT_RATIO",
        value_parser = verify_f64_less_than_one,
        global = true
    )]
    target_bundle_block_gas_limit_ratio: f64,

    #[arg(
        long = "max_bundle_block_gas_limit_ratio",
        name = "max_bundle_block_gas_limit_ratio",
        default_value = "0.9",
        env = "MAX_BUNDLE_BLOCK_GAS_LIMIT_RATIO",
        value_parser = verify_f64_less_than_one,
        global = true
    )]
    max_bundle_block_gas_limit_ratio: f64,

    #[arg(
        long = "min_stake_value",
        name = "min_stake_value",
        env = "MIN_STAKE_VALUE",
        default_value = "1000000000000000000",
        global = true
    )]
    min_stake_value: u128,

    #[arg(
        long = "min_unstake_delay",
        name = "min_unstake_delay",
        env = "MIN_UNSTAKE_DELAY",
        default_value = "84600",
        global = true
    )]
    min_unstake_delay: u32,

    /// String representation of the timeout of a custom tracer in a format that is parsable by the
    /// `ParseDuration` function on the ethereum node. See Docs: https://pkg.go.dev/time#ParseDuration
    #[arg(
        long = "tracer_timeout",
        name = "tracer_timeout",
        env = "TRACER_TIMEOUT",
        default_value = "10s",
        global = true
    )]
    tracer_timeout: String,

    /// If set, allows the simulator to fallback to unsafe mode if the simulation tracer fails
    #[arg(
        long = "enable_unsafe_fallback",
        name = "enable_unsafe_fallback",
        env = "ENABLE_UNSAFE_FALLBACK"
    )]
    enable_unsafe_fallback: bool,

    /// Amount of blocks to search when calling eth_getUserOperationByHash.
    /// Defaults from 0 to latest block
    #[arg(
        long = "user_operation_event_block_distance",
        name = "user_operation_event_block_distance",
        env = "USER_OPERATION_EVENT_BLOCK_DISTANCE",
        global = true
    )]
    user_operation_event_block_distance: Option<u64>,

    /// Amount of blocks to search when calling eth_getUserOperationByHash during a fallback.
    ///
    /// Defaults to unset. If set, will be used in the case that a first query for events fails.
    #[arg(
        long = "user_operation_event_block_distance_fallback",
        name = "user_operation_event_block_distance_fallback",
        env = "USER_OPERATION_EVENT_BLOCK_DISTANCE_FALLBACK",
        global = true
    )]
    user_operation_event_block_distance_fallback: Option<u64>,

    #[arg(
        long = "verification_estimation_gas_fee",
        name = "verification_estimation_gas_fee",
        env = "VERIFICATION_ESTIMATION_GAS_FEE",
        default_value = "1000000000000", // 10K gwei
        global = true
    )]
    verification_estimation_gas_fee: u128,

    #[arg(
        long = "bundle_base_fee_overhead_percent",
        name = "bundle_base_fee_overhead_percent",
        env = "BUNDLE_BASE_FEE_OVERHEAD_PERCENT",
        default_value = "27", // 2 12.5% EIP-1559 increases
        global = true
    )]
    bundle_base_fee_overhead_percent: u32,

    #[arg(
        long = "bundle_priority_fee_overhead_percent",
        name = "bundle_priority_fee_overhead_percent",
        env = "BUNDLE_PRIORITY_FEE_OVERHEAD_PERCENT",
        default_value = "0",
        global = true
    )]
    bundle_priority_fee_overhead_percent: u32,

    #[arg(
        long = "priority_fee_mode_kind",
        name = "priority_fee_mode_kind",
        env = "PRIORITY_FEE_MODE_KIND",
        value_parser = PossibleValuesParser::new(["base_fee_percent", "priority_fee_increase_percent"]),
        default_value = "priority_fee_increase_percent",
        global = true
    )]
    priority_fee_mode_kind: String,

    #[arg(
        long = "priority_fee_mode_value",
        name = "priority_fee_mode_value",
        env = "PRIORITY_FEE_MODE_VALUE",
        default_value = "0",
        global = true
    )]
    priority_fee_mode_value: u32,

    #[arg(
        long = "base_fee_accept_percent",
        name = "base_fee_accept_percent",
        env = "BASE_FEE_ACCEPT_PERCENT",
        default_value = "50",
        global = true
    )]
    base_fee_accept_percent: u32,

    #[arg(
        long = "pre_verification_gas_accept_percent",
        name = "pre_verification_gas_accept_percent",
        env = "PRE_VERIFICATION_GAS_ACCEPT_PERCENT",
        default_value = "50",
        global = true
    )]
    pre_verification_gas_accept_percent: u32,

    #[arg(
        long = "execution_gas_limit_efficiency_reject_threshold",
        name = "execution_gas_limit_efficiency_reject_threshold",
        env = "EXECUTION_GAS_LIMIT_EFFICIENCY_REJECT_THRESHOLD",
        default_value = "0.0"
    )]
    pub execution_gas_limit_efficiency_reject_threshold: f64,

    #[arg(
        long = "verification_gas_limit_efficiency_reject_threshold",
        name = "verification_gas_limit_efficiency_reject_threshold",
        env = "VERIFICATION_GAS_LIMIT_EFFICIENCY_REJECT_THRESHOLD",
        default_value = "0.0"
    )]
    pub verification_gas_limit_efficiency_reject_threshold: f64,

    #[arg(
        long = "verification_gas_allowed_error_pct",
        name = "verification_gas_allowed_error_pct",
        env = "VERIFICATION_GAS_ALLOWED_ERROR_PCT",
        default_value = "15",
        global = true
    )]
    pub verification_gas_allowed_error_pct: u128,

    #[arg(
        long = "call_gas_allowed_error_pct",
        name = "call_gas_allowed_error_pct",
        env = "CALL_GAS_ALLOWED_ERROR_PCT",
        default_value = "15",
        global = true
    )]
    pub call_gas_allowed_error_pct: u128,

    #[arg(
        long = "max_gas_estimation_gas",
        name = "max_gas_estimation_gas",
        env = "MAX_GAS_ESTIMATION_GAS",
        default_value = "550000000",
        global = true
    )]
    pub max_gas_estimation_gas: u64,

    #[arg(
        long = "max_gas_estimation_rounds",
        name = "max_gas_estimation_rounds",
        env = "MAX_GAS_ESTIMATION_ROUNDS",
        default_value = "3",
        global = true
    )]
    pub max_gas_estimation_rounds: u32,

    #[arg(
        long = "mempool_config_path",
        name = "mempool_config_path",
        env = "MEMPOOL_CONFIG_PATH",
        global = true
    )]
    pub mempool_config_path: Option<String>,

    #[arg(
        long = "builders_config_path",
        name = "builders_config_path",
        env = "BUILDERS_CONFIG_PATH",
        global = true
    )]
    builders_config_path: Option<String>,

    #[arg(
        long = "disable_entry_point_v0_6",
        name = "disable_entry_point_v0_6",
        env = "DISABLE_ENTRY_POINT_V0_6",
        default_value = "false",
        global = true
    )]
    pub disable_entry_point_v0_6: bool,

    #[arg(
        long = "disable_entry_point_v0_7",
        name = "disable_entry_point_v0_7",
        env = "DISABLE_ENTRY_POINT_V0_7",
        default_value = "false",
        global = true
    )]
    pub disable_entry_point_v0_7: bool,

    // Ignored if disable_entry_point_v0_6 is true
    // Ignored if entry_point_builders_path is set
    #[arg(
        long = "num_builders_v0_6",
        name = "num_builders_v0_6",
        env = "NUM_BUILDERS_V0_6",
        default_value = "1",
        global = true
    )]
    pub num_builders_v0_6: u64,

    // Ignored if disable_entry_point_v0_7 is true
    // Ignored if entry_point_builders_path is set
    #[arg(
        long = "num_builders_v0_7",
        name = "num_builders_v0_7",
        env = "NUM_BUILDERS_V0_7",
        default_value = "1",
        global = true
    )]
    pub num_builders_v0_7: u64,

    #[arg(
        long = "da_gas_tracking_enabled",
        name = "da_gas_tracking_enabled",
        env = "DA_GAS_TRACKING_ENABLED",
        default_value = "false",
        global = true
    )]
    pub da_gas_tracking_enabled: bool,

    #[arg(
        long = "provider_client_timeout_seconds",
        name = "provider_client_timeout_seconds",
        env = "PROVIDER_CLIENT_TIMEOUT_SECONDS",
        default_value = "10",
        global = true
    )]
    pub provider_client_timeout_seconds: u64,

    #[arg(
        long = "max_expected_storage_slots",
        name = "max_expected_storage_slots",
        env = "MAX_EXPECTED_STORAGE_SLOTS",
        global = true
    )]
    pub max_expected_storage_slots: Option<usize>,

    #[arg(
        long = "enabled_aggregators",
        name = "enabled_aggregators",
        env = "ENABLED_AGGREGATORS",
        global = true,
        value_delimiter = ','
    )]
    pub enabled_aggregators: Vec<AggregatorType>,

    #[arg(
        long = "aggregator_options",
        env = "AGGREGATOR_OPTIONS",
        global = true,
        value_delimiter = ',',
        value_parser = ValueParser::new(parse_key_val)
    )]
    pub aggregator_options: Vec<(String, String)>,

    /// Enable paymaster relay service
    #[arg(
        long = "paymaster.enabled",
        name = "paymaster.enabled",
        env = "PAYMASTER_ENABLED",
        global = true
    )]
    pub paymaster_enabled: bool,

    /// Paymaster private key for signing
    #[arg(
        long = "paymaster.private_key",
        name = "paymaster.private_key", 
        env = "PAYMASTER_PRIVATE_KEY",
        value_parser = parse_secret,
        global = true
    )]
    pub paymaster_private_key: Option<SecretString>,

    /// Path to paymaster policy configuration file
    #[arg(
        long = "paymaster.policy_file",
        name = "paymaster.policy_file",
        env = "PAYMASTER_POLICY_FILE",
        global = true
    )]
    pub paymaster_policy_file: Option<String>,
}

/// Converts a &str into a SecretString
pub(crate) fn parse_secret(s: &str) -> Result<SecretString, String> {
    Ok(s.into())
}

fn parse_key_val(s: &str) -> Result<(String, String), anyhow::Error> {
    let pos = s
        .find('=')
        .ok_or_else(|| anyhow::anyhow!(format!("invalid KEY=value: no `=` found in `{}`", s)))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

fn verify_f64_less_than_one(v: &str) -> Result<f64, String> {
    let Ok(v) = v.parse() else {
        return Err("invalid float".to_string());
    };
    if v < 1.0 {
        Ok(v)
    } else {
        Err(format!("value {v} is not less than 1.0"))
    }
}

const SIMULATION_GAS_OVERHEAD: u128 = 100_000;

impl TryFromWithSpec<&CommonArgs> for EstimationSettings {
    type Error = anyhow::Error;

    fn try_from_with_spec(value: &CommonArgs, chain_spec: &ChainSpec) -> Result<Self, Self::Error> {
        let max_bundle_execution_gas =
            chain_spec.block_gas_limit_mult(value.max_bundle_block_gas_limit_ratio);

        if value.max_verification_gas
            > max_bundle_execution_gas.saturating_sub(SIMULATION_GAS_OVERHEAD) as u64
        {
            anyhow::bail!(
                "max_verification_gas ({}) must be less than max_bundle_execution_gas ({}) by at least {}",
                value.max_verification_gas,
                max_bundle_execution_gas,
                SIMULATION_GAS_OVERHEAD
            );
        }

        if max_bundle_execution_gas < MIN_CALL_GAS_LIMIT {
            anyhow::bail!(
                "max_bundle_execution_gas ({}) must be greater than or equal to {}",
                max_bundle_execution_gas,
                MIN_CALL_GAS_LIMIT
            );
        }

        Ok(Self {
            max_verification_gas: value.max_verification_gas as u128,
            max_paymaster_verification_gas: value.max_verification_gas as u128,
            max_paymaster_post_op_gas: max_bundle_execution_gas,
            max_bundle_execution_gas,
            max_gas_estimation_gas: value.max_gas_estimation_gas,
            verification_estimation_gas_fee: value.verification_estimation_gas_fee,
            verification_gas_limit_efficiency_reject_threshold: value
                .verification_gas_limit_efficiency_reject_threshold,
            verification_gas_allowed_error_pct: value.verification_gas_allowed_error_pct,
            call_gas_allowed_error_pct: value.call_gas_allowed_error_pct,
            max_gas_estimation_rounds: value.max_gas_estimation_rounds,
        })
    }
}

impl TryFromWithSpec<&CommonArgs> for PrecheckSettings {
    type Error = anyhow::Error;

    fn try_from_with_spec(value: &CommonArgs, chain_spec: &ChainSpec) -> Result<Self, Self::Error> {
        Ok(Self {
            max_verification_gas: value.max_verification_gas as u128,
            max_bundle_execution_gas: chain_spec
                .block_gas_limit_mult(value.max_bundle_block_gas_limit_ratio),
            max_uo_cost: value.max_uo_cost.unwrap_or(U256::MAX),
            bundle_priority_fee_overhead_percent: value.bundle_priority_fee_overhead_percent,
            priority_fee_mode: PriorityFeeMode::try_from(
                value.priority_fee_mode_kind.as_str(),
                value.priority_fee_mode_value,
            )?,
            base_fee_accept_percent: value.base_fee_accept_percent,
            pre_verification_gas_accept_percent: value.pre_verification_gas_accept_percent,
            verification_gas_limit_efficiency_reject_threshold: value
                .verification_gas_limit_efficiency_reject_threshold,
        })
    }
}

impl TryFrom<&CommonArgs> for SimulationSettings {
    type Error = anyhow::Error;

    fn try_from(value: &CommonArgs) -> Result<Self, Self::Error> {
        if go_parse_duration::parse_duration(&value.tracer_timeout).is_err() {
            bail!("Invalid value for tracer_timeout, must be parsable by the ParseDuration function. See docs https://pkg.go.dev/time#ParseDuration")
        }

        Ok(Self {
            min_unstake_delay: value.min_unstake_delay,
            min_stake_value: U256::from(value.min_stake_value),
            tracer_timeout: value.tracer_timeout.clone(),
            enable_unsafe_fallback: value.enable_unsafe_fallback,
        })
    }
}

/// CLI options for the metrics server
#[derive(Debug, Args)]
#[command(next_help_heading = "Metrics")]
pub struct MetricsArgs {
    /// Port to listen on for metrics requests
    #[arg(
        long = "metrics.port",
        name = "metrics.port",
        env = "METRICS_PORT",
        default_value = "8080",
        global = true
    )]
    port: u16,

    /// Host to listen on for metrics requests
    #[arg(
        long = "metrics.host",
        name = "metrics.host",
        env = "METRICS_HOST",
        default_value = "0.0.0.0",
        global = true
    )]
    host: String,

    /// Tags for metrics
    ///
    /// Format: key1=value1,key2=value2,...
    #[arg(
        long = "metrics.tags",
        name = "metrics.tags",
        env = "METRICS_TAGS",
        default_values_t = Vec::<String>::new(),
        value_delimiter = ',',
        global = true
    )]
    tags: Vec<String>,

    /// Sample interval for sampling metrics
    #[arg(
        long = "metrics.sample_interval_millis",
        name = "metrics.sample_interval_millis",
        env = "METRICS_SAMPLE_INTERVAL_MILLIS",
        default_value = "1000",
        global = true
    )]
    sample_interval_millis: u64,

    /// Bucket for histogram metric.
    #[arg(
        long = "metrics.histogram_buckets", 
        name = "metrics.histogram_buckets",
        env = "METRICS_HISTOGRAM_BUCKETS",
        default_values_t = vec![
            // Exponential buckets from 0 to 256
            0.0, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0,
            // Fine-grained buckets from 500 to 1000
            500.0, 600.0, 700.0, 800.0, 900.0, 1000.0,
            // Coarser buckets from 1000 to 10,000
            2000.0, 3000.0, 4000.0, 5000.0, 7500.0, 10000.0,
        ],
        global = true
    )]
    buckets: Vec<f64>,
}

/// CLI options for logging
#[derive(Debug, Args)]
#[command(next_help_heading = "Logging")]
pub struct LogsArgs {
    /// Log file
    ///
    /// If not provided, logs will be written to stdout
    #[arg(
        long = "log.file",
        name = "log.file",
        env = "LOG_FILE",
        default_value = None,
        global = true
    )]
    file: Option<String>,

    /// Log JSON
    ///
    /// If set, logs will be written in JSON format
    #[arg(
        long = "log.json",
        name = "log.json",
        env = "LOG_JSON",
        required = false,
        num_args = 0,
        global = true
    )]
    json: bool,

    /// Log OTLP Endpoint
    ///
    /// If set, tracing spans will be forwarded to the provided gRPC OTLP endpoint
    #[arg(
        long = "log.otlp_grpc_endpoint",
        name = "log.otlp_grpc_endpoint",
        env = "LOG_OTLP_GRPC_ENDPOINT",
        default_value = None,
        global = true
    )]
    otlp_grpc_endpoint: Option<String>,
}

/// CLI options
#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,

    #[clap(flatten)]
    common: CommonArgs,

    #[clap(flatten)]
    metrics: MetricsArgs,

    #[clap(flatten)]
    logs: LogsArgs,
}

#[derive(Clone)]
pub struct RundlerProviders<P, EP06, EP07, D, DS, F> {
    provider: P,
    ep_v0_6: Option<EP06>,
    ep_v0_7: Option<EP07>,
    da_gas_oracle: D,
    da_gas_oracle_sync: Option<DS>,
    fee_estimator: F,
}

impl<P, EP06, EP07, D, DS, F> Providers for RundlerProviders<P, EP06, EP07, D, DS, F>
where
    P: EvmProvider + Clone,
    EP06: EntryPointProvider<UserOperationV0_6> + Clone,
    EP07: EntryPointProvider<UserOperationV0_7> + Clone,
    D: DAGasOracle + Clone,
    DS: DAGasOracleSync + Clone,
    F: FeeEstimator + Clone,
{
    type Evm = P;
    type EntryPointV0_6 = EP06;
    type EntryPointV0_7 = EP07;
    type DAGasOracle = D;
    type DAGasOracleSync = DS;
    type FeeEstimator = F;

    fn evm(&self) -> &Self::Evm {
        &self.provider
    }

    fn ep_v0_6(&self) -> &Option<Self::EntryPointV0_6> {
        &self.ep_v0_6
    }

    fn ep_v0_7(&self) -> &Option<Self::EntryPointV0_7> {
        &self.ep_v0_7
    }

    fn da_gas_oracle(&self) -> &Self::DAGasOracle {
        &self.da_gas_oracle
    }

    fn da_gas_oracle_sync(&self) -> &Option<Self::DAGasOracleSync> {
        &self.da_gas_oracle_sync
    }

    fn fee_estimator(&self) -> &Self::FeeEstimator {
        &self.fee_estimator
    }
}

#[derive(Clone)]
pub struct DummyEvmProvider;

#[async_trait::async_trait]
impl EvmProvider for DummyEvmProvider {
    async fn request<P, R>(&self, _method: &'static str, _params: P) -> ProviderResult<R>
    where
        P: RpcSend + 'static,
        R: RpcRecv,
    {
        todo!()
    }
    async fn fee_history(
        &self,
        _block_count: u64,
        _block_number: BlockNumberOrTag,
        _reward_percentiles: &[f64],
    ) -> ProviderResult<FeeHistory> {
        todo!()
    }
    async fn call(
        &self,
        _tx: TransactionRequest,
        _block: Option<BlockId>,
        _state_overrides: Option<StateOverride>,
    ) -> ProviderResult<Bytes> {
        todo!()
    }
    async fn send_raw_transaction(&self, _tx: Bytes) -> ProviderResult<TxHash> {
        todo!()
    }
    async fn send_raw_transaction_conditional(
        &self,
        _tx: Bytes,
        _expected_storage: &ExpectedStorage,
    ) -> ProviderResult<TxHash> {
        todo!()
    }
    async fn get_block_number(&self) -> ProviderResult<u64> {
        todo!()
    }
    async fn get_block(&self, _block_id: BlockId) -> ProviderResult<Option<Block>> {
        todo!()
    }
    async fn get_full_block(&self, _block_id: BlockId) -> ProviderResult<Option<Block>> {
        todo!()
    }
    async fn get_balance(
        &self,
        _address: Address,
        _block: Option<BlockId>,
    ) -> ProviderResult<U256> {
        todo!()
    }
    async fn get_transaction_by_hash(&self, _tx: TxHash) -> ProviderResult<Option<Transaction>> {
        todo!()
    }
    async fn get_transaction_receipt(
        &self,
        _tx: TxHash,
    ) -> ProviderResult<Option<TransactionReceipt>> {
        todo!()
    }
    async fn debug_trace_transaction(
        &self,
        _tx_hash: TxHash,
        _trace_options: GethDebugTracingOptions,
    ) -> ProviderResult<GethTrace> {
        todo!()
    }
    async fn debug_trace_call(
        &self,
        _tx: TransactionRequest,
        _block_id: Option<BlockId>,
        _trace_options: GethDebugTracingCallOptions,
    ) -> ProviderResult<GethTrace> {
        todo!()
    }
    async fn get_latest_block_hash_and_number(&self) -> ProviderResult<(B256, u64)> {
        todo!()
    }
    async fn get_pending_base_fee(&self) -> ProviderResult<u128> {
        todo!()
    }
    async fn get_max_priority_fee(&self) -> ProviderResult<u128> {
        todo!()
    }
    async fn get_code(&self, _address: Address, _block: Option<BlockId>) -> ProviderResult<Bytes> {
        todo!()
    }
    async fn get_transaction_count(&self, _address: Address) -> ProviderResult<u64> {
        todo!()
    }
    async fn get_logs(&self, _filter: &Filter) -> ProviderResult<Vec<Log>> {
        todo!()
    }
    async fn get_gas_used(&self, _call: EvmCall) -> ProviderResult<GasUsedResult> {
        todo!()
    }
    async fn batch_get_storage_at(
        &self,
        _address: Address,
        _slots: Vec<B256>,
    ) -> ProviderResult<Vec<B256>> {
        todo!()
    }
    async fn get_code_hash(
        &self,
        _addresses: Vec<Address>,
        _block: Option<BlockId>,
    ) -> ProviderResult<B256> {
        todo!()
    }
    async fn get_balances(&self, _addresses: Vec<Address>) -> ProviderResult<Vec<(Address, U256)>> {
        todo!()
    }
}

#[derive(Clone)]
pub struct DummyEntryPointProviderV06;

#[async_trait::async_trait]
impl SignatureAggregator for DummyEntryPointProviderV06 {
    type UO = UserOperationV0_6;
    async fn aggregate_signatures(
        &self,
        _aggregator_address: Address,
        _ops: Vec<UserOperationV0_6>,
    ) -> ProviderResult<Option<Bytes>> {
        todo!()
    }
    async fn validate_user_op_signature(
        &self,
        _aggregator_address: Address,
        _user_op: UserOperationV0_6,
    ) -> ProviderResult<AggregatorOut> {
        todo!()
    }
}

#[async_trait::async_trait]
impl BundleHandler for DummyEntryPointProviderV06 {
    type UO = UserOperationV0_6;
    async fn call_handle_ops(
        &self,
        _ops_per_aggregator: Vec<UserOpsPerAggregator<UserOperationV0_6>>,
        _sender_eoa: Address,
        _gas_limit: u64,
        _gas_fees: GasFees,
        _proxy: Option<Address>,
        _validation_only: bool,
    ) -> ProviderResult<HandleOpsOut> {
        todo!()
    }
    fn get_send_bundle_transaction(
        &self,
        _ops_per_aggregator: Vec<UserOpsPerAggregator<UserOperationV0_6>>,
        _sender_eoa: Address,
        _gas_limit: u64,
        _gas_fees: GasFees,
        _proxy: Option<Address>,
    ) -> TransactionRequest {
        todo!()
    }
    fn decode_handle_ops_revert(
        _message: &str,
        _revert_data: &Option<Bytes>,
    ) -> Option<HandleOpsOut> {
        todo!()
    }
    fn decode_ops_from_calldata(
        _chain_spec: &ChainSpec,
        _calldata: &Bytes,
    ) -> Vec<UserOpsPerAggregator<UserOperationV0_6>> {
        todo!()
    }
}

#[async_trait::async_trait]
impl SimulationProvider for DummyEntryPointProviderV06 {
    type UO = UserOperationV0_6;
    fn get_tracer_simulate_validation_call(
        &self,
        _user_op: UserOperationV0_6,
    ) -> ProviderResult<(TransactionRequest, StateOverride)> {
        todo!()
    }
    async fn simulate_validation(
        &self,
        _user_op: UserOperationV0_6,
        _block_id: Option<BlockId>,
    ) -> ProviderResult<Result<ValidationOutput, ValidationRevert>> {
        todo!()
    }
    async fn simulate_handle_op(
        &self,
        _op: UserOperationV0_6,
        _target: Address,
        _target_call_data: Bytes,
        _block_id: BlockId,
        _state_override: StateOverride,
    ) -> ProviderResult<Result<ExecutionResult, ValidationRevert>> {
        todo!()
    }
    async fn simulate_handle_op_estimate_gas(
        &self,
        _op: UserOperationV0_6,
        _target: Address,
        _target_call_data: Bytes,
        _block_id: BlockId,
        _state_override: StateOverride,
    ) -> ProviderResult<Result<ExecutionResult, ValidationRevert>> {
        todo!()
    }
    fn decode_simulate_handle_ops_revert(
        _revert_data: &Bytes,
    ) -> ProviderResult<Result<ExecutionResult, ValidationRevert>> {
        todo!()
    }
    fn simulation_should_revert(&self) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl DAGasProvider for DummyEntryPointProviderV06 {
    type UO = UserOperationV0_6;
    async fn calc_da_gas(
        &self,
        _uo: UserOperationV0_6,
        _block: BlockHashOrNumber,
        _gas_price: u128,
        _bundle_size: usize,
    ) -> ProviderResult<(u128, DAGasData, DAGasBlockData)> {
        todo!()
    }
}

#[async_trait::async_trait]
impl EntryPoint for DummyEntryPointProviderV06 {
    fn version(&self) -> EntryPointVersion {
        todo!()
    }
    fn address(&self) -> &Address {
        static ZERO: Address = Address::ZERO;
        &ZERO
    }
    async fn balance_of(
        &self,
        _address: Address,
        _block_id: Option<BlockId>,
    ) -> ProviderResult<U256> {
        todo!()
    }
    async fn get_deposit_info(&self, _address: Address) -> ProviderResult<DepositInfo> {
        todo!()
    }
    async fn get_balances(&self, _addresses: Vec<Address>) -> ProviderResult<Vec<U256>> {
        todo!()
    }
}

impl EntryPointProvider<UserOperationV0_6> for DummyEntryPointProviderV06 {}

#[derive(Clone)]
pub struct DummyEntryPointProviderV07;

#[async_trait::async_trait]
impl SignatureAggregator for DummyEntryPointProviderV07 {
    type UO = UserOperationV0_7;
    async fn aggregate_signatures(
        &self,
        _aggregator_address: Address,
        _ops: Vec<UserOperationV0_7>,
    ) -> ProviderResult<Option<Bytes>> {
        todo!()
    }
    async fn validate_user_op_signature(
        &self,
        _aggregator_address: Address,
        _user_op: UserOperationV0_7,
    ) -> ProviderResult<AggregatorOut> {
        todo!()
    }
}

#[async_trait::async_trait]
impl BundleHandler for DummyEntryPointProviderV07 {
    type UO = UserOperationV0_7;
    async fn call_handle_ops(
        &self,
        _ops_per_aggregator: Vec<UserOpsPerAggregator<UserOperationV0_7>>,
        _sender_eoa: Address,
        _gas_limit: u64,
        _gas_fees: GasFees,
        _proxy: Option<Address>,
        _validation_only: bool,
    ) -> ProviderResult<HandleOpsOut> {
        todo!()
    }
    fn get_send_bundle_transaction(
        &self,
        _ops_per_aggregator: Vec<UserOpsPerAggregator<UserOperationV0_7>>,
        _sender_eoa: Address,
        _gas_limit: u64,
        _gas_fees: GasFees,
        _proxy: Option<Address>,
    ) -> TransactionRequest {
        todo!()
    }
    fn decode_handle_ops_revert(
        _message: &str,
        _revert_data: &Option<Bytes>,
    ) -> Option<HandleOpsOut> {
        todo!()
    }
    fn decode_ops_from_calldata(
        _chain_spec: &ChainSpec,
        _calldata: &Bytes,
    ) -> Vec<UserOpsPerAggregator<UserOperationV0_7>> {
        todo!()
    }
}

#[async_trait::async_trait]
impl SimulationProvider for DummyEntryPointProviderV07 {
    type UO = UserOperationV0_7;
    fn get_tracer_simulate_validation_call(
        &self,
        _user_op: UserOperationV0_7,
    ) -> ProviderResult<(TransactionRequest, StateOverride)> {
        todo!()
    }
    async fn simulate_validation(
        &self,
        _user_op: UserOperationV0_7,
        _block_id: Option<BlockId>,
    ) -> ProviderResult<Result<ValidationOutput, ValidationRevert>> {
        todo!()
    }
    async fn simulate_handle_op(
        &self,
        _op: UserOperationV0_7,
        _target: Address,
        _target_call_data: Bytes,
        _block_id: BlockId,
        _state_override: StateOverride,
    ) -> ProviderResult<Result<ExecutionResult, ValidationRevert>> {
        todo!()
    }
    async fn simulate_handle_op_estimate_gas(
        &self,
        _op: UserOperationV0_7,
        _target: Address,
        _target_call_data: Bytes,
        _block_id: BlockId,
        _state_override: StateOverride,
    ) -> ProviderResult<Result<ExecutionResult, ValidationRevert>> {
        todo!()
    }
    fn decode_simulate_handle_ops_revert(
        _revert_data: &Bytes,
    ) -> ProviderResult<Result<ExecutionResult, ValidationRevert>> {
        todo!()
    }
    fn simulation_should_revert(&self) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl DAGasProvider for DummyEntryPointProviderV07 {
    type UO = UserOperationV0_7;
    async fn calc_da_gas(
        &self,
        _uo: UserOperationV0_7,
        _block: BlockHashOrNumber,
        _gas_price: u128,
        _bundle_size: usize,
    ) -> ProviderResult<(u128, DAGasData, DAGasBlockData)> {
        todo!()
    }
}

#[async_trait::async_trait]
impl EntryPoint for DummyEntryPointProviderV07 {
    fn version(&self) -> EntryPointVersion {
        todo!()
    }
    fn address(&self) -> &Address {
        static ZERO: Address = Address::ZERO;
        &ZERO
    }
    async fn balance_of(
        &self,
        _address: Address,
        _block_id: Option<BlockId>,
    ) -> ProviderResult<U256> {
        todo!()
    }
    async fn get_deposit_info(&self, _address: Address) -> ProviderResult<DepositInfo> {
        todo!()
    }
    async fn get_balances(&self, _addresses: Vec<Address>) -> ProviderResult<Vec<U256>> {
        todo!()
    }
}

impl EntryPointProvider<UserOperationV0_7> for DummyEntryPointProviderV07 {}

#[derive(Clone)]
pub struct DummyDAGasOracle;

#[async_trait::async_trait]
impl DAGasOracle for DummyDAGasOracle {
    async fn estimate_da_gas(
        &self,
        _bytes: Bytes,
        _to: Address,
        _block: BlockHashOrNumber,
        _gas_price: u128,
        _extra_data_len: usize,
    ) -> ProviderResult<(u128, DAGasData, DAGasBlockData)> {
        todo!()
    }
}

#[derive(Clone)]
pub struct DummyDAGasOracleSync;

#[async_trait::async_trait]
impl DAGasOracleSync for DummyDAGasOracleSync {
    async fn da_block_data(&self, _block: BlockHashOrNumber) -> ProviderResult<DAGasBlockData> {
        todo!()
    }
    async fn da_gas_data(
        &self,
        _gas_data: Bytes,
        _to: Address,
        _block: BlockHashOrNumber,
    ) -> ProviderResult<DAGasData> {
        todo!()
    }
    fn calc_da_gas_sync(
        &self,
        _gas_data: &DAGasData,
        _block_data: &DAGasBlockData,
        _gas_price: u128,
        _extra_data_len: usize,
    ) -> u128 {
        todo!()
    }
}

#[async_trait::async_trait]
impl DAGasOracle for DummyDAGasOracleSync {
    async fn estimate_da_gas(
        &self,
        _bytes: Bytes,
        _to: Address,
        _block: BlockHashOrNumber,
        _gas_price: u128,
        _extra_data_len: usize,
    ) -> ProviderResult<(u128, DAGasData, DAGasBlockData)> {
        todo!()
    }
}

#[derive(Clone)]
pub struct DummyFeeEstimator;

#[async_trait::async_trait]
impl FeeEstimator for DummyFeeEstimator {
    async fn required_bundle_fees(
        &self,
        _block_hash: B256,
        _min_fees: Option<GasFees>,
    ) -> anyhow::Result<(GasFees, u128)> {
        todo!()
    }
    async fn latest_bundle_fees(&self) -> anyhow::Result<(GasFees, u128)> {
        todo!()
    }
    fn required_op_fees(&self, _bundle_fees: GasFees) -> GasFees {
        todo!()
    }
}

#[allow(clippy::type_complexity)]
pub fn construct_providers(
    args: &CommonArgs,
    chain_spec: &ChainSpec,
) -> anyhow::Result<impl Providers + 'static> {
    let provider = Arc::new(rundler_provider::new_alloy_provider(
        args.node_http.as_ref().context("must provide node_http")?,
        args.provider_client_timeout_seconds,
    )?);
    let (da_gas_oracle, da_gas_oracle_sync) =
        rundler_provider::new_alloy_da_gas_oracle(chain_spec, provider.clone());

    let evm = AlloyEvmProvider::new(provider.clone());

    let ep_v0_6 = if args.disable_entry_point_v0_6 {
        None
    } else {
        Some(AlloyEntryPointV0_6::new(
            chain_spec.clone(),
            args.max_verification_gas,
            args.max_simulate_handle_op_gas,
            args.max_gas_estimation_gas,
            args.max_aggregation_gas,
            provider.clone(),
            da_gas_oracle.clone(),
        ))
    };

    let ep_v0_7 = if args.disable_entry_point_v0_7 {
        None
    } else {
        Some(AlloyEntryPointV0_7::new(
            chain_spec.clone(),
            args.max_verification_gas,
            args.max_simulate_handle_op_gas,
            args.max_gas_estimation_gas,
            args.max_aggregation_gas,
            provider.clone(),
            da_gas_oracle.clone(),
        ))
    };

    let priority_fee_mode = PriorityFeeMode::try_from(
        args.priority_fee_mode_kind.as_str(),
        args.priority_fee_mode_value,
    )?;
    let fee_estimator = Arc::new(new_fee_estimator(
        chain_spec,
        evm.clone(),
        priority_fee_mode,
        args.bundle_base_fee_overhead_percent,
        args.bundle_priority_fee_overhead_percent,
    ));

    Ok(RundlerProviders {
        provider: evm,
        ep_v0_6,
        ep_v0_7,
        da_gas_oracle,
        da_gas_oracle_sync,
        fee_estimator,
    })
}

fn lint_da_gas_tracking(da_gas_tracking_enabled: bool, chain_spec: &ChainSpec) -> bool {
    if !da_gas_tracking_enabled {
        return false;
    }

    if !chain_spec.da_pre_verification_gas {
        tracing::warn!("DA tracking is disabled because DA pre-verification gas is not enabled");
        false
    } else if !(chain_spec.da_gas_oracle_type == DAGasOracleType::CachedNitro
        || chain_spec.da_gas_oracle_type == DAGasOracleType::LocalBedrock)
    {
        tracing::warn!("DA tracking is disabled because DA gas oracle contract type {:?} does not support caching", chain_spec.da_gas_oracle_type);
        false
    } else {
        true
    }
}

async fn load_configs(
    args: &CommonArgs,
) -> anyhow::Result<(Option<MempoolConfigs>, Option<EntryPointBuilderConfigs>)> {
    let mempool_configs = if let Some(mempool_config_path) = &args.mempool_config_path {
        let mempool_configs = get_json_config::<MempoolConfigs>(mempool_config_path)
            .await
            .with_context(|| format!("should load mempool config from {mempool_config_path}"))?;

        tracing::info!("Mempool configs: {:?}", mempool_configs);

        // For now only allow one mempool defined per entry point
        let mut entry_points = vec![];
        for mempool_config in mempool_configs.0.values() {
            let ep = mempool_config.entry_point();
            if entry_points.contains(&ep) {
                bail!("multiple mempool configs defined for entry point {:?}", ep);
            }
            entry_points.push(ep);
        }

        Some(mempool_configs)
    } else {
        None
    };

    let builders_config = if let Some(builders_config_path) = &args.builders_config_path {
        let builders_config = get_json_config::<EntryPointBuilderConfigs>(builders_config_path)
            .await
            .with_context(|| format!("should load builders config from {builders_config_path}"))?;

        tracing::info!("Entry point builders: {:?}", builders_config);

        Some(builders_config)
    } else {
        None
    };

    Ok((mempool_configs, builders_config))
}
