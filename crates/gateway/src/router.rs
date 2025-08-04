use std::sync::Arc;

use alloy_primitives::{Address, Bytes, U256};
use ethers::types::H160;
use rundler_paymaster_relay::PaymasterRelayService;
use rundler_pool::LocalPoolHandle;
use rundler_types::{chain::ChainSpec, v0_6, v0_7, UserOperation, UserOperationVariant};
use serde_json::{json, Value};
use tracing::{debug, error, warn};

use crate::{
    error::{GatewayError, GatewayResult},
    gateway::JsonRpcRequest,
};

/// Router that handles request routing to appropriate rundler components
#[derive(Clone)]
pub struct GatewayRouter {
    /// Supported EntryPoint addresses
    supported_entry_points: Vec<Address>,
    /// Pool handle for mempool operations
    pool_handle: Option<Arc<LocalPoolHandle>>,
    /// Chain ID for this network
    chain_id: u64,
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
            chain_id: 31337, // Anvil default
        }
    }

    /// Create a new router with rundler components
    pub fn with_rundler_components(
        pool_handle: Arc<LocalPoolHandle>,
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
            chain_id,
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
            chain_id: if config.chain_id == 0 {
                31337
            } else {
                config.chain_id
            },
        }
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

        // 4. Call paymaster service for sponsorship
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

        // TODO: Parse UserOperation from JSON and call pool.add_op()
        // let user_op_variant = self.parse_user_operation(user_op)?;
        // let user_op_hash = pool.add_op(user_op_variant, ...).await?;

        // For now, generate a deterministic mock hash
        let user_op_str = serde_json::to_string(user_op).map_err(|_| {
            GatewayError::InvalidRequest("Invalid UserOperation format".to_string())
        })?;

        // Simple hash generation (not cryptographically secure, just for testing)
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };
        let mut hasher = DefaultHasher::new();
        user_op_str.hash(&mut hasher);
        let hash_u64 = hasher.finish();
        let mock_hash = format!("0x{:016x}{:016x}", hash_u64, hash_u64);

        debug!("Generated UserOperation hash: {}", mock_hash);
        Ok(json!(mock_hash))
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

        // TODO: Use actual pool.get_user_operation_by_hash() method
        // For now, return null (not found)
        Ok(Value::Null)
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

        // TODO: Use actual pool.get_user_operation_receipt() method
        // For now, return null (not found)
        Ok(Value::Null)
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
}

impl Default for GatewayRouter {
    fn default() -> Self {
        Self::new()
    }
}
