use std::collections::HashMap;

use async_trait::async_trait;
use ethers::{
    signers::Signer,
    types::{Address, Signature, H256},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

/// KMS-related errors
#[derive(Error, Debug)]
pub enum KmsError {
    #[error("KMS key not found: {key_id}")]
    KeyNotFound { key_id: String },
    #[error("KMS service unavailable: {reason}")]
    ServiceUnavailable { reason: String },
    #[error("KMS authentication failed: {reason}")]
    AuthenticationFailed { reason: String },
    #[error("KMS signature operation failed: {reason}")]
    SignatureFailed { reason: String },
    #[error("KMS hardware wallet error: {reason}")]
    HardwareWalletError { reason: String },
    #[error("Invalid KMS configuration: {reason}")]
    InvalidConfiguration { reason: String },
}

/// KMS key types supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KmsKeyType {
    /// AWS KMS managed key
    AwsKms,
    /// Azure Key Vault managed key
    AzureKeyVault,
    /// Google Cloud KMS managed key
    GoogleCloudKms,
    /// Hardware Security Module (HSM)
    HardwareSecurityModule,
    /// Hardware wallet (e.g., Ledger, Trezor)
    HardwareWallet,
    /// Software-based key (for testing)
    SoftwareKey,
}

/// KMS key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KmsKeyInfo {
    /// Unique key identifier
    pub key_id: String,
    /// Key type
    pub key_type: KmsKeyType,
    /// Ethereum address derived from key
    pub address: Address,
    /// Key description
    pub description: String,
    /// Whether key is enabled for signing
    pub enabled: bool,
    /// Key usage permissions
    pub permissions: Vec<String>,
    /// Key metadata
    pub metadata: HashMap<String, String>,
}

/// KMS signing request
#[derive(Debug, Clone)]
pub struct KmsSigningRequest {
    /// Key ID to use for signing
    pub key_id: String,
    /// Hash to sign (32 bytes)
    pub message_hash: H256,
    /// Additional context for audit logging
    pub context: SigningContext,
}

/// Context information for signing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningContext {
    /// Operation type (e.g., "user_operation", "transaction")
    pub operation_type: String,
    /// User operation hash for audit trail
    pub user_operation_hash: Option<H256>,
    /// Sender address
    pub sender_address: Option<Address>,
    /// Entry point address
    pub entry_point: Option<Address>,
    /// Gas estimates for the operation
    pub gas_estimates: Option<GasEstimates>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Gas estimates for signing context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimates {
    pub call_gas_limit: u128,
    pub verification_gas_limit: u128,
    pub pre_verification_gas: u128,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
}

/// KMS signing response
#[derive(Debug, Clone)]
pub struct KmsSigningResponse {
    /// Signature produced by KMS
    pub signature: Signature,
    /// Key ID used for signing
    pub key_id: String,
    /// Signature timestamp
    pub timestamp: i64,
    /// Audit information
    pub audit_info: SigningAuditInfo,
}

/// Audit information for signing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningAuditInfo {
    /// Request ID for tracking
    pub request_id: String,
    /// KMS service response metadata
    pub service_metadata: HashMap<String, String>,
    /// Signing duration in milliseconds
    pub duration_ms: u64,
    /// Whether hardware validation was performed
    pub hardware_validated: bool,
}

/// KMS configuration
#[derive(Debug, Clone)]
pub struct KmsConfig {
    /// Primary key ID for paymaster operations
    pub primary_key_id: String,
    /// Backup key IDs for failover
    pub backup_key_ids: Vec<String>,
    /// KMS service endpoint (for cloud providers)
    pub service_endpoint: Option<String>,
    /// Authentication credentials
    pub credentials: KmsCredentials,
    /// Signing timeout in seconds
    pub signing_timeout_seconds: u32,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Maximum signing requests per minute
    pub rate_limit_per_minute: u32,
}

/// KMS authentication credentials
#[derive(Debug, Clone)]
pub struct KmsCredentials {
    /// Access key or client ID
    pub access_key: String,
    /// Secret key or client secret
    pub secret_key: String,
    /// Region or tenant ID
    pub region_or_tenant: String,
    /// Additional auth parameters
    pub additional_params: HashMap<String, String>,
}

/// KMS provider trait for different implementations
#[async_trait]
pub trait KmsProvider {
    /// Sign a message hash with the specified key
    async fn sign(
        &mut self,
        request: KmsSigningRequest,
        context: SigningContext,
    ) -> Result<KmsSigningResponse, KmsError>;

