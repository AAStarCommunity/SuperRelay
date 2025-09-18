// Configuration system for module-driven architecture
//
// This module provides comprehensive configuration management for the modular
// security pipeline, supporting both file-based and environment-driven configuration
// with hot-reloading capabilities.

use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, warn};

use crate::{
    error::{GatewayError, GatewayResult},
    module_system::{ModuleConfig, PipelineConfig},
};

/// Main configuration structure for SuperRelay Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfiguration {
    /// Server configuration
    pub server: ServerConfig,
    /// Module pipeline configuration
    pub pipeline: PipelineConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Rundler integration configuration
    pub rundler: RundlerConfig,
    /// Monitoring and logging configuration
    pub monitoring: MonitoringConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to bind to
    pub port: u16,
    /// Enable CORS
    pub enable_cors: bool,
    /// Enable request logging
    pub enable_logging: bool,
    /// Max concurrent connections
    pub max_connections: u32,
    /// Request timeout in seconds
    pub request_timeout: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            enable_cors: true,
            enable_logging: true,
            max_connections: 1000,
            request_timeout: 30,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Rate limit per IP (requests per minute)
    pub rate_limit_per_ip: u32,
    /// Enable request validation
    pub enable_request_validation: bool,
    /// Enable security headers
    pub enable_security_headers: bool,
    /// Trusted IP addresses (bypass rate limiting)
    pub trusted_ips: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_rate_limiting: true,
            rate_limit_per_ip: 60,
            enable_request_validation: true,
            enable_security_headers: true,
            trusted_ips: vec!["127.0.0.1".to_string()],
        }
    }
}

/// Rundler integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RundlerConfig {
    /// Supported EntryPoint addresses
    pub entry_points: Vec<String>,
    /// Chain ID
    pub chain_id: u64,
    /// RPC URL for chain interactions
    pub rpc_url: Option<String>,
    /// Enable pool integration
    pub enable_pool_integration: bool,
    /// Enable builder integration
    pub enable_builder_integration: bool,
}

impl Default for RundlerConfig {
    fn default() -> Self {
        Self {
            entry_points: vec![
                "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789".to_string(), // v0.6
                "0x0000000071727De22E5E9d8BAf0edAc6f37da032".to_string(), // v0.7
            ],
            chain_id: 31337, // Anvil default
            rpc_url: None,
            enable_pool_integration: true,
            enable_builder_integration: true,
        }
    }
}

/// Monitoring and logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable Prometheus metrics
    pub enable_metrics: bool,
    /// Enable health checks
    pub enable_health_checks: bool,
    /// Enable request tracing
    pub enable_tracing: bool,
    /// Log level
    pub log_level: String,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            enable_health_checks: true,
            enable_tracing: true,
            log_level: "info".to_string(),
            enable_performance_monitoring: true,
        }
    }
}

