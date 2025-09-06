use std::collections::{HashMap, HashSet};

use alloy_primitives::Address;
use num_traits::ToPrimitive;
use rundler_types::{UserOperation, UserOperationVariant};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::error::GatewayResult;

/// Security check result for UserOperation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResult {
    /// Overall security status
    pub is_secure: bool,
    /// Security score (0-100)
    pub security_score: u8,
    /// Individual check results
    pub check_results: HashMap<String, SecurityCheck>,
    /// Critical security violations
    pub critical_violations: Vec<String>,
    /// Security warnings
    pub warnings: Vec<String>,
    /// Security assessment summary
    pub summary: String,
    /// Security analysis metadata
    pub metadata: SecurityMetadata,
}

/// Individual security check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheck {
    /// Check name
    pub check_name: String,
    /// Check passed status
    pub passed: bool,
    /// Check result message
    pub message: String,
    /// Security risk level
    pub risk_level: SecurityRiskLevel,
    /// Additional security context
    pub context: Option<serde_json::Value>,
}

/// Security risk levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SecurityRiskLevel {
    /// Critical - immediate security threat
    Critical,
    /// High - significant security risk
    High,
    /// Medium - moderate security concern
    Medium,
    /// Low - minor security note
    Low,
    /// Info - informational only
    Info,
}

/// Security analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetadata {
    /// Timestamp of security check
    pub timestamp: i64,
    /// Anomaly detection score
    pub anomaly_score: Option<f64>,
    /// Phishing risk assessment
    pub phishing_risk_level: Option<String>,
    /// Smart contract risk score
    pub contract_risk_score: Option<u8>,
    /// Transaction pattern analysis
    pub pattern_analysis: Option<Vec<String>>,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable smart contract verification
    pub enable_contract_verification: bool,
    /// Enable transaction pattern analysis
    pub enable_pattern_analysis: bool,
    /// Enable phishing detection
    pub enable_phishing_detection: bool,
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
    /// Enable MEV protection checks
    pub enable_mev_protection: bool,
    /// Enable calldata analysis
    pub enable_calldata_analysis: bool,
    /// Maximum call gas limit allowed
    pub max_call_gas_limit: u128,
    /// Maximum verification gas limit
    pub max_verification_gas_limit: u128,
    /// Maximum calldata size (bytes)
    pub max_calldata_size: usize,
    /// Maximum init code size (bytes)
    pub max_init_code_size: usize,
    /// Minimum reputation score required
    pub min_contract_reputation: u8,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_contract_verification: true,
            enable_pattern_analysis: true,
            enable_phishing_detection: true,
            enable_anomaly_detection: false, // CPU intensive, disabled by default
            enable_mev_protection: true,
            enable_calldata_analysis: true,
            max_call_gas_limit: 10_000_000,        // 10M gas
            max_verification_gas_limit: 5_000_000, // 5M gas
            max_calldata_size: 10_000,             // 10KB
            max_init_code_size: 50_000,            // 50KB
            min_contract_reputation: 50,
        }
    }
}

/// Comprehensive security checker for UserOperations
pub struct SecurityChecker {
    /// Security configuration
    config: SecurityConfig,
    /// Known malicious addresses
    malicious_addresses: HashSet<Address>,
    /// Known phishing patterns
    phishing_patterns: Vec<String>,
    /// Contract reputation scores
    contract_reputation: HashMap<Address, u8>,
    /// Suspicious transaction patterns
    suspicious_patterns: Vec<TransactionPattern>,
}

/// Transaction pattern for anomaly detection
#[derive(Debug, Clone)]
struct TransactionPattern {
    /// Pattern name
    name: String,
    /// Pattern description
    description: String,
    /// Risk score (0-100)
    risk_score: u8,
    /// Pattern matching function placeholder
    pattern_matcher: fn(&UserOperationVariant) -> bool,
}

impl SecurityChecker {
    /// Create a new security checker with default configuration
    pub fn new() -> Self {
        Self {
            config: SecurityConfig::default(),
            malicious_addresses: HashSet::new(),
            phishing_patterns: Vec::new(),
            contract_reputation: HashMap::new(),
            suspicious_patterns: Self::default_suspicious_patterns(),
        }
    }