    /// Get the Ethereum address for a KMS key
    async fn get_key_address(&self, key_id: &str) -> Result<Address, KmsError>;

    /// List available keys
    async fn list_keys(&self) -> Result<Vec<KmsKeyInfo>, KmsError>;

    /// Perform health check on KMS provider
    async fn health_check(&self) -> Result<bool, KmsError>;
}

/// Mock KMS implementation for testing and development
#[derive(Debug, Clone)]
pub struct MockKmsProvider {
    /// Available keys
    keys: HashMap<String, KmsKeyInfo>,
    /// Simulated signing keys (for testing only)
    signing_keys: HashMap<String, ethers::signers::LocalWallet>,
    /// Configuration
    config: KmsConfig,
    /// Audit log
    audit_log: Vec<SigningAuditInfo>,
}

impl MockKmsProvider {
    /// Create a new mock KMS provider
    pub fn new(config: KmsConfig) -> Result<Self, KmsError> {
        let mut provider = Self {
            keys: HashMap::new(),
            signing_keys: HashMap::new(),
            config,
            audit_log: Vec::new(),
        };

        // Initialize with default test keys
        provider.initialize_test_keys()?;

        Ok(provider)
    }

    /// Initialize test keys for development
    fn initialize_test_keys(&mut self) -> Result<(), KmsError> {
        // Add primary test key (matches Anvil's first account)
        let primary_private_key =
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let primary_wallet = ethers::signers::LocalWallet::from(
            ethers::core::k256::ecdsa::SigningKey::from_slice(
                &hex::decode(primary_private_key).map_err(|e| KmsError::InvalidConfiguration {
                    reason: format!("Invalid primary key: {}", e),
                })?,
            )
            .map_err(|e| KmsError::InvalidConfiguration {
                reason: format!("Invalid signing key: {}", e),
            })?,
        );

        let primary_key_info = KmsKeyInfo {
            key_id: self.config.primary_key_id.clone(),
            key_type: KmsKeyType::SoftwareKey,
            address: primary_wallet.address(),
            description: "Primary paymaster signing key (test)".to_string(),
            enabled: true,
            permissions: vec!["sign".to_string(), "verify".to_string()],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("environment".to_string(), "development".to_string());
                meta.insert("purpose".to_string(), "paymaster_operations".to_string());
                meta
            },
        };

        self.keys
            .insert(self.config.primary_key_id.clone(), primary_key_info);
        self.signing_keys
            .insert(self.config.primary_key_id.clone(), primary_wallet);

        // Add backup keys if configured
        for (i, backup_key_id) in self.config.backup_key_ids.iter().enumerate() {
            let backup_private_key = format!(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd{:02x}",
                i + 1
            );

            let backup_wallet = ethers::signers::LocalWallet::from(
                ethers::core::k256::ecdsa::SigningKey::from_slice(
                    &hex::decode(&backup_private_key).map_err(|e| {
                        KmsError::InvalidConfiguration {
                            reason: format!("Invalid backup key {}: {}", i, e),
                        }
                    })?,
                )
                .map_err(|e| KmsError::InvalidConfiguration {
                    reason: format!("Invalid backup signing key {}: {}", i, e),
                })?,
            );

            let backup_key_info = KmsKeyInfo {
                key_id: backup_key_id.clone(),
                key_type: KmsKeyType::SoftwareKey,
                address: backup_wallet.address(),
                description: format!("Backup paymaster signing key {} (test)", i + 1),
                enabled: true,
                permissions: vec!["sign".to_string()],
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("environment".to_string(), "development".to_string());
                    meta.insert("purpose".to_string(), "paymaster_backup".to_string());
                    meta.insert("backup_index".to_string(), i.to_string());
                    meta
                },
            };

