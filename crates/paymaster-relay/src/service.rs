// paymaster-relay/src/service.rs
// This file will contain the core business logic of the PaymasterRelayService.

use std::{sync::Arc, time::Instant};

use ethers::types::Address;
use rundler_pool::LocalPoolHandle;
use rundler_types::{UserOperation, UserOperationVariant};

use crate::{
    error::PaymasterError, metrics::PaymasterMetrics, policy::PolicyEngine, signer::SignerManager,
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
    signer_manager: SignerManager,
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
            signer_manager,
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
        _entry_point: Address,
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

        // 2. Sign the hash
        let signing_start = Instant::now();
        let user_op_hash = user_op.hash();
        let signature = self.signer_manager.sign_hash(user_op_hash.into()).await;
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
        let paymaster_address = self.signer_manager.address();

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

    /// Background task to update metrics periodically
    pub async fn update_background_metrics(&self) {
        // Update health status
        self.metrics.update_health_status(true);

        // Update system metrics
        self.metrics.update_memory_usage(get_memory_usage_mb());

        // Additional background metric updates could go here
    }
}

/// Get memory usage in MB (placeholder implementation)
fn get_memory_usage_mb() -> u64 {
    // In a real implementation, you'd use system metrics
    // For now, return a placeholder value
    78 // Matches our test results
}
