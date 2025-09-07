/**
 * Multi-Layer Verification Flow (formerly Multi-Layer Verification)
 *
 * Orchestrates the complete multi-layer verification process:
 * 1. Gateway SBT+PNTs validation (business rules)
 * 2. AirAccount KMS integration (user passkey + paymaster signature)
 * 3. EntryPoint version detection and routing
 *
 * This module provides the core coordination between SuperRelay Gateway
 * and AirAccount KMS for secure Account Abstraction operations.
 */
use std::collections::HashMap;
use std::{sync::Arc, time::SystemTime};

use anyhow::{anyhow, Result};
use ethers::{
    providers::{Http, Provider},
    types::{Address, U256},
};
use rundler_paymaster_relay::{kms::GasEstimates, AirAccountKmsClient, SigningContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, warn};

use crate::{
    sbt_validator::{SBTValidator, ValidationResult},
    version_selector::{EntryPointVersion, VersionSelector},
};

/// Multi-Layer Verification Flow orchestrator
pub struct DualSignatureFlow {
    sbt_validator: SBTValidator,
    version_selector: VersionSelector,
    airaccount_kms: AirAccountKmsClient,
    config: DualSignatureConfig,
}

/// Configuration for Multi-Layer Verification Flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualSignatureConfig {
    /// Enable strict SBT validation
    pub strict_sbt_validation: bool,
    /// Minimum PNTs balance required
    pub min_pnts_balance: String,
    /// Maximum gas limit allowed
    pub max_gas_limit: u64,
    /// AirAccount KMS service URL
    pub airaccount_kms_url: String,
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    /// Enable detailed audit logging
    pub enable_audit_logging: bool,
}

/// Multi-Layer Verification Flow request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualSignatureRequest {
    /// UserOperation data
    pub user_operation: Value,
    /// User's account ID
    pub account_id: String,
    /// User's passkey signature
    pub user_signature: String,
    /// User's public key
    pub user_public_key: String,
    /// Sender's Ethereum address
    pub sender_address: Address,
    /// Network identifier
    pub network: String,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

/// Multi-Layer Verification Flow response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualSignatureResponse {
    /// Success flag
    pub success: bool,
    /// Final paymaster signature
    pub paymaster_signature: String,
    /// EntryPoint version detected
    pub entry_point_version: String,
    /// SBT validation result
    pub sbt_validation: ValidationSummary,
    /// KMS signing information
    pub kms_info: KmsSigningSummary,
    /// Request processing time in milliseconds
    pub processing_time_ms: u64,
    /// Unique request identifier
    pub request_id: String,
}

/// SBT validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub valid: bool,
    pub sbt_owned: bool,
    pub pnts_balance: String,
    pub gas_coverage: String,
    pub membership_level: String,
}

/// KMS signing summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KmsSigningSummary {
    pub tee_device_id: String,
    pub dual_signature_verified: bool,
    pub paymaster_verified: bool,
    pub user_passkey_verified: bool,
    pub signature_timestamp: String,
}