            self.keys.insert(backup_key_id.clone(), backup_key_info);
            self.signing_keys
                .insert(backup_key_id.clone(), backup_wallet);
        }

        info!(
            "🔑 Initialized mock KMS with {} keys: primary={}, backups={:?}",
            self.keys.len(),
            self.config.primary_key_id,
            self.config.backup_key_ids
        );

        Ok(())
    }

    /// List available keys
    pub fn list_keys(&self) -> Vec<&KmsKeyInfo> {
        self.keys.values().collect()
    }

    /// Get key information
    pub fn get_key_info(&self, key_id: &str) -> Result<&KmsKeyInfo, KmsError> {
        self.keys.get(key_id).ok_or_else(|| KmsError::KeyNotFound {
            key_id: key_id.to_string(),
        })
    }

    /// Sign a message hash using KMS
    pub async fn sign(
        &mut self,
        request: KmsSigningRequest,
    ) -> Result<KmsSigningResponse, KmsError> {
        let start_time = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();

        debug!(
            "🔐 KMS signing request: key_id={}, hash={:?}, context={:?}",
            request.key_id, request.message_hash, request.context
        );

        // Validate key exists and is enabled
        let key_info = self.get_key_info(&request.key_id)?;
        if !key_info.enabled {
            return Err(KmsError::SignatureFailed {
                reason: format!("Key {} is disabled", request.key_id),
            });
        }

        // Simulate KMS latency (realistic timing for cloud KMS services)
        let simulated_latency_ms = match key_info.key_type {
            KmsKeyType::AwsKms => 150,
            KmsKeyType::AzureKeyVault => 120,
            KmsKeyType::GoogleCloudKms => 100,
            KmsKeyType::HardwareSecurityModule => 50,
            KmsKeyType::HardwareWallet => 2000, // Hardware wallets are slower
            KmsKeyType::SoftwareKey => 10,
        };

        tokio::time::sleep(tokio::time::Duration::from_millis(simulated_latency_ms)).await;

        // Get the signing key
        let signing_key =
            self.signing_keys
                .get(&request.key_id)
                .ok_or_else(|| KmsError::SignatureFailed {
                    reason: format!("Signing key not found for key_id: {}", request.key_id),
                })?;

        // Perform the actual signing
        let signature =
            signing_key
                .sign_hash(request.message_hash)
                .map_err(|e| KmsError::SignatureFailed {
                    reason: format!("Signature generation failed: {}", e),
                })?;

        let duration = start_time.elapsed();

        // Create audit information
        let audit_info = SigningAuditInfo {
            request_id: request_id.clone(),
            service_metadata: {
                let mut meta = HashMap::new();
                meta.insert("key_type".to_string(), format!("{:?}", key_info.key_type));
                meta.insert(
                    "simulated_latency_ms".to_string(),
                    simulated_latency_ms.to_string(),
                );
                meta.insert(
                    "operation_type".to_string(),
                    request.context.operation_type.clone(),
                );
                meta
            },
            duration_ms: duration.as_millis() as u64,
            hardware_validated: matches!(
                key_info.key_type,
                KmsKeyType::HardwareSecurityModule | KmsKeyType::HardwareWallet
            ),
        };

        // Log audit information
        if self.config.enable_audit_logging {
            self.audit_log.push(audit_info.clone());
            info!(
                "🔍 KMS signing audit: request_id={}, key_id={}, duration={}ms, context={:?}",
                request_id,
                request.key_id,
                duration.as_millis(),
                request.context
            );
        }

        let response = KmsSigningResponse {
            signature,
            key_id: request.key_id.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            audit_info,
        };

        debug!(
            "✅ KMS signing completed: key_id={}, signature={:?}, duration={}ms",
            request.key_id,
            response.signature,
            duration.as_millis()
        );

        Ok(response)
    }

    /// Test KMS connectivity
    pub async fn test_connectivity(&self) -> Result<(), KmsError> {
        debug!("🔧 Testing KMS connectivity for {} keys", self.keys.len());

        // Simulate connectivity test latency
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // In a real implementation, this would test actual KMS service connectivity
        for key_info in self.keys.values() {
            if !key_info.enabled {
                continue;
            }

            // Simulate different failure scenarios for testing
            match key_info.key_type {
                KmsKeyType::AwsKms => {
                    // Simulate occasional AWS KMS throttling
                    if rand::random::<f32>() < 0.05 {
                        return Err(KmsError::ServiceUnavailable {
                            reason: "AWS KMS throttling detected".to_string(),
                        });
                    }
                }
                KmsKeyType::HardwareWallet => {
                    // Simulate hardware wallet disconnection
                    if rand::random::<f32>() < 0.02 {
                        return Err(KmsError::HardwareWalletError {
                            reason: "Hardware wallet not connected".to_string(),
                        });
                    }
                }
                _ => {}
            }
        }

        info!(
            "✅ KMS connectivity test passed for all {} keys",
            self.keys.len()
        );
        Ok(())
    }

    /// Get audit log entries
    pub fn get_audit_log(&self) -> &[SigningAuditInfo] {
        &self.audit_log
    }

    /// Clear audit log (for testing)
    pub fn clear_audit_log(&mut self) {
        self.audit_log.clear();
    }

    /// Simulate KMS key rotation
    pub async fn rotate_key(&mut self, key_id: &str) -> Result<(), KmsError> {
        warn!("🔄 Simulating key rotation for key_id: {}", key_id);

        // In a real implementation, this would:
        // 1. Generate a new key version in the KMS
        // 2. Update key metadata
        // 3. Gradually migrate signing operations to the new key
        // 4. Retire the old key version

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if let Some(key_info) = self.keys.get_mut(key_id) {
            // Update metadata to indicate rotation
            key_info
                .metadata
                .insert("last_rotated".to_string(), chrono::Utc::now().to_rfc3339());
            key_info.metadata.insert(
                "rotation_count".to_string(),
                (key_info
                    .metadata
                    .get("rotation_count")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0)
                    + 1)
                .to_string(),
            );

            info!("✅ Key rotation completed for key_id: {}", key_id);
            Ok(())
        } else {
            Err(KmsError::KeyNotFound {
                key_id: key_id.to_string(),
            })
        }
    }
}

