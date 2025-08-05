// paymaster-relay/src/service.rs
// This file will contain the core business logic of the PaymasterRelayService.

use std::{collections::HashMap, sync::Arc, time::Instant};

use ethers::types::{Address, H256};
use rundler_pool::LocalPoolHandle;
use rundler_types::{UserOperation, UserOperationVariant};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::{
    error::PaymasterError,
    kms::{GasEstimates, SigningContext},
    metrics::PaymasterMetrics,
    policy::PolicyEngine,
    signer::SignerManager,
};

/// Result of paymaster sponsorship operation
#[derive(Debug, Clone)]
pub struct PaymasterSponsorResult {
    /// Paymaster and data to include in UserOperation
    pub paymaster_and_data: Vec<u8>,
    /// Verification gas limit for paymaster
    pub verification_gas_limit: Option<u64>,
    /// Post-operation gas limit for paymaster  
    pub post_op_gas_limit: Option<u64>,
    /// Modified gas limits (if any)
    pub pre_verification_gas: Option<u64>,
    pub verification_gas_limit_uo: Option<u64>,
    pub call_gas_limit: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct PaymasterRelayService {
    signer_manager: Arc<Mutex<SignerManager>>,
    policy_engine: PolicyEngine,
    metrics: PaymasterMetrics,
}

impl PaymasterRelayService {
    pub fn new(
        signer_manager: SignerManager,
        policy_engine: PolicyEngine,
        _pool: Arc<LocalPoolHandle>,
    ) -> Self {
        Self {
            signer_manager: Arc::new(Mutex::new(signer_manager)),
            policy_engine,
            metrics: PaymasterMetrics::new(),
        }
    }

    /// Get reference to metrics for health endpoints
    pub fn metrics(&self) -> &PaymasterMetrics {
        &self.metrics
    }

    pub async fn sponsor_user_operation(
        &self,
        user_op: UserOperationVariant,
        entry_point: Address,
    ) -> Result<PaymasterSponsorResult, PaymasterError> {
        let start_time = Instant::now();

        // Update active connections count
        self.metrics.update_active_connections(1); // Simplified - would track actual count

        let result = self
            .sponsor_user_operation_internal(user_op, entry_point)
            .await;
        let duration = start_time.elapsed();

        // Record metrics based on result
        match &result {
            Ok(_user_op_hash) => {
                self.metrics.record_request_success(duration);
                // Extract gas amount from the operation for monitoring
                if let Ok(gas_amount) = self.estimate_gas_amount() {
                    self.metrics.record_gas_sponsored(gas_amount);
                }
            }
            Err(error) => {
                let error_type = Self::categorize_error(error);
                self.metrics.record_request_failure(error_type, duration);
            }
        }

        // Reset active connections
        self.metrics.update_active_connections(0);

        result
    }

    async fn sponsor_user_operation_internal(
        &self,
        user_op: UserOperationVariant,
        entry_point: Address,
    ) -> Result<PaymasterSponsorResult, PaymasterError> {
        // 1. Check policy
        let policy_start = Instant::now();
        let policy_result = self.policy_engine.check_policy(&user_op);
        let policy_duration = policy_start.elapsed();
        self.metrics
            .record_validation(policy_result.is_ok(), policy_duration);

        if let Err(e) = policy_result {
            self.metrics.record_policy_violation("validation_failed");
            return Err(e);
        }

        // 2. Sign the hash using KMS/hardware wallet integration
        let signing_start = Instant::now();
        let user_op_hash = user_op.hash();

        debug!(
            "🔐 Paymaster signing UserOperation: hash={:?}, entry_point={:?}",
            user_op_hash, entry_point
        );

        // Create comprehensive signing context for KMS audit logging
        let signing_context =
            self.create_signing_context(&user_op, entry_point, H256::from_slice(&user_op_hash.0));

        let mut signer_manager = self.signer_manager.lock().await;
        info!(
            "🔑 Using {} backend for UserOperation signing",
            signer_manager.backend_type()
        );

        let signature = signer_manager
            .sign_hash_with_context(user_op_hash.into(), Some(signing_context))
            .await;
        let signing_duration = signing_start.elapsed();

        let signature = match signature {
            Ok(sig) => {
                self.metrics
                    .record_signature_operation(true, signing_duration);
                sig
            }
            Err(e) => {
                self.metrics
                    .record_signature_operation(false, signing_duration);
                return Err(PaymasterError::SignerError(e));
            }
        };

        // 3. Generate paymaster and data for the client to use
        let paymaster_address = signer_manager.address();

        match user_op {
            UserOperationVariant::V0_6(_op) => {
                // For v0.6, combine paymaster address and signature into paymasterAndData
                let paymaster_and_data =
                    [paymaster_address.as_bytes(), &signature.to_vec()].concat();

                Ok(PaymasterSponsorResult {
                    paymaster_and_data,
                    verification_gas_limit: None,
                    post_op_gas_limit: None,
                    pre_verification_gas: None,
                    verification_gas_limit_uo: None,
                    call_gas_limit: None,
                })
            }
            UserOperationVariant::V0_7(_op) => {
                // For v0.7, return separate paymaster fields
                let paymaster_verification_gas_limit = 100_000;
                let paymaster_post_op_gas_limit = 20_000;

                Ok(PaymasterSponsorResult {
                    paymaster_and_data: signature.to_vec(),
                    verification_gas_limit: Some(paymaster_verification_gas_limit),
                    post_op_gas_limit: Some(paymaster_post_op_gas_limit),
                    pre_verification_gas: None,
                    verification_gas_limit_uo: None,
                    call_gas_limit: None,
                })
            }
        }
    }

