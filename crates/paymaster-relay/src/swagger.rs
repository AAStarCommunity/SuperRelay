use crate::api_docs::ApiDoc;
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub async fn serve_swagger_ui(port: u16) {
    let app = Router::new().merge(
        SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind Swagger UI to port {}: {}", port, e);
            return;
        }
    };
    tracing::info!("Swagger UI available at http://{}/swagger-ui", addr);

    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("Failed to start Swagger UI server: {}", e);
    }
} 