use std::collections::HashMap;

use alloy_primitives::U256;
use num_traits::ToPrimitive;
use rundler_types::{v0_6, v0_7, UserOperation, UserOperationVariant};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::GatewayResult;
use crate::signature_validator::{SignatureValidator, SignatureValidationResult};

/// Data integrity validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataIntegrityResult {
    /// Overall validation status
    pub is_valid: bool,
    /// Validation score (0-100)
    pub validation_score: u8,
    /// Field-specific validation results
    pub field_validations: HashMap<String, FieldValidation>,
    /// Critical issues that must be addressed
    pub critical_issues: Vec<String>,
    /// Warnings that should be reviewed
    pub warnings: Vec<String>,
    /// Validation summary message
    pub summary: String,
}

/// Individual field validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidation {
    /// Field name
    pub field: String,
    /// Is field valid
    pub is_valid: bool,
    /// Field value (truncated if too long)
    pub value: String,
    /// Validation message
    pub message: String,
    /// Validation severity
    pub severity: ValidationSeverity,
}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    /// Critical error - blocks processing
    Critical,
    /// Error - should be fixed
    Error,
    /// Warning - should be reviewed
    Warning,
    /// Info - informational only
    Info,
}

/// Comprehensive data integrity checker for UserOperations
pub struct DataIntegrityChecker {
    /// Configuration for validation rules
    config: ValidationConfig,
    /// ECDSA signature validator
    signature_validator: SignatureValidator,
}

/// Configuration for data integrity validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Minimum gas limit allowed
    pub min_gas_limit: u64,
    /// Maximum gas limit allowed
    pub max_gas_limit: u64,
    /// Maximum call data size in bytes
    pub max_call_data_size: usize,
    /// Require valid signature format
    pub require_signature: bool,
    /// Check address format strictly
    pub strict_address_format: bool,
    /// Maximum number of verification gas units
    pub max_verification_gas: u64,
    /// Maximum number of pre-verification gas units
    pub max_pre_verification_gas: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            min_gas_limit: 21000,            // Minimum Ethereum transaction gas
            max_gas_limit: 30_000_000,       // Maximum reasonable gas limit
            max_call_data_size: 1024 * 1024, // 1MB max call data
            require_signature: true,
            strict_address_format: true,
            max_verification_gas: 5_000_000,
            max_pre_verification_gas: 1_000_000,
        }
    }
}

