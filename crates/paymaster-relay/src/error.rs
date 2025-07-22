//! Error types for SuperRelay Paymaster Service

use thiserror::Error;

/// Main error type for paymaster operations
#[derive(Debug, Error)]
pub enum PaymasterError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Policy validation error
    #[error("Policy validation failed: {0}")]
    PolicyRejected(String),

    /// Signing error
    #[error("Signing error: {0}")]
    Signing(#[from] SigningError),

    /// RPC communication error
    #[error("RPC communication error: {0}")]
    RpcError(String),

    /// Invalid UserOperation
    #[error("Invalid UserOperation: {0}")]
    InvalidUserOperation(String),

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to parse configuration file
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    /// Missing required configuration
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    /// Invalid configuration value
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

/// Signing-related errors
#[derive(Debug, Error)]
pub enum SigningError {
    /// Private key not found
    #[error("Private key not found")]
    KeyNotFound,

    /// Invalid private key format
    #[error("Invalid private key format")]
    InvalidKeyFormat,

    /// Signing operation failed
    #[error("Signing operation failed: {0}")]
    SigningFailed(String),
}

/// Result type for paymaster operations
pub type Result<T> = std::result::Result<T, PaymasterError>;
