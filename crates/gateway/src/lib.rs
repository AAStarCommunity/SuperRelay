// SuperRelay Gateway - Enterprise API Gateway for ERC-4337
//
// This module provides an API gateway that sits in front of rundler components,
// adding enterprise features like authentication, rate limiting, and policy enforcement
// while maintaining zero-invasion of the upstream rundler codebase.

#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![allow(missing_docs)] // Allow missing docs for auto-generated contract bindings

//! SuperRelay Gateway - API Gateway with enterprise features

/// Complete API documentation with OpenAPI/Swagger support
pub mod api_docs;
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
/// Multi-Layer Verification Flow orchestration (Gateway -> AirAccount KMS)
pub mod multi_layer_verification_flow;
/// Request routing logic
pub mod router;
/// SBT + PNTs balance validation for user eligibility
pub mod sbt_validator;
/// Security analysis and threat detection for UserOperations
pub mod security;
/// TEE Security Engine - Advanced security validation within TEE environment
pub mod tee_security_engine;
/// Data integrity validation for UserOperations
pub mod validation;
/// EntryPoint version selection and routing
pub mod version_selector;

pub use authorization::{AuthorizationChecker, AuthorizationConfig, AuthorizationResult};
pub use e2e_validator::{quick_e2e_health_check, E2EValidationResult, E2EValidator};
pub use error::{GatewayError, GatewayResult};
pub use gateway::PaymasterGateway;
pub use health::{HealthChecker, HealthStatus, SystemStatus};
pub use multi_layer_verification_flow::{
    DualSignatureConfig, DualSignatureFlow, DualSignatureRequest, DualSignatureResponse,
    KmsSigningSummary, ValidationSummary,
};
pub use router::GatewayRouter;
pub use sbt_validator::{SBTValidator, SBTValidatorConfig, ValidationResult};
pub use security::{SecurityChecker, SecurityConfig, SecurityResult};
pub use tee_security_engine::{
    TeeSecurityConfig, TeeSecurityEngine, TeeSecurityResult, ThreatIntelligence,
};
pub use validation::{DataIntegrityChecker, DataIntegrityResult, ValidationConfig};
pub use version_selector::{
    DetectionMethod, EntryPointVersion, Network, VersionSelection, VersionSelector,
    VersionSelectorConfig,
};

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