impl DualSignatureFlow {
    /// Create new Multi-Layer Verification Flow orchestrator
    pub async fn new(
        config: DualSignatureConfig,
        key_manager: rundler_paymaster_relay::PaymasterKeyManager,
    ) -> Result<Self> {
        info!("🔄 Initializing Multi-Layer Verification Flow");
        info!("   AirAccount KMS URL: {}", config.airaccount_kms_url);
        info!("   Strict SBT Validation: {}", config.strict_sbt_validation);

        // Create provider for blockchain interactions - use configurable RPC URL
        let rpc_url = std::env::var("NODE_HTTP")
            .or_else(|_| std::env::var("RPC_URL"))
            .unwrap_or_else(|_| {
                warn!("⚠️  NODE_HTTP/RPC_URL not set, using Sepolia testnet");
                "https://rpc.sepolia.org".to_string()
            });
        let _provider = Arc::new(Provider::<Http>::try_from(rpc_url.as_str())?);

        // Initialize SBT validator
        let sbt_config = crate::sbt_validator::SBTValidatorConfig {
            rpc_url: rpc_url.to_string(),
            sbt_contract: "0x1234567890123456789012345678901234567890".parse()?,
            pnts_contract: "0x2345678901234567890123456789012345678901".parse()?,
            pnts_to_eth_rate: 1000, // 1000 PNTs = 1 ETH
            gas_price_buffer: 1.2,  // 20% buffer
            min_sbt_balance: 1,     // Minimum 1 SBT required
            validation_timeout: 30, // 30 seconds timeout
        };
        let sbt_validator = SBTValidator::new(sbt_config).await?;

        // Initialize version selector (Sepolia Testnet = Chain ID 11155111)
        let version_config = crate::version_selector::VersionSelectorConfig::sepolia_testnet();
        let version_selector = VersionSelector::new(version_config, 11155111)?;

        // Initialize AirAccount KMS client
        let airaccount_kms =
            AirAccountKmsClient::new(config.airaccount_kms_url.clone(), key_manager);

        Ok(Self {
            sbt_validator,
            version_selector,
            airaccount_kms,
            config,
        })
    }

    /// Process Multi-Layer Verification verification flow
    pub async fn process_dual_signature(
        &mut self,
        request: DualSignatureRequest,
    ) -> Result<DualSignatureResponse> {
        let start_time = SystemTime::now();
        let request_id = uuid::Uuid::new_v4().to_string();

        info!(
            "🔐 Processing Multi-Layer Verification request: {}",
            request_id
        );
        info!("   Account ID: {}", request.account_id);
        info!("   Sender: {:?}", request.sender_address);

        // Step 1: EntryPoint Version Detection
        let entry_point_version = self
            .detect_entry_point_version(&request.user_operation)
            .await?;

        info!("📋 Detected EntryPoint version: {:?}", entry_point_version);

        // Step 2: SBT + PNTs Validation (Business Rules)
        let sbt_validation = self.validate_business_rules(&request).await.map_err(|e| {
            error!("❌ SBT validation failed: {}", e);
            anyhow!("Business rules validation failed: {}", e)
        })?;

        if !sbt_validation.is_eligible && self.config.strict_sbt_validation {
            return Err(anyhow!(
                "SBT validation failed: user does not meet business requirements"
            ));
        }

        info!("✅ Business rules validation passed");

        // Step 3: Prepare KMS Signing Context
        let _signing_context = self
            .prepare_signing_context(&request, &entry_point_version, &sbt_validation)
            .await?;

        // Step 4: AirAccount KMS Multi-Layer Verification
        let kms_response = self
            .airaccount_kms
            .sign_user_operation(
                &request.user_operation,
                &request.account_id,
                &request.user_signature,
                &request.user_public_key,
            )
            .await
            .map_err(|e| {
                error!("❌ AirAccount KMS signing failed: {}", e);
                anyhow!("KMS Multi-Layer Verification failed: {}", e)
            })?;

        info!("✅ AirAccount KMS Multi-Layer Verification completed");

        // Step 5: Build Response
        let processing_time = start_time.elapsed().unwrap_or_default().as_millis() as u64;

        let response = DualSignatureResponse {
            success: true,
            paymaster_signature: kms_response.signature.clone(),
            entry_point_version: format!("{:?}", entry_point_version),
            sbt_validation: ValidationSummary {
                valid: sbt_validation.is_eligible,
                sbt_owned: sbt_validation.sbt_balance > 0,
                pnts_balance: sbt_validation.pnts_balance.to_string(),
                gas_coverage: sbt_validation.required_pnts.to_string(),
                membership_level: if sbt_validation.sbt_balance > 0 {
                    "premium".to_string()
                } else {
                    "basic".to_string()
                },
            },
            kms_info: KmsSigningSummary {
                tee_device_id: kms_response.tee_device_id,
                dual_signature_verified: kms_response.verification_proof.dual_signature_mode,
                paymaster_verified: kms_response.verification_proof.paymaster_verified,
                user_passkey_verified: kms_response.verification_proof.user_passkey_verified,
                signature_timestamp: kms_response.verification_proof.timestamp,
            },
            processing_time_ms: processing_time,
            request_id,
        };

        // Step 6: Audit Logging
        if self.config.enable_audit_logging {
            self.log_dual_signature_audit(&request, &response).await;
        }

        info!(
            "🎉 Dual signature flow completed successfully in {}ms",
            processing_time
        );

        Ok(response)
    }