    /// Create a new security checker with custom configuration
    pub fn with_config(config: SecurityConfig) -> Self {
        Self {
            config,
            malicious_addresses: HashSet::new(),
            phishing_patterns: Vec::new(),
            contract_reputation: HashMap::new(),
            suspicious_patterns: Self::default_suspicious_patterns(),
        }
    }

    /// Load security configuration and threat intelligence
    pub async fn load_threat_intelligence(&mut self) -> GatewayResult<()> {
        // TODO: In production, load from threat intelligence feeds
        // For now, add some default threat indicators

        // Add known malicious addresses (example addresses)
        self.add_malicious_address(
            "0x0000000000000000000000000000000000000bad"
                .parse()
                .unwrap(),
        );
        self.add_malicious_address(
            "0x000000000000000000000000000000000000dead"
                .parse()
                .unwrap(),
        );

        // Add phishing patterns
        self.add_phishing_pattern("transfer.*0x00000000".to_string());
        self.add_phishing_pattern("approve.*999999999".to_string());

        // Set contract reputation scores
        self.set_contract_reputation(
            "0xA0b86a33E6441d6e42D2B6b63B3b66F2c7A77fD8"
                .parse()
                .unwrap(),
            95, // High reputation contract
        );

        debug!("✅ Security threat intelligence loaded successfully");
        Ok(())
    }

    /// Perform comprehensive security analysis on UserOperation
    pub async fn check_security(
        &self,
        user_op: &UserOperationVariant,
        entry_point: &Address,
        _client_ip: Option<&str>,
    ) -> GatewayResult<SecurityResult> {
        debug!("🔍 Starting comprehensive security analysis");

        let mut check_results = HashMap::new();
        let mut critical_violations = Vec::new();
        let mut warnings = Vec::new();
        let mut security_score = 100u8;
        let timestamp = chrono::Utc::now().timestamp();

        // Extract key addresses and data
        let sender = match user_op {
            UserOperationVariant::V0_6(op) => op.sender(),
            UserOperationVariant::V0_7(op) => op.sender(),
        };

        debug!(
            "Security analysis for sender: {:?} on entry point: {:?}",
            sender, entry_point
        );

        // 1. Malicious Address Check
        let malicious_check = self.check_malicious_addresses(user_op);
        self.process_security_check(
            malicious_check,
            &mut check_results,
            &mut critical_violations,
            &mut warnings,
            &mut security_score,
        );

        // 2. Gas Limits Validation
        let gas_limits_check = self.check_gas_limits(user_op);
        self.process_security_check(
            gas_limits_check,
            &mut check_results,
            &mut critical_violations,
            &mut warnings,
            &mut security_score,
        );

        // 3. Calldata Size and Content Analysis
        if self.config.enable_calldata_analysis {
            let calldata_check = self.check_calldata_security(user_op);
            self.process_security_check(
                calldata_check,
                &mut check_results,
                &mut critical_violations,
                &mut warnings,
                &mut security_score,
            );
        }

        // 4. Smart Contract Verification
        if self.config.enable_contract_verification {
            let contract_check = self.check_contract_security(&sender).await?;
            self.process_security_check(
                contract_check,
                &mut check_results,
                &mut critical_violations,
                &mut warnings,
                &mut security_score,
            );
        }

        // 5. Transaction Pattern Analysis
        if self.config.enable_pattern_analysis {
            let pattern_check = self.check_transaction_patterns(user_op);
            self.process_security_check(
                pattern_check,
                &mut check_results,
                &mut critical_violations,
                &mut warnings,
                &mut security_score,
            );
        }

        // 6. Phishing Detection
        if self.config.enable_phishing_detection {
            let phishing_check = self.check_phishing_indicators(user_op);
            self.process_security_check(
                phishing_check,
                &mut check_results,
                &mut critical_violations,
                &mut warnings,
                &mut security_score,
            );
        }

        // 7. MEV Protection Check
        if self.config.enable_mev_protection {
            let mev_check = self.check_mev_protection(user_op);
            self.process_security_check(
                mev_check,
                &mut check_results,
                &mut critical_violations,
                &mut warnings,
                &mut security_score,
            );
        }

        // 8. Init Code Security (for v0.6 and v0.7 factory deployments)
        let init_code_check = self.check_init_code_security(user_op);
        self.process_security_check(
            init_code_check,
            &mut check_results,
            &mut critical_violations,
            &mut warnings,
            &mut security_score,
        );

        // Determine overall security status
        let is_secure = critical_violations.is_empty();

        let summary = if is_secure {
            if warnings.is_empty() {
                format!(
                    "✅ UserOperation passed all security checks (score: {})",
                    security_score
                )
            } else {
                format!(
                    "⚠️ UserOperation passed security checks with {} warnings (score: {})",
                    warnings.len(),
                    security_score
                )
            }
        } else {
            format!(
                "🚨 UserOperation failed security checks: {} critical violations",
                critical_violations.len()
            )
        };

        // Build security metadata
        let metadata = SecurityMetadata {
            timestamp,
            anomaly_score: if self.config.enable_anomaly_detection {
                Some(self.calculate_anomaly_score(user_op))
            } else {
                None
            },
            phishing_risk_level: Some(self.assess_phishing_risk_level(user_op)),
            contract_risk_score: Some(self.calculate_contract_risk_score(&sender)),
            pattern_analysis: Some(self.get_detected_patterns(user_op)),
        };

        debug!("Security analysis completed: {}", summary);

        Ok(SecurityResult {
            is_secure,
            security_score,
            check_results,
            critical_violations,
            warnings,
            summary,
            metadata,
        })
    }

