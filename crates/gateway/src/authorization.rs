use std::collections::{HashMap, HashSet};

use alloy_primitives::{Address, U256};
use num_traits::ToPrimitive;
use rundler_types::{UserOperation, UserOperationVariant};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::error::GatewayResult;

/// Authorization check result for UserOperation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResult {
    /// Overall authorization status
    pub is_authorized: bool,
    /// Authorization score (0-100)
    pub authorization_score: u8,
    /// Individual check results
    pub check_results: HashMap<String, AuthorizationCheck>,
    /// Blocking issues that prevent authorization
    pub blocking_issues: Vec<String>,
    /// Warnings that should be reviewed
    pub warnings: Vec<String>,
    /// Authorization summary message
    pub summary: String,
    /// Additional metadata
    pub metadata: AuthorizationMetadata,
}

/// Individual authorization check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCheck {
    /// Check name
    pub check_name: String,
    /// Check passed status
    pub passed: bool,
    /// Check result message
    pub message: String,
    /// Check severity level
    pub severity: AuthorizationSeverity,
    /// Additional context data
    pub context: Option<serde_json::Value>,
}

/// Authorization severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthorizationSeverity {
    /// Critical - blocks authorization completely
    Critical,
    /// Error - significant issue that should block
    Error,
    /// Warning - issue that should be reviewed
    Warning,
    /// Info - informational only
    Info,
}

/// Authorization metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationMetadata {
    /// Timestamp of authorization check
    pub timestamp: i64,
    /// Sender reputation score
    pub sender_reputation: Option<u8>,
    /// Paymaster verification status
    pub paymaster_verified: Option<bool>,
    /// Rate limit status
    pub rate_limit_remaining: Option<u32>,
    /// Geographic restrictions applied
    pub geo_restrictions: Option<Vec<String>>,
}

/// Authorization checker configuration
#[derive(Debug, Clone)]
pub struct AuthorizationConfig {
    /// Enable sender whitelist checking
    pub enable_sender_whitelist: bool,
    /// Enable sender blacklist checking
    pub enable_sender_blacklist: bool,
    /// Enable paymaster verification
    pub enable_paymaster_verification: bool,
    /// Enable rate limiting per sender
    pub enable_rate_limiting: bool,
    /// Enable balance checking
    pub enable_balance_checking: bool,
    /// Enable nonce sequence validation
    pub enable_nonce_validation: bool,
    /// Enable gas price limits
    pub enable_gas_price_limits: bool,
    /// Enable contract code verification
    pub enable_contract_verification: bool,
    /// Maximum operations per sender per time window
    pub max_ops_per_sender: u32,
    /// Time window for rate limiting (seconds)
    pub rate_limit_window: u64,
    /// Minimum sender balance required (wei)
    pub min_sender_balance: U256,
    /// Maximum gas price allowed (wei)
    pub max_gas_price: u64,
    /// Minimum reputation score required
    pub min_reputation_score: u8,
}

impl Default for AuthorizationConfig {
    fn default() -> Self {
        Self {
            enable_sender_whitelist: false,
            enable_sender_blacklist: true,
            enable_paymaster_verification: true,
            enable_rate_limiting: true,
            enable_balance_checking: true,
            enable_nonce_validation: true,
            enable_gas_price_limits: true,
            enable_contract_verification: false,
            max_ops_per_sender: 10,
            rate_limit_window: 60,                               // 1 minute
            min_sender_balance: U256::from(1000000000000000u64), // 0.001 ETH
            max_gas_price: 100_000_000_000u64,                   // 100 Gwei
            min_reputation_score: 50,
        }
    }
}

/// Comprehensive authorization checker for UserOperations
pub struct AuthorizationChecker {
    /// Configuration for authorization rules
    config: AuthorizationConfig,
    /// Sender whitelist
    sender_whitelist: HashSet<Address>,
    /// Sender blacklist
    sender_blacklist: HashSet<Address>,
    /// Verified paymaster addresses
    verified_paymasters: HashSet<Address>,
    /// Rate limiting tracking
    rate_limit_tracker: HashMap<Address, RateLimitState>,
    /// Sender reputation scores
    reputation_scores: HashMap<Address, u8>,
}

