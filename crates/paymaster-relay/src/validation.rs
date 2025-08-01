// Enhanced input validation for PaymasterRelay service
use std::str::FromStr;

use alloy_primitives::U256;
use ethers::types::Address;
use rundler_types::UserOperationVariant;
use serde_json::Value;
use thiserror::Error;

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Gas limit too high: {limit} exceeds maximum {max}")]
    GasLimitTooHigh { limit: u128, max: u128 },

    #[error("Gas price too low: {price} below minimum {min}")]
    GasPriceTooLow { price: u128, min: u128 },

    #[error("Invalid user operation: {0}")]
    InvalidUserOperation(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid JSON structure: {0}")]
    InvalidJson(String),

    #[error("Value too large: {0}")]
    ValueTooLarge(String),

    #[error("Invalid signature format: {0}")]
    InvalidSignature(String),

    #[error("Suspicious pattern detected: {0}")]
    SuspiciousPattern(String),
}

/// Validation limits and constraints
#[derive(Debug, Clone)]
pub struct ValidationLimits {
    /// Maximum gas limit allowed
    pub max_gas_limit: u128,
    /// Minimum gas price (in wei)
    pub min_gas_price: u128,
    /// Maximum calldata size (bytes)
    pub max_calldata_size: usize,
    /// Maximum signature size (bytes)
    pub max_signature_size: usize,
    /// Maximum nonce value
    pub max_nonce: U256,
    /// Maximum preVerificationGas
    pub max_pre_verification_gas: u128,
    /// Maximum verificationGasLimit
    pub max_verification_gas_limit: u128,
    /// Maximum callGasLimit
    pub max_call_gas_limit: u128,
}

impl Default for ValidationLimits {
    fn default() -> Self {
        Self {
            max_gas_limit: 10_000_000,             // 10M gas
            min_gas_price: 1_000_000_000,          // 1 gwei
            max_calldata_size: 64 * 1024,          // 64KB
            max_signature_size: 1024,              // 1KB
            max_nonce: U256::from(u64::MAX),       // Max u64
            max_pre_verification_gas: 1_000_000,   // 1M gas
            max_verification_gas_limit: 5_000_000, // 5M gas
            max_call_gas_limit: 10_000_000,        // 10M gas
        }
    }
}

/// Enhanced input validator
#[derive(Debug)]
pub struct InputValidator {
    limits: ValidationLimits,
}

impl InputValidator {
    pub fn new(limits: ValidationLimits) -> Self {
        Self { limits }
    }

    /// Validate entry point address
    pub fn validate_entry_point(&self, entry_point_str: &str) -> Result<Address, ValidationError> {
        // Check for basic format
        if entry_point_str.is_empty() {
            return Err(ValidationError::MissingField("entry_point".to_string()));
        }

        // Parse as Ethereum address
        let address = Address::from_str(entry_point_str)
            .map_err(|e| ValidationError::InvalidAddress(format!("{}: {}", entry_point_str, e)))?;

        // Check if it's not zero address
        if address == Address::zero() {
            return Err(ValidationError::InvalidAddress(
                "Cannot be zero address".to_string(),
            ));
        }

        // Check for suspicious patterns (all same digit, etc.)
        if self.is_suspicious_address(&address) {
            return Err(ValidationError::SuspiciousPattern(format!(
                "Suspicious address pattern: {}",
                address
            )));
        }

        Ok(address)
    }

