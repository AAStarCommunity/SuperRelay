//! Policy engine for SuperRelay Paymaster Service

use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, SystemTime},
};

use alloy_primitives::Address;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    config::PolicyAction,
    error::{ConfigError, Result},
    rpc::UserOperationRequest,
};

/// Policy engine for validating UserOperations
pub struct PolicyEngine {
    /// Current policy rules
    rules: RwLock<PolicyRules>,
    /// Rate limiting state
    rate_limiter: RwLock<HashMap<Address, RateLimit>>,
    /// Path to policy configuration file
    config_path: String,
    /// Last configuration reload time
    last_reload: RwLock<SystemTime>,
    /// Reload interval
    reload_interval: Duration,
}

/// Policy rules loaded from configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolicyRules {
    /// Default action for undefined rules
    #[serde(default)]
    pub default_action: PolicyAction,

    /// Global settings
    #[serde(default)]
    pub global: GlobalPolicy,

    /// Per-address policy rules
    #[serde(default)]
    pub addresses: HashMap<String, AddressPolicy>,

    /// Contract interaction policies
    #[serde(default)]
    pub contracts: HashMap<String, ContractPolicy>,
}

/// Global policy settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalPolicy {
    /// Maximum gas limit across all operations
    pub max_gas_limit: Option<u64>,

    /// Maximum verification gas
    pub max_verification_gas: Option<u64>,

    /// Maximum call gas
    pub max_call_gas: Option<u64>,

    /// Global rate limit (requests per minute)
    pub rate_limit_per_minute: Option<u32>,

    /// Allowed sender addresses (if specified, only these are allowed)
    pub allowed_senders: Option<Vec<String>>,

    /// Denied sender addresses
    pub denied_senders: Option<Vec<String>>,
}

/// Policy for a specific address
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddressPolicy {
    /// Action for this address
    pub action: PolicyAction,

    /// Maximum gas limit for this address
    pub max_gas_limit: Option<u64>,

    /// Rate limit for this address (requests per minute)
    pub rate_limit_per_minute: Option<u32>,

    /// Allowed target contracts for this address
    pub allowed_targets: Option<Vec<String>>,

    /// Custom validation rules
    pub custom_rules: Option<HashMap<String, String>>,
}

/// Policy for contract interactions
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContractPolicy {
    /// Action for interactions with this contract
    pub action: PolicyAction,

    /// Maximum gas for interactions with this contract
    pub max_gas_limit: Option<u64>,

    /// Allowed function selectors
    pub allowed_functions: Option<Vec<String>>,
}

/// Rate limiting state for an address
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Number of requests in current window
    pub requests: u32,
    /// Window start time
    pub window_start: SystemTime,
    /// Requests per minute limit
    pub limit: u32,
}

/// Policy validation result
#[derive(Debug, Clone)]
pub struct PolicyResult {
    /// Whether the operation is allowed
    pub allowed: bool,
    /// Reason for the decision
    pub reason: String,
    /// Applied policy type
    pub policy_type: String,
    /// Gas limit override (if any)
    pub gas_limit_override: Option<u64>,
}

impl PolicyEngine {
    /// Create a new PolicyEngine
    pub async fn new<P: AsRef<Path>>(config_path: P, reload_interval: Duration) -> Result<Self> {
        let config_path_str = config_path.as_ref().to_string_lossy().to_string();

        // Load initial policy rules
        let rules = Self::load_policy_rules(&config_path_str).await?;

        Ok(Self {
            rules: RwLock::new(rules),
            rate_limiter: RwLock::new(HashMap::new()),
            config_path: config_path_str,
            last_reload: RwLock::new(SystemTime::now()),
            reload_interval,
        })
    }

