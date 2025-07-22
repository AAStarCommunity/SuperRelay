//! Configuration module for SuperRelay Paymaster Service

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::{ConfigError, Result};

/// Main configuration for the SuperRelay Paymaster Service
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,

    /// Paymaster configuration
    pub paymaster: PaymasterConfig,

    /// Rundler integration configuration
    pub rundler: RundlerConfig,

    /// Policy configuration
    pub policy: PolicyConfig,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to bind to
    #[serde(default = "default_port")]
    pub port: u16,

    /// Maximum number of connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,
}

/// Paymaster configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymasterConfig {
    /// Private key for signing (environment variable name)
    pub private_key_env: String,

    /// Chain ID
    pub chain_id: u64,

    /// Supported EntryPoint addresses
    pub entry_points: HashMap<String, String>, // version -> address

    /// Gas limits
    pub gas_limits: GasLimits,
}

/// Gas limit configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GasLimits {
    /// Maximum verification gas
    #[serde(default = "default_max_verification_gas")]
    pub max_verification_gas: u64,

    /// Maximum call gas
    #[serde(default = "default_max_call_gas")]
    pub max_call_gas: u64,

    /// Maximum pre-verification gas
    #[serde(default = "default_max_pre_verification_gas")]
    pub max_pre_verification_gas: u64,
}

/// Rundler integration configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RundlerConfig {
    /// Rundler RPC endpoint
    #[serde(default = "default_rundler_url")]
    pub url: String,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub timeout: u64,

    /// Maximum retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Retry delay in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
}

/// Policy configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolicyConfig {
    /// Path to policy rules file
    pub rules_path: String,

    /// Policy reload interval in seconds
    #[serde(default = "default_policy_reload_interval")]
    pub reload_interval: u64,

    /// Default policy action for undefined rules
    #[serde(default = "default_policy_action")]
    pub default_action: PolicyAction,
}

/// Policy action
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PolicyAction {
    /// Allow the operation
    Allow,
    /// Deny the operation
    #[default]
    Deny,
}

impl Config {
    /// Load configuration from TOML file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(path.as_ref())
            .await
            .map_err(|e| ConfigError::ParseError(format!("Failed to read config file: {}", e)))?;

        Self::parse_from_str(&content)
    }

    /// Parse configuration from string
    pub fn parse_from_str(content: &str) -> Result<Self> {
        toml::from_str(content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse TOML: {}", e)).into())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue("Server port cannot be 0".to_string()).into());
        }

        if self.paymaster.private_key_env.is_empty() {
            return Err(
                ConfigError::MissingRequired("paymaster.private_key_env".to_string()).into(),
            );
        }

        if self.paymaster.entry_points.is_empty() {
            return Err(ConfigError::MissingRequired("paymaster.entry_points".to_string()).into());
        }

        if self.policy.rules_path.is_empty() {
            return Err(ConfigError::MissingRequired("policy.rules_path".to_string()).into());
        }

        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            max_connections: default_max_connections(),
            request_timeout: default_request_timeout(),
        }
    }
}

impl Default for PaymasterConfig {
    fn default() -> Self {
        Self {
            private_key_env: "PAYMASTER_PRIVATE_KEY".to_string(),
            chain_id: 31337, // Anvil default
            entry_points: {
                let mut map = HashMap::new();
                map.insert(
                    "v0.6".to_string(),
                    "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789".to_string(),
                );
                map.insert(
                    "v0.7".to_string(),
                    "0x0000000071727De22E5E9d8BAf0edAc6f37da032".to_string(),
                );
                map
            },
            gas_limits: GasLimits::default(),
        }
    }
}

impl Default for GasLimits {
    fn default() -> Self {
        Self {
            max_verification_gas: default_max_verification_gas(),
            max_call_gas: default_max_call_gas(),
            max_pre_verification_gas: default_max_pre_verification_gas(),
        }
    }
}

impl Default for RundlerConfig {
    fn default() -> Self {
        Self {
            url: default_rundler_url(),
            timeout: default_connection_timeout(),
            max_retries: default_max_retries(),
            retry_delay: default_retry_delay(),
        }
    }
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            rules_path: "config/paymaster-policies.toml".to_string(),
            reload_interval: default_policy_reload_interval(),
            default_action: default_policy_action(),
        }
    }
}

// Default value functions
fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3001
}

fn default_max_connections() -> u32 {
    1000
}

fn default_request_timeout() -> u64 {
    30
}

fn default_max_verification_gas() -> u64 {
    5_000_000
}

fn default_max_call_gas() -> u64 {
    10_000_000
}

fn default_max_pre_verification_gas() -> u64 {
    1_000_000
}

fn default_rundler_url() -> String {
    "http://localhost:3000".to_string()
}

fn default_connection_timeout() -> u64 {
    10
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    1000
}

fn default_policy_reload_interval() -> u64 {
    60
}

fn default_policy_action() -> PolicyAction {
    PolicyAction::Deny
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.server.port, 3001);
        assert_eq!(config.paymaster.chain_id, 31337);
    }

    #[test]
    fn test_config_from_toml() {
        let toml = r#"
            [server]
            host = "0.0.0.0"
            port = 3001

            [paymaster]
            private_key_env = "PAYMASTER_PRIVATE_KEY"
            chain_id = 1

            [paymaster.entry_points]
            "v0.6" = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
            "v0.7" = "0x0000000071727De22E5E9d8BAf0edAc6f37da032"

            [paymaster.gas_limits]
            max_verification_gas = 5000000

            [rundler]
            url = "http://localhost:3000"

            [policy]
            rules_path = "config/policies.toml"
        "#;

        let config = Config::parse_from_str(toml).unwrap();
        assert!(config.validate().is_ok());
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.paymaster.chain_id, 1);
    }

    #[test]
    fn test_config_validation_errors() {
        let mut config = Config::default();

        // Test invalid port
        config.server.port = 0;
        assert!(config.validate().is_err());

        // Test missing private key env
        config.server.port = 3001;
        config.paymaster.private_key_env = "".to_string();
        assert!(config.validate().is_err());

        // Test missing entry points
        config.paymaster.private_key_env = "PAYMASTER_PRIVATE_KEY".to_string();
        config.paymaster.entry_points.clear();
        assert!(config.validate().is_err());
    }
}