    /// Check for known malicious addresses
    fn check_malicious_addresses(&self, user_op: &UserOperationVariant) -> SecurityCheck {
        let sender = match user_op {
            UserOperationVariant::V0_6(op) => op.sender(),
            UserOperationVariant::V0_7(op) => op.sender(),
        };

        // Check sender
        if self.malicious_addresses.contains(&sender) {
            return SecurityCheck {
                check_name: "malicious_address".to_string(),
                passed: false,
                message: format!("Sender address {:?} is known to be malicious", sender),
                risk_level: SecurityRiskLevel::Critical,
                context: Some(serde_json::json!({
                    "sender": format!("{:?}", sender),
                    "threat_type": "known_malicious"
                })),
            };
        }

        // Check paymaster (if present)
        let paymaster_address = self.extract_paymaster_address(user_op);
        if let Some(paymaster) = paymaster_address {
            if self.malicious_addresses.contains(&paymaster) {
                return SecurityCheck {
                    check_name: "malicious_address".to_string(),
                    passed: false,
                    message: format!("Paymaster address {:?} is known to be malicious", paymaster),
                    risk_level: SecurityRiskLevel::Critical,
                    context: Some(serde_json::json!({
                        "paymaster": format!("{:?}", paymaster),
                        "threat_type": "malicious_paymaster"
                    })),
                };
            }
        }

        SecurityCheck {
            check_name: "malicious_address".to_string(),
            passed: true,
            message: "No known malicious addresses detected".to_string(),
            risk_level: SecurityRiskLevel::Info,
            context: Some(serde_json::json!({
                "checked_addresses": [format!("{:?}", sender)]
            })),
        }
    }

