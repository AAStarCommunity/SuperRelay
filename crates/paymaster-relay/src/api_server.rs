// api_server.rs
// Axum HTTP server with utoipa OpenAPI integration

use std::sync::Arc;

use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api_handlers::health_check_handler,
    api_schemas::ApiDoc,
    rpc::{PaymasterRelayApiServer, PaymasterRelayApiServerImpl},
};

pub type AppState = Arc<PaymasterRelayApiServerImpl>;

/// JSON-RPC request structure
#[derive(serde::Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: Option<serde_json::Value>,
}

/// JSON-RPC response structure
#[derive(serde::Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Option<serde_json::Value>,
}

/// JSON-RPC error structure
#[derive(serde::Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// Handle JSON-RPC requests at the root path
async fn json_rpc_handler(
    State(rpc_service): State<Arc<PaymasterRelayApiServerImpl>>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, StatusCode> {
    // Validate JSON-RPC 2.0 format
    if request.jsonrpc != "2.0" {
        let error_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: Some(serde_json::json!("JSON-RPC version must be 2.0")),
            }),
            id: request.id,
        };
        return Ok(Json(error_response));
    }

    // Route to appropriate method handler
    let result = match request.method.as_str() {
        "pm_sponsorUserOperation" => {
            handle_sponsor_user_operation_rpc(rpc_service, request.params).await
        }
        "health" => handle_health_check_rpc().await,
        _ => {
            let error_response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: Some(serde_json::json!(format!(
                        "Unknown method: {}",
                        request.method
                    ))),
                }),
                id: request.id,
            };
            return Ok(Json(error_response));
        }
    };

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id: request.id,
    };

    Ok(Json(response))
}

/// Handle pm_sponsorUserOperation JSON-RPC method
async fn handle_sponsor_user_operation_rpc(
    rpc_service: Arc<PaymasterRelayApiServerImpl>,
    params: serde_json::Value,
) -> serde_json::Value {
    // Parse parameters (expecting [user_op, entry_point])
    if let serde_json::Value::Array(params_array) = params {
        if params_array.len() >= 2 {
            let user_op = params_array[0].clone();
            let entry_point = params_array[1].as_str().unwrap_or("").to_string();

            // Call the RPC implementation
            match PaymasterRelayApiServer::sponsor_user_operation(
                &*rpc_service,
                user_op,
                entry_point,
            )
            .await
            {
                Ok(result) => serde_json::json!(result),
                Err(e) => serde_json::json!({
                    "error": {
                        "code": -32603,
                        "message": "Internal error",
                        "data": e.to_string()
                    }
                }),
            }
        } else {
            serde_json::json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params",
                    "data": "Expected [user_op, entry_point] parameters"
                }
            })
        }
    } else {
        serde_json::json!({
            "error": {
                "code": -32602,
                "message": "Invalid params",
                "data": "Parameters must be an array"
            }
        })
    }
}

/// Handle health check JSON-RPC method
async fn handle_health_check_rpc() -> serde_json::Value {
    serde_json::json!({
        "status": "UP",
        "version": "0.2.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })
}

/// Create the Axum router with all API endpoints and Swagger UI
pub fn create_api_router(app_state: AppState) -> Router {
    Router::new()
        // JSON-RPC endpoint (root path for blockchain tools compatibility)
        .route("/", post(json_rpc_handler))
        .route("/health", get(health_check_handler))
        // Swagger UI - serve the auto-generated OpenAPI spec
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        // CORS configuration for web UI integration
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(Any),
        )
        // Shared state
        .with_state(app_state)
}

/// Start the HTTP API server with utoipa-generated documentation
pub async fn start_api_server(
    bind_address: &str,
    rpc_impl: Arc<PaymasterRelayApiServerImpl>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_api_router(rpc_impl);

    let listener = tokio::net::TcpListener::bind(bind_address).await?;

    tracing::info!(
        "🚀 Starting SuperPaymaster Dual-Protocol API server on http://{}",
        bind_address
    );
    tracing::info!("🔄 JSON-RPC endpoint: http://{}/", bind_address);
    tracing::info!(
        "🌐 HTTP REST endpoint: http://{}/api/v1/sponsor",
        bind_address
    );
    tracing::info!("📊 Swagger UI: http://{}/swagger-ui", bind_address);
    tracing::info!("🏥 Health check: http://{}/health", bind_address);

    axum::serve(listener, app)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
}

#[cfg(test)]
mod tests {
    use utoipa::OpenApi;

    use crate::api_schemas::ApiDoc;

    #[test]
    fn test_openapi_document_generation() {
        // Test that OpenAPI document can be generated
        let openapi = ApiDoc::openapi();

        // Verify basic structure
        assert_eq!(openapi.info.title, "SuperPaymaster Relay API");
        assert_eq!(openapi.info.version, "0.2.0");

        // Verify paths are present
        assert!(openapi.paths.paths.contains_key("/api/v1/sponsor"));
        assert!(openapi.paths.paths.contains_key("/health"));

        // Verify components/schemas
        if let Some(components) = &openapi.components {
            assert!(components
                .schemas
                .contains_key("SponsorUserOperationRequest"));
            assert!(components
                .schemas
                .contains_key("SponsorUserOperationResponse"));
            assert!(components.schemas.contains_key("HealthResponse"));
        }
    }
}