    /// Validate JSON user operation structure
    pub fn validate_user_operation_json(
        &self,
        user_op_json: &Value,
    ) -> Result<(), ValidationError> {
        let obj = user_op_json.as_object().ok_or_else(|| {
            ValidationError::InvalidJson("User operation must be an object".to_string())
        })?;

        // Required fields check
        let required_fields = ["sender", "nonce", "callData"];
        for field in &required_fields {
            if !obj.contains_key(*field) {
                return Err(ValidationError::MissingField(field.to_string()));
            }
        }

        // Validate sender address
        if let Some(sender) = obj.get("sender") {
            let sender_str = sender.as_str().ok_or_else(|| {
                ValidationError::InvalidJson("sender must be a string".to_string())
            })?;
            self.validate_address(sender_str, "sender")?;
        }

        // Validate nonce
        if let Some(nonce) = obj.get("nonce") {
            self.validate_nonce_value(nonce)?;
        }

        // Validate gas limits
        self.validate_gas_fields(obj)?;

        // Validate callData size
        if let Some(call_data) = obj.get("callData") {
            self.validate_call_data(call_data)?;
        }

        // Validate signature if present
        if let Some(signature) = obj.get("signature") {
            self.validate_signature(signature)?;
        }

        Ok(())
    }

    /// Validate converted UserOperation
    /// Note: This is a simplified validation as UserOperation fields are private
    /// The main validation happens at the JSON level before conversion
    pub fn validate_user_operation(
        &self,
        _user_op: &UserOperationVariant,
    ) -> Result<(), ValidationError> {
        // Since UserOperation fields are private in rundler_types,
        // we rely on the JSON validation before conversion.
        // This method is kept for future extensibility when field access becomes available.
        Ok(())
    }

    fn validate_address(
        &self,
        addr_str: &str,
        field_name: &str,
    ) -> Result<Address, ValidationError> {
        Address::from_str(addr_str).map_err(|e| {
            ValidationError::InvalidAddress(format!("{}: {} - {}", field_name, addr_str, e))
        })
    }