    /// Check gas limits for potential abuse
    fn check_gas_limits(&self, user_op: &UserOperationVariant) -> SecurityCheck {
        let (call_gas_limit, verification_gas_limit) = match user_op {
            UserOperationVariant::V0_6(op) => (
                op.call_gas_limit().to_u128().unwrap_or(u128::MAX),
                op.verification_gas_limit().to_u128().unwrap_or(u128::MAX),
            ),
            UserOperationVariant::V0_7(op) => (
                op.call_gas_limit().to_u128().unwrap_or(u128::MAX),
                op.verification_gas_limit().to_u128().unwrap_or(u128::MAX),
            ),
        };

        // Check call gas limit
        if call_gas_limit > self.config.max_call_gas_limit {
            return SecurityCheck {
                check_name: "gas_limits".to_string(),
                passed: false,
                message: format!(
                    "Call gas limit {} exceeds maximum allowed {} (potential DoS)",
                    call_gas_limit, self.config.max_call_gas_limit
                ),
                risk_level: SecurityRiskLevel::High,
                context: Some(serde_json::json!({
                    "call_gas_limit": call_gas_limit,
                    "max_allowed": self.config.max_call_gas_limit,
                    "risk_type": "resource_exhaustion"
                })),
            };
        }

        // Check verification gas limit
        if verification_gas_limit > self.config.max_verification_gas_limit {
            return SecurityCheck {
                check_name: "gas_limits".to_string(),
                passed: false,
                message: format!(
                    "Verification gas limit {} exceeds maximum allowed {} (potential DoS)",
                    verification_gas_limit, self.config.max_verification_gas_limit
                ),
                risk_level: SecurityRiskLevel::High,
                context: Some(serde_json::json!({
                    "verification_gas_limit": verification_gas_limit,
                    "max_allowed": self.config.max_verification_gas_limit,
                    "risk_type": "resource_exhaustion"
                })),
            };
        }

        SecurityCheck {
            check_name: "gas_limits".to_string(),
            passed: true,
            message: format!(
                "Gas limits within safe bounds: call={}, verification={}",
                call_gas_limit, verification_gas_limit
            ),
            risk_level: SecurityRiskLevel::Info,
            context: Some(serde_json::json!({
                "call_gas_limit": call_gas_limit,
                "verification_gas_limit": verification_gas_limit
            })),
        }
    }

    /// Check calldata for security issues
    fn check_calldata_security(&self, user_op: &UserOperationVariant) -> SecurityCheck {
        let call_data = match user_op {
            UserOperationVariant::V0_6(op) => op.call_data(),
            UserOperationVariant::V0_7(op) => op.call_data(),
        };

        // Check calldata size
        if call_data.len() > self.config.max_calldata_size {
            return SecurityCheck {
                check_name: "calldata_security".to_string(),
                passed: false,
                message: format!(
                    "Calldata size {} exceeds maximum allowed {} bytes",
                    call_data.len(),
                    self.config.max_calldata_size
                ),
                risk_level: SecurityRiskLevel::Medium,
                context: Some(serde_json::json!({
                    "calldata_size": call_data.len(),
                    "max_allowed": self.config.max_calldata_size,
                    "risk_type": "oversized_calldata"
                })),
            };
        }

        // Check for suspicious patterns in calldata
        let calldata_hex = hex::encode(call_data);
        for pattern in &self.phishing_patterns {
            if calldata_hex.contains(&pattern.replace(".*", "")) {
                return SecurityCheck {
                    check_name: "calldata_security".to_string(),
                    passed: false,
                    message: format!("Suspicious pattern detected in calldata: {}", pattern),
                    risk_level: SecurityRiskLevel::High,
                    context: Some(serde_json::json!({
                        "detected_pattern": pattern,
                        "risk_type": "suspicious_calldata_pattern"
                    })),
                };
            }
        }

        SecurityCheck {
            check_name: "calldata_security".to_string(),
            passed: true,
            message: format!("Calldata appears safe ({} bytes)", call_data.len()),
            risk_level: SecurityRiskLevel::Info,
            context: Some(serde_json::json!({
                "calldata_size": call_data.len()
            })),
        }
    }

    /// Check smart contract security and reputation
    async fn check_contract_security(
        &self,
        contract_address: &Address,
    ) -> GatewayResult<SecurityCheck> {
        // TODO: In production, query contract verification services
        // For now, use local reputation scores

        let reputation_score = self
            .contract_reputation
            .get(contract_address)
            .copied()
            .unwrap_or(50);

        if reputation_score < self.config.min_contract_reputation {
            Ok(SecurityCheck {
                check_name: "contract_security".to_string(),
                passed: false,
                message: format!(
                    "Contract {:?} has low reputation score {} (min required: {})",
                    contract_address, reputation_score, self.config.min_contract_reputation
                ),
                risk_level: SecurityRiskLevel::Medium,
                context: Some(serde_json::json!({
                    "contract_address": format!("{:?}", contract_address),
                    "reputation_score": reputation_score,
                    "min_required": self.config.min_contract_reputation,
                    "risk_type": "low_reputation_contract"
                })),
            })
        } else {
            Ok(SecurityCheck {
                check_name: "contract_security".to_string(),
                passed: true,
                message: format!(
                    "Contract {:?} has acceptable reputation score {}",
                    contract_address, reputation_score
                ),
                risk_level: SecurityRiskLevel::Info,
                context: Some(serde_json::json!({
                    "contract_address": format!("{:?}", contract_address),
                    "reputation_score": reputation_score
                })),
            })
        }
    }

