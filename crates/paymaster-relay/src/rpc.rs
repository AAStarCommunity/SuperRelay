//! RPC API definitions for SuperRelay Paymaster Service

use alloy_primitives::{Address, B256};
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use rundler_types::{
    v0_6::UserOperation as UserOperationV0_6, v0_7::UserOperation as UserOperationV0_7,
    UserOperation,
};
use serde::{Deserialize, Serialize};
use serde_json;

/// Paymaster Relay API trait
///
/// This trait defines the RPC interface for the SuperRelay paymaster service.
/// All methods use the "pm_" namespace to avoid conflicts with standard ERC-4337 methods.
#[rpc(server, namespace = "pm")]
pub trait PaymasterRelayApi {
    /// Sponsor a UserOperation by validating it against policies, signing it, and submitting to mempool
    ///
    /// This is the core paymaster functionality that:
    /// 1. Validates the UserOperation against configured policies
    /// 2. Signs the UserOperation as paymaster
    /// 3. Submits it to the Rundler mempool via eth_sendUserOperation
    /// 4. Returns the UserOperation hash if successful
    ///
    /// # Arguments
    /// * `user_op` - The UserOperation to sponsor (supports both v0.6 and v0.7)
    /// * `entry_point` - The EntryPoint contract address
    ///
    /// # Returns
    /// * `UserOpHash` - Hash of the sponsored UserOperation
    #[method(name = "sponsorUserOperation")]
    async fn sponsor_user_operation(
        &self,
        user_op: serde_json::Value, // Temporary workaround for serialization
        entry_point: Address,
    ) -> RpcResult<UserOpHash>;

    /// Get supported EntryPoint addresses
    ///
    /// Returns a list of EntryPoint contract addresses that this paymaster supports.
    ///
    /// # Returns
    /// * `Vec<Address>` - List of supported EntryPoint addresses
    #[method(name = "getSupportedEntryPoints")]
    async fn get_supported_entry_points(&self) -> RpcResult<Vec<Address>>;

    /// Get current chain ID
    ///
    /// Returns the chain ID that this paymaster service is configured for.
    ///
    /// # Returns
    /// * `u64` - Chain ID
    #[method(name = "getChainId")]
    async fn get_chain_id(&self) -> RpcResult<u64>;

    /// Get service statistics
    ///
    /// Returns usage statistics for the paymaster service including:
    /// - Total sponsored transactions
    /// - Total gas sponsored
    /// - Number of unique users
    ///
    /// # Returns
    /// * `PaymasterStatistics` - Service usage statistics
    #[method(name = "getStatistics")]
    async fn get_statistics(&self) -> RpcResult<PaymasterStatistics>;

    /// Health check endpoint
    ///
    /// Returns service health status. Used for monitoring and load balancer health checks.
    ///
    /// # Returns
    /// * `String` - "ok" if service is healthy
    #[method(name = "health")]
    async fn health(&self) -> RpcResult<String>;

    /// Get paymaster policy information
    ///
    /// Returns the current policy configuration for the given sender address.
    /// Useful for clients to understand sponsorship eligibility.
    ///
    /// # Arguments
    /// * `sender` - The sender address to check policy for
    ///
    /// # Returns
    /// * `PolicyInfo` - Policy information for the sender
    #[method(name = "getPolicyInfo")]
    async fn get_policy_info(&self, sender: Address) -> RpcResult<PolicyInfo>;
}

/// UserOperation request wrapper that supports both v0.6 and v0.7
#[derive(Debug, Clone)]
pub enum UserOperationRequest {
    /// ERC-4337 v0.6 UserOperation
    V0_6(Box<UserOperationV0_6>),
    /// ERC-4337 v0.7 UserOperation  
    V0_7(Box<UserOperationV0_7>),
}

/// UserOperation hash response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOpHash {
    /// The hash of the UserOperation
    pub hash: B256,
    /// The EntryPoint version used
    pub version: String,
}

/// Paymaster service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymasterStatistics {
    /// Total number of sponsored transactions
    pub total_transactions: u64,
    /// Total number of unique users served
    pub unique_users: u64,
    /// Total gas sponsored (in wei)
    pub total_gas_sponsored: String,
    /// Service uptime in seconds
    pub uptime_seconds: u64,
    /// Current service version
    pub version: String,
    /// Statistics by EntryPoint version
    pub by_entry_point: std::collections::HashMap<String, EntryPointStatistics>,
}

