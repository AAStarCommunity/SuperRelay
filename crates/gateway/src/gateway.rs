use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use rundler_builder::LocalBuilderHandle;
use rundler_paymaster_relay::PaymasterRelayService;
use rundler_pool::LocalPoolHandle;
use serde_json::Value;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{debug, error, info, warn};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api_docs::CompleteApiDoc,
    e2e_validator::quick_e2e_health_check,
    error::{GatewayError, GatewayResult},
    health::health_routes,
    router::{EthApiConfig, GatewayRouter},
    GatewayConfig,
};

/// Main gateway service that orchestrates requests between clients and rundler components
#[derive(Clone)]
pub struct PaymasterGateway {
    config: GatewayConfig,
    paymaster_service: Option<Arc<PaymasterRelayService>>,
    router: GatewayRouter,
}

/// Gateway state shared across requests
#[derive(Clone)]
pub struct GatewayState {
    /// Optional paymaster service instance
    pub paymaster_service: Option<Arc<PaymasterRelayService>>,
    /// Request router instance
    pub router: GatewayRouter,
    /// Gateway configuration
    pub config: GatewayConfig,
}

impl PaymasterGateway {
    /// Create a new gateway instance
    pub fn new(
        config: GatewayConfig,
        paymaster_service: Option<Arc<PaymasterRelayService>>,
    ) -> Self {
        let router = GatewayRouter::new();

        Self {
            config,
            paymaster_service,
            router,
        }
    }

    /// Create a new gateway instance with rundler components
    pub fn with_rundler_components(
        config: GatewayConfig,
        paymaster_service: Option<Arc<PaymasterRelayService>>,
        pool_handle: Arc<LocalPoolHandle>,
        builder_handle: Arc<LocalBuilderHandle>,
        eth_config: EthApiConfig,
    ) -> Self {
        let router =
            GatewayRouter::with_rundler_components(pool_handle, Some(builder_handle), eth_config);

        Self {
            config,
            paymaster_service,
            router,
        }
    }

    /// Start the gateway server
    pub async fn start(self) -> GatewayResult<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        info!("🌐 Starting SuperRelay Gateway on {}", addr);

        let state = GatewayState {
            paymaster_service: self.paymaster_service.clone(),
            router: self.router.clone(),
            config: self.config.clone(),
        };

        let app = self.create_router(state);

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| GatewayError::ServerError(format!("Failed to bind to {}: {}", addr, e)))?;

        info!("✅ Gateway server listening on {}", addr);
        info!("📋 Available endpoints:");
        info!("  • POST /              - JSON-RPC API (25 methods)");
        info!("  • GET /swagger-ui     - Complete API Documentation");
        info!("  • GET /health         - Comprehensive health check");
        info!("  • GET /ready          - Readiness check");
        info!("  • GET /live           - Liveness check");
        info!("  • GET /e2e            - End-to-end validation");
        info!("  • GET /metrics        - Prometheus metrics");
        info!("");
        info!("🌐 Swagger UI: http://{}/swagger-ui/", addr);
        info!("🔥 Complete SuperRelay API Documentation Available!");

        axum::serve(listener, app)
            .await
            .map_err(|e| GatewayError::ServerError(format!("Server error: {}", e)))?;

        Ok(())
    }

    fn create_router(&self, state: GatewayState) -> Router {
        let mut router = Router::new()
            // JSON-RPC API endpoint
            .route("/", post(handle_jsonrpc))
            // Monitoring and health endpoints
            .route("/e2e", get(handle_e2e_validation))
            .route("/metrics", get(handle_metrics))
            .merge(health_routes())
            // Swagger UI integration - Complete API documentation
            .merge(
                SwaggerUi::new("/swagger-ui")
                    .url("/api-docs/openapi.json", CompleteApiDoc::openapi()),
            )
            .with_state(state);

        // Add middleware layers
        if self.config.enable_cors {
            let cors = CorsLayer::permissive();
            router = router.layer(cors);
        }

        router = router.layer(TraceLayer::new_for_http());

        router
    }
}

/// Handle JSON-RPC requests with enterprise features
async fn handle_jsonrpc(
    State(state): State<GatewayState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Parse JSON-RPC request
    let request = match parse_jsonrpc_request(&payload) {
        Ok(req) => req,
        Err(e) => {
            warn!("Invalid JSON-RPC request: {}", e);
            return Ok(Json(jsonrpc_error(-32700, "Parse error", None)));
        }
    };

    // Route request based on method
    let response = match request.method.as_str() {
        // Paymaster methods
        "pm_sponsorUserOperation" => handle_paymaster_request(&state, &request).await,

        // Standard eth methods - forward to rundler
        method if method.starts_with("eth_") => handle_rundler_request(&state, &request).await,

        // Rundler-specific methods
        method if method.starts_with("rundler_") => handle_rundler_request(&state, &request).await,

        // Debug methods
        method if method.starts_with("debug_") => handle_rundler_request(&state, &request).await,

        // Admin methods
        method if method.starts_with("admin_") => handle_rundler_request(&state, &request).await,

        _ => {
            warn!("Unknown method: {}", request.method);
            jsonrpc_error(-32601, "Method not found", Some(request.id))
        }
    };

    Ok(Json(response))
}

