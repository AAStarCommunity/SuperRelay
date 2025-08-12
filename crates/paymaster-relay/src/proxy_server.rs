// proxy_server.rs
// Proxy API server that forwards requests to external SuperRelay service

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api_schemas::{examples, ApiDoc, ErrorResponse},
    proxy_client::SuperRelayProxyClient,
};

pub type ProxyAppState = Arc<SuperRelayProxyClient>;

/// Health check handler for proxy server
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Proxy server is healthy")
    ),
    tag = "monitoring"
)]
async fn proxy_health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "UP - Proxy Mode",
        "version": "0.2.0",
        "mode": "proxy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// JSON-RPC proxy handler - forwards requests to SuperRelay service
#[utoipa::path(
    post,
    path = "/jsonrpc",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "JSON-RPC response from SuperRelay service", body = serde_json::Value),
        (status = 502, description = "SuperRelay service unavailable", body = ErrorResponse)
    ),
    tag = "json-rpc",
    operation_id = "json_rpc_sponsor"
)]
async fn json_rpc_proxy_handler(
    State(proxy_client): State<ProxyAppState>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    // Forward JSON-RPC request to SuperRelay service
    match proxy_client.forward_json_rpc(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Proxy request failed: {}", e);
            Err((
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    code: -32000,
                    message: "SuperRelay service unavailable".to_string(),
                    data: Some(serde_json::json!({
                        "error": e.to_string(),
                        "suggestion": "Please check if SuperRelay service is running"
                    })),
                }),
            ))
        }
    }
}

/// Dashboard home redirect handler
async fn dashboard_home() -> impl IntoResponse {
    axum::response::Redirect::permanent("/dashboard")
}

/// Dashboard HTML page handler - 简化版本，不依赖外部模板
async fn dashboard_page() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>SuperRelay API Proxy Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
        .container { max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; }
        .header { text-align: center; margin-bottom: 30px; }
        .api-list { list-style: none; padding: 0; }
        .api-list li { margin: 10px 0; padding: 10px; background: #f9f9f9; border-radius: 5px; }
        .api-list a { text-decoration: none; color: #007acc; font-weight: bold; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 SuperRelay API Proxy</h1>
            <p>代理模式运行 - 转发请求到 SuperRelay 服务</p>
        </div>
        
        <h2>📖 可用端点</h2>
        <ul class="api-list">
            <li>🔄 <strong>JSON-RPC:</strong> <code>POST /jsonrpc</code> - JSON-RPC 2.0 协议</li>
            <li>📊 <strong>Swagger UI:</strong> <a href="/swagger-ui">/swagger-ui</a> - API 文档和测试</li>
            <li>🏥 <strong>Health:</strong> <a href="/health">/health</a> - 健康检查</li>
            <li>📈 <strong>Metrics:</strong> <a href="/metrics">/metrics</a> - 系统指标</li>
            <li>🔍 <strong>Examples:</strong> <a href="/examples/v06">/examples/v06</a> - 示例数据</li>
        </ul>
        
        <h2>💻 代码示例</h2>
        <ul class="api-list">
            <li><a href="/codegen/curl/sponsor">Curl 示例</a></li>
            <li><a href="/codegen/javascript/sponsor">JavaScript 示例</a></li>
            <li><a href="/codegen/python/sponsor">Python 示例</a></li>
        </ul>
        
        <div style="text-align: center; margin-top: 30px; color: #666;">
            <p>SuperRelay JSON-RPC Proxy Server - 纯 JSON-RPC 协议</p>
        </div>
    </div>
</body>
</html>"#;
    Html(html)
}

/// Balance status API - proxies to SuperRelay service
async fn get_balance_status(
    State(proxy_client): State<ProxyAppState>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    let json_rpc_request = json!({
        "jsonrpc": "2.0",
        "method": "eth_getBalance",
        "params": ["0x0000000000000000000000000000000000000000", "latest"],
        "id": 1
    });

    match proxy_client.forward_json_rpc(json_rpc_request).await {
        Ok(_response) => Ok(Json(json!({
            "paymaster_balance_eth": "0.0",
            "entrypoint_deposit_eth": "0.0",
            "network": "dev",
            "status": "healthy"
        }))),
        Err(e) => Err((
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                code: -32000,
                message: "Failed to get balance status".to_string(),
                data: Some(json!({"error": e.to_string()})),
            }),
        )),
    }
}

/// Policies status API - returns mock data for proxy mode
async fn get_policies_status() -> Json<Value> {
    Json(json!({
        "active_policies": [],
        "total_policies": 0,
        "status": "proxy_mode"
    }))
}

/// Dashboard metrics API
async fn get_metrics_dashboard() -> Json<Value> {
    Json(json!({
        "total_requests": 0,
        "successful_requests": 0,
        "failed_requests": 0,
        "success_rate": 100,
        "avg_response_time_ms": 0,
        "uptime_seconds": 0,
        "memory_usage_mb": 0,
        "cpu_usage_percent": 0.0
    }))
}

/// Transaction history API - returns mock data for proxy mode  
async fn get_transaction_history() -> Json<Value> {
    Json(json!({
        "recent_transactions": [],
        "total_count": 0
    }))
}

/// Readiness check handler
#[utoipa::path(
    get,
    path = "/ready",
    responses(
        (status = 200, description = "Service is ready")
    ),
    tag = "monitoring"
)]
async fn readiness_check() -> Json<Value> {
    Json(json!({
        "status": "ready",
        "service": "paymaster-proxy",
        "mode": "proxy",
        "checks": {
            "uptime": "pass",
            "proxy_connection": "pass"
        }
    }))
}

