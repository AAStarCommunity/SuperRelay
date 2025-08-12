// api_server.rs
// Axum HTTP server with utoipa OpenAPI integration

use std::sync::Arc;

use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api_handlers::{health_check_handler, sponsor_user_operation_handler},
    api_schemas::ApiDoc,
    rpc::PaymasterRelayApiServerImpl,
};

pub type AppState = Arc<PaymasterRelayApiServerImpl>;

/// Create the Axum router with all API endpoints and Swagger UI
pub fn create_api_router(app_state: AppState) -> Router {
    Router::new()
        // API endpoints
        .route("/api/v1/sponsor", post(sponsor_user_operation_handler))
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
        "Starting SuperPaymaster API server on http://{}",
        bind_address
    );
    tracing::info!("Swagger UI available at http://{}/swagger-ui", bind_address);

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
