// SuperRelay Gateway - Enterprise API Gateway for ERC-4337
//
// This module provides an API gateway that sits in front of rundler components,
// adding enterprise features like authentication, rate limiting, and policy enforcement
// while maintaining zero-invasion of the upstream rundler codebase.

#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]

//! SuperRelay Gateway - API Gateway with enterprise features

/// Authorization and eligibility checking for UserOperations
pub mod authorization;
/// End-to-end transaction validation
pub mod e2e_validator;
/// Error types and result helpers
pub mod error;
/// Main gateway implementation
pub mod gateway;
/// Health check and system monitoring
pub mod health;
/// HTTP middleware for enterprise features
pub mod middleware;
/// Request routing logic
pub mod router;
/// Security analysis and threat detection for UserOperations
pub mod security;
/// Data integrity validation for UserOperations
pub mod validation;

pub use authorization::{AuthorizationChecker, AuthorizationConfig, AuthorizationResult};
pub use e2e_validator::{quick_e2e_health_check, E2EValidationResult, E2EValidator};
pub use error::{GatewayError, GatewayResult};
pub use gateway::PaymasterGateway;
pub use health::{HealthChecker, HealthStatus, SystemStatus};
pub use router::GatewayRouter;
pub use security::{SecurityChecker, SecurityConfig, SecurityResult};
pub use validation::{DataIntegrityChecker, DataIntegrityResult, ValidationConfig};

/// Gateway configuration
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Host to bind to
    pub host: String,
    /// Port to bind to
    pub port: u16,
    /// Enable request logging
    pub enable_logging: bool,
    /// Enable CORS
    pub enable_cors: bool,
    /// Max concurrent connections
    pub max_connections: u32,
    /// Request timeout in seconds
    pub request_timeout: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            enable_logging: true,
            enable_cors: true,
            max_connections: 1000,
            request_timeout: 30,
        }
    }
}
