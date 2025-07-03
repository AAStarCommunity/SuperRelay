// paymaster-relay/src/error.rs
// This file will define custom error types for the paymaster-relay crate.

use rundler_types::pool::PoolError;
use thiserror::Error;

/// Main error type for paymaster operations
#[derive(Debug, Error)]
pub enum PaymasterError {
    #[error("Invalid UserOperation: {0}")]
    InvalidUserOperation(String),

    #[error("Internal signer error: {0}")]
    SignerError(#[from] eyre::Report),

    #[error("Sponsorship policy rejected: {0}")]
    PolicyRejected(String),

    #[error("Mempool submission error: {0}")]
    PoolError(#[from] PoolError),
}

impl PaymasterError {
    /// Get error category for metrics
    pub fn category(&self) -> &'static str {
        match self {
            PaymasterError::InvalidUserOperation(_) => "validation_error",
            PaymasterError::SignerError(_) => "signer_error",
            PaymasterError::PolicyRejected(_) => "policy_rejection",
            PaymasterError::PoolError(_) => "pool_error",
        }
    }
}

impl From<PaymasterError> for jsonrpsee::types::ErrorObjectOwned {
    fn from(error: PaymasterError) -> Self {
        match error {
            PaymasterError::InvalidUserOperation(msg) => jsonrpsee::types::ErrorObjectOwned::owned(
                -32602,
                "Invalid UserOperation",
                Some(msg),
            ),
            PaymasterError::SignerError(err) => jsonrpsee::types::ErrorObjectOwned::owned(
                -32603,
                "Signer error",
                Some(err.to_string()),
            ),
            PaymasterError::PolicyRejected(msg) => {
                jsonrpsee::types::ErrorObjectOwned::owned(-32604, "Policy rejected", Some(msg))
            }
            PaymasterError::PoolError(err) => jsonrpsee::types::ErrorObjectOwned::owned(
                -32605,
                "Pool error",
                Some(err.to_string()),
            ),
        }
    }
}
