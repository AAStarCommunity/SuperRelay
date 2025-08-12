// api_handlers.rs
// Axum HTTP handlers that integrate with utoipa for OpenAPI generation

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json};
use utoipa::ToSchema;

use crate::{
    api_schemas::{ErrorResponse, SponsorUserOperationRequest, SponsorUserOperationResponse},
    rpc::PaymasterRelayApiServerImpl,
};

/// HTTP handler for sponsoring user operations
///
/// This endpoint provides HTTP REST interface on top of the JSON-RPC implementation.
/// It transforms HTTP requests to RPC calls and returns JSON responses.
#[utoipa::path(
    post,
    path = "/api/v1/sponsor",
    request_body = SponsorUserOperationRequest,
    responses(
        (status = 200, description = "Successfully sponsored the user operation", body = SponsorUserOperationResponse),
        (status = 400, description = "Invalid request parameters", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "paymaster"
)]
pub async fn sponsor_user_operation_handler(
    State(_rpc_service): State<Arc<PaymasterRelayApiServerImpl>>,
    Json(_request): Json<SponsorUserOperationRequest>,
) -> Result<Json<SponsorUserOperationResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Implement actual RPC call integration
    // For now, return a placeholder response
    Ok(Json(SponsorUserOperationResponse {
        paymaster_and_data: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8000000000000000000000000000000000000000000000000000000006678c5500000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }))
}

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