impl Default for GatewayConfiguration {
    fn default() -> Self {
        let mut pipeline_config = PipelineConfig::default();

        // Configure default modules
        pipeline_config.modules.insert(
            "authorization".to_string(),
            ModuleConfig {
                enabled: true,
                priority: Some(100),
                settings: HashMap::new(),
            },
        );

        pipeline_config.modules.insert(
            "user_data_encryption".to_string(),
            ModuleConfig {
                enabled: false, // Disabled by default for performance
                priority: Some(200),
                settings: HashMap::new(),
            },
        );

        pipeline_config.modules.insert(
            "bls_protection".to_string(),
            ModuleConfig {
                enabled: true,
                priority: Some(300),
                settings: HashMap::new(),
            },
        );

        pipeline_config.modules.insert(
            "contract_security".to_string(),
            ModuleConfig {
                enabled: true,
                priority: Some(400),
                settings: HashMap::new(),
            },
        );

        pipeline_config.modules.insert(
            "rundler_integration".to_string(),
            ModuleConfig {
                enabled: true,
                priority: Some(1000), // Always last
                settings: HashMap::new(),
            },
        );

        Self {
            server: ServerConfig::default(),
            pipeline: pipeline_config,
            security: SecurityConfig::default(),
            rundler: RundlerConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

/// Configuration manager for loading and managing configurations
pub struct ConfigurationManager {
    /// Current configuration
    config: GatewayConfiguration,
    /// Configuration file path
    config_path: Option<String>,
}

impl ConfigurationManager {
    /// Create a new configuration manager with default configuration
    pub fn new() -> Self {
        Self {
            config: GatewayConfiguration::default(),
            config_path: None,
        }
    }

    /// Load configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> GatewayResult<Self> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy().to_string();

        info!("📋 Loading configuration from: {}", path_str);

        let content = fs::read_to_string(path).map_err(|e| {
            GatewayError::ConfigError(format!("Failed to read config file {}: {}", path_str, e))
        })?;

        let config: GatewayConfiguration = toml::from_str(&content).map_err(|e| {
            GatewayError::ConfigError(format!("Failed to parse config file {}: {}", path_str, e))
        })?;

        info!("✅ Configuration loaded successfully");
        debug!(
            "📋 Active modules: {:?}",
            config.pipeline.modules.keys().collect::<Vec<_>>()
        );

        Ok(Self {
            config,
            config_path: Some(path_str),
        })
    }

    /// Load configuration from environment variables (for cloud deployments)
    pub fn load_from_env() -> GatewayResult<Self> {
        info!("📋 Loading configuration from environment variables");

        let mut config = GatewayConfiguration::default();

        // Server configuration
        if let Ok(host) = std::env::var("GATEWAY_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("GATEWAY_PORT") {
            config.server.port = port
                .parse()
                .map_err(|_| GatewayError::ConfigError("Invalid GATEWAY_PORT".to_string()))?;
        }
        if let Ok(cors) = std::env::var("GATEWAY_ENABLE_CORS") {
            config.server.enable_cors = cors.parse().map_err(|_| {
                GatewayError::ConfigError("Invalid GATEWAY_ENABLE_CORS".to_string())
            })?;
        }

        // Module configuration
        Self::load_module_config_from_env(&mut config, "authorization")?;
        Self::load_module_config_from_env(&mut config, "user_data_encryption")?;
        Self::load_module_config_from_env(&mut config, "bls_protection")?;
        Self::load_module_config_from_env(&mut config, "contract_security")?;

        // Rundler configuration
        if let Ok(chain_id) = std::env::var("RUNDLER_CHAIN_ID") {
            config.rundler.chain_id = chain_id
                .parse()
                .map_err(|_| GatewayError::ConfigError("Invalid RUNDLER_CHAIN_ID".to_string()))?;
        }
        if let Ok(rpc_url) = std::env::var("RUNDLER_RPC_URL") {
            config.rundler.rpc_url = Some(rpc_url);
        }

        info!("✅ Configuration loaded from environment");

        Ok(Self {
            config,
            config_path: None,
        })
    }

    /// Load module configuration from environment variables
    fn load_module_config_from_env(
        config: &mut GatewayConfiguration,
        module_name: &str,
    ) -> GatewayResult<()> {
        let enabled_var = format!("MODULE_{}_ENABLED", module_name.to_uppercase());
        let priority_var = format!("MODULE_{}_PRIORITY", module_name.to_uppercase());

        let module_config = config
            .pipeline
            .modules
            .entry(module_name.to_string())
            .or_insert_with(ModuleConfig::default);

        if let Ok(enabled) = std::env::var(&enabled_var) {
            module_config.enabled = enabled
                .parse()
                .map_err(|_| GatewayError::ConfigError(format!("Invalid {}", enabled_var)))?;
        }

        if let Ok(priority) = std::env::var(&priority_var) {
            module_config.priority = Some(
                priority
                    .parse()
                    .map_err(|_| GatewayError::ConfigError(format!("Invalid {}", priority_var)))?,
            );
        }

        Ok(())
    }

    /// Save current configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> GatewayResult<()> {
        let path = path.as_ref();
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| GatewayError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, content).map_err(|e| {
            GatewayError::ConfigError(format!("Failed to write config file: {}", e))
        })?;

        info!("💾 Configuration saved to: {}", path.to_string_lossy());
        Ok(())
    }

