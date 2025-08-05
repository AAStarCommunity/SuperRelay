// paymaster-relay/src/signer.rs
// This file will implement the SignerManager for handling private keys.

use std::{collections::HashMap, str::FromStr};

use ethers::{
    signers::{LocalWallet, Signer},
    types::{Address, Signature, H256},
};
use eyre::Result;
use secrecy::{ExposeSecret, SecretString};
use tracing::{debug, info, warn};

use crate::kms::{KmsConfig, KmsError, KmsSigningRequest, MockKmsProvider, SigningContext};

/// Signer backend type
#[derive(Debug, Clone)]
pub enum SignerBackend {
    /// Direct private key (legacy mode)
    DirectKey(LocalWallet),
    /// KMS-based signing (enterprise mode)
    Kms(MockKmsProvider),
}

#[derive(Clone, Debug)]
pub struct SignerManager {
    backend: SignerBackend,
    /// Primary paymaster address
    primary_address: Address,
    /// Configuration metadata
    config_metadata: HashMap<String, String>,
}

impl SignerManager {
    /// Create SignerManager with direct private key (legacy mode)
    pub fn new(private_key: SecretString) -> Result<Self> {
        let signer = LocalWallet::from_str(private_key.expose_secret())?;
        let primary_address = signer.address();

        let mut config_metadata = HashMap::new();
        config_metadata.insert("backend_type".to_string(), "direct_key".to_string());
        config_metadata.insert("created_at".to_string(), chrono::Utc::now().to_rfc3339());

        Ok(Self {
            backend: SignerBackend::DirectKey(signer),
            primary_address,
            config_metadata,
        })
    }

    /// Create SignerManager with KMS backend (enterprise mode)
    pub fn new_with_kms(kms_config: KmsConfig) -> Result<Self, KmsError> {
        debug!("🔑 Initializing SignerManager with KMS backend");

        let kms_provider = MockKmsProvider::new(kms_config.clone())?;

        // Get primary key address from KMS
        let primary_key_info = kms_provider
            .get_key_info(&kms_config.primary_key_id)
            .map_err(|e| KmsError::InvalidConfiguration {
                reason: format!("Primary key not found: {}", e),
            })?;

        let primary_address = primary_key_info.address;

        let mut config_metadata = HashMap::new();
        config_metadata.insert("backend_type".to_string(), "kms".to_string());
        config_metadata.insert(
            "primary_key_id".to_string(),
            kms_config.primary_key_id.clone(),
        );
        config_metadata.insert(
            "backup_keys_count".to_string(),
            kms_config.backup_key_ids.len().to_string(),
        );
        config_metadata.insert(
            "kms_audit_enabled".to_string(),
            kms_config.enable_audit_logging.to_string(),
        );
        config_metadata.insert("created_at".to_string(), chrono::Utc::now().to_rfc3339());

        info!(
            "✅ SignerManager initialized with KMS: primary_address={:?}, backup_keys={}",
            primary_address,
            kms_config.backup_key_ids.len()
        );

        Ok(Self {
            backend: SignerBackend::Kms(kms_provider),
            primary_address,
            config_metadata,
        })
    }

    /// Sign a hash using the configured backend
    pub async fn sign_hash(&mut self, hash: [u8; 32]) -> Result<Signature> {
        self.sign_hash_with_context(hash, None).await
    }

    /// Sign a hash with additional context for audit logging
    pub async fn sign_hash_with_context(
        &mut self,
        hash: [u8; 32],
        context: Option<SigningContext>,
    ) -> Result<Signature> {
        match &mut self.backend {
            SignerBackend::DirectKey(signer) => {
                debug!("🔐 Signing with direct key: address={:?}", signer.address());
                let signature = signer.sign_hash(hash.into())?;
                Ok(signature)
            }
            SignerBackend::Kms(kms_provider) => {
                debug!("🔐 Signing with KMS provider");

                let signing_context = context.unwrap_or_else(|| SigningContext {
                    operation_type: "paymaster_operation".to_string(),
                    user_operation_hash: None,
                    sender_address: None,
                    entry_point: None,
                    gas_estimates: None,
                    metadata: HashMap::new(),
                });

                // Use the primary key ID from KMS provider's keys
                let primary_key_id = kms_provider
                    .list_keys()
                    .into_iter()
                    .find(|key| key.permissions.contains(&"sign".to_string()))
                    .ok_or_else(|| eyre::eyre!("No signing key found in KMS provider"))?
                    .key_id
                    .clone();

                let kms_request = KmsSigningRequest {
                    key_id: primary_key_id,
                    message_hash: H256::from_slice(&hash),
                    context: signing_context,
                };

                let kms_response = kms_provider
                    .sign(kms_request)
                    .await
                    .map_err(|e| eyre::eyre!("KMS signing failed: {}", e))?;

                debug!(
                    "✅ KMS signing completed: key_id={}, audit_id={}",
                    kms_response.key_id, kms_response.audit_info.request_id
                );

                Ok(kms_response.signature)
            }
        }
    }

    /// Get the primary paymaster address
    pub fn address(&self) -> Address {
        self.primary_address
    }

    /// Get signer backend type
    pub fn backend_type(&self) -> &str {
        match self.backend {
            SignerBackend::DirectKey(_) => "direct_key",
            SignerBackend::Kms(_) => "kms",
        }
    }