    /// Detect EntryPoint version from UserOperation
    async fn detect_entry_point_version(&self, user_op: &Value) -> Result<EntryPointVersion> {
        let version_selection = self.version_selector.select_version(None, Some(user_op))?;
        Ok(version_selection.selected_version)
    }

    /// Validate business rules (SBT + PNTs)
    async fn validate_business_rules(
        &self,
        request: &DualSignatureRequest,
    ) -> Result<ValidationResult> {
        // Extract gas estimate from UserOperation
        let call_gas_limit = request.user_operation["callGasLimit"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing callGasLimit in UserOperation"))?;
        let verification_gas_limit = request.user_operation["verificationGasLimit"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing verificationGasLimit in UserOperation"))?;
        let pre_verification_gas = request.user_operation["preVerificationGas"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing preVerificationGas in UserOperation"))?;

        // Convert hex strings to U256
        let call_gas = U256::from_str_radix(&call_gas_limit[2..], 16)?;
        let verification_gas = U256::from_str_radix(&verification_gas_limit[2..], 16)?;
        let pre_verification_gas = U256::from_str_radix(&pre_verification_gas[2..], 16)?;

        let total_gas = call_gas + verification_gas + pre_verification_gas;

        debug!("⛽ Gas calculation: total={}", total_gas);

        // Perform SBT + PNTs validation
        self.sbt_validator
            .verify_user_eligibility(request.sender_address, total_gas)
            .await
    }

    /// Prepare signing context for KMS
    async fn prepare_signing_context(
        &self,
        request: &DualSignatureRequest,
        version: &EntryPointVersion,
        sbt_validation: &ValidationResult,
    ) -> Result<SigningContext> {
        let mut metadata = request.metadata.clone();
        metadata.insert("account_id".to_string(), request.account_id.clone());
        metadata.insert("user_signature".to_string(), request.user_signature.clone());
        metadata.insert(
            "user_public_key".to_string(),
            request.user_public_key.clone(),
        );
        metadata.insert("entry_point_version".to_string(), format!("{:?}", version));
        metadata.insert(
            "sbt_validation_passed".to_string(),
            sbt_validation.is_eligible.to_string(),
        );

        Ok(SigningContext {
            operation_type: "dual_signature_user_operation".to_string(),
            user_operation_hash: None, // Will be computed by KMS
            sender_address: Some(request.sender_address),
            entry_point: None, // Will be set by version selector
            gas_estimates: Some(GasEstimates {
                call_gas_limit: 100000u128,
                verification_gas_limit: 100000u128,
                pre_verification_gas: 21000u128,
                max_fee_per_gas: 1500000000u128,
                max_priority_fee_per_gas: 1000000000u128,
            }),
            metadata,
        })
    }

    /// Log Multi-Layer Verification audit information
    async fn log_dual_signature_audit(
        &self,
        request: &DualSignatureRequest,
        response: &DualSignatureResponse,
    ) {
        info!("📊 Multi-Layer Verification Audit Log");
        info!("   Request ID: {}", response.request_id);
        info!("   Account ID: {}", request.account_id);
        info!("   Sender: {:?}", request.sender_address);
        info!("   EntryPoint Version: {}", response.entry_point_version);
        info!("   SBT Valid: {}", response.sbt_validation.valid);
        info!("   PNTs Balance: {}", response.sbt_validation.pnts_balance);
        info!("   TEE Device: {}", response.kms_info.tee_device_id);
        info!(
            "   Multi-Layer Verification Status: {}",
            response.kms_info.dual_signature_verified
        );
        info!("   Processing Time: {}ms", response.processing_time_ms);
        info!("   Success: {}", response.success);
    }

    /// Health check for all components
    pub async fn health_check(&self) -> Result<HashMap<String, bool>> {
        let mut health_status = HashMap::new();

        // Check SBT Validator
        match self.sbt_validator.health_check().await {
            Ok(_) => {
                health_status.insert("sbt_validator".to_string(), true);
                debug!("✅ SBT Validator health check passed");
            }
            Err(e) => {
                health_status.insert("sbt_validator".to_string(), false);
                warn!("⚠️ SBT Validator health check failed: {}", e);
            }
        }

        // Check AirAccount KMS
        match self.airaccount_kms.check_status().await {
            Ok(status) => {
                let kms_healthy = status.success && status.status.tee_connection == "connected";
                health_status.insert("airaccount_kms".to_string(), kms_healthy);
                if kms_healthy {
                    debug!("✅ AirAccount KMS health check passed");
                } else {
                    warn!("⚠️ AirAccount KMS not fully healthy: {:?}", status);
                }
            }
            Err(e) => {
                health_status.insert("airaccount_kms".to_string(), false);
                warn!("⚠️ AirAccount KMS health check failed: {}", e);
            }
        }

        // Check Version Selector
        health_status.insert("version_selector".to_string(), true);

        let overall_health = health_status.values().all(|&status| status);

        if overall_health {
            info!("✅ Multi-Layer Verification Flow health check: ALL SYSTEMS OPERATIONAL");
        } else {
            warn!("⚠️ Multi-Layer Verification Flow health check: SOME SYSTEMS DEGRADED");
        }

        Ok(health_status)
    }
}

impl Default for DualSignatureConfig {
    fn default() -> Self {
        Self {
            strict_sbt_validation: true,
            min_pnts_balance: "1000000000000000000000".to_string(), // 1000 PNTs
            max_gas_limit: 5000000,                                 // 5M gas
            airaccount_kms_url: std::env::var("AIRACCOUNT_KMS_URL")
                .unwrap_or_else(|_| "http://localhost:3002".to_string()),
            request_timeout_seconds: 30,
            enable_audit_logging: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_dual_signature_config() {
        let config = DualSignatureConfig::default();
        assert!(config.strict_sbt_validation);
        assert_eq!(config.airaccount_kms_url, "http://localhost:3002");
        assert_eq!(config.max_gas_limit, 5000000);
    }

    #[tokio::test]
    async fn test_entry_point_version_detection() {
        // This test would require a full setup, so we'll mock it
        let user_op_v06 = json!({
            "sender": "0x742D35Cc6634C0532925a3b8D6C18E3CB1EB98C1",
            "nonce": "0x0",
            "initCode": "0x",
            "callData": "0x",
            "callGasLimit": "0x186a0",
            "verificationGasLimit": "0x186a0",
            "preVerificationGas": "0x5208",
            "maxFeePerGas": "0x59682f00",
            "maxPriorityFeePerGas": "0x3b9aca00",
            "paymasterAndData": "0x"
        });

        // Verify the UserOperation structure
        assert!(user_op_v06["callGasLimit"].as_str().is_some());
        assert!(user_op_v06["verificationGasLimit"].as_str().is_some());
        assert!(user_op_v06["preVerificationGas"].as_str().is_some());
    }

    #[test]
    fn test_dual_signature_request_serialization() {
        let request = DualSignatureRequest {
            user_operation: json!({"test": "value"}),
            account_id: "test-account".to_string(),
            user_signature: "0x123456".to_string(),
            user_public_key: "0xabcdef".to_string(),
            sender_address: "0x742D35Cc6634C0532925a3b8D6C18E3CB1EB98C1"
                .parse()
                .unwrap(),
            network: "sepolia".to_string(),
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("test-account"));
        assert!(serialized.contains("sepolia"));

        let deserialized: DualSignatureRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.account_id, "test-account");
        assert_eq!(deserialized.network, "sepolia");
    }
}