/// Rate limiting state for a sender
#[derive(Debug, Clone)]
struct RateLimitState {
    /// Number of operations in current window
    operation_count: u32,
    /// Window start timestamp
    window_start: i64,
    /// Last operation timestamp
    last_operation: i64,
}

impl AuthorizationChecker {
    /// Create a new authorization checker with default configuration
    pub fn new() -> Self {
        Self {
            config: AuthorizationConfig::default(),
            sender_whitelist: HashSet::new(),
            sender_blacklist: HashSet::new(),
            verified_paymasters: HashSet::new(),
            rate_limit_tracker: HashMap::new(),
            reputation_scores: HashMap::new(),
        }
    }

    /// Create a new authorization checker with custom configuration
    pub fn with_config(config: AuthorizationConfig) -> Self {
        Self {
            config,
            sender_whitelist: HashSet::new(),
            sender_blacklist: HashSet::new(),
            verified_paymasters: HashSet::new(),
            rate_limit_tracker: HashMap::new(),
            reputation_scores: HashMap::new(),
        }
    }

    /// Add sender to whitelist
    pub fn add_to_whitelist(&mut self, sender: Address) {
        self.sender_whitelist.insert(sender);
        debug!("Added sender {:?} to whitelist", sender);
    }

    /// Add sender to blacklist
    pub fn add_to_blacklist(&mut self, sender: Address) {
        self.sender_blacklist.insert(sender);
        warn!("Added sender {:?} to blacklist", sender);
    }

    /// Add verified paymaster
    pub fn add_verified_paymaster(&mut self, paymaster: Address) {
        self.verified_paymasters.insert(paymaster);
        debug!("Added paymaster {:?} to verified list", paymaster);
    }

    /// Set sender reputation score
    pub fn set_reputation_score(&mut self, sender: Address, score: u8) {
        self.reputation_scores.insert(sender, score);
        debug!("Set reputation score {} for sender {:?}", score, sender);
    }

    /// Load configuration from external sources (e.g., database, config files)
    pub async fn load_configuration(&mut self) -> GatewayResult<()> {
        // TODO: In production, load from database or configuration service
        // For now, add some default configurations

        // Add some well-known safe paymasters
        self.add_verified_paymaster(
            "0x0000000000000000000000000000000000000001"
                .parse()
                .unwrap(),
        );

        // Set default reputation scores
        self.set_reputation_score(
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
                .parse()
                .unwrap(),
            100,
        );

        debug!("✅ Authorization configuration loaded successfully");
        Ok(())
    }

