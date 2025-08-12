// api_handlers.rs
// Basic HTTP handlers for health checks and monitoring

use axum::response::Json;
use utoipa::ToSchema;

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "monitoring"
)]
pub async fn health_check_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "UP".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Health check response structure
#[derive(serde::Serialize, serde::Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    #[schema(example = "UP")]
    pub status: String,

    /// Service version
    #[schema(example = "0.2.0")]
    pub version: String,

    /// Response timestamp
    #[schema(example = "2024-01-01T12:00:00Z")]
    pub timestamp: String,
}
