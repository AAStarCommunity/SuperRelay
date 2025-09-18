// Module system for configurable pipeline architecture
//
// This module provides the foundation for a configuration-driven modular system
// that enables loading/unloading security modules based on configuration,
// maintaining clear upstream/downstream relationships in a pipeline architecture.

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

use crate::{
    error::{GatewayError, GatewayResult},
    gateway::JsonRpcRequest,
};

/// Processing context passed through the module pipeline
#[derive(Debug, Clone)]
pub struct ProcessingContext {
    /// Original JSON-RPC request
    pub request: JsonRpcRequest,
    /// Request metadata
    pub metadata: RequestMetadata,
    /// Module-specific data store
    pub data: HashMap<String, Value>,
    /// Processing start time
    pub start_time: Instant,
}

/// Request metadata collected during processing
#[derive(Debug, Clone)]
pub struct RequestMetadata {
    /// Client IP address
    pub client_ip: Option<String>,
    /// Request ID for tracing
    pub request_id: String,
    /// Security flags set by modules
    pub security_flags: Vec<String>,
    /// Performance metrics
    pub metrics: HashMap<String, f64>,
}

/// Result of module processing
#[derive(Debug, Clone)]
pub struct ModuleResult {
    /// Whether processing should continue
    pub should_continue: bool,
    /// Optional response (for early termination)
    pub response: Option<Value>,
    /// Error if processing failed
    pub error: Option<String>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

impl ModuleResult {
    /// Create a successful result that continues processing
    pub fn continue_processing() -> Self {
        Self {
            should_continue: true,
            response: None,
            error: None,
            processing_time_ms: 0,
        }
    }

    /// Create a result that terminates processing with a response
    pub fn terminate_with_response(response: Value) -> Self {
        Self {
            should_continue: false,
            response: Some(response),
            error: None,
            processing_time_ms: 0,
        }
    }

    /// Create a result that terminates processing with an error
    pub fn terminate_with_error(error: String) -> Self {
        Self {
            should_continue: false,
            response: None,
            error: Some(error),
            processing_time_ms: 0,
        }
    }
}

/// Core trait that all security modules must implement
#[async_trait::async_trait]
pub trait SecurityModule: Send + Sync {
    /// Process the request context
    async fn process(&self, context: &mut ProcessingContext) -> ModuleResult;

    /// Check if this module is enabled
    fn is_enabled(&self) -> bool;

    /// Get the module name for logging and identification
    fn module_name(&self) -> &'static str;

    /// Get the module priority (lower numbers run first)
    fn priority(&self) -> u32 {
        1000 // Default priority
    }

    /// Check if this module should process the given request
    fn should_process(&self, context: &ProcessingContext) -> bool {
        // Default: process all requests if enabled
        self.is_enabled()
    }

    /// Initialize the module with configuration
    async fn initialize(&mut self, config: &ModuleConfig) -> GatewayResult<()> {
        debug!("Initializing module: {}", self.module_name());
        let _ = config; // Suppress unused variable warning
        Ok(())
    }

    /// Shutdown the module gracefully
    async fn shutdown(&mut self) -> GatewayResult<()> {
        debug!("Shutting down module: {}", self.module_name());
        Ok(())
    }
}

/// Configuration for individual modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    /// Whether the module is enabled
    pub enabled: bool,
    /// Module priority (lower numbers run first)
    pub priority: Option<u32>,
    /// Module-specific configuration
    pub settings: HashMap<String, Value>,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            priority: None,
            settings: HashMap::new(),
        }
    }
}

/// Configuration for the entire module pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Whether to enable parallel processing where possible
    pub enable_parallel_processing: bool,
    /// Maximum processing time per module (ms)
    pub module_timeout_ms: u64,
    /// Maximum total pipeline processing time (ms)
    pub pipeline_timeout_ms: u64,
    /// Individual module configurations
    pub modules: HashMap<String, ModuleConfig>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_parallel_processing: false,
            module_timeout_ms: 5000,    // 5 seconds per module
            pipeline_timeout_ms: 30000, // 30 seconds total
            modules: HashMap::new(),
        }
    }
}

/// The main module pipeline that orchestrates request processing
pub struct ModulePipeline {
    /// Registered modules in priority order
    modules: Vec<Box<dyn SecurityModule>>,
    /// Pipeline configuration
    config: PipelineConfig,
    /// Runtime statistics
    stats: PipelineStats,
}

/// Runtime statistics for the pipeline
#[derive(Debug, Default)]
pub struct PipelineStats {
    /// Total requests processed
    pub total_requests: u64,
    /// Total successful requests
    pub successful_requests: u64,
    /// Total failed requests
    pub failed_requests: u64,
    /// Average processing time (ms)
    pub avg_processing_time_ms: f64,
    /// Per-module statistics
    pub module_stats: HashMap<String, ModuleStats>,
}

/// Statistics for individual modules
#[derive(Debug, Default)]
pub struct ModuleStats {
    /// Requests processed by this module
    pub requests_processed: u64,
    /// Requests rejected by this module
    pub requests_rejected: u64,
    /// Average processing time (ms)
    pub avg_processing_time_ms: f64,
    /// Error count
    pub error_count: u64,
}