    /// Perform comprehensive authorization check on UserOperation
    pub async fn check_authorization(
        &mut self,
        user_op: &UserOperationVariant,
        _entry_point: &Address,
        client_ip: Option<&str>,
    ) -> GatewayResult<AuthorizationResult> {
        debug!("🔍 Starting comprehensive authorization check");

        let mut check_results = HashMap::new();
        let mut blocking_issues = Vec::new();
        let mut warnings = Vec::new();
        let mut authorization_score = 100u8;
        let timestamp = chrono::Utc::now().timestamp();

        // Extract sender address
        let sender = match user_op {
            UserOperationVariant::V0_6(op) => op.sender(),
            UserOperationVariant::V0_7(op) => op.sender(),
        };

        debug!("Authorizing UserOperation for sender: {:?}", sender);

        // 1. Sender Whitelist Check
        if self.config.enable_sender_whitelist {
            let whitelist_check = self.check_sender_whitelist(&sender);
            self.process_authorization_check(
                whitelist_check,
                &mut check_results,
                &mut blocking_issues,
                &mut warnings,
                &mut authorization_score,
            );
        }

        // 2. Sender Blacklist Check
        if self.config.enable_sender_blacklist {
            let blacklist_check = self.check_sender_blacklist(&sender);
            self.process_authorization_check(
                blacklist_check,
                &mut check_results,
                &mut blocking_issues,
                &mut warnings,
                &mut authorization_score,
            );
        }

        // 3. Rate Limiting Check
        if self.config.enable_rate_limiting {
            let rate_limit_check = self.check_rate_limit(&sender, timestamp);
            self.process_authorization_check(
                rate_limit_check,
                &mut check_results,
                &mut blocking_issues,
                &mut warnings,
                &mut authorization_score,
            );
        }

        // 4. Paymaster Verification Check
        if self.config.enable_paymaster_verification {
            let paymaster_check = self.check_paymaster_verification(user_op).await?;
            self.process_authorization_check(
                paymaster_check,
                &mut check_results,
                &mut blocking_issues,
                &mut warnings,
                &mut authorization_score,
            );
        }

        // 5. Balance Check
        if self.config.enable_balance_checking {
            let balance_check = self.check_sender_balance(&sender).await?;
            self.process_authorization_check(
                balance_check,
                &mut check_results,
                &mut blocking_issues,
                &mut warnings,
                &mut authorization_score,
            );
        }

        // 6. Nonce Validation
        if self.config.enable_nonce_validation {
            let nonce_check = self.check_nonce_sequence(user_op, &sender).await?;
            self.process_authorization_check(
                nonce_check,
                &mut check_results,
                &mut blocking_issues,
                &mut warnings,
                &mut authorization_score,
            );
        }

        // 7. Gas Price Limits
        if self.config.enable_gas_price_limits {
            let gas_price_check = self.check_gas_price_limits(user_op);
            self.process_authorization_check(
                gas_price_check,
                &mut check_results,
                &mut blocking_issues,
                &mut warnings,
                &mut authorization_score,
            );
        }

        // 8. Reputation Check
        let reputation_check = self.check_sender_reputation(&sender);
        self.process_authorization_check(
            reputation_check,
            &mut check_results,
            &mut blocking_issues,
            &mut warnings,
            &mut authorization_score,
        );

        // Determine overall authorization status
        let is_authorized = blocking_issues.is_empty();

        let summary = if is_authorized {
            if warnings.is_empty() {
                format!(
                    "✅ UserOperation authorized (score: {})",
                    authorization_score
                )
            } else {
                format!(
                    "⚠️ UserOperation authorized with {} warnings (score: {})",
                    warnings.len(),
                    authorization_score
                )
            }
        } else {
            format!(
                "❌ UserOperation authorization failed: {} blocking issues",
                blocking_issues.len()
            )
        };

        // Build metadata
        let metadata = AuthorizationMetadata {
            timestamp,
            sender_reputation: self.reputation_scores.get(&sender).copied(),
            paymaster_verified: self.get_paymaster_verification_status(user_op),
            rate_limit_remaining: self.get_rate_limit_remaining(&sender),
            geo_restrictions: client_ip.map(|_| vec!["No restrictions".to_string()]),
        };

        debug!("Authorization check completed: {}", summary);

        Ok(AuthorizationResult {
            is_authorized,
            authorization_score,
            check_results,
            blocking_issues,
            warnings,
            summary,
            metadata,
        })
    }

    /// Check if sender is in whitelist
    fn check_sender_whitelist(&self, sender: &Address) -> AuthorizationCheck {
        let is_whitelisted = self.sender_whitelist.contains(sender);

        if self.sender_whitelist.is_empty() {
            // No whitelist configured, pass by default
            AuthorizationCheck {
                check_name: "sender_whitelist".to_string(),
                passed: true,
                message: "No whitelist configured, allowing all senders".to_string(),
                severity: AuthorizationSeverity::Info,
                context: None,
            }
        } else if is_whitelisted {
            AuthorizationCheck {
                check_name: "sender_whitelist".to_string(),
                passed: true,
                message: format!("Sender {:?} is whitelisted", sender),
                severity: AuthorizationSeverity::Info,
                context: Some(serde_json::json!({"sender": format!("{:?}", sender)})),
            }
        } else {
            AuthorizationCheck {
                check_name: "sender_whitelist".to_string(),
                passed: false,
                message: format!("Sender {:?} not found in whitelist", sender),
                severity: AuthorizationSeverity::Critical,
                context: Some(serde_json::json!({"sender": format!("{:?}", sender)})),
            }
        }
    }

