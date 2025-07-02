// paymaster-relay/src/error.rs
// This file will define custom error types for the paymaster-relay crate.

use rundler_types::pool::PoolError;
use thiserror::Error;

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

impl From<PaymasterError> for jsonrpsee::types::ErrorObjectOwned {
    fn from(err: PaymasterError) -> Self {
        jsonrpsee::types::ErrorObjectOwned::owned(
            jsonrpsee::types::ErrorCode::InternalError.code(),
            err.to_string(),
            None::<()>,
        )
    }
}
