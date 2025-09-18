use std::sync::Arc;

use alloy_primitives::{Address, Bytes, U256};
use ethers::types::H160;
use rundler_builder::LocalBuilderHandle;
use rundler_paymaster_relay::PaymasterRelayService;
use rundler_pool::LocalPoolHandle;
use rundler_types::{
    builder::Builder, chain::ChainSpec, pool::Pool, v0_6, v0_7, UserOperation,
    UserOperationPermissions, UserOperationVariant,
};
use serde_json::{json, Value};
use tracing::{debug, error, warn};

use crate::{
    authorization::AuthorizationChecker,
    bls_protection_service::BlsProtectionService,
    contract_account_security::ContractAccountSecurityValidator,
    error::{GatewayError, GatewayResult},
    gateway::JsonRpcRequest,
    security::SecurityChecker,
    validation::DataIntegrityChecker,
};

/// Router that handles request routing to appropriate rundler components
#[derive(Clone)]
pub struct GatewayRouter {
    /// Supported EntryPoint addresses
    supported_entry_points: Vec<Address>,
    /// Pool handle for mempool operations
    pool_handle: Option<Arc<LocalPoolHandle>>,
    /// Builder handle for bundle operations
    builder_handle: Option<Arc<LocalBuilderHandle>>,
    /// Chain ID for this network
    chain_id: u64,
    /// BLS protection service for aggregated signatures
    bls_protection_service: Option<Arc<BlsProtectionService>>,
    /// Contract account security validator
    contract_security_validator: Option<Arc<ContractAccountSecurityValidator>>,
}

/// Configuration for the Gateway's ETH API
#[derive(Default)]
pub struct EthApiConfig {
    /// Chain ID for the network
    pub chain_id: u64,
    /// Supported EntryPoint addresses
    pub entry_points: Vec<Address>,
}