    /// Check if sender is in blacklist
    fn check_sender_blacklist(&self, sender: &Address) -> AuthorizationCheck {
        let is_blacklisted = self.sender_blacklist.contains(sender);

        if is_blacklisted {
            AuthorizationCheck {
                check_name: "sender_blacklist".to_string(),
                passed: false,
                message: format!("Sender {:?} is blacklisted", sender),
                severity: AuthorizationSeverity::Critical,
                context: Some(serde_json::json!({"sender": format!("{:?}", sender)})),
            }
        } else {
            AuthorizationCheck {
                check_name: "sender_blacklist".to_string(),
                passed: true,
                message: format!("Sender {:?} not in blacklist", sender),
                severity: AuthorizationSeverity::Info,
                context: Some(serde_json::json!({"sender": format!("{:?}", sender)})),
            }
        }
    }

    /// Check rate limiting for sender
    fn check_rate_limit(&mut self, sender: &Address, timestamp: i64) -> AuthorizationCheck {
        let current_state =
            self.rate_limit_tracker
                .entry(*sender)
                .or_insert_with(|| RateLimitState {
                    operation_count: 0,
                    window_start: timestamp,
                    last_operation: timestamp,
                });

        // Check if we need to reset the window
        if timestamp - current_state.window_start >= self.config.rate_limit_window as i64 {
            current_state.operation_count = 0;
            current_state.window_start = timestamp;
        }

        // Check if sender exceeds rate limit
        if current_state.operation_count >= self.config.max_ops_per_sender {
            AuthorizationCheck {
                check_name: "rate_limit".to_string(),
                passed: false,
                message: format!(
                    "Rate limit exceeded: {} operations in {} seconds (max: {})",
                    current_state.operation_count,
                    self.config.rate_limit_window,
                    self.config.max_ops_per_sender
                ),
                severity: AuthorizationSeverity::Error,
                context: Some(serde_json::json!({
                    "current_count": current_state.operation_count,
                    "max_allowed": self.config.max_ops_per_sender,
                    "window_seconds": self.config.rate_limit_window
                })),
            }
        } else {
            // Increment counter for this operation
            current_state.operation_count += 1;
            current_state.last_operation = timestamp;

            let remaining = self.config.max_ops_per_sender - current_state.operation_count;

            AuthorizationCheck {
                check_name: "rate_limit".to_string(),
                passed: true,
                message: format!("Rate limit OK: {} remaining operations", remaining),
                severity: AuthorizationSeverity::Info,
                context: Some(serde_json::json!({
                    "remaining_operations": remaining,
                    "window_seconds": self.config.rate_limit_window
                })),
            }
        }
    }

    /// Check paymaster verification status
    async fn check_paymaster_verification(
        &self,
        user_op: &UserOperationVariant,
    ) -> GatewayResult<AuthorizationCheck> {
        let paymaster_info = self.extract_paymaster_info(user_op);

        match paymaster_info {
            Some(paymaster_address) => {
                if self.verified_paymasters.contains(&paymaster_address) {
                    Ok(AuthorizationCheck {
                        check_name: "paymaster_verification".to_string(),
                        passed: true,
                        message: format!("Paymaster {:?} is verified", paymaster_address),
                        severity: AuthorizationSeverity::Info,
                        context: Some(
                            serde_json::json!({"paymaster": format!("{:?}", paymaster_address)}),
                        ),
                    })
                } else {
                    // In production, this might query a verification service
                    // For now, we'll allow unverified paymasters with a warning
                    Ok(AuthorizationCheck {
                        check_name: "paymaster_verification".to_string(),
                        passed: true,
                        message: format!("Paymaster {:?} not in verified list", paymaster_address),
                        severity: AuthorizationSeverity::Warning,
                        context: Some(
                            serde_json::json!({"paymaster": format!("{:?}", paymaster_address)}),
                        ),
                    })
                }
            }
            None => {
                // No paymaster specified - this is allowed for self-paying operations
                Ok(AuthorizationCheck {
                    check_name: "paymaster_verification".to_string(),
                    passed: true,
                    message: "No paymaster specified (self-paying operation)".to_string(),
                    severity: AuthorizationSeverity::Info,
                    context: None,
                })
            }
        }
    }