    /// Check for suspicious transaction patterns
    fn check_transaction_patterns(&self, user_op: &UserOperationVariant) -> SecurityCheck {
        for pattern in &self.suspicious_patterns {
            if (pattern.pattern_matcher)(user_op) {
                return SecurityCheck {
                    check_name: "transaction_patterns".to_string(),
                    passed: false,
                    message: format!("Suspicious pattern detected: {}", pattern.description),
                    risk_level: if pattern.risk_score > 75 {
                        SecurityRiskLevel::Critical
                    } else if pattern.risk_score > 50 {
                        SecurityRiskLevel::High
                    } else {
                        SecurityRiskLevel::Medium
                    },
                    context: Some(serde_json::json!({
                        "pattern_name": pattern.name,
                        "risk_score": pattern.risk_score,
                        "risk_type": "suspicious_pattern"
                    })),
                };
            }
        }

        SecurityCheck {
            check_name: "transaction_patterns".to_string(),
            passed: true,
            message: "No suspicious transaction patterns detected".to_string(),
            risk_level: SecurityRiskLevel::Info,
            context: None,
        }
    }

    /// Check for phishing indicators
    fn check_phishing_indicators(&self, user_op: &UserOperationVariant) -> SecurityCheck {
        let call_data = match user_op {
            UserOperationVariant::V0_6(op) => op.call_data(),
            UserOperationVariant::V0_7(op) => op.call_data(),
        };

        let calldata_hex = hex::encode(call_data);

        // Check for common phishing patterns
        let phishing_indicators = vec![
            ("transfer_to_zero", "transfer.*0x000000"),
            ("approve_max", "approve.*ffffffff"),
            ("suspicious_multicall", "multicall.*batch"),
        ];

        for (indicator_name, pattern) in phishing_indicators {
            if calldata_hex.contains(&pattern.replace(".*", "")) {
                return SecurityCheck {
                    check_name: "phishing_detection".to_string(),
                    passed: false,
                    message: format!("Potential phishing indicator detected: {}", indicator_name),
                    risk_level: SecurityRiskLevel::High,
                    context: Some(serde_json::json!({
                        "indicator": indicator_name,
                        "pattern": pattern,
                        "risk_type": "phishing_attempt"
                    })),
                };
            }
        }

        SecurityCheck {
            check_name: "phishing_detection".to_string(),
            passed: true,
            message: "No phishing indicators detected".to_string(),
            risk_level: SecurityRiskLevel::Info,
            context: None,
        }
    }

    /// Check MEV protection measures
    fn check_mev_protection(&self, user_op: &UserOperationVariant) -> SecurityCheck {
        let (max_fee_per_gas, max_priority_fee) = match user_op {
            UserOperationVariant::V0_6(op) => (
                op.max_fee_per_gas().to_u64().unwrap_or(0),
                op.max_priority_fee_per_gas().to_u64().unwrap_or(0),
            ),
            UserOperationVariant::V0_7(op) => (
                op.max_fee_per_gas().to_u64().unwrap_or(0),
                op.max_priority_fee_per_gas().to_u64().unwrap_or(0),
            ),
        };

        // Check for unusually high priority fees (potential MEV extraction)
        let high_priority_threshold = 50_000_000_000u64; // 50 Gwei
        if max_priority_fee > high_priority_threshold {
            return SecurityCheck {
                check_name: "mev_protection".to_string(),
                passed: false,
                message: format!(
                    "Unusually high priority fee {} (threshold: {}) - potential MEV exploitation",
                    max_priority_fee, high_priority_threshold
                ),
                risk_level: SecurityRiskLevel::Medium,
                context: Some(serde_json::json!({
                    "max_priority_fee": max_priority_fee,
                    "threshold": high_priority_threshold,
                    "risk_type": "mev_exploitation"
                })),
            };
        }

        SecurityCheck {
            check_name: "mev_protection".to_string(),
            passed: true,
            message: format!(
                "Gas fees within normal range: max={}, priority={}",
                max_fee_per_gas, max_priority_fee
            ),
            risk_level: SecurityRiskLevel::Info,
            context: Some(serde_json::json!({
                "max_fee_per_gas": max_fee_per_gas,
                "max_priority_fee_per_gas": max_priority_fee
            })),
        }
    }

