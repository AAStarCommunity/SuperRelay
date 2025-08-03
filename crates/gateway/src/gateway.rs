use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use rundler_paymaster_relay::PaymasterRelayService;
use serde_json::Value;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

use crate::{
    error::{GatewayError, GatewayResult},
    router::GatewayRouter,
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
    paymaster_service: Option<Arc<PaymasterRelayService>>,
    router: GatewayRouter,
    config: GatewayConfig,
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
        info!("  • POST /        - JSON-RPC API");
        info!("  • GET /health   - Health check");
        info!("  • GET /metrics  - Prometheus metrics");

        axum::serve(listener, app)
            .await
            .map_err(|e| GatewayError::ServerError(format!("Server error: {}", e)))?;

        Ok(())
    }

    fn create_router(&self, state: GatewayState) -> Router {
        let mut router = Router::new()
            .route("/", post(handle_jsonrpc))
            .route("/health", get(handle_health))
            .route("/metrics", get(handle_metrics))
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

/// Health check endpoint
async fn handle_health(State(state): State<GatewayState>) -> Json<Value> {
    let mut health = serde_json::Map::new();
    health.insert("status".to_string(), Value::String("healthy".to_string()));
    health.insert("gateway".to_string(), Value::String("running".to_string()));

    if state.paymaster_service.is_some() {
        health.insert(
            "paymaster".to_string(),
            Value::String("available".to_string()),
        );
    } else {
        health.insert(
            "paymaster".to_string(),
            Value::String("disabled".to_string()),
        );
    }

    Json(Value::Object(health))
}

/// Metrics endpoint (placeholder)
async fn handle_metrics() -> String {
    // TODO: Implement Prometheus metrics
    "# Metrics endpoint - TODO: implement Prometheus integration\n".to_string()
}

/// JSON-RPC request structure
#[derive(Debug)]
pub struct JsonRpcRequest {
    pub id: Value,
    pub method: String,
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