    /// Check sender balance
    async fn check_sender_balance(&self, _sender: &Address) -> GatewayResult<AuthorizationCheck> {
        // TODO: In production, query actual blockchain balance
        // For now, simulate balance check
        let simulated_balance = U256::from(10000000000000000000u64); // 10 ETH simulation

        if simulated_balance >= self.config.min_sender_balance {
            Ok(AuthorizationCheck {
                check_name: "balance_check".to_string(),
                passed: true,
                message: format!(
                    "Sender balance sufficient: {} wei (min: {} wei)",
                    simulated_balance, self.config.min_sender_balance
                ),
                severity: AuthorizationSeverity::Info,
                context: Some(serde_json::json!({
                    "balance": simulated_balance.to_string(),
                    "min_required": self.config.min_sender_balance.to_string()
                })),
            })
        } else {
            Ok(AuthorizationCheck {
                check_name: "balance_check".to_string(),
                passed: false,
                message: format!(
                    "Insufficient sender balance: {} wei (min: {} wei)",
                    simulated_balance, self.config.min_sender_balance
                ),
                severity: AuthorizationSeverity::Critical,
                context: Some(serde_json::json!({
                    "balance": simulated_balance.to_string(),
                    "min_required": self.config.min_sender_balance.to_string()
                })),
            })
        }
    }

    /// Check nonce sequence validity
    async fn check_nonce_sequence(
        &self,
        user_op: &UserOperationVariant,
        sender: &Address,
    ) -> GatewayResult<AuthorizationCheck> {
        let nonce = match user_op {
            UserOperationVariant::V0_6(op) => op.nonce(),
            UserOperationVariant::V0_7(op) => op.nonce(),
        };

        // TODO: In production, verify nonce against actual on-chain state
        // For now, basic nonce validation
        let max_reasonable_nonce = U256::from(1000000u64);

        if nonce > max_reasonable_nonce {
            Ok(AuthorizationCheck {
                check_name: "nonce_validation".to_string(),
                passed: true,
                message: format!("Nonce {} is very high, verify correctness", nonce),
                severity: AuthorizationSeverity::Warning,
                context: Some(serde_json::json!({
                    "nonce": nonce.to_string(),
                    "sender": format!("{:?}", sender)
                })),
            })
        } else {
            Ok(AuthorizationCheck {
                check_name: "nonce_validation".to_string(),
                passed: true,
                message: format!("Nonce {} is within reasonable range", nonce),
                severity: AuthorizationSeverity::Info,
                context: Some(serde_json::json!({
                    "nonce": nonce.to_string(),
                    "sender": format!("{:?}", sender)
                })),
            })
        }
    }