/// Statistics for a specific EntryPoint version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPointStatistics {
    /// Number of sponsored transactions for this EntryPoint
    pub transactions: u64,
    /// Total gas sponsored for this EntryPoint (in wei)
    pub gas_sponsored: String,
    /// Average gas per transaction
    pub avg_gas_per_tx: String,
}

/// Policy information for a sender
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyInfo {
    /// Whether the sender is allowed by policy
    pub allowed: bool,
    /// Policy type applied (allowlist, denylist, etc.)
    pub policy_type: String,
    /// Maximum gas limit allowed for this sender
    pub max_gas_limit: Option<u64>,
    /// Rate limit information
    pub rate_limit: Option<RateLimit>,
    /// Policy details/reason
    pub details: String,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window in seconds
    pub window_seconds: u32,
    /// Current usage in the window
    pub current_usage: u32,
    /// Time until window resets (seconds)
    pub reset_time: u32,
}

impl UserOperationRequest {
    /// Get the sender address from the UserOperation
    pub fn sender(&self) -> Address {
        match self {
            UserOperationRequest::V0_6(uo) => uo.sender(),
            UserOperationRequest::V0_7(uo) => uo.sender(),
        }
    }

    /// Get the nonce from the UserOperation
    pub fn nonce(&self) -> alloy_primitives::U256 {
        match self {
            UserOperationRequest::V0_6(uo) => uo.nonce(),
            UserOperationRequest::V0_7(uo) => uo.nonce(),
        }
    }

    /// Get the call gas limit from the UserOperation
    pub fn call_gas_limit(&self) -> alloy_primitives::U256 {
        match self {
            UserOperationRequest::V0_6(uo) => alloy_primitives::U256::from(uo.call_gas_limit()),
            UserOperationRequest::V0_7(uo) => alloy_primitives::U256::from(uo.call_gas_limit()),
        }
    }

    /// Get the verification gas limit from the UserOperation
    pub fn verification_gas_limit(&self) -> alloy_primitives::U256 {
        match self {
            UserOperationRequest::V0_6(uo) => {
                alloy_primitives::U256::from(uo.verification_gas_limit())
            }
            UserOperationRequest::V0_7(uo) => {
                alloy_primitives::U256::from(uo.verification_gas_limit())
            }
        }
    }

    /// Check if this is a v0.6 UserOperation
    pub fn is_v0_6(&self) -> bool {
        matches!(self, UserOperationRequest::V0_6(_))
    }

    /// Check if this is a v0.7 UserOperation
    pub fn is_v0_7(&self) -> bool {
        matches!(self, UserOperationRequest::V0_7(_))
    }
}

impl std::fmt::Display for UserOperationRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserOperationRequest::V0_6(_) => write!(f, "UserOperation v0.6"),
            UserOperationRequest::V0_7(_) => write!(f, "UserOperation v0.7"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // Note: UserOperation construction tests are commented out due to
    // rundler_types::UserOperation being non-exhaustive. Integration tests
    // will be used instead for full end-to-end testing.

    #[test]
    fn test_user_op_hash_serialization() {
        let hash = UserOpHash {
            hash: B256::ZERO,
            version: "v0.6".to_string(),
        };

        let json = serde_json::to_string(&hash).unwrap();
        let deserialized: UserOpHash = serde_json::from_str(&json).unwrap();
        assert_eq!(hash.hash, deserialized.hash);
        assert_eq!(hash.version, deserialized.version);
    }

    #[test]
    fn test_policy_info_creation() {
        let policy_info = PolicyInfo {
            allowed: true,
            policy_type: "allowlist".to_string(),
            max_gas_limit: Some(1_000_000),
            rate_limit: Some(RateLimit {
                max_requests: 10,
                window_seconds: 60,
                current_usage: 3,
                reset_time: 45,
            }),
            details: "Sender is on allowlist".to_string(),
        };

        assert!(policy_info.allowed);
        assert!(policy_info.rate_limit.is_some());
    }
}