/// Handle paymaster-specific requests
async fn handle_paymaster_request(state: &GatewayState, request: &JsonRpcRequest) -> Value {
    if let Some(ref paymaster_service) = state.paymaster_service {
        // Forward to paymaster service
        match state
            .router
            .route_to_paymaster(paymaster_service, request)
            .await
        {
            Ok(result) => jsonrpc_success(result, request.id.clone()),
            Err(e) => {
                warn!("Paymaster request failed: {}", e);
                jsonrpc_error(
                    -32603,
                    &format!("Paymaster error: {}", e),
                    Some(request.id.clone()),
                )
            }
        }
    } else {
        jsonrpc_error(
            -32601,
            "Paymaster service not available",
            Some(request.id.clone()),
        )
    }
}

/// Handle rundler requests by forwarding to appropriate rundler components
async fn handle_rundler_request(state: &GatewayState, request: &JsonRpcRequest) -> Value {
    match state.router.route_to_rundler(request).await {
        Ok(result) => jsonrpc_success(result, request.id.clone()),
        Err(e) => {
            warn!("Rundler request failed: {}", e);
            jsonrpc_error(
                -32603,
                &format!("Rundler error: {}", e),
                Some(request.id.clone()),
            )
        }
    }
}

// Health check endpoints now handled by health module

/// End-to-end validation endpoint
async fn handle_e2e_validation(
    State(state): State<GatewayState>,
) -> Result<Json<crate::e2e_validator::E2EValidationResult>, StatusCode> {
    debug!("Processing end-to-end validation request");

    let result = quick_e2e_health_check(&state).await;

    match result.status {
        crate::e2e_validator::E2EStatus::Success => {
            info!("E2E validation passed: all systems operational");
        }
        crate::e2e_validator::E2EStatus::PartialSuccess => {
            warn!("E2E validation partial success: some components have warnings");
        }
        crate::e2e_validator::E2EStatus::Failed => {
            error!("E2E validation failed: critical issues detected");
        }
        crate::e2e_validator::E2EStatus::InProgress => {
            info!("E2E validation in progress");
        }
    }

    Ok(Json(result))
}

/// Metrics endpoint - integrate with rundler metrics
async fn handle_metrics(State(state): State<GatewayState>) -> String {
    let mut metrics = String::new();

    // Gateway-specific metrics (minimal additions)
    metrics.push_str("# Gateway metrics\n");
    metrics.push_str("superrelay_gateway_requests_total 0\n");
    metrics.push_str("superrelay_gateway_active_connections 0\n");

    // If paymaster service exists, include its basic info
    if let Some(ref _paymaster_service) = state.paymaster_service {
        metrics.push_str("\n# Paymaster service status\n");
        metrics.push_str("paymaster_service_available 1\n");
        // Note: Actual paymaster metrics are handled by Prometheus directly
    } else {
        metrics.push_str("\n# Paymaster service status\n");
        metrics.push_str("paymaster_service_available 0\n");
    }

    // TODO: Proxy rundler metrics endpoint (/metrics) here for unified access
    metrics.push_str("\n# Note: Rundler metrics available at original endpoints\n");

    metrics
}

/// JSON-RPC request structure
#[derive(Debug)]
pub struct JsonRpcRequest {
    /// Request identifier
    pub id: Value,
    /// RPC method name
    pub method: String,
    /// Method parameters
    pub params: Vec<Value>,
}

/// Parse JSON-RPC request
fn parse_jsonrpc_request(payload: &Value) -> GatewayResult<JsonRpcRequest> {
    let obj = payload
        .as_object()
        .ok_or_else(|| GatewayError::InvalidRequest("Request must be an object".to_string()))?;

    let id = obj.get("id").cloned().unwrap_or(Value::Null);

    let method = obj
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| GatewayError::InvalidRequest("Missing method".to_string()))?
        .to_string();

    let params = obj
        .get("params")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(JsonRpcRequest { id, method, params })
}

/// Create JSON-RPC success response
fn jsonrpc_success(result: Value, id: Value) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    })
}

/// Create JSON-RPC error response
fn jsonrpc_error(code: i32, message: &str, id: Option<Value>) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": code,
            "message": message
        },
        "id": id.unwrap_or(Value::Null)
    })
}