impl ModulePipeline {
    /// Create a new module pipeline
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            modules: Vec::new(),
            config,
            stats: PipelineStats::default(),
        }
    }

    /// Register a security module
    pub async fn register_module(
        &mut self,
        mut module: Box<dyn SecurityModule>,
    ) -> GatewayResult<()> {
        let module_name = module.module_name();

        // Get module configuration
        let module_config = self
            .config
            .modules
            .get(module_name)
            .cloned()
            .unwrap_or_default();

        // Initialize the module
        module.initialize(&module_config).await?;

        // Only register if enabled
        if module.is_enabled() {
            info!("🔧 Registering security module: {}", module_name);
            self.modules.push(module);

            // Sort modules by priority
            self.modules.sort_by_key(|m| m.priority());

            // Initialize stats
            self.stats
                .module_stats
                .insert(module_name.to_string(), ModuleStats::default());
        } else {
            info!("⏭️ Skipping disabled module: {}", module_name);
        }

        Ok(())
    }

    /// Process a request through the module pipeline
    pub async fn process_request(
        &mut self,
        mut context: ProcessingContext,
    ) -> GatewayResult<Value> {
        let start_time = Instant::now();

        debug!(
            "🚀 Starting pipeline processing for request: {} (modules: {})",
            context.metadata.request_id,
            self.modules.len()
        );

        // Update total request count
        self.stats.total_requests += 1;

        // Process through each module in priority order
        for module in &self.modules {
            let module_name = module.module_name();

            // Skip if module shouldn't process this request
            if !module.should_process(&context) {
                debug!(
                    "⏭️ Module {} skipping request {}",
                    module_name, context.metadata.request_id
                );
                continue;
            }

            let module_start = Instant::now();
            debug!("🔄 Processing with module: {}", module_name);

            // Process the request
            let result = match tokio::time::timeout(
                std::time::Duration::from_millis(self.config.module_timeout_ms),
                module.process(&mut context),
            )
            .await
            {
                Ok(result) => result,
                Err(_) => {
                    error!("⏰ Module {} timed out", module_name);
                    ModuleResult::terminate_with_error(format!("Module {} timed out", module_name))
                }
            };

            let processing_time = module_start.elapsed().as_millis() as u64;

            // Update module statistics
            if let Some(stats) = self.stats.module_stats.get_mut(module_name) {
                stats.requests_processed += 1;
                let new_avg = (stats.avg_processing_time_ms
                    * (stats.requests_processed - 1) as f64
                    + processing_time as f64)
                    / stats.requests_processed as f64;
                stats.avg_processing_time_ms = new_avg;

                if result.error.is_some() {
                    stats.error_count += 1;
                }
                if !result.should_continue {
                    stats.requests_rejected += 1;
                }
            }

            debug!(
                "✅ Module {} completed in {}ms (continue: {})",
                module_name, processing_time, result.should_continue
            );

            // Handle the result
            if let Some(error) = result.error {
                error!("❌ Module {} failed: {}", module_name, error);
                self.stats.failed_requests += 1;
                return Err(GatewayError::ValidationError(format!(
                    "Module {} failed: {}",
                    module_name, error
                )));
            }

            if !result.should_continue {
                info!("🛑 Pipeline terminated by module: {}", module_name);
                if let Some(response) = result.response {
                    self.stats.successful_requests += 1;
                    return Ok(response);
                } else {
                    self.stats.failed_requests += 1;
                    return Err(GatewayError::ValidationError(format!(
                        "Module {} terminated processing without response",
                        module_name
                    )));
                }
            }
        }

        let total_time = start_time.elapsed().as_millis() as f64;

        // Update pipeline statistics
        let new_avg = (self.stats.avg_processing_time_ms * (self.stats.total_requests - 1) as f64
            + total_time)
            / self.stats.total_requests as f64;
        self.stats.avg_processing_time_ms = new_avg;

        info!(
            "🎉 Pipeline processing completed for request {} in {}ms",
            context.metadata.request_id, total_time as u64
        );

        // All modules passed - this should be handled by the final Rundler integration module
        // If we get here, it means no module provided a final response
        self.stats.failed_requests += 1;
        Err(GatewayError::InternalError(
            "No module provided a final response".to_string(),
        ))
    }

    /// Get pipeline statistics
    pub fn get_stats(&self) -> &PipelineStats {
        &self.stats
    }

    /// Reset pipeline statistics
    pub fn reset_stats(&mut self) {
        self.stats = PipelineStats::default();

        // Reinitialize module stats
        for module in &self.modules {
            self.stats
                .module_stats
                .insert(module.module_name().to_string(), ModuleStats::default());
        }
    }

    /// Get list of active modules
    pub fn get_active_modules(&self) -> Vec<String> {
        self.modules
            .iter()
            .filter(|m| m.is_enabled())
            .map(|m| m.module_name().to_string())
            .collect()
    }

    /// Shutdown all modules
    pub async fn shutdown(&mut self) -> GatewayResult<()> {
        info!("🔄 Shutting down module pipeline...");

        for module in &mut self.modules {
            if let Err(e) = module.shutdown().await {
                warn!(
                    "⚠️ Failed to shutdown module {}: {}",
                    module.module_name(),
                    e
                );
            }
        }

        info!("✅ Module pipeline shutdown complete");
        Ok(())
    }
}

impl ProcessingContext {
    /// Create a new processing context
    pub fn new(request: JsonRpcRequest, request_id: String) -> Self {
        Self {
            request,
            metadata: RequestMetadata {
                client_ip: None,
                request_id,
                security_flags: Vec::new(),
                metrics: HashMap::new(),
            },
            data: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    /// Add a security flag
    pub fn add_security_flag(&mut self, flag: String) {
        self.metadata.security_flags.push(flag);
    }

    /// Add a metric
    pub fn add_metric(&mut self, name: String, value: f64) {
        self.metadata.metrics.insert(name, value);
    }

    /// Store module-specific data
    pub fn store_data(&mut self, key: String, value: Value) {
        self.data.insert(key, value);
    }

    /// Retrieve module-specific data
    pub fn get_data(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }
}