    /// Check init code security
    fn check_init_code_security(&self, user_op: &UserOperationVariant) -> SecurityCheck {
        let init_code = match user_op {
            UserOperationVariant::V0_6(op) => op.init_code().clone(),
            UserOperationVariant::V0_7(op) => {
                // v0.7 uses factory + factory_data
                if op.factory().is_some() {
                    op.factory_data().clone()
                } else {
                    return SecurityCheck {
                        check_name: "init_code_security".to_string(),
                        passed: true,
                        message: "No init code (account already deployed)".to_string(),
                        risk_level: SecurityRiskLevel::Info,
                        context: None,
                    };
                }
            }
        };

        // Check init code size
        if init_code.len() > self.config.max_init_code_size {
            return SecurityCheck {
                check_name: "init_code_security".to_string(),
                passed: false,
                message: format!(
                    "Init code size {} exceeds maximum allowed {} bytes",
                    init_code.len(),
                    self.config.max_init_code_size
                ),
                risk_level: SecurityRiskLevel::Medium,
                context: Some(serde_json::json!({
                    "init_code_size": init_code.len(),
                    "max_allowed": self.config.max_init_code_size,
                    "risk_type": "oversized_init_code"
                })),
            };
        }

        SecurityCheck {
            check_name: "init_code_security".to_string(),
            passed: true,
            message: format!("Init code size within limits ({} bytes)", init_code.len()),
            risk_level: SecurityRiskLevel::Info,
            context: Some(serde_json::json!({
                "init_code_size": init_code.len()
            })),
        }
    }

    /// Add malicious address to blacklist
    pub fn add_malicious_address(&mut self, address: Address) {
        self.malicious_addresses.insert(address);
        warn!(
            "Added malicious address {:?} to security blacklist",
            address
        );
    }

    /// Add phishing pattern
    pub fn add_phishing_pattern(&mut self, pattern: String) {
        self.phishing_patterns.push(pattern.clone());
        debug!("Added phishing pattern: {}", pattern);
    }

    /// Set contract reputation score
    pub fn set_contract_reputation(&mut self, contract: Address, score: u8) {
        self.contract_reputation.insert(contract, score);
        debug!("Set reputation score {} for contract {:?}", score, contract);
    }

    /// Extract paymaster address from UserOperation
    fn extract_paymaster_address(&self, user_op: &UserOperationVariant) -> Option<Address> {
        match user_op {
            UserOperationVariant::V0_6(op) => {
                let paymaster_and_data = op.paymaster_and_data();
                if paymaster_and_data.is_empty() {
                    None
                } else if paymaster_and_data.len() >= 20 {
                    Some(Address::from_slice(&paymaster_and_data[0..20]))
                } else {
                    None
                }
            }
            UserOperationVariant::V0_7(op) => op.paymaster(),
        }
    }

    /// Calculate anomaly score for the transaction
    fn calculate_anomaly_score(&self, _user_op: &UserOperationVariant) -> f64 {
        // TODO: Implement machine learning-based anomaly detection
        // For now, return a placeholder score
        0.1 // Low anomaly score
    }

    /// Assess phishing risk level
    fn assess_phishing_risk_level(&self, _user_op: &UserOperationVariant) -> String {
        // TODO: Implement sophisticated phishing risk assessment
        "Low".to_string()
    }

    /// Calculate contract risk score
    fn calculate_contract_risk_score(&self, contract: &Address) -> u8 {
        100 - self
            .contract_reputation
            .get(contract)
            .copied()
            .unwrap_or(50)
    }

    /// Get detected patterns
    fn get_detected_patterns(&self, _user_op: &UserOperationVariant) -> Vec<String> {
        // TODO: Return actual detected patterns
        vec!["Normal transaction pattern".to_string()]
    }