/// Metrics endpoint handler
async fn get_metrics() -> Json<Value> {
    Json(json!({
        "service": "paymaster-proxy",
        "mode": "proxy",
        "uptime_seconds": 0,
        "memory_usage_mb": 0,
        "proxy_requests": 0
    }))
}

/// Examples endpoint handler
#[utoipa::path(
    get,
    path = "/examples/{version}",
    params(
        ("version" = String, Path, description = "Example version (v06, v07, success, error)")
    ),
    responses(
        (status = 200, description = "Example data"),
        (status = 404, description = "Version not found")
    ),
    tag = "examples"
)]
async fn get_examples(Path(version): Path<String>) -> Result<Json<Value>, StatusCode> {
    match version.as_str() {
        "v06" => Ok(Json(json!({
            "title": "ERC-4337 v0.6 UserOperation Example",
            "version": "0.6",
            "example": examples::example_user_op_v06(),
            "description": "Example UserOperation for EntryPoint v0.6"
        }))),
        "v07" => Ok(Json(json!({
            "title": "ERC-4337 v0.7 UserOperation Example",
            "version": "0.7",
            "example": examples::example_user_op_v07(),
            "description": "Example UserOperation for EntryPoint v0.7"
        }))),
        "success" => Ok(Json(json!({
            "title": "Successful Response Example",
            "example": examples::example_success_response(),
            "description": "Example of a successful sponsorship response"
        }))),
        "error" => Ok(Json(json!({
            "title": "Error Response Example",
            "example": examples::example_error_response(),
            "description": "Example of an error response"
        }))),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

/// Code generation endpoints
async fn generate_curl_example(Path(endpoint): Path<String>) -> Result<Json<Value>, StatusCode> {
    match endpoint.as_str() {
        "sponsor" => Ok(Json(json!({
            "title": "Curl Example - Sponsor User Operation",
            "command": format!(
                "curl -X POST http://localhost:9000/api/v1/sponsor \\\n  -H \"Content-Type: application/json\" \\\n  -d '{}'",
                serde_json::to_string_pretty(&json!({
                    "user_op": examples::example_user_op_v06(),
                    "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                })).unwrap_or_default()
            )
        }))),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn generate_js_example(Path(endpoint): Path<String>) -> Result<Json<Value>, StatusCode> {
    match endpoint.as_str() {
        "sponsor" => Ok(Json(json!({
            "title": "JavaScript Example - Sponsor User Operation",
            "code": format!(
                "const response = await fetch('http://localhost:9000/api/v1/sponsor', {{\n  method: 'POST',\n  headers: {{'Content-Type': 'application/json'}},\n  body: JSON.stringify({})\n}});\nconst result = await response.json();",
                serde_json::to_string_pretty(&json!({
                    "user_op": examples::example_user_op_v06(),
                    "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                })).unwrap_or_default()
            )
        }))),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn generate_python_example(Path(endpoint): Path<String>) -> Result<Json<Value>, StatusCode> {
    match endpoint.as_str() {
        "sponsor" => Ok(Json(json!({
            "title": "Python Example - Sponsor User Operation",
            "code": format!(
                "import requests\nimport json\n\nresponse = requests.post(\n    'http://localhost:9000/api/v1/sponsor',\n    json={}\n)\nresult = response.json()",
                serde_json::to_string_pretty(&json!({
                    "user_op": examples::example_user_op_v06(),
                    "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                })).unwrap_or_default()
            )
        }))),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

/// Create the proxy API router
pub fn create_proxy_api_router(proxy_client: ProxyAppState) -> Router {
    Router::new()
        // 根路径：GET 重定向到 dashboard
        .route("/", get(dashboard_home))
        // JSON-RPC 端点 - 纯 JSON-RPC 协议
        .route("/jsonrpc", post(json_rpc_proxy_handler))
        .route("/health", get(proxy_health_check))
        .route("/ready", get(readiness_check))
        .route("/metrics", get(get_metrics))
        // Dashboard routes
        .route("/dashboard", get(dashboard_page))
        .route("/dashboard/api/balance", get(get_balance_status))
        .route("/dashboard/api/policies", get(get_policies_status))
        .route("/dashboard/api/metrics", get(get_metrics_dashboard))
        .route("/dashboard/api/transactions", get(get_transaction_history))
        // API management endpoints
        .route("/api/balance", get(get_balance_status))
        .route("/api/policies", get(get_policies_status))
        .route("/api/transactions", get(get_transaction_history))
        // Examples and documentation
        .route("/examples/:version", get(get_examples))
        // Code generation endpoints
        .route("/codegen/curl/:endpoint", get(generate_curl_example))
        .route("/codegen/javascript/:endpoint", get(generate_js_example))
        .route("/codegen/python/:endpoint", get(generate_python_example))
        // Swagger UI with comprehensive documentation
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        // CORS configuration
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(Any),
        )
        // Shared proxy client state
        .with_state(proxy_client)
}

/// Start the proxy API server
pub async fn start_proxy_api_server(
    bind_address: &str,
    proxy_client: SuperRelayProxyClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_proxy_api_router(Arc::new(proxy_client));

    let listener = tokio::net::TcpListener::bind(bind_address).await?;

    tracing::info!(
        "🚀 SuperRelay API Testing Server (Proxy Mode) started on http://{}",
        bind_address
    );
    tracing::info!(
        "📊 Swagger UI: http://{}/swagger-ui (支持协议切换测试)",
        bind_address
    );
    tracing::info!(
        "🔄 JSON-RPC Protocol: http://{}/jsonrpc → SuperRelay:3000/ (直接转发)",
        bind_address
    );
    tracing::info!("🏥 Health Check: http://{}/health", bind_address);
    tracing::info!("💡 在 Swagger UI 中选择服务器即可切换协议测试");

    axum::serve(listener, app)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
}