    /// Get configuration metadata
    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.config_metadata
    }

    /// Test KMS connectivity (only for KMS backend)
    pub async fn test_kms_connectivity(&self) -> Result<()> {
        match &self.backend {
            SignerBackend::DirectKey(_) => {
                debug!("📡 Connectivity test: Direct key backend (always available)");
                Ok(())
            }
            SignerBackend::Kms(kms_provider) => {
                debug!("📡 Testing KMS connectivity...");
                kms_provider
                    .test_connectivity()
                    .await
                    .map_err(|e| eyre::eyre!("KMS connectivity test failed: {}", e))?;
                info!("✅ KMS connectivity test passed");
                Ok(())
            }
        }
    }

    /// Get KMS audit log (only for KMS backend)
    pub fn get_kms_audit_log(&self) -> Option<Vec<crate::kms::SigningAuditInfo>> {
        match &self.backend {
            SignerBackend::DirectKey(_) => None,
            SignerBackend::Kms(kms_provider) => Some(kms_provider.get_audit_log().to_vec()),
        }
    }

    /// Rotate KMS key (only for KMS backend)
    pub async fn rotate_kms_key(&mut self, key_id: &str) -> Result<()> {
        match &mut self.backend {
            SignerBackend::DirectKey(_) => {
                warn!("🔄 Key rotation requested for direct key backend (not supported)");
                Err(eyre::eyre!(
                    "Key rotation not supported for direct key backend"
                ))
            }
            SignerBackend::Kms(kms_provider) => {
                info!("🔄 Initiating KMS key rotation: key_id={}", key_id);
                kms_provider
                    .rotate_key(key_id)
                    .await
                    .map_err(|e| eyre::eyre!("KMS key rotation failed: {}", e))?;
                info!("✅ KMS key rotation completed: key_id={}", key_id);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethers::types::H256;

    use super::*;
    use crate::kms::{GasEstimates, KmsConfig};

    #[tokio::test]
    async fn test_signer_manager_direct_key() {
        // A common test private key.
        // Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
        let private_key =
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string();
        let secret_key = SecretString::new(private_key.into());

        // 1. Create a new SignerManager with direct key
        let mut signer_manager =
            SignerManager::new(secret_key).expect("Failed to create signer manager");

        // 2. Check if the address is correct
        let expected_address =
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        assert_eq!(signer_manager.address(), expected_address);
        assert_eq!(signer_manager.backend_type(), "direct_key");

        // 3. Sign a sample hash
        let hash = H256::random().to_fixed_bytes();
        let signature = signer_manager
            .sign_hash(hash)
            .await
            .expect("Failed to sign hash");

        // 4. Verify the signature
        signature
            .verify(hash, expected_address)
            .expect("Signature verification failed");

        // 5. Test connectivity
        signer_manager
            .test_kms_connectivity()
            .await
            .expect("Connectivity test failed");
    }

    #[tokio::test]
    async fn test_signer_manager_kms() {
        // 1. Create KMS config
        let kms_config = KmsConfig::default();

        // 2. Create SignerManager with KMS backend
        let mut signer_manager =
            SignerManager::new_with_kms(kms_config).expect("Failed to create KMS signer manager");

        // 3. Check backend type
        assert_eq!(signer_manager.backend_type(), "kms");
        assert!(signer_manager.get_metadata().contains_key("primary_key_id"));

        // 4. Test connectivity
        signer_manager
            .test_kms_connectivity()
            .await
            .expect("KMS connectivity test failed");

        // 5. Sign a sample hash with context
        let hash = H256::random().to_fixed_bytes();
        let context = SigningContext {
            operation_type: "test_user_operation".to_string(),
            user_operation_hash: Some(H256::random()),
            sender_address: Some(Address::random()),
            entry_point: Some(Address::random()),
            gas_estimates: Some(GasEstimates {
                call_gas_limit: 100_000,
                verification_gas_limit: 100_000,
                pre_verification_gas: 21_000,
                max_fee_per_gas: 1_000_000_000,
                max_priority_fee_per_gas: 1_000_000_000,
            }),
            metadata: HashMap::new(),
        };

        let signature = signer_manager
            .sign_hash_with_context(hash, Some(context))
            .await
            .expect("Failed to sign hash with KMS");

        // 6. Verify signature is valid
        assert!(signature.r != ethers::types::U256::zero());
        assert!(signature.s != ethers::types::U256::zero());

        // 7. Check audit log
        let audit_log = signer_manager.get_kms_audit_log().unwrap();
        assert!(
            !audit_log.is_empty(),
            "Audit log should contain signing entry"
        );
        assert_eq!(
            audit_log[0].service_metadata.get("operation_type"),
            Some(&"test_user_operation".to_string())
        );

        // 8. Test key rotation
        signer_manager
            .rotate_kms_key("paymaster-primary-key")
            .await
            .expect("Key rotation should succeed");
    }

    #[tokio::test]
    async fn test_signer_manager_kms_failover() {
        // Test KMS with backup keys
        let kms_config = KmsConfig {
            backup_key_ids: vec!["backup-key-1".to_string(), "backup-key-2".to_string()],
            ..Default::default()
        };

        let signer_manager = SignerManager::new_with_kms(kms_config)
            .expect("Failed to create KMS signer manager with backups");

        assert_eq!(signer_manager.backend_type(), "kms");
        assert_eq!(
            signer_manager.get_metadata().get("backup_keys_count"),
            Some(&"2".to_string())
        );
    }
}