    /// Generate default configuration file
    pub fn generate_default_config<P: AsRef<Path>>(path: P) -> GatewayResult<()> {
        let default_config = GatewayConfiguration::default();
        let content = toml::to_string_pretty(&default_config).map_err(|e| {
            GatewayError::ConfigError(format!("Failed to serialize default config: {}", e))
        })?;

        fs::write(path.as_ref(), content).map_err(|e| {
            GatewayError::ConfigError(format!("Failed to write default config: {}", e))
        })?;

        info!(
            "📝 Default configuration generated at: {}",
            path.as_ref().to_string_lossy()
        );
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &GatewayConfiguration {
        &self.config
    }

    /// Update module configuration
    pub fn update_module_config(
        &mut self,
        module_name: &str,
        config: ModuleConfig,
    ) -> GatewayResult<()> {
        info!("🔄 Updating configuration for module: {}", module_name);

        self.config
            .pipeline
            .modules
            .insert(module_name.to_string(), config);

        // Save to file if we have a path
        if let Some(ref path) = self.config_path {
            self.save_to_file(path)?;
        }

        Ok(())
    }

    /// Enable/disable a module
    pub fn set_module_enabled(&mut self, module_name: &str, enabled: bool) -> GatewayResult<()> {
        info!(
            "🔄 {} module: {}",
            if enabled { "Enabling" } else { "Disabling" },
            module_name
        );

        let module_config = self
            .config
            .pipeline
            .modules
            .entry(module_name.to_string())
            .or_insert_with(ModuleConfig::default);

        module_config.enabled = enabled;

        // Save to file if we have a path
        if let Some(ref path) = self.config_path {
            self.save_to_file(path)?;
        }

        Ok(())
    }

    /// Get module configuration
    pub fn get_module_config(&self, module_name: &str) -> Option<&ModuleConfig> {
        self.config.pipeline.modules.get(module_name)
    }

    /// List all configured modules
    pub fn list_modules(&self) -> Vec<(&String, &ModuleConfig)> {
        self.config.pipeline.modules.iter().collect()
    }

    /// Validate configuration
    pub fn validate_config(&self) -> GatewayResult<Vec<String>> {
        let mut warnings = Vec::new();

        // Check server configuration
        if self.config.server.port < 1024 && cfg!(not(target_os = "linux")) {
            warnings.push("Port below 1024 may require elevated privileges".to_string());
        }

        // Check if critical modules are enabled
        let critical_modules = ["rundler_integration"];
        for module in &critical_modules {
            if let Some(config) = self.config.pipeline.modules.get(*module) {
                if !config.enabled {
                    return Err(GatewayError::ConfigError(format!(
                        "Critical module '{}' is disabled",
                        module
                    )));
                }
            } else {
                return Err(GatewayError::ConfigError(format!(
                    "Critical module '{}' is not configured",
                    module
                )));
            }
        }

        // Check for duplicate priorities
        let mut priorities = Vec::new();
        for (name, config) in &self.config.pipeline.modules {
            if config.enabled {
                if let Some(priority) = config.priority {
                    if priorities.contains(&priority) {
                        warnings.push(format!(
                            "Module '{}' has duplicate priority {}",
                            name, priority
                        ));
                    } else {
                        priorities.push(priority);
                    }
                }
            }
        }

        // Check Rundler configuration
        if self.config.rundler.entry_points.is_empty() {
            warnings.push("No EntryPoint addresses configured".to_string());
        }

        if warnings.is_empty() {
            info!("✅ Configuration validation passed");
        } else {
            warn!(
                "⚠️ Configuration validation completed with {} warnings",
                warnings.len()
            );
        }

        Ok(warnings)
    }

    /// Hot reload configuration from file
    pub fn reload_config(&mut self) -> GatewayResult<()> {
        if let Some(ref path) = self.config_path.clone() {
            info!("🔄 Hot reloading configuration from: {}", path);

            let new_manager = Self::load_from_file(path)?;

            // Validate the new configuration before applying
            let warnings = new_manager.validate_config()?;
            if !warnings.is_empty() {
                warn!(
                    "⚠️ Configuration reload has {} warnings: {:?}",
                    warnings.len(),
                    warnings
                );
            }

            self.config = new_manager.config;

            info!("✅ Configuration hot reloaded successfully");
            Ok(())
        } else {
            Err(GatewayError::ConfigError(
                "No config file path available for reload".to_string(),
            ))
        }
    }

    /// Watch for configuration file changes and auto-reload
    pub async fn start_config_watcher(&mut self) -> GatewayResult<()> {
        if let Some(ref config_path) = self.config_path.clone() {
            info!(
                "👁️ Starting configuration file watcher for: {}",
                config_path
            );

            // Create a simple file modification time watcher
            // In a full implementation, this would use a proper file watcher library
            let path = config_path.clone();
            let _watcher_handle = tokio::spawn(async move {
                let mut last_modified = std::time::SystemTime::UNIX_EPOCH;

                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified > last_modified {
                                info!("📁 Configuration file changed, triggering reload...");
                                last_modified = modified;
                                // In a real implementation, this would send a signal to reload
                            }
                        }
                    }
                }
            });