impl DataIntegrityChecker {
    /// Create a new data integrity checker with default configuration
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::default(),
            signature_validator: SignatureValidator::new(),
        }
    }

    /// Create a new data integrity checker with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { 
            config,
            signature_validator: SignatureValidator::new(),
        }
    }

    /// Create a new data integrity checker with custom signature validation settings
    pub fn with_signature_validation(config: ValidationConfig, lenient_signatures: bool) -> Self {
        let signature_validator = if lenient_signatures {
            SignatureValidator::lenient()
        } else {
            SignatureValidator::new()
        };
        
        Self { 
            config,
            signature_validator,
        }
    }

    /// Perform comprehensive data integrity validation on UserOperation
    pub async fn validate_user_operation(
        &self,
        user_op: &UserOperationVariant,
        entry_point: &str,
    ) -> GatewayResult<DataIntegrityResult> {
        debug!("🔍 Starting comprehensive data integrity validation");

        let mut field_validations = HashMap::new();
        let mut critical_issues = Vec::new();
        let mut warnings = Vec::new();
        let mut validation_score = 100u8;

        // Validate entry point address
        let entry_point_validation = self.validate_entry_point(entry_point);
        if !entry_point_validation.is_valid
            && entry_point_validation.severity == ValidationSeverity::Critical
        {
            critical_issues.push(entry_point_validation.message.clone());
            validation_score = validation_score.saturating_sub(20);
        } else if !entry_point_validation.is_valid {
            warnings.push(entry_point_validation.message.clone());
            validation_score = validation_score.saturating_sub(5);
        }
        field_validations.insert("entry_point".to_string(), entry_point_validation);

        // Validate UserOperation fields based on version
        match user_op {
            UserOperationVariant::V0_6(op) => {
                self.validate_v0_6_fields(
                    op,
                    &mut field_validations,
                    &mut critical_issues,
                    &mut warnings,
                    &mut validation_score,
                )
                .await?;
            }
            UserOperationVariant::V0_7(op) => {
                self.validate_v0_7_fields(
                    op,
                    &mut field_validations,
                    &mut critical_issues,
                    &mut warnings,
                    &mut validation_score,
                )
                .await?;
            }
        }

        // Cross-field validation
        self.validate_cross_field_consistency(
            user_op,
            &mut critical_issues,
            &mut warnings,
            &mut validation_score,
        )
        .await;

        // Determine overall validity
        let is_valid = critical_issues.is_empty();

        let summary = if is_valid {
            if warnings.is_empty() {
                format!(
                    "✅ UserOperation passes all data integrity checks (score: {})",
                    validation_score
                )
            } else {
                format!(
                    "⚠️ UserOperation valid with {} warnings (score: {})",
                    warnings.len(),
                    validation_score
                )
            }
        } else {
            format!(
                "❌ UserOperation failed data integrity validation: {} critical issues",
                critical_issues.len()
            )
        };

        debug!("Data integrity validation completed: {}", summary);

        Ok(DataIntegrityResult {
            is_valid,
            validation_score,
            field_validations,
            critical_issues,
            warnings,
            summary,
        })
    }

    /// Validate entry point address format and properties
    fn validate_entry_point(&self, entry_point: &str) -> FieldValidation {
        if entry_point.is_empty() {
            return FieldValidation {
                field: "entry_point".to_string(),
                is_valid: false,
                value: entry_point.to_string(),
                message: "Entry point address is required".to_string(),
                severity: ValidationSeverity::Critical,
            };
        }

        if !entry_point.starts_with("0x") || entry_point.len() != 42 {
            return FieldValidation {
                field: "entry_point".to_string(),
                is_valid: false,
                value: entry_point.to_string(),
                message: "Entry point must be a valid Ethereum address (0x + 40 hex chars)"
                    .to_string(),
                severity: ValidationSeverity::Critical,
            };
        }

        // Check if hex characters are valid
        if !entry_point[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return FieldValidation {
                field: "entry_point".to_string(),
                is_valid: false,
                value: entry_point.to_string(),
                message: "Entry point contains invalid hexadecimal characters".to_string(),
                severity: ValidationSeverity::Critical,
            };
        }

        FieldValidation {
            field: "entry_point".to_string(),
            is_valid: true,
            value: entry_point.to_string(),
            message: "Entry point address format is valid".to_string(),
            severity: ValidationSeverity::Info,
        }
    }

    /// Validate v0.6 UserOperation fields
    async fn validate_v0_6_fields(
        &self,
        op: &v0_6::UserOperation,
        field_validations: &mut HashMap<String, FieldValidation>,
        critical_issues: &mut Vec<String>,
        warnings: &mut Vec<String>,
        validation_score: &mut u8,
    ) -> GatewayResult<()> {
        debug!("Validating v0.6 UserOperation fields");

        // Validate sender
        let sender_validation = self.validate_address_field(
            "sender",
            &format!("{:?}", op.sender()),
            "Sender address validation",
        );
        self.process_field_validation(
            sender_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate nonce
        let nonce_validation = self.validate_nonce_field(op.nonce());
        self.process_field_validation(
            nonce_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate init code
        let init_code_validation = self.validate_init_code_field(&op.init_code().0);
        self.process_field_validation(
            init_code_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate call data
        let call_data_validation = self.validate_call_data_field(&op.call_data().0);
        self.process_field_validation(
            call_data_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate gas fields
        self.validate_v0_6_gas_fields(
            op,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate paymaster and data
        let paymaster_validation =
            self.validate_paymaster_and_data_field(&op.paymaster_and_data().0);
        self.process_field_validation(
            paymaster_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate signature
        let signature_validation = self.validate_signature_field(&op.signature().0).await;
        self.process_field_validation(
            signature_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        Ok(())
    }

    /// Validate v0.7 UserOperation fields
    async fn validate_v0_7_fields(
        &self,
        op: &v0_7::UserOperation,
        field_validations: &mut HashMap<String, FieldValidation>,
        critical_issues: &mut Vec<String>,
        warnings: &mut Vec<String>,
        validation_score: &mut u8,
    ) -> GatewayResult<()> {
        debug!("Validating v0.7 UserOperation fields");

        // Validate sender
        let sender_validation = self.validate_address_field(
            "sender",
            &format!("{:?}", op.sender()),
            "Sender address validation",
        );
        self.process_field_validation(
            sender_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate nonce
        let nonce_validation = self.validate_nonce_field(op.nonce());
        self.process_field_validation(
            nonce_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate factory and factory data (v0.7 specific)
        if let Some(factory) = op.factory() {
            let factory_validation = self.validate_address_field(
                "factory",
                &format!("{:?}", factory),
                "Factory address validation",
            );
            self.process_field_validation(
                factory_validation,
                field_validations,
                critical_issues,
                warnings,
                validation_score,
            );
        }

        let factory_data = op.factory_data();
        if !factory_data.is_empty() {
            let factory_data_validation = self.validate_factory_data_field(&factory_data.0);
            self.process_field_validation(
                factory_data_validation,
                field_validations,
                critical_issues,
                warnings,
                validation_score,
            );
        }

        // Validate call data
        let call_data_validation = self.validate_call_data_field(&op.call_data().0);
        self.process_field_validation(
            call_data_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate gas fields
        self.validate_v0_7_gas_fields(
            op,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate paymaster fields (v0.7 specific)
        if let Some(paymaster) = op.paymaster() {
            let paymaster_validation = self.validate_address_field(
                "paymaster",
                &format!("{:?}", paymaster),
                "Paymaster address validation",
            );
            self.process_field_validation(
                paymaster_validation,
                field_validations,
                critical_issues,
                warnings,
                validation_score,
            );
        }

        // Validate signature
        let signature_validation = self.validate_signature_field(&op.signature().0).await;
        self.process_field_validation(
            signature_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        Ok(())
    }

    /// Validate gas fields for v0.6 UserOperation
    fn validate_v0_6_gas_fields(
        &self,
        op: &v0_6::UserOperation,
        field_validations: &mut HashMap<String, FieldValidation>,
        critical_issues: &mut Vec<String>,
        warnings: &mut Vec<String>,
        validation_score: &mut u8,
    ) {
        // Validate call gas limit
        let call_gas_validation = self.validate_gas_field(
            "call_gas_limit",
            op.call_gas_limit().to_u64().unwrap_or(u64::MAX),
            self.config.min_gas_limit,
            self.config.max_gas_limit,
        );
        self.process_field_validation(
            call_gas_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate verification gas limit
        let verification_gas_validation = self.validate_gas_field(
            "verification_gas_limit",
            op.verification_gas_limit().to_u64().unwrap_or(u64::MAX),
            0,
            self.config.max_verification_gas,
        );
        self.process_field_validation(
            verification_gas_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate pre-verification gas
        let pre_verification_gas_validation = self.validate_gas_field(
            "pre_verification_gas",
            op.pre_verification_gas().to_u64().unwrap_or(u64::MAX),
            0,
            self.config.max_pre_verification_gas,
        );
        self.process_field_validation(
            pre_verification_gas_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate max fee per gas
        let max_fee_validation = self.validate_fee_field(
            "max_fee_per_gas",
            op.max_fee_per_gas().to_u64().unwrap_or(u64::MAX),
        );
        self.process_field_validation(
            max_fee_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        // Validate max priority fee per gas
        let max_priority_fee_validation = self.validate_fee_field(
            "max_priority_fee_per_gas",
            op.max_priority_fee_per_gas().to_u64().unwrap_or(u64::MAX),
        );
        self.process_field_validation(
            max_priority_fee_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );
    }

    /// Validate gas fields for v0.7 UserOperation
    fn validate_v0_7_gas_fields(
        &self,
        op: &v0_7::UserOperation,
        field_validations: &mut HashMap<String, FieldValidation>,
        critical_issues: &mut Vec<String>,
        warnings: &mut Vec<String>,
        validation_score: &mut u8,
    ) {
        // Similar to v0.6 but with potential differences in v0.7 gas calculation
        let call_gas_validation = self.validate_gas_field(
            "call_gas_limit",
            op.call_gas_limit().to_u64().unwrap_or(u64::MAX),
            self.config.min_gas_limit,
            self.config.max_gas_limit,
        );
        self.process_field_validation(
            call_gas_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        let verification_gas_validation = self.validate_gas_field(
            "verification_gas_limit",
            op.verification_gas_limit().to_u64().unwrap_or(u64::MAX),
            0,
            self.config.max_verification_gas,
        );
        self.process_field_validation(
            verification_gas_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        let pre_verification_gas_validation = self.validate_gas_field(
            "pre_verification_gas",
            op.pre_verification_gas().to_u64().unwrap_or(u64::MAX),
            0,
            self.config.max_pre_verification_gas,
        );
        self.process_field_validation(
            pre_verification_gas_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        let max_fee_validation = self.validate_fee_field(
            "max_fee_per_gas",
            op.max_fee_per_gas().to_u64().unwrap_or(u64::MAX),
        );
        self.process_field_validation(
            max_fee_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );

        let max_priority_fee_validation = self.validate_fee_field(
            "max_priority_fee_per_gas",
            op.max_priority_fee_per_gas().to_u64().unwrap_or(u64::MAX),
        );
        self.process_field_validation(
            max_priority_fee_validation,
            field_validations,
            critical_issues,
            warnings,
            validation_score,
        );
    }

    /// Validate address field format
    fn validate_address_field(
        &self,
        field_name: &str,
        address: &str,
        context: &str,
    ) -> FieldValidation {
        if address.is_empty() {
            return FieldValidation {
                field: field_name.to_string(),
                is_valid: false,
                value: address.to_string(),
                message: format!("{}: Address is required", context),
                severity: ValidationSeverity::Critical,
            };
        }

        if self.config.strict_address_format {
            if !address.starts_with("0x") || address.len() != 42 {
                return FieldValidation {
                    field: field_name.to_string(),
                    is_valid: false,
                    value: address.to_string(),
                    message: format!(
                        "{}: Invalid address format (expected 0x + 40 hex chars)",
                        context
                    ),
                    severity: ValidationSeverity::Critical,
                };
            }

            if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                return FieldValidation {
                    field: field_name.to_string(),
                    is_valid: false,
                    value: address.to_string(),
                    message: format!(
                        "{}: Address contains invalid hexadecimal characters",
                        context
                    ),
                    severity: ValidationSeverity::Critical,
                };
            }
        }

        FieldValidation {
            field: field_name.to_string(),
            is_valid: true,
            value: address.to_string(),
            message: format!("{}: Address format is valid", context),
            severity: ValidationSeverity::Info,
        }
    }

    /// Validate nonce field
    fn validate_nonce_field(&self, nonce: U256) -> FieldValidation {
        // Nonce should be reasonable (not extremely high)
        let max_reasonable_nonce = U256::from(1_000_000u64);

        if nonce > max_reasonable_nonce {
            FieldValidation {
                field: "nonce".to_string(),
                is_valid: true,
                value: format!("{}", nonce),
                message: "Nonce is unusually high, verify correctness".to_string(),
                severity: ValidationSeverity::Warning,
            }
        } else {
            FieldValidation {
                field: "nonce".to_string(),
                is_valid: true,
                value: format!("{}", nonce),
                message: "Nonce value is within expected range".to_string(),
                severity: ValidationSeverity::Info,
            }
        }
    }

    /// Validate init code field
    fn validate_init_code_field(&self, init_code: &[u8]) -> FieldValidation {
        if init_code.len() > self.config.max_call_data_size {
            return FieldValidation {
                field: "init_code".to_string(),
                is_valid: false,
                value: format!(
                    "0x{}...",
                    hex::encode(&init_code[..20.min(init_code.len())])
                ),
                message: format!(
                    "Init code exceeds maximum size limit ({} bytes)",
                    self.config.max_call_data_size
                ),
                severity: ValidationSeverity::Critical,
            };
        }

        FieldValidation {
            field: "init_code".to_string(),
            is_valid: true,
            value: if init_code.is_empty() {
                "0x".to_string()
            } else {
                format!(
                    "0x{}... ({} bytes)",
                    hex::encode(&init_code[..10.min(init_code.len())]),
                    init_code.len()
                )
            },
            message: format!("Init code size is acceptable ({} bytes)", init_code.len()),
            severity: ValidationSeverity::Info,
        }
    }

    /// Validate call data field
    fn validate_call_data_field(&self, call_data: &[u8]) -> FieldValidation {
        if call_data.is_empty() {
            return FieldValidation {
                field: "call_data".to_string(),
                is_valid: false,
                value: "0x".to_string(),
                message: "Call data is required for UserOperation execution".to_string(),
                severity: ValidationSeverity::Critical,
            };
        }

        if call_data.len() > self.config.max_call_data_size {
            return FieldValidation {
                field: "call_data".to_string(),
                is_valid: false,
                value: format!(
                    "0x{}...",
                    hex::encode(&call_data[..20.min(call_data.len())])
                ),
                message: format!(
                    "Call data exceeds maximum size limit ({} bytes)",
                    self.config.max_call_data_size
                ),
                severity: ValidationSeverity::Critical,
            };
        }

        FieldValidation {
            field: "call_data".to_string(),
            is_valid: true,
            value: format!(
                "0x{}... ({} bytes)",
                hex::encode(&call_data[..10.min(call_data.len())]),
                call_data.len()
            ),
            message: format!("Call data size is acceptable ({} bytes)", call_data.len()),
            severity: ValidationSeverity::Info,
        }
    }

    /// Validate factory data field (v0.7 specific)
    fn validate_factory_data_field(&self, factory_data: &[u8]) -> FieldValidation {
        if factory_data.len() > self.config.max_call_data_size {
            return FieldValidation {
                field: "factory_data".to_string(),
                is_valid: false,
                value: format!(
                    "0x{}...",
                    hex::encode(&factory_data[..20.min(factory_data.len())])
                ),
                message: format!(
                    "Factory data exceeds maximum size limit ({} bytes)",
                    self.config.max_call_data_size
                ),
                severity: ValidationSeverity::Critical,
            };
        }

        FieldValidation {
            field: "factory_data".to_string(),
            is_valid: true,
            value: if factory_data.is_empty() {
                "0x".to_string()
            } else {
                format!(
                    "0x{}... ({} bytes)",
                    hex::encode(&factory_data[..10.min(factory_data.len())]),
                    factory_data.len()
                )
            },
            message: format!(
                "Factory data size is acceptable ({} bytes)",
                factory_data.len()
            ),
            severity: ValidationSeverity::Info,
        }
    }

    /// Validate gas field
    fn validate_gas_field(
        &self,
        field_name: &str,
        gas_value: u64,
        min_gas: u64,
        max_gas: u64,
    ) -> FieldValidation {
        if gas_value < min_gas {
            return FieldValidation {
                field: field_name.to_string(),
                is_valid: false,
                value: gas_value.to_string(),
                message: format!(
                    "Gas value {} is below minimum required ({})",
                    gas_value, min_gas
                ),
                severity: ValidationSeverity::Critical,
            };
        }

        if gas_value > max_gas {
            return FieldValidation {
                field: field_name.to_string(),
                is_valid: false,
                value: gas_value.to_string(),
                message: format!(
                    "Gas value {} exceeds maximum allowed ({})",
                    gas_value, max_gas
                ),
                severity: ValidationSeverity::Critical,
            };
        }

        FieldValidation {
            field: field_name.to_string(),
            is_valid: true,
            value: gas_value.to_string(),
            message: format!("Gas value {} is within acceptable range", gas_value),
            severity: ValidationSeverity::Info,
        }
    }

    /// Validate fee field
    fn validate_fee_field(&self, field_name: &str, fee_value: u64) -> FieldValidation {
        if fee_value == 0 {
            return FieldValidation {
                field: field_name.to_string(),
                is_valid: false,
                value: fee_value.to_string(),
                message: "Fee cannot be zero for transaction processing".to_string(),
                severity: ValidationSeverity::Critical,
            };
        }

        // Check for extremely high fees (potential mistake)
        let max_reasonable_fee = 1_000_000_000_000u64; // 1000 Gwei
        if fee_value > max_reasonable_fee {
            return FieldValidation {
                field: field_name.to_string(),
                is_valid: true,
                value: fee_value.to_string(),
                message: format!("Fee {} is extremely high, verify correctness", fee_value),
                severity: ValidationSeverity::Warning,
            };
        }

        FieldValidation {
            field: field_name.to_string(),
            is_valid: true,
            value: fee_value.to_string(),
            message: format!("Fee value {} is reasonable", fee_value),
            severity: ValidationSeverity::Info,
        }
    }

    /// Validate paymaster and data field (v0.6)
    fn validate_paymaster_and_data_field(&self, paymaster_data: &[u8]) -> FieldValidation {
        if paymaster_data.len() > self.config.max_call_data_size {
            return FieldValidation {
                field: "paymaster_and_data".to_string(),
                is_valid: false,
                value: format!(
                    "0x{}...",
                    hex::encode(&paymaster_data[..20.min(paymaster_data.len())])
                ),
                message: format!(
                    "Paymaster data exceeds maximum size limit ({} bytes)",
                    self.config.max_call_data_size
                ),
                severity: ValidationSeverity::Critical,
            };
        }

        FieldValidation {
            field: "paymaster_and_data".to_string(),
            is_valid: true,
            value: if paymaster_data.is_empty() {
                "0x".to_string()
            } else {
                format!(
                    "0x{}... ({} bytes)",
                    hex::encode(&paymaster_data[..10.min(paymaster_data.len())]),
                    paymaster_data.len()
                )
            },
            message: format!(
                "Paymaster data size is acceptable ({} bytes)",
                paymaster_data.len()
            ),
            severity: ValidationSeverity::Info,
        }
    }

    /// Validate signature field using enhanced ECDSA validation
    async fn validate_signature_field(&self, signature: &[u8]) -> FieldValidation {
        if self.config.require_signature && signature.is_empty() {
            return FieldValidation {
                field: "signature".to_string(),
                is_valid: false,
                value: "0x".to_string(),
                message: "Signature is required for UserOperation validation".to_string(),
                severity: ValidationSeverity::Critical,
            };
        }

        if signature.is_empty() {
            return FieldValidation {
                field: "signature".to_string(),
                is_valid: true,
                value: "0x".to_string(),
                message: "Empty signature (acceptable for certain UserOperation types)".to_string(),
                severity: ValidationSeverity::Info,
            };
        }

        // Use enhanced ECDSA signature validation
        match self.signature_validator.validate_signature(signature).await {
            Ok(result) => {
                let value = format!(
                    "0x{}... ({} bytes, format: {:?})",
                    hex::encode(&signature[..10.min(signature.len())]),
                    signature.len(),
                    result.signature_format
                );

                let mut message = result.message.clone();
                if !result.security_issues.is_empty() {
                    message.push_str(&format!(
                        " | Security issues: {}",
                        result.security_issues.join("; ")
                    ));
                }

                FieldValidation {
                    field: "signature".to_string(),
                    is_valid: result.is_valid,
                    value,
                    message,
                    severity: result.severity,
                }
            }
            Err(e) => {
                FieldValidation {
                    field: "signature".to_string(),
                    is_valid: false,
                    value: format!(
                        "0x{}... ({} bytes)",
                        hex::encode(&signature[..10.min(signature.len())]),
                        signature.len()
                    ),
                    message: format!("Signature validation failed: {}", e),
                    severity: ValidationSeverity::Critical,
                }
            }
        }
    }

    /// Validate cross-field consistency
    async fn validate_cross_field_consistency(
        &self,
        user_op: &UserOperationVariant,
        critical_issues: &mut Vec<String>,
        warnings: &mut Vec<String>,
        validation_score: &mut u8,
    ) {
        debug!("Performing cross-field consistency validation");

        // Check gas consistency
        match user_op {
            UserOperationVariant::V0_6(op) => {
                let max_fee = op.max_fee_per_gas().to_u64().unwrap_or(u64::MAX);
                let max_priority_fee = op.max_priority_fee_per_gas().to_u64().unwrap_or(u64::MAX);

                if max_priority_fee > max_fee {
                    critical_issues
                        .push("Max priority fee per gas cannot exceed max fee per gas".to_string());
                    *validation_score = validation_score.saturating_sub(15);
                }
            }
            UserOperationVariant::V0_7(op) => {
                let max_fee = op.max_fee_per_gas().to_u64().unwrap_or(u64::MAX);
                let max_priority_fee = op.max_priority_fee_per_gas().to_u64().unwrap_or(u64::MAX);

                if max_priority_fee > max_fee {
                    critical_issues
                        .push("Max priority fee per gas cannot exceed max fee per gas".to_string());
                    *validation_score = validation_score.saturating_sub(15);
                }
            }
        }

        // Additional consistency checks can be added here
        // For example: init_code consistency with factory fields in v0.7
        if let UserOperationVariant::V0_7(op) = user_op {
            let has_factory = op.factory().is_some();
            let has_factory_data = !op.factory_data().is_empty();

            if has_factory != has_factory_data {
                warnings.push(
                    "Factory and factory_data should both be present or both be absent".to_string(),
                );
                *validation_score = validation_score.saturating_sub(5);
            }
        }
    }

    /// Process field validation result and update tracking collections
    fn process_field_validation(
        &self,
        validation: FieldValidation,
        field_validations: &mut HashMap<String, FieldValidation>,
        critical_issues: &mut Vec<String>,
        warnings: &mut Vec<String>,
        validation_score: &mut u8,
    ) {
        let field_name = validation.field.clone();

        match validation.severity {
            ValidationSeverity::Critical if !validation.is_valid => {
                critical_issues.push(validation.message.clone());
                *validation_score = validation_score.saturating_sub(20);
            }
            ValidationSeverity::Error if !validation.is_valid => {
                warnings.push(validation.message.clone());
                *validation_score = validation_score.saturating_sub(10);
            }
            ValidationSeverity::Warning => {
                warnings.push(validation.message.clone());
                *validation_score = validation_score.saturating_sub(5);
            }
            _ => {} // Info or successful validations don't affect score negatively
        }

        field_validations.insert(field_name, validation);
    }
}

impl Default for DataIntegrityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert_eq!(config.min_gas_limit, 21000);
        assert_eq!(config.max_gas_limit, 30_000_000);
        assert!(config.require_signature);
        assert!(config.strict_address_format);
    }

    #[test]
    fn test_entry_point_validation() {
        let checker = DataIntegrityChecker::new();

        // Valid entry point
        let valid_result =
            checker.validate_entry_point("0x1234567890123456789012345678901234567890");
        assert!(valid_result.is_valid);

        // Invalid entry point - wrong length
        let invalid_result = checker.validate_entry_point("0x123");
        assert!(!invalid_result.is_valid);
        assert_eq!(invalid_result.severity, ValidationSeverity::Critical);

        // Invalid entry point - no 0x prefix
        let invalid_result2 =
            checker.validate_entry_point("1234567890123456789012345678901234567890");
        assert!(!invalid_result2.is_valid);
        assert_eq!(invalid_result2.severity, ValidationSeverity::Critical);
    }

    #[test]
    fn test_gas_field_validation() {
        let checker = DataIntegrityChecker::new();

        // Valid gas
        let valid_result = checker.validate_gas_field("test_gas", 100000, 21000, 1000000);
        assert!(valid_result.is_valid);

        // Too low gas
        let low_result = checker.validate_gas_field("test_gas", 10000, 21000, 1000000);
        assert!(!low_result.is_valid);
        assert_eq!(low_result.severity, ValidationSeverity::Critical);

        // Too high gas
        let high_result = checker.validate_gas_field("test_gas", 2000000, 21000, 1000000);
        assert!(!high_result.is_valid);
        assert_eq!(high_result.severity, ValidationSeverity::Critical);
    }

    #[test]
    fn test_fee_field_validation() {
        let checker = DataIntegrityChecker::new();

        // Valid fee
        let valid_result = checker.validate_fee_field("test_fee", 1000000000); // 1 Gwei
        assert!(valid_result.is_valid);

        // Zero fee
        let zero_result = checker.validate_fee_field("test_fee", 0);
        assert!(!zero_result.is_valid);
        assert_eq!(zero_result.severity, ValidationSeverity::Critical);

        // Extremely high fee
        let high_result = checker.validate_fee_field("test_fee", 2_000_000_000_000u64);
        assert!(high_result.is_valid); // Still valid but with warning
        assert_eq!(high_result.severity, ValidationSeverity::Warning);
    }
}