    /// Load policy rules from file
    async fn load_policy_rules(path: &str) -> Result<PolicyRules> {
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            ConfigError::ParseError(format!("Failed to read policy file {}: {}", path, e))
        })?;

        let rules: PolicyRules = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse policy TOML: {}", e)))?;

        Ok(rules)
    }

    /// Reload policy rules if needed
    pub async fn maybe_reload_policies(&self) -> Result<()> {
        let last_reload = *self.last_reload.read().await;
        let now = SystemTime::now();

        if now.duration_since(last_reload).unwrap_or_default() >= self.reload_interval {
            tracing::info!("Reloading policy configuration from {}", self.config_path);

            match Self::load_policy_rules(&self.config_path).await {
                Ok(new_rules) => {
                    *self.rules.write().await = new_rules;
                    *self.last_reload.write().await = now;
                    tracing::info!("Policy configuration reloaded successfully");
                }
                Err(e) => {
                    tracing::warn!("Failed to reload policy configuration: {}", e);
                    // Continue with existing rules
                }
            }
        }

        Ok(())
    }

    /// Validate a UserOperation against policies
    pub async fn validate_user_operation(
        &self,
        user_op: &UserOperationRequest,
        _entry_point: Address,
    ) -> Result<PolicyResult> {
        // Maybe reload policies
        self.maybe_reload_policies().await?;

        let rules = self.rules.read().await;
        let sender = user_op.sender();

        // Check rate limits
        if let Err(reason) = self.check_rate_limit(sender, &rules).await {
            return Ok(PolicyResult {
                allowed: false,
                reason,
                policy_type: "rate_limit".to_string(),
                gas_limit_override: None,
            });
        }

        // Check global denied senders
        if let Some(denied_senders) = &rules.global.denied_senders {
            if Self::address_in_list(sender, denied_senders) {
                return Ok(PolicyResult {
                    allowed: false,
                    reason: "Sender is on denied list".to_string(),
                    policy_type: "global_deny".to_string(),
                    gas_limit_override: None,
                });
            }
        }

        // Check global allowed senders (if specified, only these are allowed)
        if let Some(allowed_senders) = &rules.global.allowed_senders {
            if !Self::address_in_list(sender, allowed_senders) {
                return Ok(PolicyResult {
                    allowed: false,
                    reason: "Sender is not on allowed list".to_string(),
                    policy_type: "global_allowlist".to_string(),
                    gas_limit_override: None,
                });
            }
        }

        // Check address-specific policy
        if let Some(address_policy) = rules.addresses.get(&sender.to_string().to_lowercase()) {
            return Ok(self.apply_address_policy(user_op, address_policy));
        }

        // Check global gas limits
        if let Err(reason) = self.check_global_gas_limits(user_op, &rules) {
            return Ok(PolicyResult {
                allowed: false,
                reason,
                policy_type: "gas_limit".to_string(),
                gas_limit_override: None,
            });
        }

        // Apply default action
        Ok(PolicyResult {
            allowed: matches!(rules.default_action, PolicyAction::Allow),
            reason: format!("Default policy action: {:?}", rules.default_action),
            policy_type: "default".to_string(),
            gas_limit_override: None,
        })
    }

    /// Check rate limit for a sender
    async fn check_rate_limit(
        &self,
        sender: Address,
        rules: &PolicyRules,
    ) -> std::result::Result<(), String> {
        let mut rate_limiter = self.rate_limiter.write().await;
        let now = SystemTime::now();

        // Determine rate limit for this sender
        let rate_limit =
            if let Some(address_policy) = rules.addresses.get(&sender.to_string().to_lowercase()) {
                address_policy.rate_limit_per_minute.unwrap_or(60) // Default 60 req/min
            } else {
                rules.global.rate_limit_per_minute.unwrap_or(60)
            };

        // Get or create rate limit state
        let rate_state = rate_limiter.entry(sender).or_insert_with(|| RateLimit {
            requests: 0,
            window_start: now,
            limit: rate_limit,
        });

        // Check if we need to reset the window (1 minute)
        if now
            .duration_since(rate_state.window_start)
            .unwrap_or_default()
            >= Duration::from_secs(60)
        {
            rate_state.requests = 0;
            rate_state.window_start = now;
        }

        // Check if rate limit exceeded
        if rate_state.requests >= rate_state.limit {
            return Err(format!(
                "Rate limit exceeded: {} requests per minute",
                rate_state.limit
            ));
        }

        // Increment request count
        rate_state.requests += 1;
        Ok(())
    }

    /// Apply address-specific policy
    fn apply_address_policy(
        &self,
        user_op: &UserOperationRequest,
        policy: &AddressPolicy,
    ) -> PolicyResult {
        match policy.action {
            PolicyAction::Deny => PolicyResult {
                allowed: false,
                reason: "Address policy denies this sender".to_string(),
                policy_type: "address_deny".to_string(),
                gas_limit_override: None,
            },
            PolicyAction::Allow => {
                // Check gas limits if specified
                if let Some(max_gas) = policy.max_gas_limit {
                    let total_gas = user_op.call_gas_limit().to::<u64>()
                        + user_op.verification_gas_limit().to::<u64>();

                    if total_gas > max_gas {
                        return PolicyResult {
                            allowed: false,
                            reason: format!(
                                "Gas limit {} exceeds address policy maximum {}",
                                total_gas, max_gas
                            ),
                            policy_type: "address_gas_limit".to_string(),
                            gas_limit_override: Some(max_gas),
                        };
                    }
                }

                PolicyResult {
                    allowed: true,
                    reason: "Address policy allows this sender".to_string(),
                    policy_type: "address_allow".to_string(),
                    gas_limit_override: policy.max_gas_limit,
                }
            }
        }
    }

    /// Check global gas limits
    fn check_global_gas_limits(
        &self,
        user_op: &UserOperationRequest,
        rules: &PolicyRules,
    ) -> std::result::Result<(), String> {
        if let Some(max_gas) = rules.global.max_gas_limit {
            let total_gas =
                user_op.call_gas_limit().to::<u64>() + user_op.verification_gas_limit().to::<u64>();

            if total_gas > max_gas {
                return Err(format!(
                    "Total gas {} exceeds global limit {}",
                    total_gas, max_gas
                ));
            }
        }

        if let Some(max_verification_gas) = rules.global.max_verification_gas {
            if user_op.verification_gas_limit().to::<u64>() > max_verification_gas {
                return Err(format!(
                    "Verification gas {} exceeds global limit {}",
                    user_op.verification_gas_limit(),
                    max_verification_gas
                ));
            }
        }

        if let Some(max_call_gas) = rules.global.max_call_gas {
            if user_op.call_gas_limit().to::<u64>() > max_call_gas {
                return Err(format!(
                    "Call gas {} exceeds global limit {}",
                    user_op.call_gas_limit(),
                    max_call_gas
                ));
            }
        }

        Ok(())
    }

    /// Check if address is in the given list
    fn address_in_list(address: Address, list: &[String]) -> bool {
        let address_str = address.to_string().to_lowercase();
        list.iter().any(|addr| addr.to_lowercase() == address_str)
    }

    /// Get current policy information for an address
    pub async fn get_policy_info(&self, sender: Address) -> crate::rpc::PolicyInfo {
        let rules = self.rules.read().await;

        // Check if address has specific policy
        if let Some(address_policy) = rules.addresses.get(&sender.to_string().to_lowercase()) {
            return crate::rpc::PolicyInfo {
                allowed: matches!(address_policy.action, PolicyAction::Allow),
                policy_type: "address_specific".to_string(),
                max_gas_limit: address_policy.max_gas_limit,
                rate_limit: address_policy.rate_limit_per_minute.map(|limit| {
                    crate::rpc::RateLimit {
                        max_requests: limit,
                        window_seconds: 60,
                        current_usage: 0, // Would need to calculate from rate_limiter
                        reset_time: 0,    // Would need to calculate from window_start
                    }
                }),
                details: format!("Address-specific policy: {:?}", address_policy.action),
            };
        }

        // Check global policies
        let allowed = if let Some(allowed_senders) = &rules.global.allowed_senders {
            Self::address_in_list(sender, allowed_senders)
        } else if let Some(denied_senders) = &rules.global.denied_senders {
            !Self::address_in_list(sender, denied_senders)
        } else {
            matches!(rules.default_action, PolicyAction::Allow)
        };

        crate::rpc::PolicyInfo {
            allowed,
            policy_type: "global".to_string(),
            max_gas_limit: rules.global.max_gas_limit,
            rate_limit: rules
                .global
                .rate_limit_per_minute
                .map(|limit| crate::rpc::RateLimit {
                    max_requests: limit,
                    window_seconds: 60,
                    current_usage: 0,
                    reset_time: 0,
                }),
            details: format!(
                "Global policy with default action: {:?}",
                rules.default_action
            ),
        }
    }
}

