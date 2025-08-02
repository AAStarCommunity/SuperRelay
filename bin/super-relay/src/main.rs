// SuperRelay - Enterprise Account Abstraction Paymaster Solution
// A wrapper around rundler with SuperPaymaster enhancements

use std::{fs, process::Command};

use clap::{Parser, Subcommand};
use eyre::Result;
use serde::Deserialize;

#[derive(Parser)]
#[command(
    name = "super-relay",
    about = "SuperRelay - Enterprise Account Abstraction Solution",
    version = "0.1.4"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the full SuperRelay node (rundler + paymaster + monitoring)
    Node {
        /// Path to configuration file
        #[arg(long, default_value = "config/config.toml")]
        config: String,

        /// Additional rundler arguments
        #[arg(last = true)]
        rundler_args: Vec<String>,
    },
    /// Run only the pool service
    Pool {
        /// Additional rundler arguments
        #[arg(last = true)]
        rundler_args: Vec<String>,
    },
    /// Run only the builder service
    Builder {
        /// Additional rundler arguments
        #[arg(last = true)]
        rundler_args: Vec<String>,
    },
    /// Run admin commands
    Admin {
        /// Additional rundler arguments
        #[arg(last = true)]
        rundler_args: Vec<String>,
    },
    /// Show version information
    Version,
    /// Check service status
    Status,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct SuperRelayConfig {
    node: NodeConfig,
    pool: PoolConfig,
    rpc: RpcConfig,
    paymaster_relay: PaymasterRelayConfig,
    mempool: MempoolConfig,
    rate_limiting: Option<RateLimitingConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct NodeConfig {
    http_api: Option<String>,
    max_entries_per_chain: Option<u32>,
    max_mem_entries_per_chain: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct PoolConfig {
    max_expire_duration_seconds: Option<u64>,
    max_ops_per_unstaked_sender: Option<u32>,
    throttled_entity_mempool_count: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct RpcConfig {
    max_verification_gas: Option<u64>,
    max_call_gas: Option<u64>,
    api: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct PaymasterRelayConfig {
    enabled: Option<bool>,
    private_key: Option<String>,
    policy_file: Option<String>,
    entry_points: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct MempoolConfig {
    max_send_bundle_txns: Option<u32>,
    bundle_max_length: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct RateLimitingConfig {
    enabled: Option<bool>,
    requests_per_second: Option<u32>,
    burst_capacity: Option<u32>,
    cleanup_interval_seconds: Option<u64>,
    entry_expiry_seconds: Option<u64>,
}

impl Cli {
    async fn run(self) -> Result<()> {
        // Show SuperRelay branding
        self.show_banner();

        match self.command {
            Commands::Node {
                ref config,
                ref rundler_args,
            } => {
                println!("🚀 Starting SuperRelay Node...\n");

                // Parse TOML configuration with environment variable expansion
                let config_content = fs::read_to_string(config)
                    .map_err(|e| eyre::eyre!("Failed to read config file '{}': {}", config, e))?;

                // Expand environment variables in config content
                let expanded_content = expand_env_vars(&config_content);

                let super_config: SuperRelayConfig = toml::from_str(&expanded_content)
                    .map_err(|e| eyre::eyre!("Failed to parse config file: {}", e))?;

                // Convert config to rundler arguments, avoiding duplicates
                let config_args = self.config_to_rundler_args(&super_config)?;
                let mut rundler_args_final = config_args;

                // Only add additional args that don't conflict with config args
                for arg in rundler_args.iter() {
                    if !arg.starts_with("--network")
                        && !arg.starts_with("--node_http")
                        && !arg.starts_with("--metrics.port")
                    {
                        rundler_args_final.push(arg.clone());
                    }
                }

                // Use the built rundler binary directly instead of cargo run
                let rundler_path = std::env::current_dir()?
                    .join("target")
                    .join("release")
                    .join("rundler");

                let mut cmd = Command::new(&rundler_path);
                cmd.arg("node");
                cmd.args(&rundler_args_final);

                println!(
                    "🔧 Executing: {} node {}",
                    rundler_path.display(),
                    rundler_args_final.join(" ")
                );

                let status = cmd.status()?;
                if !status.success() {
                    eyre::bail!("rundler node failed with exit code: {:?}", status.code());
                }
            }
            Commands::Pool { rundler_args } => {
                let rundler_path = std::env::current_dir()?
                    .join("target")
                    .join("release")
                    .join("rundler");

                let mut cmd = Command::new(&rundler_path);
                cmd.arg("pool");
                cmd.args(&rundler_args);

                let status = cmd.status()?;
                if !status.success() {
                    eyre::bail!("rundler pool failed with exit code: {:?}", status.code());
                }
            }
            Commands::Builder { rundler_args } => {
                let rundler_path = std::env::current_dir()?
                    .join("target")
                    .join("release")
                    .join("rundler");

                let mut cmd = Command::new(&rundler_path);
                cmd.arg("builder");
                cmd.args(&rundler_args);

                let status = cmd.status()?;
                if !status.success() {
                    eyre::bail!("rundler builder failed with exit code: {:?}", status.code());
                }
            }
            Commands::Admin { rundler_args } => {
                let rundler_path = std::env::current_dir()?
                    .join("target")
                    .join("release")
                    .join("rundler");

                let mut cmd = Command::new(&rundler_path);
                cmd.arg("admin");
                cmd.args(&rundler_args);

                let status = cmd.status()?;
                if !status.success() {
                    eyre::bail!("rundler admin failed with exit code: {:?}", status.code());
                }
            }
            Commands::Version => {
                self.show_version();
            }
            Commands::Status => {
                self.check_status().await?;
            }
        }

        Ok(())
    }

    fn config_to_rundler_args(&self, config: &SuperRelayConfig) -> Result<Vec<String>> {
        // Read network and node settings from environment variables or config file, supporting local development
        let network = std::env::var("NETWORK")
            .or_else(|_| std::env::var("CHAIN_NETWORK"))
            .unwrap_or_else(|_| "dev".to_string());

        let node_http = std::env::var("RPC_URL")
            .or_else(|_| std::env::var("NODE_HTTP"))
            .unwrap_or_else(|_| "http://localhost:8545".to_string());

        // Smart private key management: prioritize environment variables, support .env files for testing
        let signer_keys = std::env::var("SIGNER_PRIVATE_KEYS")
            .or_else(|_| {
                // Testing/development phase: try loading from .env file
                if let Ok(env_content) = std::fs::read_to_string(".env") {
                    for line in env_content.lines() {
                        if line.starts_with("SIGNER_PRIVATE_KEYS=") {
                            return Ok(line.split('=').nth(1).unwrap_or("").to_string());
                        }
                    }
                }
                Err(std::env::VarError::NotPresent)
            })
            .map_err(|_| {
                eyre::eyre!(
                    "🔐 Private key configuration required!\n\
                \n\
                🧪 For TESTING/DEVELOPMENT:\n\
                   • Set SIGNER_PRIVATE_KEYS in .env file\n\
                   • Or use: source ./scripts/load_dev_env.sh\n\
                \n\
                🏭 For PRODUCTION:\n\
                   • Set SIGNER_PRIVATE_KEYS environment variable\n\
                   • Future: Hardware wallet API support planned\n\
                \n\
                ⚠️  NEVER use test keys in production!"
                )
            })?;

        let mut args = vec![
            "--network".to_string(),
            network,
            "--node_http".to_string(),
            node_http,
            "--signer.private_keys".to_string(),
            signer_keys,
        ];

        // Node configuration
        if let Some(ref http_api) = config.node.http_api {
            let parts: Vec<&str> = http_api.split(':').collect();
            if parts.len() == 2 {
                args.push("--rpc.host".to_string());
                args.push(parts[0].to_string());
                args.push("--rpc.port".to_string());
                args.push(parts[1].to_string());
            }
        }

        // Pool configuration
        if let Some(max_ops) = config.pool.max_ops_per_unstaked_sender {
            args.push("--pool.same_sender_mempool_count".to_string());
            args.push(max_ops.to_string());
        }

        if let Some(max_ops) = config.pool.throttled_entity_mempool_count {
            args.push("--pool.throttled_entity_mempool_count".to_string());
            args.push(max_ops.to_string());
        }

        // RPC configuration
        if let Some(max_verification_gas) = config.rpc.max_verification_gas {
            args.push("--max_verification_gas".to_string());
            args.push(max_verification_gas.to_string());
        }

        // Paymaster relay configuration
        if let Some(enabled) = config.paymaster_relay.enabled {
            if enabled {
                args.push("--paymaster.enabled".to_string());

                if let Some(ref private_key) = config.paymaster_relay.private_key {
                    args.push("--paymaster.private_key".to_string());
                    // Environment variables in config file have been expanded through expand_env_vars
                    args.push(private_key.clone());
                }

                if let Some(ref policy_file) = config.paymaster_relay.policy_file {
                    args.push("--paymaster.policy_file".to_string());
                    args.push(policy_file.clone());
                }
            }
        }

        // Always enable necessary APIs
        args.push("--rpc.api".to_string());
        args.push("eth,rundler,paymaster".to_string());

        Ok(args)
    }

    fn show_banner(&self) {
        println!("🚀 SuperRelay v0.1.4 - Enterprise Account Abstraction Service");
        println!("📊 Enhanced with PaymasterRelay, Monitoring & Swagger UI");
        println!("🌐 Swagger UI: http://localhost:9000/swagger-ui/");
        println!("📈 Monitoring: http://localhost:9000/health");
        println!("🔧 Built on Rundler v0.9.0 with SuperPaymaster Extensions");
        println!();
    }

    fn show_version(&self) {
        println!("SuperRelay v0.1.4");
        println!("Built on Rundler v0.9.0");
        println!();
        println!("🚀 Enterprise Account Abstraction Features:");
        println!("  - ERC-4337 compliant bundler");
        println!("  - Advanced paymaster policies");
        println!("  - Real-time monitoring & metrics");
        println!("  - Swagger UI documentation");
        println!("  - Enterprise-grade policies");
    }

    async fn check_status(&self) -> Result<()> {
        println!("🔍 Checking SuperRelay service status...\n");

        // Check main RPC service
        match self
            .check_endpoint("http://localhost:3000", "Main RPC Service")
            .await
        {
            Ok(_) => println!("✅ Main RPC Service: RUNNING"),
            Err(_) => println!("❌ Main RPC Service: NOT RUNNING"),
        }

        // Check Swagger UI
        match self
            .check_endpoint("http://localhost:9000/health", "Swagger UI & Monitoring")
            .await
        {
            Ok(_) => println!("✅ Swagger UI & Monitoring: RUNNING"),
            Err(_) => println!("❌ Swagger UI & Monitoring: NOT RUNNING"),
        }

        // Check Prometheus metrics
        match self
            .check_endpoint("http://localhost:8080/metrics", "Prometheus Metrics")
            .await
        {
            Ok(_) => println!("✅ Prometheus Metrics: RUNNING"),
            Err(_) => println!("❌ Prometheus Metrics: NOT RUNNING"),
        }

        println!("\n📋 Service URLs:");
        println!("  🌐 Swagger UI: http://localhost:9000/swagger-ui/");
        println!("  🏥 Health Check: http://localhost:9000/health");
        println!("  📊 Metrics: http://localhost:9000/metrics");
        println!("  📈 Prometheus: http://localhost:8080/metrics");
        println!("  🔧 Main RPC: http://localhost:3000");

        Ok(())
    }

    async fn check_endpoint(&self, url: &str, _service: &str) -> Result<()> {
        // Simple TCP connection check (avoiding external dependencies)
        use std::{net::TcpStream, time::Duration};

        let url_parts: Vec<&str> = url.split("://").collect();
        if url_parts.len() != 2 {
            return Err(eyre::eyre!("Invalid URL format"));
        }

        let host_port: Vec<&str> = url_parts[1].split('/').next().unwrap().split(':').collect();
        if host_port.len() != 2 {
            return Err(eyre::eyre!("Invalid host:port format"));
        }

        let host = host_port[0];
        let port: u16 = host_port[1].parse()?;

        match TcpStream::connect_timeout(
            &format!("{}:{}", host, port).parse()?,
            Duration::from_secs(3),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre::eyre!("Connection failed: {}", e)),
        }
    }
}

/// Expand environment variables in the form ${VAR_NAME} in the given string
fn expand_env_vars(content: &str) -> String {
    let mut result = content.to_string();

    // Find all ${VAR_NAME} patterns and replace them
    let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();

    for captures in re.captures_iter(content) {
        let full_match = captures.get(0).unwrap().as_str();
        let var_name = captures.get(1).unwrap().as_str();

        if let Ok(var_value) = std::env::var(var_name) {
            result = result.replace(full_match, &var_value);
        } else {
            eprintln!(
                "⚠️  Environment variable {} not set, keeping original value",
                var_name
            );
        }
    }

    result
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await
}
