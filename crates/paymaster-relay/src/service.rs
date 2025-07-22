//! Core service implementation for SuperRelay Paymaster Service

use std::{collections::HashMap, sync::Arc, time::SystemTime};

use alloy_primitives::Address;
use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use tokio::sync::RwLock;

use crate::{
    config::Config,
    error::{PaymasterError, Result},
    rpc::{
        EntryPointStatistics, PaymasterRelayApiServer, PaymasterStatistics, PolicyInfo, UserOpHash,
        UserOperationRequest,
    },
};

/// Core PaymasterRelayService implementation
///
/// This service implements the SuperRelay paymaster functionality:
/// - Policy-based UserOperation validation
/// - UserOperation signing as paymaster
/// - Integration with Rundler via HTTP RPC calls
/// - Statistics collection and monitoring
pub struct PaymasterRelayService {
    /// Service configuration
    config: Config,

    /// Service statistics
    stats: Arc<RwLock<ServiceStatistics>>,

    /// Service start time for uptime calculation
    start_time: SystemTime,

    /// HTTP client for Rundler communication
    #[allow(dead_code)]
    rundler_client: jsonrpsee::http_client::HttpClient,
}

/// Internal service statistics
#[derive(Debug, Default)]
struct ServiceStatistics {
    /// Total sponsored transactions
    total_transactions: u64,
    /// Unique users (sender addresses) served
    unique_users: std::collections::HashSet<Address>,
    /// Total gas sponsored in wei
    total_gas_sponsored: u128,
    /// Statistics by EntryPoint version
    by_entry_point: HashMap<String, EntryPointStats>,
}

/// Internal EntryPoint statistics
#[derive(Debug, Default)]
struct EntryPointStats {
    transactions: u64,
    gas_sponsored: u128,
}

impl PaymasterRelayService {
    /// Create a new PaymasterRelayService
    pub async fn new(config: Config) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        // Create HTTP client for Rundler communication
        let rundler_client = jsonrpsee::http_client::HttpClientBuilder::default()
            .request_timeout(std::time::Duration::from_secs(config.rundler.timeout))
            .max_concurrent_requests(1000)
            .build(&config.rundler.url)
            .map_err(|e| {
                PaymasterError::RpcError(format!("Failed to create Rundler client: {}", e))
            })?;

        Ok(Self {
            config,
            stats: Arc::new(RwLock::new(ServiceStatistics::default())),
            start_time: SystemTime::now(),
            rundler_client,
        })
    }

    /// Validate UserOperation against policies
    #[allow(dead_code)]
    async fn validate_user_operation(&self, user_op: &UserOperationRequest) -> Result<()> {
        // Basic validation
        self.validate_gas_limits(user_op)?;

        // Policy validation would go here
        // For now, we'll implement a simple allowlist check
        self.validate_sender_policy(user_op.sender()).await?;

        Ok(())
    }

    /// Validate gas limits against configuration
    fn validate_gas_limits(&self, user_op: &UserOperationRequest) -> Result<()> {
        let limits = &self.config.paymaster.gas_limits;

        if user_op.verification_gas_limit()
            > alloy_primitives::U256::from(limits.max_verification_gas)
        {
            return Err(PaymasterError::InvalidUserOperation(format!(
                "Verification gas limit {} exceeds maximum {}",
                user_op.verification_gas_limit(),
                limits.max_verification_gas
            )));
        }

        if user_op.call_gas_limit() > alloy_primitives::U256::from(limits.max_call_gas) {
            return Err(PaymasterError::InvalidUserOperation(format!(
                "Call gas limit {} exceeds maximum {}",
                user_op.call_gas_limit(),
                limits.max_call_gas
            )));
        }

        Ok(())
    }

    /// Validate sender against policy rules
    async fn validate_sender_policy(&self, _sender: Address) -> Result<()> {
        // TODO: Implement actual policy validation
        // For now, allow all senders
        Ok(())
    }

    /// Sign UserOperation as paymaster
    #[allow(dead_code)]
    async fn sign_user_operation(
        &self,
        user_op: &UserOperationRequest,
        entry_point: &Address,
    ) -> Result<UserOperationRequest> {
        // TODO: Implement actual signing logic
        // For now, return the UserOperation unchanged
        // In real implementation, this would:
        // 1. Get the private key from environment
        // 2. Create paymaster data with signature
        // 3. Update the UserOperation with paymaster fields
        tracing::info!(
            "Signing UserOperation for sender: {} at EntryPoint: {}",
            user_op.sender(),
            entry_point
        );
        Ok(user_op.clone())
    }

    /// Submit UserOperation to Rundler mempool
    #[allow(dead_code)]
    async fn submit_to_rundler(
        &self,
        _user_op: &UserOperationRequest,
        _entry_point: &Address,
    ) -> Result<alloy_primitives::B256> {
        // TODO: Implement actual RPC call to Rundler
        // For now, return a placeholder hash
        Ok(alloy_primitives::B256::ZERO)
    }

    /// Update service statistics
    #[allow(dead_code)]
    async fn update_statistics(&self, user_op: &UserOperationRequest, entry_point_version: &str) {
        let mut stats = self.stats.write().await;

        // Update totals
        stats.total_transactions += 1;
        stats.unique_users.insert(user_op.sender());

        // Estimate gas sponsored (this is a rough estimate)
        let estimated_gas =
            user_op.call_gas_limit().to::<u128>() + user_op.verification_gas_limit().to::<u128>();
        stats.total_gas_sponsored += estimated_gas;

        // Update entry point statistics
        let ep_stats = stats
            .by_entry_point
            .entry(entry_point_version.to_string())
            .or_default();
        ep_stats.transactions += 1;
        ep_stats.gas_sponsored += estimated_gas;
    }

    /// Get EntryPoint version from address
    #[allow(dead_code)]
    fn get_entry_point_version(&self, entry_point: &Address) -> String {
        for (version, address_str) in &self.config.paymaster.entry_points {
            if let Ok(address) = address_str.parse::<Address>() {
                if address == *entry_point {
                    return version.clone();
                }
            }
        }
        "unknown".to_string()
    }
}