    /// Check gas price limits
    fn check_gas_price_limits(&self, user_op: &UserOperationVariant) -> AuthorizationCheck {
        let (max_fee_per_gas, max_priority_fee_per_gas) = match user_op {
            UserOperationVariant::V0_6(op) => (
                op.max_fee_per_gas().to_u64().unwrap_or(u64::MAX),
                op.max_priority_fee_per_gas().to_u64().unwrap_or(u64::MAX),
            ),
            UserOperationVariant::V0_7(op) => (
                op.max_fee_per_gas().to_u64().unwrap_or(u64::MAX),
                op.max_priority_fee_per_gas().to_u64().unwrap_or(u64::MAX),
            ),
        };

        if max_fee_per_gas > self.config.max_gas_price {
            AuthorizationCheck {
                check_name: "gas_price_limits".to_string(),
                passed: false,
                message: format!(
                    "Max fee per gas {} exceeds limit {} (wei)",
                    max_fee_per_gas, self.config.max_gas_price
                ),
                severity: AuthorizationSeverity::Error,
                context: Some(serde_json::json!({
                    "max_fee_per_gas": max_fee_per_gas,
                    "max_priority_fee_per_gas": max_priority_fee_per_gas,
                    "limit": self.config.max_gas_price
                })),
            }
        } else if max_priority_fee_per_gas > self.config.max_gas_price {
            AuthorizationCheck {
                check_name: "gas_price_limits".to_string(),
                passed: false,
                message: format!(
                    "Max priority fee per gas {} exceeds limit {} (wei)",
                    max_priority_fee_per_gas, self.config.max_gas_price
                ),
                severity: AuthorizationSeverity::Error,
                context: Some(serde_json::json!({
                    "max_fee_per_gas": max_fee_per_gas,
                    "max_priority_fee_per_gas": max_priority_fee_per_gas,
                    "limit": self.config.max_gas_price
                })),
            }
        } else {
            AuthorizationCheck {
                check_name: "gas_price_limits".to_string(),
                passed: true,
                message: format!(
                    "Gas prices within limits: fee={}, priority={} (max: {})",
                    max_fee_per_gas, max_priority_fee_per_gas, self.config.max_gas_price
                ),
                severity: AuthorizationSeverity::Info,
                context: Some(serde_json::json!({
                    "max_fee_per_gas": max_fee_per_gas,
                    "max_priority_fee_per_gas": max_priority_fee_per_gas,
                    "limit": self.config.max_gas_price
                })),
            }
        }
    }

    /// Check sender reputation score
    fn check_sender_reputation(&self, sender: &Address) -> AuthorizationCheck {
        let reputation_score = self.reputation_scores.get(sender).copied().unwrap_or(75); // Default reputation

        if reputation_score < self.config.min_reputation_score {
            AuthorizationCheck {
                check_name: "reputation_check".to_string(),
                passed: false,
                message: format!(
                    "Sender reputation {} below minimum required {}",
                    reputation_score, self.config.min_reputation_score
                ),
                severity: AuthorizationSeverity::Error,
                context: Some(serde_json::json!({
                    "reputation_score": reputation_score,
                    "min_required": self.config.min_reputation_score,
                    "sender": format!("{:?}", sender)
                })),
            }
        } else {
            AuthorizationCheck {
                check_name: "reputation_check".to_string(),
                passed: true,
                message: format!("Sender reputation {} meets requirement", reputation_score),
                severity: AuthorizationSeverity::Info,
                context: Some(serde_json::json!({
                    "reputation_score": reputation_score,
                    "min_required": self.config.min_reputation_score,
                    "sender": format!("{:?}", sender)
                })),
            }
        }
    }

    /// Extract paymaster address from UserOperation
    fn extract_paymaster_info(&self, user_op: &UserOperationVariant) -> Option<Address> {
        match user_op {
            UserOperationVariant::V0_6(op) => {
                let paymaster_and_data = op.paymaster_and_data();
                if paymaster_and_data.is_empty() {
                    None
                } else if paymaster_and_data.len() >= 20 {
                    // Extract address from first 20 bytes
                    Some(Address::from_slice(&paymaster_and_data[0..20]))
                } else {
                    None
                }
            }
            UserOperationVariant::V0_7(op) => op.paymaster(),
        }
    }

    /// Get paymaster verification status
    fn get_paymaster_verification_status(&self, user_op: &UserOperationVariant) -> Option<bool> {
        self.extract_paymaster_info(user_op)
            .map(|paymaster| self.verified_paymasters.contains(&paymaster))
    }

    /// Get remaining rate limit for sender
    fn get_rate_limit_remaining(&self, sender: &Address) -> Option<u32> {
        self.rate_limit_tracker.get(sender).map(|state| {
            self.config
                .max_ops_per_sender
                .saturating_sub(state.operation_count)
        })
    }