    /// Default suspicious patterns
    fn default_suspicious_patterns() -> Vec<TransactionPattern> {
        vec![
            TransactionPattern {
                name: "high_value_transfer".to_string(),
                description: "Unusually high value transfer".to_string(),
                risk_score: 60,
                pattern_matcher: |_user_op| {
                    // TODO: Implement pattern matching logic
                    false
                },
            },
            TransactionPattern {
                name: "rapid_nonce_increment".to_string(),
                description: "Rapid nonce increments suggesting automated abuse".to_string(),
                risk_score: 75,
                pattern_matcher: |_user_op| {
                    // TODO: Implement pattern matching logic
                    false
                },
            },
            TransactionPattern {
                name: "suspicious_multicall".to_string(),
                description: "Complex multicall with potential for abuse".to_string(),
                risk_score: 50,
                pattern_matcher: |_user_op| {
                    // TODO: Implement pattern matching logic
                    false
                },
            },
        ]
    }

    /// Process security check result and update tracking collections
    fn process_security_check(
        &self,
        check: SecurityCheck,
        check_results: &mut HashMap<String, SecurityCheck>,
        critical_violations: &mut Vec<String>,
        warnings: &mut Vec<String>,
        security_score: &mut u8,
    ) {
        let check_name = check.check_name.clone();

        match check.risk_level {
            SecurityRiskLevel::Critical if !check.passed => {
                critical_violations.push(check.message.clone());
                *security_score = security_score.saturating_sub(40);
            }
            SecurityRiskLevel::High if !check.passed => {
                critical_violations.push(check.message.clone());
                *security_score = security_score.saturating_sub(25);
            }
            SecurityRiskLevel::Medium if !check.passed => {
                warnings.push(check.message.clone());
                *security_score = security_score.saturating_sub(15);
            }
            SecurityRiskLevel::Low if !check.passed => {
                warnings.push(check.message.clone());
                *security_score = security_score.saturating_sub(5);
            }
            _ => {} // Info or successful checks don't affect score negatively
        }

        check_results.insert(check_name, check);
    }
}

impl Default for SecurityChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SecurityChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecurityChecker")
            .field("config", &self.config)
            .field("malicious_addresses", &self.malicious_addresses)
            .field("phishing_patterns", &self.phishing_patterns)
            .field("contract_reputation", &self.contract_reputation)
            .field("suspicious_patterns_count", &self.suspicious_patterns.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(config.enable_contract_verification);
        assert!(config.enable_pattern_analysis);
        assert!(config.enable_phishing_detection);
        assert!(!config.enable_anomaly_detection); // Disabled by default
        assert_eq!(config.max_call_gas_limit, 10_000_000);
        assert_eq!(config.max_calldata_size, 10_000);
    }

    #[test]
    fn test_malicious_address_detection() {
        let mut checker = SecurityChecker::new();
        let malicious_addr: Address = "0x0000000000000000000000000000000000000bad"
            .parse()
            .unwrap();

        checker.add_malicious_address(malicious_addr);
        assert!(checker.malicious_addresses.contains(&malicious_addr));
    }

    #[test]
    fn test_phishing_pattern_addition() {
        let mut checker = SecurityChecker::new();
        let pattern = "transfer.*0x00000000".to_string();

        checker.add_phishing_pattern(pattern.clone());
        assert!(checker.phishing_patterns.contains(&pattern));
    }

    #[test]
    fn test_contract_reputation_management() {
        let mut checker = SecurityChecker::new();
        let contract: Address = "0xA0b86a33E6441d6e42D2B6b63B3b66F2c7A77fD8"
            .parse()
            .unwrap();

        checker.set_contract_reputation(contract, 95);
        assert_eq!(checker.contract_reputation.get(&contract), Some(&95));
        assert_eq!(checker.calculate_contract_risk_score(&contract), 5);
    }

    #[test]
    fn test_security_risk_levels() {
        let critical_check = SecurityCheck {
            check_name: "test".to_string(),
            passed: false,
            message: "Critical issue".to_string(),
            risk_level: SecurityRiskLevel::Critical,
            context: None,
        };

        assert_eq!(critical_check.risk_level, SecurityRiskLevel::Critical);
    }
}
