use thiserror::Error;

/// Gateway error types
#[derive(Error, Debug)]
pub enum GatewayError {
    /// Invalid request format
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Policy violation
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Internal rundler error
    #[error("Rundler error: {0}")]
    RundlerError(String),

    /// Paymaster service error
    #[error("Paymaster error: {0}")]
    PaymasterError(String),

    /// Server error
    #[error("Server error: {0}")]
    ServerError(String),

    /// JSON-RPC error
    #[error("JSON-RPC error: {0}")]
    JsonRpcError(String),

    /// Timeout error
    #[error("Request timeout")]
    Timeout,
}

impl From<anyhow::Error> for GatewayError {
    fn from(err: anyhow::Error) -> Self {
        GatewayError::ServerError(err.to_string())
    }
}

// JSON-RPC error conversion would be implemented based on the specific jsonrpsee version
// impl From<jsonrpsee::core::Error> for GatewayError {
//     fn from(err: jsonrpsee::core::Error) -> Self {
//         GatewayError::JsonRpcError(err.to_string())
//     }
// }

/// Result type for gateway operations
pub type GatewayResult<T> = Result<T, GatewayError>;