impl Default for KmsConfig {
    fn default() -> Self {
        Self {
            primary_key_id: "paymaster-primary-key".to_string(),
            backup_key_ids: vec![
                "paymaster-backup-key-1".to_string(),
                "paymaster-backup-key-2".to_string(),
            ],
            service_endpoint: None,
            credentials: KmsCredentials {
                access_key: "mock-access-key".to_string(),
                secret_key: "mock-secret-key".to_string(),
                region_or_tenant: "us-east-1".to_string(),
                additional_params: HashMap::new(),
            },
            signing_timeout_seconds: 30,
            enable_audit_logging: true,
            rate_limit_per_minute: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_kms_initialization() {
        let config = KmsConfig::default();
        let kms = MockKmsProvider::new(config).unwrap();

        // Should have primary + backup keys
        assert_eq!(kms.keys.len(), 3);
        assert!(kms.keys.contains_key("paymaster-primary-key"));
        assert!(kms.keys.contains_key("paymaster-backup-key-1"));
        assert!(kms.keys.contains_key("paymaster-backup-key-2"));
    }

    #[tokio::test]
    async fn test_kms_signing() {
        let config = KmsConfig::default();
        let mut kms = MockKmsProvider::new(config).unwrap();

        let request = KmsSigningRequest {
            key_id: "paymaster-primary-key".to_string(),
            message_hash: H256::random(),
            context: SigningContext {
                operation_type: "user_operation".to_string(),
                user_operation_hash: Some(H256::random()),
                sender_address: Some(Address::random()),
                entry_point: Some(Address::random()),
                gas_estimates: None,
                metadata: HashMap::new(),
            },
        };

        let response = kms.sign(request).await.unwrap();

        assert_eq!(response.key_id, "paymaster-primary-key");
        assert!(response.signature.r != ethers::types::U256::zero());
        assert!(response.signature.s != ethers::types::U256::zero());
        assert!(response.audit_info.duration_ms > 0);
    }

    #[tokio::test]
    async fn test_kms_connectivity() {
        let config = KmsConfig::default();
        let kms = MockKmsProvider::new(config).unwrap();

        // Connectivity test should pass for mock implementation
        kms.test_connectivity().await.unwrap();
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let config = KmsConfig::default();
        let mut kms = MockKmsProvider::new(config).unwrap();

        // Test key rotation
        kms.rotate_key("paymaster-primary-key").await.unwrap();

        let key_info = kms.get_key_info("paymaster-primary-key").unwrap();
        assert!(key_info.metadata.contains_key("last_rotated"));
        assert!(key_info.metadata.contains_key("rotation_count"));
    }

    #[tokio::test]
    async fn test_audit_logging() {
        let config = KmsConfig::default();
        let mut kms = MockKmsProvider::new(config).unwrap();

        let request = KmsSigningRequest {
            key_id: "paymaster-primary-key".to_string(),
            message_hash: H256::random(),
            context: SigningContext {
                operation_type: "test_signing".to_string(),
                user_operation_hash: None,
                sender_address: None,
                entry_point: None,
                gas_estimates: None,
                metadata: HashMap::new(),
            },
        };

        // Audit log should be empty initially
        assert_eq!(kms.get_audit_log().len(), 0);

        let _response = kms.sign(request).await.unwrap();

        // Audit log should contain one entry after signing
        assert_eq!(kms.get_audit_log().len(), 1);
        assert_eq!(
            kms.get_audit_log()[0]
                .service_metadata
                .get("operation_type"),
            Some(&"test_signing".to_string())
        );
    }
}