            info!("✅ Configuration watcher started");
            Ok(())
        } else {
            Err(GatewayError::ConfigError(
                "Cannot watch configuration file: no file path available".to_string(),
            ))
        }
    }

    /// Export current configuration to different formats
    pub fn export_config(&self, format: ConfigExportFormat) -> GatewayResult<String> {
        match format {
            ConfigExportFormat::Toml => toml::to_string_pretty(&self.config)
                .map_err(|e| GatewayError::ConfigError(format!("TOML export failed: {}", e))),
            ConfigExportFormat::Json => serde_json::to_string_pretty(&self.config)
                .map_err(|e| GatewayError::ConfigError(format!("JSON export failed: {}", e))),
            ConfigExportFormat::Yaml => {
                // For YAML support, we'd need to add serde_yaml dependency
                // For now, return an error
                Err(GatewayError::ConfigError(
                    "YAML export not yet supported".to_string(),
                ))
            }
        }
    }

    /// Compare two configurations and show differences
    pub fn diff_config(&self, other: &GatewayConfiguration) -> ConfigDiff {
        let mut changes = Vec::new();

        // Server configuration changes
        if self.config.server.host != other.server.host {
            changes.push(format!(
                "server.host: {} → {}",
                self.config.server.host, other.server.host
            ));
        }
        if self.config.server.port != other.server.port {
            changes.push(format!(
                "server.port: {} → {}",
                self.config.server.port, other.server.port
            ));
        }

        // Module configuration changes
        for (module_name, current_config) in &self.config.pipeline.modules {
            if let Some(other_config) = other.pipeline.modules.get(module_name) {
                if current_config.enabled != other_config.enabled {
                    changes.push(format!(
                        "modules.{}.enabled: {} → {}",
                        module_name, current_config.enabled, other_config.enabled
                    ));
                }
                if current_config.priority != other_config.priority {
                    changes.push(format!(
                        "modules.{}.priority: {:?} → {:?}",
                        module_name, current_config.priority, other_config.priority
                    ));
                }
            } else {
                changes.push(format!("modules.{}: removed", module_name));
            }
        }

        // New modules in other config
        for module_name in other.pipeline.modules.keys() {
            if !self.config.pipeline.modules.contains_key(module_name) {
                changes.push(format!("modules.{}: added", module_name));
            }
        }

        ConfigDiff { changes }
    }
}

/// Supported configuration export formats
#[derive(Debug, Clone)]
pub enum ConfigExportFormat {
    Toml,
    Json,
    Yaml,
}

/// Configuration difference result
#[derive(Debug)]
pub struct ConfigDiff {
    pub changes: Vec<String>,
}

impl ConfigDiff {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn summary(&self) -> String {
        if self.changes.is_empty() {
            "No configuration changes detected".to_string()
        } else {
            format!(
                "{} configuration changes detected:\n{}",
                self.changes.len(),
                self.changes.join("\n")
            )
        }
    }
}
