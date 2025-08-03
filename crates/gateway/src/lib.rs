// SuperRelay Gateway - Enterprise API Gateway for ERC-4337
//
// This module provides an API gateway that sits in front of rundler components,
// adding enterprise features like authentication, rate limiting, and policy enforcement
// while maintaining zero-invasion of the upstream rundler codebase.

#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]

//! SuperRelay Gateway - API Gateway with enterprise features

pub mod error;
pub mod gateway;
pub mod middleware;
pub mod router;

pub use error::{GatewayError, GatewayResult};
pub use gateway::PaymasterGateway;
pub use router::GatewayRouter;

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
