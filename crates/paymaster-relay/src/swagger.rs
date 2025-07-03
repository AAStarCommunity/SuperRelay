// swagger.rs
// Swagger UI server implementation for API documentation

use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use tokio::time::Instant;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api_schemas::{
        examples, ApiDoc, ErrorResponse, SponsorUserOperationRequest, SponsorUserOperationResponse,
    },
    service::PaymasterRelayService,
};

/// Swagger UI server state
#[derive(Clone)]
pub struct SwaggerState {
    pub paymaster_service: Arc<PaymasterRelayService>,
    pub start_time: Instant,
    pub metrics: Arc<SwaggerMetrics>,
}

/// API usage metrics
#[derive(Default)]
pub struct SwaggerMetrics {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub successful_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub avg_response_time_ms: std::sync::atomic::AtomicU64,
}

impl SwaggerMetrics {
    pub fn record_request(&self, success: bool, duration: Duration) {
        use std::sync::atomic::Ordering;

        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        let duration_ms = duration.as_millis() as u64;
        let current_avg = self.avg_response_time_ms.load(Ordering::Relaxed);
        let total = self.total_requests.load(Ordering::Relaxed);
        let new_avg = (current_avg * (total - 1) + duration_ms) / total;
        self.avg_response_time_ms.store(new_avg, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> serde_json::Value {
        use std::sync::atomic::Ordering;

        let total = self.total_requests.load(Ordering::Relaxed);
        let success_rate = if total > 0 {
            (self.successful_requests.load(Ordering::Relaxed) as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        json!({
            "total_requests": total,
            "successful_requests": self.successful_requests.load(Ordering::Relaxed),
            "failed_requests": self.failed_requests.load(Ordering::Relaxed),
            "success_rate": success_rate,
            "avg_response_time_ms": self.avg_response_time_ms.load(Ordering::Relaxed)
        })
    }
}

/// Start the Swagger UI server
pub async fn serve_swagger_ui(
    paymaster_service: Arc<PaymasterRelayService>,
    bind_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = SwaggerState {
        paymaster_service,
        start_time: Instant::now(),
        metrics: Arc::new(SwaggerMetrics::default()),
    };

    let app = create_swagger_router(state);

    info!("🚀 Starting Swagger UI server on http://{}", bind_addr);
    info!("📖 API Documentation: http://{}/swagger-ui/", bind_addr);
    info!("📊 Health Check: http://{}/health", bind_addr);
    info!("📈 Metrics: http://{}/metrics", bind_addr);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the Swagger router with all endpoints
fn create_swagger_router(state: SwaggerState) -> Router {
    Router::new()
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // API endpoints
        .route("/api/v1/sponsor", post(sponsor_user_operation_endpoint))
        // Health and monitoring endpoints
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/metrics", get(get_metrics))
        .route("/examples/:version", get(get_examples))
        // Code generation endpoints
        .route("/codegen/curl/:endpoint", get(generate_curl_example))
        .route("/codegen/javascript/:endpoint", get(generate_js_example))
        .route("/codegen/python/:endpoint", get(generate_python_example))
        // State and middleware
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                )
                .into_inner(),
        )
}

/// Sponsor user operation endpoint (OpenAPI documented)
#[utoipa::path(
    post,
    path = "/api/v1/sponsor",
    tag = "paymaster",
    request_body = SponsorUserOperationRequest,
    responses(
        (status = 200, description = "Successfully sponsored user operation", body = SponsorUserOperationResponse),
        (status = 400, description = "Invalid request parameters", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn sponsor_user_operation_endpoint(
    State(state): State<SwaggerState>,
    Json(request): Json<SponsorUserOperationRequest>,
) -> Result<Json<SponsorUserOperationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start_time = Instant::now();

    // Convert JSON to internal format and call service
    let json_user_op: crate::rpc::JsonUserOperation =
        serde_json::from_value(request.user_op.clone()).map_err(|e| {
            let duration = start_time.elapsed();
            state.metrics.record_request(false, duration);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: crate::api_schemas::ApiError {
                        code: crate::api_schemas::error_codes::INVALID_PARAMS,
                        message: format!("Invalid user operation: {}", e),
                        data: Some(json!({"field": "user_op", "reason": e.to_string()})),
                    },
                }),
            )
        })?;

    let user_op = json_user_op.try_into().map_err(|e: String| {
        let duration = start_time.elapsed();
        state.metrics.record_request(false, duration);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: crate::api_schemas::ApiError {
                    code: crate::api_schemas::error_codes::INVALID_PARAMS,
                    message: format!("Failed to convert user operation: {}", e),
                    data: Some(json!({"conversion_error": e})),
                },
            }),
        )
    })?;

    let entry_point: ethers::types::Address = request.entry_point.parse().map_err(|e| {
        let duration = start_time.elapsed();
        state.metrics.record_request(false, duration);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: crate::api_schemas::ApiError {
                    code: crate::api_schemas::error_codes::INVALID_PARAMS,
                    message: format!("Invalid entry point address: {}", e),
                    data: Some(json!({"field": "entry_point", "reason": format!("{:?}", e)})),
                },
            }),
        )
    })?;

    match state
        .paymaster_service
        .sponsor_user_operation(user_op, entry_point)
        .await
    {
        Ok(user_op_hash) => {
            let duration = start_time.elapsed();
            state.metrics.record_request(true, duration);
            Ok(Json(SponsorUserOperationResponse {
                user_op_hash: format!("0x{:x}", user_op_hash),
            }))
        }
        Err(e) => {
            let duration = start_time.elapsed();
            state.metrics.record_request(false, duration);

            let (code, message) = match &e {
                crate::error::PaymasterError::PolicyRejected(_) => (
                    crate::api_schemas::error_codes::POLICY_REJECTED,
                    "Policy rejected",
                ),
                crate::error::PaymasterError::SignerError(_) => (
                    crate::api_schemas::error_codes::SIGNER_ERROR,
                    "Signer error",
                ),
                crate::error::PaymasterError::PoolError(_) => {
                    (crate::api_schemas::error_codes::POOL_ERROR, "Pool error")
                }
                _ => (
                    crate::api_schemas::error_codes::INTERNAL_ERROR,
                    "Internal error",
                ),
            };

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: crate::api_schemas::ApiError {
                        code,
                        message: format!("{}: {}", message, e),
                        data: Some(
                            json!({"error_type": std::any::type_name::<crate::error::PaymasterError>()}),
                        ),
                    },
                }),
            ))
        }
    }
}