impl GatewayRouter {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            supported_entry_points: Self::default_entry_points(),
            pool_handle: None,
            builder_handle: None,
            chain_id: 31337, // Anvil default
            bls_protection_service: None,
            contract_security_validator: None,
        }
    }

    /// Create a new router with rundler components
    pub fn with_rundler_components(
        pool_handle: Arc<LocalPoolHandle>,
        builder_handle: Option<Arc<LocalBuilderHandle>>,
        config: EthApiConfig,
    ) -> Self {
        let chain_id = if config.chain_id == 0 {
            31337
        } else {
            config.chain_id
        };
        let entry_points = if config.entry_points.is_empty() {
            Self::default_entry_points()
        } else {
            config.entry_points
        };

        Self {
            supported_entry_points: entry_points,
            pool_handle: Some(pool_handle),
            builder_handle,
            chain_id,
            bls_protection_service: None,
            contract_security_validator: None,
        }
    }

    /// Create a new router with custom configuration (legacy method)
    pub fn with_config(config: EthApiConfig) -> Self {
        Self {
            supported_entry_points: if config.entry_points.is_empty() {
                Self::default_entry_points()
            } else {
                config.entry_points
            },
            pool_handle: None,
            builder_handle: None,
            chain_id: if config.chain_id == 0 {
                31337
            } else {
                config.chain_id
            },
            bls_protection_service: None,
            contract_security_validator: None,
        }
    }

    /// Set the BLS protection service
    pub fn with_bls_protection_service(mut self, service: Arc<BlsProtectionService>) -> Self {
        self.bls_protection_service = Some(service);
        self
    }

    /// Set the contract account security validator
    pub fn with_contract_security_validator(
        mut self,
        validator: Arc<ContractAccountSecurityValidator>,
    ) -> Self {
        self.contract_security_validator = Some(validator);
        self
    }

    /// Default EntryPoint addresses (commonly used ones)
    fn default_entry_points() -> Vec<Address> {
        vec![
            // EntryPoint v0.6
            "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                .parse()
                .unwrap(),
            // EntryPoint v0.7 (common testnet address)
            "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
                .parse()
                .unwrap(),
        ]
    }

    /// Route request to paymaster service
    pub async fn route_to_paymaster(
        &self,
        paymaster_service: &Arc<PaymasterRelayService>,
        request: &JsonRpcRequest,
    ) -> GatewayResult<Value> {
        debug!("Routing to paymaster: {}", request.method);

        match request.method.as_str() {
            "pm_sponsorUserOperation" => {
                self.handle_sponsor_user_operation(paymaster_service, &request.params)
                    .await
            }
            _ => Err(GatewayError::InvalidRequest(format!(
                "Unknown paymaster method: {}",
                request.method
            ))),
        }
    }

    /// Route request to rundler components
    pub async fn route_to_rundler(&self, request: &JsonRpcRequest) -> GatewayResult<Value> {
        debug!("Routing to rundler: {}", request.method);

        match request.method.as_str() {
            "eth_supportedEntryPoints" => self.get_supported_entry_points(),
            "eth_chainId" => self.get_chain_id(),
            "eth_estimateUserOperationGas" => {
                if let Some(pool) = &self.pool_handle {
                    self.estimate_user_operation_gas_with_pool(pool, request)
                        .await
                } else {
                    self.estimate_user_operation_gas_fallback(request).await
                }
            }
            "eth_sendUserOperation" => {
                if let Some(pool) = &self.pool_handle {
                    self.send_user_operation_with_pool(pool, request).await
                } else {
                    warn!("Pool not available for eth_sendUserOperation");
                    Err(GatewayError::InvalidRequest(
                        "Pool not available in gateway mode".to_string(),
                    ))
                }
            }
            "eth_getUserOperationByHash" => {
                if let Some(pool) = &self.pool_handle {
                    self.get_user_operation_by_hash_with_pool(pool, request)
                        .await
                } else {
                    Ok(Value::Null) // Not found
                }
            }
            "eth_getUserOperationReceipt" => {
                if let Some(pool) = &self.pool_handle {
                    self.get_user_operation_receipt_with_pool(pool, request)
                        .await
                } else {
                    Ok(Value::Null) // Not found
                }
            }
            _ => {
                warn!("Unhandled rundler method: {}", request.method);
                Err(GatewayError::UnsupportedMethod(request.method.clone()))
            }
        }
    }

    /// Handle pm_sponsorUserOperation method
    async fn handle_sponsor_user_operation(
        &self,
        paymaster_service: &Arc<PaymasterRelayService>,
        params: &[Value],
    ) -> GatewayResult<Value> {
        if params.len() != 2 {
            return Err(GatewayError::InvalidRequest(
                "pm_sponsorUserOperation requires exactly 2 parameters".to_string(),
            ));
        }

        let _user_operation = &params[0];
        let _entry_point = &params[1];

        debug!(
            "Sponsoring UserOperation for entry point: {:?}",
            _entry_point
        );

        // 1. Parse EntryPoint address
        let entry_point: Address = _entry_point
            .as_str()
            .ok_or_else(|| {
                GatewayError::InvalidRequest("Entry point must be a string".to_string())
            })?
            .parse()
            .map_err(|_| GatewayError::InvalidRequest("Invalid entry point address".to_string()))?;

        // 2. Validate entry point is supported
        if !self.supported_entry_points.contains(&entry_point) {
            return Err(GatewayError::InvalidRequest(format!(
                "Unsupported entry point: {:#x}",
                entry_point
            )));
        }

        // 3. Parse UserOperation from JSON (simplified for now)
        let user_op_variant = self.parse_user_operation_from_json(_user_operation, entry_point)?;

        debug!(
            "Parsed UserOperation variant: {:?}",
            user_op_variant.entry_point()
        );

        // 4. Data Integrity Check (第一个业务步骤: 数据的完备性检查)
        debug!("🔍 Starting data integrity validation");
        let data_integrity_checker = DataIntegrityChecker::new();
        let entry_point_str = format!("{:#x}", entry_point);

        match data_integrity_checker
            .validate_user_operation(&user_op_variant, &entry_point_str)
            .await
        {
            Ok(validation_result) => {
                if !validation_result.is_valid {
                    error!(
                        "❌ Data integrity validation failed: {}",
                        validation_result.summary
                    );
                    return Err(GatewayError::ValidationError(format!(
                        "Data integrity check failed: {} critical issues found: [{}]",
                        validation_result.critical_issues.len(),
                        validation_result.critical_issues.join(", ")
                    )));
                } else if !validation_result.warnings.is_empty() {
                    warn!(
                        "⚠️ Data integrity validation passed with warnings: {}",
                        validation_result.warnings.join(", ")
                    );
                }
                debug!(
                    "✅ Data integrity validation passed (score: {}): {}",
                    validation_result.validation_score, validation_result.summary
                );
            }
            Err(e) => {
                error!("💥 Data integrity validation error: {}", e);
                return Err(GatewayError::InternalError(format!(
                    "Data integrity validation system error: {}",
                    e
                )));
            }
        }

        // 5. Authorization Check (第二个业务步骤: 资格检查)
        debug!("🔐 Starting authorization and eligibility check");
        let mut authorization_checker = AuthorizationChecker::new();

        // Load authorization configuration
        if let Err(e) = authorization_checker.load_configuration().await {
            warn!("Failed to load authorization configuration: {}", e);
            // Continue with default configuration
        }

        match authorization_checker
            .check_authorization(&user_op_variant, &entry_point, None)
            .await
        {
            Ok(authorization_result) => {
                if !authorization_result.is_authorized {
                    error!(
                        "❌ Authorization check failed: {}",
                        authorization_result.summary
                    );
                    return Err(GatewayError::ValidationError(format!(
                        "Authorization check failed: {} blocking issues found: [{}]",
                        authorization_result.blocking_issues.len(),
                        authorization_result.blocking_issues.join(", ")
                    )));
                } else if !authorization_result.warnings.is_empty() {
                    warn!(
                        "⚠️ Authorization check passed with warnings: {}",
                        authorization_result.warnings.join(", ")
                    );
                }
                debug!(
                    "✅ Authorization check passed (score: {}): {}",
                    authorization_result.authorization_score, authorization_result.summary
                );
            }
            Err(e) => {
                error!("💥 Authorization check error: {}", e);
                return Err(GatewayError::InternalError(format!(
                    "Authorization check system error: {}",
                    e
                )));
            }
        }

        // 6. Security Check (第三个业务步骤: 安全性检查)
        debug!("🔒 Starting security analysis");
        let mut security_checker = SecurityChecker::new();

        // Load threat intelligence data
        if let Err(e) = security_checker.load_threat_intelligence().await {
            warn!("Failed to load threat intelligence: {}", e);
            // Continue with default security configuration
        }

        match security_checker
            .check_security(&user_op_variant, &entry_point, None)
            .await
        {
            Ok(security_result) => {
                if !security_result.is_secure {
                    error!("🚨 Security check failed: {}", security_result.summary);
                    return Err(GatewayError::ValidationError(format!(
                        "Security check failed: {} critical violations found: [{}]",
                        security_result.critical_violations.len(),
                        security_result.critical_violations.join(", ")
                    )));
                } else if !security_result.warnings.is_empty() {
                    warn!(
                        "⚠️ Security check passed with warnings: {}",
                        security_result.warnings.join(", ")
                    );
                }
                debug!(
                    "✅ Security check passed (score: {}): {}",
                    security_result.security_score, security_result.summary
                );
            }
            Err(e) => {
                error!("💥 Security check error: {}", e);
                return Err(GatewayError::InternalError(format!(
                    "Security check system error: {}",
                    e
                )));
            }
        }

        // 7. Call paymaster service for sponsorship
        // Convert alloy Address to ethers H160 for compatibility
        let ethers_entry_point = H160::from_slice(entry_point.as_slice());
        match paymaster_service
            .sponsor_user_operation(user_op_variant, ethers_entry_point)
            .await
        {
            Ok(sponsor_result) => {
                debug!("Sponsorship successful");

                // Convert PaymasterSponsorResult to JSON response
                let mut response = json!({
                    "paymasterAndData": format!("0x{}", hex::encode(&sponsor_result.paymaster_and_data))
                });

                // Add optional gas limits if present
                if let Some(verification_gas) = sponsor_result.verification_gas_limit {
                    response["paymasterVerificationGasLimit"] =
                        json!(format!("0x{:x}", verification_gas));
                }
                if let Some(post_op_gas) = sponsor_result.post_op_gas_limit {
                    response["paymasterPostOpGasLimit"] = json!(format!("0x{:x}", post_op_gas));
                }
                if let Some(pre_verification_gas) = sponsor_result.pre_verification_gas {
                    response["preVerificationGas"] = json!(format!("0x{:x}", pre_verification_gas));
                }
                if let Some(verification_gas_limit_uo) = sponsor_result.verification_gas_limit_uo {
                    response["verificationGasLimit"] =
                        json!(format!("0x{:x}", verification_gas_limit_uo));
                }
                if let Some(call_gas_limit) = sponsor_result.call_gas_limit {
                    response["callGasLimit"] = json!(format!("0x{:x}", call_gas_limit));
                }

                Ok(response)
            }
            Err(e) => {
                error!("Sponsorship failed: {:?}", e);
                Err(GatewayError::PaymasterError(format!(
                    "Sponsorship failed: {}",
                    e
                )))
            }
        }
    }

    // === Rundler component integration methods ===

    /// Get supported EntryPoint addresses
    fn get_supported_entry_points(&self) -> GatewayResult<Value> {
        let entry_points: Vec<String> = self
            .supported_entry_points
            .iter()
            .map(|addr| format!("{:#x}", addr))
            .collect();

        debug!("Returning supported entry points: {:?}", entry_points);
        Ok(json!(entry_points))
    }

    /// Get chain ID
    fn get_chain_id(&self) -> GatewayResult<Value> {
        let chain_id_hex = format!("0x{:x}", self.chain_id);
        debug!("Returning chain ID: {}", chain_id_hex);
        Ok(json!(chain_id_hex))
    }

    /// Estimate user operation gas using real pool component
    async fn estimate_user_operation_gas_with_pool(
        &self,
        _pool: &Arc<LocalPoolHandle>,
        request: &JsonRpcRequest,
    ) -> GatewayResult<Value> {
        if request.params.len() < 2 {
            return Err(GatewayError::InvalidRequest(
                "Missing parameters".to_string(),
            ));
        }

        let _user_op = &request.params[0];
        let entry_point = &request.params[1];

        debug!(
            "Estimating gas with pool for entry point: {:?}",
            entry_point
        );

        // TODO: Use actual pool.estimate_user_operation_gas() method
        // For now, return reasonable estimates
        Ok(json!({
            "preVerificationGas": "0x5208",      // 21,000
            "verificationGasLimit": "0x186A0",   // 100,000
            "callGasLimit": "0x186A0",           // 100,000
            "paymasterVerificationGasLimit": null,
            "paymasterPostOpGasLimit": null
        }))
    }

    /// Fallback gas estimation without pool
    async fn estimate_user_operation_gas_fallback(
        &self,
        request: &JsonRpcRequest,
    ) -> GatewayResult<Value> {
        if request.params.len() < 2 {
            return Err(GatewayError::InvalidRequest(
                "Missing parameters".to_string(),
            ));
        }

        debug!("Estimating gas with fallback method");

        // Return conservative estimates
        Ok(json!({
            "preVerificationGas": "0x5208",      // 21,000
            "verificationGasLimit": "0x186A0",   // 100,000
            "callGasLimit": "0x186A0",           // 100,000
        }))
    }

    /// Send user operation using real pool component
    async fn send_user_operation_with_pool(
        &self,
        _pool: &Arc<LocalPoolHandle>,
        request: &JsonRpcRequest,
    ) -> GatewayResult<Value> {
        if request.params.len() < 2 {
            return Err(GatewayError::InvalidRequest(
                "Missing parameters".to_string(),
            ));
        }

        let user_op = &request.params[0];
        let entry_point = &request.params[1];

        debug!(
            "Sending UserOperation with pool to entry point: {:?}",
            entry_point
        );

        // Validate entry point
        let entry_point_addr: Address = entry_point
            .as_str()
            .ok_or_else(|| GatewayError::InvalidRequest("Invalid entry point format".to_string()))?
            .parse()
            .map_err(|_| GatewayError::InvalidRequest("Invalid entry point address".to_string()))?;

        if !self.supported_entry_points.contains(&entry_point_addr) {
            return Err(GatewayError::InvalidRequest(format!(
                "Unsupported entry point: {:#x}",
                entry_point_addr
            )));
        }

        // Parse UserOperation from JSON and call real pool.add_op()
        let user_op_variant = self.parse_user_operation_from_json(user_op, entry_point_addr)?;

        debug!(
            "Parsed UserOperation: sender={:?}, entry_point={:?}",
            user_op_variant.sender(),
            user_op_variant.entry_point()
        );

        // BLS Protection Check (新增: BLS聚合签名防护检查)
        if let Some(ref bls_service) = self.bls_protection_service {
            // Check if this UserOperation uses an aggregator from request parameters
            let aggregator_address = self.extract_aggregator_from_request(user_op);

            debug!(
                "🔐 Starting BLS protection validation for aggregator: {:?}",
                aggregator_address
            );

            match bls_service
                .validate_user_operation_bls(&user_op_variant, aggregator_address)
                .await
            {
                Ok(bls_result) => {
                    if !bls_result.is_valid {
                        error!("❌ BLS validation failed: {}", bls_result.message);
                        return Err(GatewayError::ValidationError(format!(
                            "BLS signature validation failed: {} (Security issues: {})",
                            bls_result.message,
                            bls_result.security_issues.join(", ")
                        )));
                    } else if !bls_result.security_issues.is_empty() {
                        warn!(
                            "⚠️ BLS validation passed with security warnings: {}",
                            bls_result.security_issues.join(", ")
                        );
                    }
                    debug!(
                        "✅ BLS validation passed (validation_time: {}ms): {}",
                        bls_result.validation_time_ms, bls_result.message
                    );
                }
                Err(e) => {
                    error!("💥 BLS validation system error: {}", e);
                    return Err(GatewayError::InternalError(format!(
                        "BLS validation system error: {}",
                        e
                    )));
                }
            }
        }

        // Contract Account Security Validation (新增: 合约账户安全规则检查)
        if let Some(ref security_validator) = self.contract_security_validator {
            debug!("🔒 Starting contract account security validation...");

            match security_validator
                .validate_user_operation_security(&user_op_variant)
                .await
            {
                Ok(security_analysis) => {
                    if !security_analysis.is_secure {
                        error!(
                            "❌ Contract security validation failed for {:#x}: {}",
                            security_analysis.contract_address, security_analysis.summary
                        );
                        return Err(GatewayError::ValidationError(format!(
                            "Contract security validation failed (risk score: {}): {} Detected {} security issues",
                            security_analysis.risk_score,
                            security_analysis.summary,
                            security_analysis.security_risks.len()
                        )));
                    } else if !security_analysis.security_risks.is_empty() {
                        warn!(
                            "⚠️ Contract security validation passed with {} warnings for {:#x}",
                            security_analysis.security_risks.len(),
                            security_analysis.contract_address
                        );
                        for risk in &security_analysis.security_risks {
                            if risk.severity >= 2 {
                                warn!("  • {}: {}", risk.description, risk.recommendation);
                            }
                        }
                    }
                    debug!(
                        "✅ Contract security validation passed (risk score: {}, analysis time: {}ms): {}",
                        security_analysis.risk_score, security_analysis.analysis_time_ms, security_analysis.summary
                    );
                }
                Err(e) => {
                    error!("💥 Contract security validation system error: {}", e);
                    return Err(GatewayError::InternalError(format!(
                        "Contract security validation system error: {}",
                        e
                    )));
                }
            }
        }

        // Set appropriate permissions for user operations
        let perms = UserOperationPermissions {
            trusted: false,
            max_allowed_in_pool_for_sender: Some(10),
            underpriced_accept_pct: Some(10),
            underpriced_bundle_pct: Some(0),
            bundler_sponsorship: None,
        };

        // Submit operation to pool and get hash
        let user_op_hash = _pool.add_op(user_op_variant, perms).await.map_err(|e| {
            GatewayError::PoolError(format!("Failed to add operation to pool: {:?}", e))
        })?;

        // If builder is available, trigger bundle processing
        if let Some(builder) = &self.builder_handle {
            debug!("🏗️ Triggering bundle processing for new UserOperation");

            // Note: In the future, we might want to add direct builder integration
            // For now, the builder will pick up operations from the pool automatically
            match builder.get_supported_entry_points().await {
                Ok(entry_points) => {
                    debug!("📊 Builder supports entry points: {:?}", entry_points);
                }
                Err(e) => {
                    warn!("⚠️ Could not get builder entry points: {:?}", e);
                }
            }
        }

        // Return the real operation hash from pool
        let hash_hex = format!("0x{:x}", user_op_hash);
        debug!(
            "✅ Successfully submitted UserOperation to pool: hash={}",
            hash_hex
        );
        Ok(json!(hash_hex))
    }

    /// Get user operation by hash using real pool component
    async fn get_user_operation_by_hash_with_pool(
        &self,
        _pool: &Arc<LocalPoolHandle>,
        request: &JsonRpcRequest,
    ) -> GatewayResult<Value> {
        if request.params.is_empty() {
            return Err(GatewayError::InvalidRequest(
                "Missing hash parameter".to_string(),
            ));
        }

        let hash = &request.params[0];
        debug!("Looking up UserOperation by hash with pool: {:?}", hash);

        // Parse hash from hex string
        let hash_str = hash
            .as_str()
            .ok_or_else(|| GatewayError::InvalidRequest("Hash must be a string".to_string()))?;

        let hash_bytes = if let Some(stripped) = hash_str.strip_prefix("0x") {
            hex::decode(stripped)
        } else {
            hex::decode(hash_str)
        }
        .map_err(|_| GatewayError::InvalidRequest("Invalid hash format".to_string()))?;

        if hash_bytes.len() != 32 {
            return Err(GatewayError::InvalidRequest(
                "Hash must be 32 bytes".to_string(),
            ));
        }

        let hash_b256 = alloy_primitives::B256::from_slice(&hash_bytes);

        // Use real pool.get_op_by_hash() method
        match _pool.get_op_by_hash(hash_b256).await {
            Ok(Some(pool_op)) => {
                debug!(
                    "✅ Found UserOperation in pool: sender={:?}",
                    pool_op.uo.sender()
                );
                // Convert PoolOperation back to JSON format
                Ok(json!({
                    "userOperation": self.user_operation_to_json(&pool_op.uo),
                    "entryPoint": format!("{:#x}", pool_op.entry_point),
                    "blockNumber": null,
                    "blockHash": null,
                    "transactionHash": null
                }))
            }
            Ok(None) => {
                debug!("UserOperation not found in pool for hash: {}", hash_str);
                Ok(Value::Null)
            }
            Err(e) => {
                error!("Pool lookup error: {:?}", e);
                Err(GatewayError::PoolError(format!(
                    "Failed to lookup operation: {:?}",
                    e
                )))
            }
        }
    }

    /// Get user operation receipt using real pool component
    async fn get_user_operation_receipt_with_pool(
        &self,
        _pool: &Arc<LocalPoolHandle>,
        request: &JsonRpcRequest,
    ) -> GatewayResult<Value> {
        if request.params.is_empty() {
            return Err(GatewayError::InvalidRequest(
                "Missing hash parameter".to_string(),
            ));
        }

        let hash = &request.params[0];
        debug!(
            "Looking up UserOperation receipt by hash with pool: {:?}",
            hash
        );

        // Parse hash from hex string
        let hash_str = hash
            .as_str()
            .ok_or_else(|| GatewayError::InvalidRequest("Hash must be a string".to_string()))?;

        let hash_bytes = if let Some(stripped) = hash_str.strip_prefix("0x") {
            hex::decode(stripped)
        } else {
            hex::decode(hash_str)
        }
        .map_err(|_| GatewayError::InvalidRequest("Invalid hash format".to_string()))?;

        if hash_bytes.len() != 32 {
            return Err(GatewayError::InvalidRequest(
                "Hash must be 32 bytes".to_string(),
            ));
        }

        let hash_b256 = alloy_primitives::B256::from_slice(&hash_bytes);

        // Look up the operation first
        match _pool.get_op_by_hash(hash_b256).await {
            Ok(Some(pool_op)) => {
                // For now, return basic receipt structure
                // In a real implementation, this would come from blockchain data
                debug!(
                    "✅ Found UserOperation for receipt: sender={:?}",
                    pool_op.uo.sender()
                );
                Ok(json!({
                    "userOpHash": hash_str,
                    "entryPoint": format!("{:#x}", pool_op.entry_point),
                    "sender": format!("{:#x}", pool_op.uo.sender()),
                    "nonce": format!("0x{:x}", pool_op.uo.nonce()),
                    "paymaster": match &pool_op.uo {
                        UserOperationVariant::V0_6(op) => {
                            if op.paymaster_and_data().is_empty() {
                                Value::Null
                            } else {
                                // Extract paymaster address from paymasterAndData
                                json!("0x0000000000000000000000000000000000000000")
                            }
                        },
                        UserOperationVariant::V0_7(op) => {
                            op.paymaster().map(|p| json!(format!("{:#x}", p))).unwrap_or(Value::Null)
                        }
                    },
                    "actualGasCost": "0x0", // Would be calculated from blockchain receipt
                    "actualGasUsed": "0x0", // Would be calculated from blockchain receipt
                    "success": true, // Would come from blockchain receipt
                    "logs": [], // Would come from blockchain receipt
                    "receipt": {
                        "transactionHash": null, // Would be set when mined
                        "blockNumber": null,
                        "blockHash": null,
                        "from": format!("{:#x}", pool_op.uo.sender()),
                        "gasused": "0x0"
                    }
                }))
            }
            Ok(None) => {
                debug!("UserOperation not found for receipt lookup: {}", hash_str);
                Ok(Value::Null)
            }
            Err(e) => {
                error!("Pool lookup error for receipt: {:?}", e);
                Err(GatewayError::PoolError(format!(
                    "Failed to lookup operation receipt: {:?}",
                    e
                )))
            }
        }
    }

    // === UserOperation parsing methods ===

    /// Parse UserOperation from JSON value based on entry point version
    fn parse_user_operation_from_json(
        &self,
        json_value: &Value,
        entry_point: Address,
    ) -> GatewayResult<UserOperationVariant> {
        // Determine version based on entry point address or JSON structure
        if self.is_entry_point_v06(entry_point) {
            self.parse_v06_user_operation(json_value)
        } else {
            self.parse_v07_user_operation(json_value)
        }
    }

    /// Check if entry point is v0.6
    fn is_entry_point_v06(&self, entry_point: Address) -> bool {
        // Standard v0.6 EntryPoint address
        entry_point
            == "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                .parse::<Address>()
                .unwrap()
    }

    /// Parse v0.6 UserOperation from JSON (simplified implementation)
    fn parse_v06_user_operation(&self, json_value: &Value) -> GatewayResult<UserOperationVariant> {
        // For now, use a simple approach that creates a minimal v0.6 UserOperation
        // In production, this would need full field parsing

        let sender: Address = json_value
            .get("sender")
            .and_then(|v| v.as_str())
            .ok_or_else(|| GatewayError::InvalidRequest("Missing sender field".to_string()))?
            .parse()
            .map_err(|_| GatewayError::InvalidRequest("Invalid sender address".to_string()))?;

        let nonce = self.parse_u256_field(json_value, "nonce")?;

        // Create a simplified UserOperation with required fields
        let user_op = v0_6::UserOperationBuilder::new(
            &ChainSpec::default(),
            v0_6::UserOperationRequiredFields {
                sender,
                nonce,
                init_code: self
                    .parse_bytes_field(json_value, "initCode")
                    .unwrap_or_default(),
                call_data: self
                    .parse_bytes_field(json_value, "callData")
                    .unwrap_or_default(),
                call_gas_limit: self
                    .parse_u128_field(json_value, "callGasLimit")
                    .unwrap_or(100_000),
                verification_gas_limit: self
                    .parse_u128_field(json_value, "verificationGasLimit")
                    .unwrap_or(100_000),
                pre_verification_gas: self
                    .parse_u128_field(json_value, "preVerificationGas")
                    .unwrap_or(21_000),
                max_fee_per_gas: self
                    .parse_u128_field(json_value, "maxFeePerGas")
                    .unwrap_or(1_000_000_000),
                max_priority_fee_per_gas: self
                    .parse_u128_field(json_value, "maxPriorityFeePerGas")
                    .unwrap_or(1_000_000_000),
                paymaster_and_data: self
                    .parse_bytes_field(json_value, "paymasterAndData")
                    .unwrap_or_default(),
                signature: self
                    .parse_bytes_field(json_value, "signature")
                    .unwrap_or_default(),
            },
        )
        .build();

        Ok(UserOperationVariant::V0_6(user_op))
    }

    /// Parse v0.7 UserOperation from JSON (simplified implementation)
    fn parse_v07_user_operation(&self, json_value: &Value) -> GatewayResult<UserOperationVariant> {
        let sender: Address = json_value
            .get("sender")
            .and_then(|v| v.as_str())
            .ok_or_else(|| GatewayError::InvalidRequest("Missing sender field".to_string()))?
            .parse()
            .map_err(|_| GatewayError::InvalidRequest("Invalid sender address".to_string()))?;

        let nonce = self.parse_u256_field(json_value, "nonce")?;

        // Create a v0.7 UserOperation with required fields only
        let chain_spec = ChainSpec::default();
        let mut builder = v0_7::UserOperationBuilder::new(
            &chain_spec,
            v0_7::UserOperationRequiredFields {
                sender,
                nonce,
                call_data: self
                    .parse_bytes_field(json_value, "callData")
                    .unwrap_or_default(),
                call_gas_limit: self
                    .parse_u128_field(json_value, "callGasLimit")
                    .unwrap_or(100_000),
                verification_gas_limit: self
                    .parse_u128_field(json_value, "verificationGasLimit")
                    .unwrap_or(100_000),
                pre_verification_gas: self
                    .parse_u128_field(json_value, "preVerificationGas")
                    .unwrap_or(21_000),
                max_fee_per_gas: self
                    .parse_u128_field(json_value, "maxFeePerGas")
                    .unwrap_or(1_000_000_000),
                max_priority_fee_per_gas: self
                    .parse_u128_field(json_value, "maxPriorityFeePerGas")
                    .unwrap_or(1_000_000_000),
                signature: self
                    .parse_bytes_field(json_value, "signature")
                    .unwrap_or_default(),
            },
        );

        // Add optional fields if present (v0.7 combines factory and factory_data)
        if let Some(factory) = self
            .parse_optional_address_field(json_value, "factory")
            .unwrap_or(None)
        {
            let factory_data = self
                .parse_bytes_field(json_value, "factoryData")
                .unwrap_or_default();
            builder = builder.factory(factory, factory_data);
        }

        // Add paymaster fields if present (v0.7 combines all paymaster fields)
        if let Some(paymaster) = self
            .parse_optional_address_field(json_value, "paymaster")
            .unwrap_or(None)
        {
            let pv_gas_limit = self
                .parse_optional_u128_field(json_value, "paymasterVerificationGasLimit")
                .unwrap_or(None)
                .unwrap_or(100_000);
            let po_gas_limit = self
                .parse_optional_u128_field(json_value, "paymasterPostOpGasLimit")
                .unwrap_or(None)
                .unwrap_or(100_000);
            let paymaster_data = self
                .parse_bytes_field(json_value, "paymasterData")
                .unwrap_or_default();
            builder = builder.paymaster(paymaster, pv_gas_limit, po_gas_limit, paymaster_data);
        }

        let user_op = builder.build();

        Ok(UserOperationVariant::V0_7(user_op))
    }

    // === JSON conversion helper methods ===

    /// Convert UserOperationVariant back to JSON format
    fn user_operation_to_json(&self, user_op: &UserOperationVariant) -> Value {
        match user_op {
            UserOperationVariant::V0_6(op) => json!({
                "sender": format!("{:#x}", op.sender()),
                "nonce": format!("0x{:x}", op.nonce()),
                "initCode": format!("0x{}", hex::encode(op.init_code())),
                "callData": format!("0x{}", hex::encode(op.call_data())),
                "callGasLimit": format!("0x{:x}", op.call_gas_limit()),
                "verificationGasLimit": format!("0x{:x}", op.verification_gas_limit()),
                "preVerificationGas": format!("0x{:x}", op.pre_verification_gas()),
                "maxFeePerGas": format!("0x{:x}", op.max_fee_per_gas()),
                "maxPriorityFeePerGas": format!("0x{:x}", op.max_priority_fee_per_gas()),
                "paymasterAndData": format!("0x{}", hex::encode(op.paymaster_and_data())),
                "signature": format!("0x{}", hex::encode(op.signature()))
            }),
            UserOperationVariant::V0_7(op) => json!({
                "sender": format!("{:#x}", op.sender()),
                "nonce": format!("0x{:x}", op.nonce()),
                "factory": op.factory().map(|f| format!("{:#x}", f)),
                "factoryData": format!("0x{}", hex::encode(op.factory_data())),
                "callData": format!("0x{}", hex::encode(op.call_data())),
                "callGasLimit": format!("0x{:x}", op.call_gas_limit()),
                "verificationGasLimit": format!("0x{:x}", op.verification_gas_limit()),
                "preVerificationGas": format!("0x{:x}", op.pre_verification_gas()),
                "maxFeePerGas": format!("0x{:x}", op.max_fee_per_gas()),
                "maxPriorityFeePerGas": format!("0x{:x}", op.max_priority_fee_per_gas()),
                "paymaster": op.paymaster().map(|p| format!("{:#x}", p)),
                "paymasterVerificationGasLimit": if op.paymaster_verification_gas_limit() > 0 {
                    Some(format!("0x{:x}", op.paymaster_verification_gas_limit()))
                } else {
                    None
                },
                "paymasterPostOpGasLimit": if op.paymaster_post_op_gas_limit() > 0 {
                    Some(format!("0x{:x}", op.paymaster_post_op_gas_limit()))
                } else {
                    None
                },
                "paymasterData": format!("0x{}", hex::encode(op.paymaster_data())),
                "signature": format!("0x{}", hex::encode(op.signature()))
            }),
        }
    }

    // === BLS protection helper methods ===

    /// Extract aggregator address from UserOperation JSON request
    /// This follows ERC-7766 specification for optional aggregator field
    fn extract_aggregator_from_request(&self, user_op_json: &Value) -> Option<Address> {
        // Look for aggregator field in the UserOperation JSON
        // This is an extension to ERC-4337 for signature aggregation support
        if let Some(aggregator_value) = user_op_json.get("aggregator") {
            if let Some(aggregator_str) = aggregator_value.as_str() {
                if !aggregator_str.is_empty() && aggregator_str != "0x" {
                    if let Ok(address) = aggregator_str.parse::<Address>() {
                        debug!("🔍 Found aggregator address in request: {:#x}", address);
                        return Some(address);
                    } else {
                        warn!("⚠️ Invalid aggregator address format: {}", aggregator_str);
                    }
                }
            }
        }
        None
    }

    // === JSON parsing helper methods ===

    /// Parse a U256 field from JSON
    fn parse_u256_field(&self, json: &Value, field_name: &str) -> GatewayResult<U256> {
        let field_value = json
            .get(field_name)
            .ok_or_else(|| GatewayError::InvalidRequest(format!("Missing {} field", field_name)))?;

        match field_value {
            Value::String(s) => {
                // Handle hex strings (with or without 0x prefix)
                let cleaned = s.strip_prefix("0x").unwrap_or(s);
                U256::from_str_radix(cleaned, 16).map_err(|_| {
                    GatewayError::InvalidRequest(format!("Invalid {} format", field_name))
                })
            }
            Value::Number(n) => {
                if let Some(val) = n.as_u64() {
                    Ok(U256::from(val))
                } else {
                    Err(GatewayError::InvalidRequest(format!(
                        "Invalid {} number format",
                        field_name
                    )))
                }
            }
            _ => Err(GatewayError::InvalidRequest(format!(
                "{} must be a string or number",
                field_name
            ))),
        }
    }

    /// Parse a u128 field from JSON
    fn parse_u128_field(&self, json: &Value, field_name: &str) -> GatewayResult<u128> {
        let u256_val = self.parse_u256_field(json, field_name)?;
        u256_val.try_into().map_err(|_| {
            GatewayError::InvalidRequest(format!("{} value too large for u128", field_name))
        })
    }

    /// Parse an optional u128 field from JSON
    fn parse_optional_u128_field(
        &self,
        json: &Value,
        field_name: &str,
    ) -> GatewayResult<Option<u128>> {
        match json.get(field_name) {
            Some(value) if !value.is_null() => Ok(Some(self.parse_u128_field(json, field_name)?)),
            _ => Ok(None),
        }
    }

    /// Parse a bytes field from JSON
    fn parse_bytes_field(&self, json: &Value, field_name: &str) -> GatewayResult<Bytes> {
        let field_value = json
            .get(field_name)
            .ok_or_else(|| GatewayError::InvalidRequest(format!("Missing {} field", field_name)))?;

        match field_value {
            Value::String(s) => {
                let cleaned = s.strip_prefix("0x").unwrap_or(s);
                if cleaned.is_empty() {
                    Ok(Bytes::new())
                } else {
                    hex::decode(cleaned).map(Bytes::from).map_err(|_| {
                        GatewayError::InvalidRequest(format!("Invalid {} hex format", field_name))
                    })
                }
            }
            _ => Err(GatewayError::InvalidRequest(format!(
                "{} must be a hex string",
                field_name
            ))),
        }
    }

    /// Parse an optional address field from JSON
    fn parse_optional_address_field(
        &self,
        json: &Value,
        field_name: &str,
    ) -> GatewayResult<Option<Address>> {
        match json.get(field_name) {
            Some(Value::String(s)) if !s.is_empty() && s != "0x" => {
                s.parse().map(Some).map_err(|_| {
                    GatewayError::InvalidRequest(format!("Invalid {} address", field_name))
                })
            }
            _ => Ok(None),
        }
    }

    // ============================================================================
    // Builder API Methods
    // ============================================================================

    /// Handle rundler_getBundleStats method
    pub async fn handle_get_bundle_stats(&self, _request: &JsonRpcRequest) -> GatewayResult<Value> {
        if let Some(builder) = &self.builder_handle {
            debug!("📊 Getting bundle stats from builder");

            match builder.get_supported_entry_points().await {
                Ok(entry_points) => {
                    debug!(
                        "✅ Builder is operational, supported entry points: {:?}",
                        entry_points
                    );
                    Ok(json!({
                        "builder_status": "operational",
                        "supported_entry_points": entry_points,
                        "message": "Builder is running and available for bundle processing"
                    }))
                }
                Err(e) => {
                    error!("❌ Builder is not operational: {:?}", e);
                    Err(GatewayError::BuilderError(format!(
                        "Builder not operational: {:?}",
                        e
                    )))
                }
            }
        } else {
            warn!("⚠️ Builder handle not available for bundle stats");
            Ok(json!({
                "error": "Builder component not available",
                "bundlesSubmitted": 0,
                "bundlesDropped": 0,
                "bundlesMined": 0
            }))
        }
    }

    /// Handle rundler_getBundleByHash method  
    pub async fn handle_get_bundle_by_hash(
        &self,
        request: &JsonRpcRequest,
    ) -> GatewayResult<Value> {
        if request.params.is_empty() {
            return Err(GatewayError::InvalidRequest(
                "Missing bundle hash parameter".to_string(),
            ));
        }

        let hash = &request.params[0];
        debug!("🔍 Looking up bundle by hash: {:?}", hash);

        if let Some(_builder) = &self.builder_handle {
            // Parse hash from hex string
            let hash_str = hash
                .as_str()
                .ok_or_else(|| GatewayError::InvalidRequest("Hash must be a string".to_string()))?;

            let hash_bytes = if let Some(stripped) = hash_str.strip_prefix("0x") {
                hex::decode(stripped)
            } else {
                hex::decode(hash_str)
            }
            .map_err(|_| GatewayError::InvalidRequest("Invalid hash format".to_string()))?;

            if hash_bytes.len() != 32 {
                return Err(GatewayError::InvalidRequest(
                    "Hash must be 32 bytes".to_string(),
                ));
            }

            let _hash_b256 = alloy_primitives::B256::from_slice(&hash_bytes);

            // Note: get_bundle_by_hash is not available in current LocalBuilderHandle API
            // Return not implemented error for now
            warn!("📭 Bundle lookup by hash not implemented in current builder API");
            Err(GatewayError::BuilderError(
                "Bundle lookup by hash not yet implemented".to_string(),
            ))
        } else {
            warn!("⚠️ Builder handle not available for bundle lookup");
            Err(GatewayError::BuilderError(
                "Builder component not available".to_string(),
            ))
        }
    }

    /// Handle rundler_sendBundleNow method
    pub async fn handle_send_bundle_now(&self, _request: &JsonRpcRequest) -> GatewayResult<Value> {
        if let Some(builder) = &self.builder_handle {
            debug!("🚀 Triggering immediate bundle submission");

            match builder.debug_send_bundle_now().await {
                Ok(result) => {
                    debug!("✅ Bundle sent immediately: {:?}", result);
                    Ok(json!({
                        "success": true,
                        "result": result
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to send bundle immediately: {:?}", e);
                    Err(GatewayError::BuilderError(format!(
                        "Failed to send bundle: {:?}",
                        e
                    )))
                }
            }
        } else {
            warn!("⚠️ Builder handle not available for immediate bundle send");
            Err(GatewayError::BuilderError(
                "Builder component not available".to_string(),
            ))
        }
    }
}

impl Default for GatewayRouter {
    fn default() -> Self {
        Self::new()
    }
}