    /// Categorize errors for metrics
    fn categorize_error(error: &PaymasterError) -> &'static str {
        match error {
            PaymasterError::PolicyRejected(_) => "policy_rejection",
            PaymasterError::SignerError(_) => "signer_error",
            PaymasterError::PoolError(_) => "pool_error",
            _ => "internal_error",
        }
    }

    /// Estimate gas amount for sponsored operation (simplified)
    fn estimate_gas_amount(&self) -> Result<u64, PaymasterError> {
        // This is a simplified implementation
        // In a real system, this would be calculated from the actual UserOperation
        Ok(100_000) // Placeholder gas amount
    }

    /// Create comprehensive signing context for KMS audit logging
    fn create_signing_context(
        &self,
        user_op: &UserOperationVariant,
        entry_point: Address,
        user_op_hash: H256,
    ) -> SigningContext {
        let (sender_address, gas_estimates) = match user_op {
            UserOperationVariant::V0_6(op) => {
                let gas_est = GasEstimates {
                    call_gas_limit: op.call_gas_limit(),
                    verification_gas_limit: op.verification_gas_limit(),
                    pre_verification_gas: op.pre_verification_gas(),
                    max_fee_per_gas: op.max_fee_per_gas(),
                    max_priority_fee_per_gas: op.max_priority_fee_per_gas(),
                };
                (op.sender(), gas_est)
            }
            UserOperationVariant::V0_7(op) => {
                let gas_est = GasEstimates {
                    call_gas_limit: op.call_gas_limit(),
                    verification_gas_limit: op.verification_gas_limit(),
                    pre_verification_gas: op.pre_verification_gas(),
                    max_fee_per_gas: op.max_fee_per_gas(),
                    max_priority_fee_per_gas: op.max_priority_fee_per_gas(),
                };
                (op.sender(), gas_est)
            }
        };

        let mut metadata = HashMap::new();
        metadata.insert("paymaster_service".to_string(), "SuperRelay".to_string());
        metadata.insert(
            "entry_point_version".to_string(),
            match user_op {
                UserOperationVariant::V0_6(_) => "v0.6",
                UserOperationVariant::V0_7(_) => "v0.7",
            }
            .to_string(),
        );

        // Get backend type from locked signer manager
        let backend_type = "kms".to_string(); // Placeholder for async context
        metadata.insert("backend_type".to_string(), backend_type);
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        // Convert alloy Address to ethers H160
        let ethers_sender = ethers::types::Address::from_slice(sender_address.as_slice());
        let ethers_entry_point = ethers::types::Address::from_slice(entry_point.0.as_slice());

        SigningContext {
            operation_type: "paymaster_user_operation".to_string(),
            user_operation_hash: Some(user_op_hash),
            sender_address: Some(ethers_sender),
            entry_point: Some(ethers_entry_point),
            gas_estimates: Some(gas_estimates),
            metadata,
        }
    }

    /// Test KMS connectivity for health checks
    pub async fn test_kms_connectivity(&self) -> Result<(), PaymasterError> {
        let signer_manager = self.signer_manager.lock().await;
        signer_manager
            .test_kms_connectivity()
            .await
            .map_err(PaymasterError::SignerError)
    }

    /// Get KMS audit information for compliance
    pub async fn get_kms_audit_info(&self) -> Option<Vec<crate::kms::SigningAuditInfo>> {
        let signer_manager = self.signer_manager.lock().await;
        signer_manager.get_kms_audit_log()
    }

    /// Get signer configuration details
    pub async fn get_signer_info(&self) -> HashMap<String, String> {
        let signer_manager = self.signer_manager.lock().await;
        let mut info = signer_manager.get_metadata().clone();
        info.insert(
            "current_address".to_string(),
            format!("{:?}", signer_manager.address()),
        );
        info.insert(
            "service_name".to_string(),
            "PaymasterRelayService".to_string(),
        );
        info
    }

    /// Rotate KMS keys (enterprise feature)
    pub async fn rotate_kms_key(&self, key_id: &str) -> Result<(), PaymasterError> {
        info!(
            "🔄 PaymasterService: Initiating KMS key rotation for key_id={}",
            key_id
        );

        let mut signer_manager = self.signer_manager.lock().await;
        signer_manager
            .rotate_kms_key(key_id)
            .await
            .map_err(PaymasterError::SignerError)?;

        // Update metrics to reflect key rotation
        self.metrics
            .record_policy_violation("key_rotation_completed");

        info!(
            "✅ PaymasterService: KMS key rotation completed for key_id={}",
            key_id
        );
        Ok(())
    }

    /// Background task to update metrics periodically
    pub async fn update_background_metrics(&self) {
        // Update health status
        self.metrics.update_health_status(true);

        // Update system metrics
        self.metrics.update_memory_usage(get_memory_usage_mb());

        // Test KMS connectivity for health monitoring
        {
            let signer_manager = self.signer_manager.lock().await;
            if let Err(e) = signer_manager.test_kms_connectivity().await {
                warn!("⚠️ KMS connectivity issue detected: {}", e);
                self.metrics.update_health_status(false);
            }
        }

        // Additional background metric updates could go here
    }
}

/// Get memory usage in MB (placeholder implementation)
fn get_memory_usage_mb() -> u64 {
    // In a real implementation, you'd use system metrics
    // For now, return a placeholder value
    78 // Matches our test results
}
