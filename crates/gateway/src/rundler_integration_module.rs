// Rundler Integration Module - Final module in the pipeline
//
// This module provides the final integration with rundler components,
// handling the actual business logic after all security modules have processed the request.

use std::sync::Arc;

use rundler_paymaster_relay::PaymasterRelayService;
use serde_json::Value;
use tracing::{debug, error, info, warn};

use crate::{
    error::{GatewayError, GatewayResult},
    gateway::JsonRpcRequest,
    module_system::{ModuleConfig, ModuleResult, ProcessingContext, SecurityModule},
    router::GatewayRouter,
};

/// Rundler Integration Module - handles the final business logic execution
pub struct RundlerIntegrationModule {
    /// Gateway router for rundler integration
    router: GatewayRouter,
    /// Optional paymaster service
    paymaster_service: Option<Arc<PaymasterRelayService>>,
    /// Module configuration
    enabled: bool,
}

impl RundlerIntegrationModule {
    /// Create a new Rundler integration module
    pub fn new(
        router: GatewayRouter,
        paymaster_service: Option<Arc<PaymasterRelayService>>,
    ) -> Self {
        Self {
            router,
            paymaster_service,
            enabled: true, // Always enabled as this is the final required module
        }
    }

    /// Process the JSON-RPC request through rundler components
    async fn process_jsonrpc_request(&self, request: &JsonRpcRequest) -> GatewayResult<Value> {
        debug!(
            "🔧 Rundler integration processing method: {}",
            request.method
        );

        match request.method.as_str() {
            // Paymaster methods
            "pm_sponsorUserOperation" => {
                if let Some(ref paymaster_service) = self.paymaster_service {
                    self.router
                        .route_to_paymaster(paymaster_service, request)
                        .await
                        .map_err(|e| {
                            error!("Paymaster routing failed: {}", e);
                            GatewayError::ValidationError(format!("Paymaster error: {}", e))
                        })
                } else {
                    Err(GatewayError::ValidationError(
                        "Paymaster service not available".to_string(),
                    ))
                }
            }

            // Standard eth methods - forward to rundler
            method if method.starts_with("eth_") => {
                self.router.route_to_rundler(request).await.map_err(|e| {
                    error!("Rundler routing failed for eth method {}: {}", method, e);
                    GatewayError::ValidationError(format!("Rundler error: {}", e))
                })
            }

            // Rundler-specific methods
            method if method.starts_with("rundler_") => {
                self.router.route_to_rundler(request).await.map_err(|e| {
                    error!(
                        "Rundler routing failed for rundler method {}: {}",
                        method, e
                    );
                    GatewayError::ValidationError(format!("Rundler error: {}", e))
                })
            }

            // Debug methods
            method if method.starts_with("debug_") => {
                self.router.route_to_rundler(request).await.map_err(|e| {
                    error!("Rundler routing failed for debug method {}: {}", method, e);
                    GatewayError::ValidationError(format!("Rundler error: {}", e))
                })
            }

            // Admin methods
            method if method.starts_with("admin_") => {
                self.router.route_to_rundler(request).await.map_err(|e| {
                    error!("Rundler routing failed for admin method {}: {}", method, e);
                    GatewayError::ValidationError(format!("Rundler error: {}", e))
                })
            }

            _ => Err(GatewayError::ValidationError(format!(
                "Unknown method: {}",
                request.method
            ))),
        }
    }
}

#[async_trait::async_trait]
impl SecurityModule for RundlerIntegrationModule {
    async fn process(&self, context: &mut ProcessingContext) -> ModuleResult {
        if !self.is_enabled() {
            warn!("🔧 Rundler integration module is disabled - this should not happen!");
            return ModuleResult::terminate_with_error(
                "Rundler integration module is required".to_string(),
            );
        }

        debug!("🔧 Processing request through Rundler integration");

        match self.process_jsonrpc_request(&context.request).await {
            Ok(result) => {
                info!("✅ Rundler integration completed successfully");
                // Store result in context for potential later use
                context.store_data("rundler_result".to_string(), result.clone());

                // Terminate with final response
                ModuleResult::terminate_with_response(result)
            }
            Err(e) => {
                error!("❌ Rundler integration failed: {}", e);
                context.add_security_flag("rundler_integration_failed".to_string());
                ModuleResult::terminate_with_error(format!("Rundler integration failed: {}", e))
            }
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn module_name(&self) -> &'static str {
        "rundler_integration"
    }

    fn priority(&self) -> u32 {
        1000 // Highest priority - runs last as the final module
    }

    fn should_process(&self, _context: &ProcessingContext) -> bool {
        // Always process - this is the required final module
        self.is_enabled()
    }

    async fn initialize(&mut self, config: &ModuleConfig) -> GatewayResult<()> {
        info!("🔧 Initializing Rundler Integration Module");

        self.enabled = config.enabled;

        if !self.enabled {
            warn!("⚠️ Rundler integration module disabled - this may break functionality!");
        } else {
            info!("✅ Rundler integration module initialized successfully");
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> GatewayResult<()> {
        info!("🔧 Shutting down Rundler Integration Module");
        // No cleanup needed for this module
        Ok(())
    }
}