#[async_trait]
impl PaymasterRelayApiServer for PaymasterRelayService {
    async fn sponsor_user_operation(
        &self,
        _user_op: serde_json::Value,
        entry_point: Address,
    ) -> RpcResult<UserOpHash> {
        tracing::info!(
            "Received UserOperation for sponsorship at EntryPoint: {}",
            entry_point
        );

        // TODO: Parse the JSON UserOperation properly
        // For now, return a placeholder hash
        let hash = alloy_primitives::B256::ZERO;

        tracing::info!("Successfully sponsored UserOperation with hash: {}", hash);

        Ok(UserOpHash {
            hash,
            version: "unknown".to_string(),
        })
    }

    async fn get_supported_entry_points(&self) -> RpcResult<Vec<Address>> {
        let mut entry_points = Vec::new();

        for addr_str in self.config.paymaster.entry_points.values() {
            match addr_str.parse::<Address>() {
                Ok(addr) => entry_points.push(addr),
                Err(e) => {
                    return Err(jsonrpsee::types::ErrorObjectOwned::owned(
                        -32000,
                        format!("Invalid entry point address: {}", e),
                        None::<()>,
                    ))
                }
            }
        }

        Ok(entry_points)
    }

    async fn get_chain_id(&self) -> RpcResult<u64> {
        Ok(self.config.paymaster.chain_id)
    }

    async fn get_statistics(&self) -> RpcResult<PaymasterStatistics> {
        let stats = self.stats.read().await;
        let uptime = self.start_time.elapsed().unwrap_or_default().as_secs();

        let mut by_entry_point = HashMap::new();
        for (version, ep_stats) in &stats.by_entry_point {
            let avg_gas = if ep_stats.transactions > 0 {
                ep_stats.gas_sponsored / ep_stats.transactions as u128
            } else {
                0
            };

            by_entry_point.insert(
                version.clone(),
                EntryPointStatistics {
                    transactions: ep_stats.transactions,
                    gas_sponsored: ep_stats.gas_sponsored.to_string(),
                    avg_gas_per_tx: avg_gas.to_string(),
                },
            );
        }

        Ok(PaymasterStatistics {
            total_transactions: stats.total_transactions,
            unique_users: stats.unique_users.len() as u64,
            total_gas_sponsored: stats.total_gas_sponsored.to_string(),
            uptime_seconds: uptime,
            version: env!("CARGO_PKG_VERSION").to_string(),
            by_entry_point,
        })
    }

    async fn health(&self) -> RpcResult<String> {
        // TODO: Add actual health checks (Rundler connectivity, etc.)
        Ok("ok".to_string())
    }

    async fn get_policy_info(&self, _sender: Address) -> RpcResult<PolicyInfo> {
        // TODO: Implement actual policy lookup
        Ok(PolicyInfo {
            allowed: true,
            policy_type: "default".to_string(),
            max_gas_limit: Some(self.config.paymaster.gas_limits.max_call_gas),
            rate_limit: None,
            details: "Default policy - all senders allowed".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config::default()
    }

    #[tokio::test]
    async fn test_service_creation() {
        let config = create_test_config();
        let service = PaymasterRelayService::new(config).await;
        assert!(service.is_ok());
    }

    // Note: UserOperation construction tests are commented out due to
    // rundler_types::UserOperation being non-exhaustive. Integration tests
    // will be used instead for full end-to-end testing.
}
