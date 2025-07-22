//! Paymaster signing functionality for SuperRelay

use std::env;

use alloy_primitives::{Address, B256, U256};
use secrecy::{ExposeSecret, SecretString};

use crate::error::{Result, SigningError};

/// Paymaster signer for ERC-4337 UserOperations
pub struct PaymasterSigner {
    /// Private key for signing
    private_key: SecretString,
    /// Paymaster address derived from private key
    address: Address,
}

/// Paymaster signature data
#[derive(Debug, Clone)]
pub struct PaymasterSignature {
    /// The signature bytes
    pub signature: Vec<u8>,
    /// The paymaster address that signed
    pub paymaster: Address,
}

impl PaymasterSigner {
    /// Create a new PaymasterSigner from environment variable
    pub fn from_env(env_var_name: &str) -> Result<Self> {
        let private_key = env::var(env_var_name).map_err(|_| SigningError::KeyNotFound)?;

        if private_key.is_empty() {
            return Err(SigningError::KeyNotFound.into());
        }

        Self::from_private_key(&private_key)
    }

    /// Create a new PaymasterSigner from private key string
    pub fn from_private_key(private_key_str: &str) -> Result<Self> {
        // Remove 0x prefix if present
        let cleaned_key = private_key_str
            .strip_prefix("0x")
            .unwrap_or(private_key_str);

        // Validate key length (64 hex characters = 32 bytes)
        if cleaned_key.len() != 64 {
            return Err(SigningError::InvalidKeyFormat.into());
        }

        // Validate hex format
        if !cleaned_key.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(SigningError::InvalidKeyFormat.into());
        }

        // TODO: In real implementation, derive the address from private key
        // For now, use a placeholder address
        let address = Address::from([0u8; 20]); // Placeholder

        Ok(Self {
            private_key: private_key_str.into(),
            address,
        })
    }

    /// Get the paymaster address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Sign a UserOperation hash for ERC-4337 v0.6
    pub async fn sign_user_operation_v0_6(
        &self,
        user_op_hash: B256,
        chain_id: u64,
        entry_point: Address,
    ) -> Result<PaymasterSignature> {
        tracing::debug!(
            "Signing v0.6 UserOperation hash: {} for chain: {} at EntryPoint: {}",
            user_op_hash,
            chain_id,
            entry_point
        );

        // TODO: Implement actual EIP-712 signing for v0.6
        // For now, return a placeholder signature
        let signature = self.create_placeholder_signature().await?;

        Ok(PaymasterSignature {
            signature,
            paymaster: self.address,
        })
    }

    /// Sign a UserOperation hash for ERC-4337 v0.7
    pub async fn sign_user_operation_v0_7(
        &self,
        user_op_hash: B256,
        chain_id: u64,
        entry_point: Address,
    ) -> Result<PaymasterSignature> {
        tracing::debug!(
            "Signing v0.7 UserOperation hash: {} for chain: {} at EntryPoint: {}",
            user_op_hash,
            chain_id,
            entry_point
        );

        // TODO: Implement actual EIP-712 signing for v0.7
        // For now, return a placeholder signature
        let signature = self.create_placeholder_signature().await?;

        Ok(PaymasterSignature {
            signature,
            paymaster: self.address,
        })
    }

    /// Create paymaster data for UserOperation v0.6
    pub fn create_paymaster_data_v0_6(&self, signature: &PaymasterSignature) -> Vec<u8> {
        // ERC-4337 v0.6 paymaster data format:
        // paymaster (20 bytes) + signature (variable length)
        let mut data = Vec::new();
        data.extend_from_slice(signature.paymaster.as_slice());
        data.extend_from_slice(&signature.signature);
        data
    }

    /// Create paymaster data for UserOperation v0.7
    pub fn create_paymaster_data_v0_7(&self, signature: &PaymasterSignature) -> Vec<u8> {
        // ERC-4337 v0.7 paymaster data format may differ
        // For now, use the same format as v0.6
        self.create_paymaster_data_v0_6(signature)
    }

    /// Estimate gas required for paymaster verification
    pub fn estimate_verification_gas(&self) -> U256 {
        // Conservative estimate for paymaster verification gas
        U256::from(50_000)
    }

    /// Estimate gas required for paymaster post-operation
    pub fn estimate_post_op_gas(&self) -> U256 {
        // Conservative estimate for paymaster post-op gas
        U256::from(10_000)
    }

    /// Create a placeholder signature for testing
    async fn create_placeholder_signature(&self) -> Result<Vec<u8>> {
        // TODO: Replace with actual signing implementation
        // For now, return a fixed-length signature (65 bytes = r + s + v)
        Ok(vec![0u8; 65])
    }

    /// Verify that the signer can access the private key
    pub fn verify_key_access(&self) -> Result<()> {
        if self.private_key.expose_secret().is_empty() {
            return Err(SigningError::KeyNotFound.into());
        }
        Ok(())
    }
}