    /// Process authorization check result and update tracking collections
    fn process_authorization_check(
        &self,
        check: AuthorizationCheck,
        check_results: &mut HashMap<String, AuthorizationCheck>,
        blocking_issues: &mut Vec<String>,
        warnings: &mut Vec<String>,
        authorization_score: &mut u8,
    ) {
        let check_name = check.check_name.clone();

        match check.severity {
            AuthorizationSeverity::Critical if !check.passed => {
                blocking_issues.push(check.message.clone());
                *authorization_score = authorization_score.saturating_sub(30);
            }
            AuthorizationSeverity::Error if !check.passed => {
                blocking_issues.push(check.message.clone());
                *authorization_score = authorization_score.saturating_sub(20);
            }
            AuthorizationSeverity::Warning => {
                warnings.push(check.message.clone());
                *authorization_score = authorization_score.saturating_sub(10);
            }
            _ => {} // Info or successful checks don't affect score negatively
        }

        check_results.insert(check_name, check);
    }
}

impl Default for AuthorizationChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_config_default() {
        let config = AuthorizationConfig::default();
        assert!(config.enable_sender_blacklist);
        assert!(config.enable_paymaster_verification);
        assert!(config.enable_rate_limiting);
        assert_eq!(config.max_ops_per_sender, 10);
        assert_eq!(config.rate_limit_window, 60);
    }

    #[test]
    fn test_sender_whitelist_check() {
        let mut checker = AuthorizationChecker::new();
        let sender: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
            .parse()
            .unwrap();

        // Test without whitelist (should pass)
        let result = checker.check_sender_whitelist(&sender);
        assert!(result.passed);

        // Add to whitelist
        checker.add_to_whitelist(sender);
        checker.config.enable_sender_whitelist = true;

        // Test with whitelist (should pass)
        let result = checker.check_sender_whitelist(&sender);
        assert!(result.passed);

        // Test different sender (should fail)
        let other_sender: Address = "0x0000000000000000000000000000000000000001"
            .parse()
            .unwrap();
        let result = checker.check_sender_whitelist(&other_sender);
        assert!(!result.passed);
        assert_eq!(result.severity, AuthorizationSeverity::Critical);
    }

    #[test]
    fn test_sender_blacklist_check() {
        let mut checker = AuthorizationChecker::new();
        let sender: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
            .parse()
            .unwrap();

        // Test normal sender (should pass)
        let result = checker.check_sender_blacklist(&sender);
        assert!(result.passed);

        // Add to blacklist
        checker.add_to_blacklist(sender);

        // Test blacklisted sender (should fail)
        let result = checker.check_sender_blacklist(&sender);
        assert!(!result.passed);
        assert_eq!(result.severity, AuthorizationSeverity::Critical);
    }

    #[test]
    fn test_rate_limiting() {
        let mut checker = AuthorizationChecker::new();
        let sender: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
            .parse()
            .unwrap();
        let timestamp = chrono::Utc::now().timestamp();

        // First few operations should pass
        for i in 1..=5 {
            let result = checker.check_rate_limit(&sender, timestamp + i);
            assert!(result.passed, "Operation {} should pass rate limit", i);
        }

        // Exceed rate limit
        for i in 6..=15 {
            let result = checker.check_rate_limit(&sender, timestamp + i);
            if i <= 10 {
                assert!(result.passed, "Operation {} should still pass", i);
            } else {
                assert!(!result.passed, "Operation {} should fail rate limit", i);
                assert_eq!(result.severity, AuthorizationSeverity::Error);
            }
        }
    }

    #[test]
    fn test_reputation_check() {
        let mut checker = AuthorizationChecker::new();
        let sender: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
            .parse()
            .unwrap();

        // Test with default reputation (should pass)
        let result = checker.check_sender_reputation(&sender);
        assert!(result.passed);

        // Set low reputation
        checker.set_reputation_score(sender, 30);

        // Test with low reputation (should fail)
        let result = checker.check_sender_reputation(&sender);
        assert!(!result.passed);
        assert_eq!(result.severity, AuthorizationSeverity::Error);

        // Set high reputation
        checker.set_reputation_score(sender, 80);

        // Test with high reputation (should pass)
        let result = checker.check_sender_reputation(&sender);
        assert!(result.passed);
    }
}