    fn validate_nonce_value(&self, nonce: &Value) -> Result<(), ValidationError> {
        match nonce {
            Value::String(s) => {
                // Try to parse as hex or decimal
                let parsed: Result<U256, _> = if s.starts_with("0x") {
                    U256::from_str(s).map_err(|e| format!("Invalid hex: {}", e))
                } else {
                    s.parse::<u64>()
                        .map(U256::from)
                        .map_err(|e| format!("Invalid decimal: {}", e))
                };

                let nonce_val = parsed.map_err(|_| {
                    ValidationError::InvalidAmount(format!("Invalid nonce format: {}", s))
                })?;

                if nonce_val > self.limits.max_nonce {
                    return Err(ValidationError::ValueTooLarge(format!(
                        "Nonce {} exceeds maximum {}",
                        nonce_val, self.limits.max_nonce
                    )));
                }
            }
            Value::Number(n) => {
                if let Some(nonce_u64) = n.as_u64() {
                    if U256::from(nonce_u64) > self.limits.max_nonce {
                        return Err(ValidationError::ValueTooLarge(format!(
                            "Nonce {} exceeds maximum {}",
                            nonce_u64, self.limits.max_nonce
                        )));
                    }
                } else {
                    return Err(ValidationError::InvalidAmount(
                        "Invalid nonce number".to_string(),
                    ));
                }
            }
            _ => {
                return Err(ValidationError::InvalidJson(
                    "Nonce must be string or number".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn validate_gas_fields(
        &self,
        obj: &serde_json::Map<String, Value>,
    ) -> Result<(), ValidationError> {
        let gas_fields = [
            ("callGasLimit", self.limits.max_call_gas_limit),
            (
                "verificationGasLimit",
                self.limits.max_verification_gas_limit,
            ),
            ("preVerificationGas", self.limits.max_pre_verification_gas),
        ];

        for (field_name, max_value) in &gas_fields {
            if let Some(gas_value) = obj.get(*field_name) {
                let gas_limit = self.parse_gas_value(gas_value, field_name)?;
                if gas_limit > *max_value {
                    return Err(ValidationError::GasLimitTooHigh {
                        limit: gas_limit,
                        max: *max_value,
                    });
                }
            }
        }

        Ok(())
    }

    fn parse_gas_value(&self, value: &Value, field_name: &str) -> Result<u128, ValidationError> {
        match value {
            Value::String(s) => if let Some(stripped) = s.strip_prefix("0x") {
                u128::from_str_radix(stripped, 16)
            } else {
                s.parse::<u128>()
            }
            .map_err(|_| {
                ValidationError::InvalidAmount(format!("Invalid {} format: {}", field_name, s))
            }),
            Value::Number(n) => n.as_u64().map(|v| v as u128).ok_or_else(|| {
                ValidationError::InvalidAmount(format!("Invalid {} number", field_name))
            }),
            _ => Err(ValidationError::InvalidJson(format!(
                "{} must be string or number",
                field_name
            ))),
        }
    }

    fn validate_call_data(&self, call_data: &Value) -> Result<(), ValidationError> {
        let call_data_str = call_data
            .as_str()
            .ok_or_else(|| ValidationError::InvalidJson("callData must be a string".to_string()))?;

        // Basic hex format check
        if !call_data_str.starts_with("0x") {
            return Err(ValidationError::InvalidJson(
                "callData must be hex string starting with 0x".to_string(),
            ));
        }

        // Check size (hex string length / 2 = byte length)
        let byte_length = (call_data_str.len() - 2) / 2;
        if byte_length > self.limits.max_calldata_size {
            return Err(ValidationError::ValueTooLarge(format!(
                "CallData size {} exceeds maximum {}",
                byte_length, self.limits.max_calldata_size
            )));
        }

        // Validate hex format
        if call_data_str[2..].chars().any(|c| !c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidJson(
                "callData contains invalid hex characters".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_signature(&self, signature: &Value) -> Result<(), ValidationError> {
        let sig_str = signature.as_str().ok_or_else(|| {
            ValidationError::InvalidSignature("Signature must be a string".to_string())
        })?;

        if !sig_str.starts_with("0x") {
            return Err(ValidationError::InvalidSignature(
                "Signature must be hex string starting with 0x".to_string(),
            ));
        }

        let byte_length = (sig_str.len() - 2) / 2;
        if byte_length > self.limits.max_signature_size {
            return Err(ValidationError::InvalidSignature(format!(
                "Signature size {} exceeds maximum {}",
                byte_length, self.limits.max_signature_size
            )));
        }

        Ok(())
    }

    fn is_suspicious_address(&self, address: &Address) -> bool {
        let addr_str = format!("{:?}", address);
        let hex_part = &addr_str[2..]; // Remove "0x" prefix

        // Check for patterns like all same digit
        let first_char = hex_part.chars().next().unwrap_or('0');
        if hex_part.chars().all(|c| c == first_char) {
            return true;
        }

        // Check for obvious test patterns
        let suspicious_patterns = [
            "1111111111111111111111111111111111111111",
            "2222222222222222222222222222222222222222",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ];

        suspicious_patterns
            .iter()
            .any(|pattern| hex_part.contains(pattern))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_validate_entry_point() {
        let validator = InputValidator::new(ValidationLimits::default());

        // Valid address
        assert!(validator
            .validate_entry_point("0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789")
            .is_ok());

        // Invalid address
        assert!(validator.validate_entry_point("invalid").is_err());

        // Zero address
        assert!(validator
            .validate_entry_point("0x0000000000000000000000000000000000000000")
            .is_err());
    }

    #[test]
    fn test_validate_user_operation_json() {
        let validator = InputValidator::new(ValidationLimits::default());

        let valid_op = json!({
            "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "nonce": "0x0",
            "callData": "0x",
            "callGasLimit": "100000",
            "verificationGasLimit": "100000",
            "preVerificationGas": "50000"
        });

        assert!(validator.validate_user_operation_json(&valid_op).is_ok());

        // Missing required field
        let invalid_op = json!({
            "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "nonce": "0x0"
            // Missing callData
        });

        assert!(validator.validate_user_operation_json(&invalid_op).is_err());
    }
}