/// Utility functions for ERC-4337 signing
pub mod utils {
    use super::*;

    /// Calculate UserOperation hash for ERC-4337 v0.6
    pub fn calculate_user_op_hash_v0_6(
        _user_op: &rundler_types::v0_6::UserOperation,
        _entry_point: Address,
        _chain_id: u64,
    ) -> B256 {
        // TODO: Implement actual EIP-712 hash calculation for v0.6
        // This should follow the ERC-4337 specification
        B256::ZERO // Placeholder
    }

    /// Calculate UserOperation hash for ERC-4337 v0.7
    pub fn calculate_user_op_hash_v0_7(
        _user_op: &rundler_types::v0_7::UserOperation,
        _entry_point: Address,
        _chain_id: u64,
    ) -> B256 {
        // TODO: Implement actual EIP-712 hash calculation for v0.7
        // This should follow the ERC-4337 specification
        B256::ZERO // Placeholder
    }

    /// Validate signature format
    pub fn validate_signature(signature: &[u8]) -> bool {
        // Basic validation: signature should be 65 bytes (r + s + v)
        signature.len() == 65
    }

    /// Extract recovery ID from signature
    pub fn extract_recovery_id(signature: &[u8]) -> Option<u8> {
        if signature.len() >= 65 {
            Some(signature[64])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_creation_from_valid_key() {
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let signer = PaymasterSigner::from_private_key(private_key);
        assert!(signer.is_ok());
    }

    #[test]
    fn test_signer_creation_from_invalid_key() {
        let invalid_key = "invalid_key";
        let signer = PaymasterSigner::from_private_key(invalid_key);
        assert!(signer.is_err());
    }

    #[test]
    fn test_signer_with_0x_prefix() {
        let private_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let signer = PaymasterSigner::from_private_key(private_key);
        assert!(signer.is_ok());
    }

    #[tokio::test]
    async fn test_signing_operations() {
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let signer = PaymasterSigner::from_private_key(private_key).unwrap();

        let user_op_hash = B256::ZERO;
        let chain_id = 1;
        let entry_point = Address::ZERO;

        let signature_v06 = signer
            .sign_user_operation_v0_6(user_op_hash, chain_id, entry_point)
            .await;
        assert!(signature_v06.is_ok());

        let signature_v07 = signer
            .sign_user_operation_v0_7(user_op_hash, chain_id, entry_point)
            .await;
        assert!(signature_v07.is_ok());
    }

    #[test]
    fn test_paymaster_data_creation() {
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let signer = PaymasterSigner::from_private_key(private_key).unwrap();

        let signature = PaymasterSignature {
            signature: vec![0u8; 65],
            paymaster: signer.address(),
        };

        let data_v06 = signer.create_paymaster_data_v0_6(&signature);
        assert_eq!(data_v06.len(), 20 + 65); // 20 bytes address + 65 bytes signature

        let data_v07 = signer.create_paymaster_data_v0_7(&signature);
        assert_eq!(data_v07.len(), 20 + 65);
    }

    #[test]
    fn test_gas_estimates() {
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let signer = PaymasterSigner::from_private_key(private_key).unwrap();

        let verification_gas = signer.estimate_verification_gas();
        let post_op_gas = signer.estimate_post_op_gas();

        assert!(verification_gas > U256::ZERO);
        assert!(post_op_gas > U256::ZERO);
    }

    #[test]
    fn test_signature_validation() {
        let valid_signature = vec![0u8; 65];
        let invalid_signature = vec![0u8; 32];

        assert!(utils::validate_signature(&valid_signature));
        assert!(!utils::validate_signature(&invalid_signature));
    }

    #[test]
    fn test_recovery_id_extraction() {
        let signature = vec![0u8; 65];
        let recovery_id = utils::extract_recovery_id(&signature);
        assert_eq!(recovery_id, Some(0));

        let short_signature = vec![0u8; 32];
        let no_recovery_id = utils::extract_recovery_id(&short_signature);
        assert_eq!(no_recovery_id, None);
    }
}