impl Default for PolicyRules {
    fn default() -> Self {
        Self {
            default_action: PolicyAction::Deny,
            global: GlobalPolicy::default(),
            addresses: HashMap::new(),
            contracts: HashMap::new(),
        }
    }
}

impl Default for GlobalPolicy {
    fn default() -> Self {
        Self {
            max_gas_limit: Some(10_000_000),
            max_verification_gas: Some(5_000_000),
            max_call_gas: Some(10_000_000),
            rate_limit_per_minute: Some(60),
            allowed_senders: None,
            denied_senders: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Write, time::Duration};

    use tempfile::NamedTempFile;

    use super::*;

    async fn create_test_policy_engine(policy_content: &str) -> PolicyEngine {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(policy_content.as_bytes()).unwrap();

        PolicyEngine::new(temp_file.path(), Duration::from_secs(60))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_policy_loading() {
        let policy_content = r#"
            default_action = "deny"
            
            [global]
            max_gas_limit = 1000000
            rate_limit_per_minute = 10
            
            [[addresses]]
            address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
            action = "allow"
            max_gas_limit = 500000
        "#;

        let engine = create_test_policy_engine(policy_content).await;
        let rules = engine.rules.read().await;

        assert!(matches!(rules.default_action, PolicyAction::Deny));
        assert_eq!(rules.global.max_gas_limit, Some(1_000_000));
        assert_eq!(rules.global.rate_limit_per_minute, Some(10));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let policy_content = r#"
            default_action = "allow"
            
            [global]
            rate_limit_per_minute = 2
        "#;

        let engine = create_test_policy_engine(policy_content).await;
        let sender = Address::ZERO;
        let rules = engine.rules.read().await;

        // First two requests should succeed
        assert!(engine.check_rate_limit(sender, &rules).await.is_ok());
        assert!(engine.check_rate_limit(sender, &rules).await.is_ok());

        // Third request should fail
        assert!(engine.check_rate_limit(sender, &rules).await.is_err());
    }

    // Note: UserOperation construction tests are commented out due to
    // rundler_types::UserOperation being non-exhaustive. Integration tests
    // will be used instead for full end-to-end testing.
}