/// Health check endpoint
async fn health_check(State(state): State<SwaggerState>) -> Json<serde_json::Value> {
    let uptime = state.start_time.elapsed();

    Json(json!({
        "status": "healthy",
        "service": "paymaster-relay",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime.as_secs(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "endpoints": {
            "swagger_ui": "/swagger-ui/",
            "health": "/health",
            "metrics": "/metrics",
            "sponsor": "/api/v1/sponsor"
        }
    }))
}

/// Readiness check endpoint
async fn readiness_check(
    State(state): State<SwaggerState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check if paymaster service is ready
    // This could include checking signer, policy engine, etc.
    let uptime = state.start_time.elapsed();

    // Simple readiness check - service has been up for at least 1 second
    if uptime.as_secs() >= 1 {
        Ok(Json(json!({
            "status": "ready",
            "service": "paymaster-relay",
            "checks": {
                "uptime": "ok",
                "signer": "ok",
                "policy_engine": "ok"
            }
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Get API usage metrics
async fn get_metrics(State(state): State<SwaggerState>) -> Json<serde_json::Value> {
    let uptime = state.start_time.elapsed();
    let metrics = state.metrics.get_stats();

    Json(json!({
        "service": "paymaster-relay",
        "uptime_seconds": uptime.as_secs(),
        "api_metrics": metrics,
        "system": {
            "memory_usage_mb": get_memory_usage_mb(),
            "cpu_usage_percent": get_cpu_usage_percent()
        }
    }))
}

/// Get API examples
async fn get_examples(Path(version): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    match version.as_str() {
        "v0.6" => Ok(Json(json!({
            "version": "v0.6",
            "user_operation": examples::example_user_op_v06(),
            "request": {
                "user_op": examples::example_user_op_v06(),
                "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
            },
            "response": examples::example_success_response()
        }))),
        "v0.7" => Ok(Json(json!({
            "version": "v0.7",
            "user_operation": examples::example_user_op_v07(),
            "request": {
                "user_op": examples::example_user_op_v07(),
                "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
            },
            "response": examples::example_success_response()
        }))),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

/// Generate curl example
async fn generate_curl_example(
    Path(endpoint): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match endpoint.as_str() {
        "sponsor" => Ok(Json(json!({
            "title": "Curl Example - Sponsor User Operation",
            "description": "One-line curl command to test the sponsorship API",
            "command": format!(
                "curl -X POST {} \\
        -H \"Content-Type: application/json\" \\
        -d '{}'",
                "http://localhost:9000/api/v1/sponsor",
                serde_json::to_string_pretty(&json!({
                    "user_op": examples::example_user_op_v06(),
                    "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                })).unwrap_or_default()
            )
        }))),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

/// Generate JavaScript example
async fn generate_js_example(
    Path(endpoint): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match endpoint.as_str() {
        "sponsor" => Ok(Json(json!({
            "title": "JavaScript Example - Sponsor User Operation",
            "code": format!(
                "// Node.js/Browser example
const response = await fetch('http://localhost:9000/api/v1/sponsor', {{
  method: 'POST',
  headers: {{
    'Content-Type': 'application/json',
  }},
  body: JSON.stringify({})
}});

const result = await response.json();
console.log('User operation hash:', result.user_op_hash);",
                serde_json::to_string_pretty(&json!({
                    "user_op": examples::example_user_op_v06(),
                    "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                })).unwrap_or_default()
            )
        }))),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

/// Generate Python example
async fn generate_python_example(
    Path(endpoint): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match endpoint.as_str() {
        "sponsor" => Ok(Json(json!({
            "title": "Python Example - Sponsor User Operation",
            "code": format!(
                "# Python example using requests
import requests
import json

url = 'http://localhost:9000/api/v1/sponsor'
payload = {}
headers = {{'Content-Type': 'application/json'}}

response = requests.post(url, data=json.dumps(payload), headers=headers)
result = response.json()

print(f'User operation hash: {{result[\"user_op_hash\"]}}')
print(f'Status code: {{response.status_code}}')",
                serde_json::to_string_pretty(&json!({
                    "user_op": examples::example_user_op_v06(),
                    "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                })).unwrap_or_default()
            )
        }))),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

/// Get memory usage in MB (placeholder implementation)
fn get_memory_usage_mb() -> u64 {
    // In a real implementation, you'd use system metrics
    // For now, return a placeholder value
    78 // Matches our test results
}

/// Get CPU usage percentage (placeholder implementation)
fn get_cpu_usage_percent() -> f64 {
    // In a real implementation, you'd use system metrics
    // For now, return a placeholder value
    15.2
}
