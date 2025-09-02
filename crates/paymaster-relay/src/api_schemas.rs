// api_schemas.rs
// OpenAPI schemas and documentation for the Paymaster Relay API

use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::json;
use utoipa::{OpenApi, ToSchema};

/// Main OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::proxy_server::json_rpc_proxy_handler,
        crate::proxy_server::proxy_health_check,
        crate::proxy_server::readiness_check,
        crate::proxy_server::get_examples
    ),
    components(
        schemas(
            JsonRpcRequest,
            JsonRpcResponse,
            JsonRpcError,
            ErrorResponse,
            ApiError,
            crate::api_handlers::HealthResponse
        )
    ),
    tags(
        (name = "json-rpc", description = "JSON-RPC 2.0 - 区块链标准协议测试 pm_sponsorUserOperation"), 
        (name = "monitoring", description = "系统监控和健康检查端点"),
        (name = "examples", description = "示例数据和代码生成工具")
    ),
    info(
        title = "SuperPaymaster Relay API",
        version = "0.2.0",
        description = r#"
SuperRelay JSON-RPC 代理服务器

🔄 **JSON-RPC 2.0**: 标准区块链协议，直接转发到 SuperRelay:3000
📋 **交互式文档**: 通过 Swagger UI 测试 JSON-RPC 方法

⚙️ **支持方法**:
- pm_sponsorUserOperation: UserOperation 赞助服务
        "#,
        contact(
            name = "SuperPaymaster Team",
            url = "https://github.com/aastar/super-relay"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:9000", description = "Swagger UI JSON-RPC 代理服务"),
        (url = "http://localhost:3000", description = "Direct SuperRelay Service (高级用户)")
    )
)]
pub struct ApiDoc;

/// JSON-RPC 2.0 Request structure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "jsonrpc": "2.0",
    "method": "pm_sponsorUserOperation",
    "params": [
        {
            "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "nonce": "0x0",
            "callData": "0x",
            "callGasLimit": "0x186A0",
            "verificationGasLimit": "0x186A0", 
            "preVerificationGas": "0x5208",
            "maxFeePerGas": "0x3B9ACA00",
            "maxPriorityFeePerGas": "0x3B9ACA00",
            "signature": "0x",
            "initCode": "0x",
            "paymasterAndData": "0x"
        },
        "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
    ],
    "id": 1
}))]
pub struct JsonRpcRequest {
    /// JSON-RPC protocol version (must be "2.0")
    #[schema(example = "2.0")]
    pub jsonrpc: String,

    /// RPC method name
    #[schema(example = "pm_sponsorUserOperation")]
    pub method: String,

    /// Method parameters array
    pub params: serde_json::Value,

    /// Request identifier
    #[schema(example = 1)]
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 Response structure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "jsonrpc": "2.0",
    "result": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8000000000000000000000000000000000000000000000000000000006678c5500000000000000000000000000000000000000000000000000000000000000000",
    "id": 1
}))]
pub struct JsonRpcResponse {
    /// JSON-RPC protocol version
    #[schema(example = "2.0")]
    pub jsonrpc: String,

    /// Method result (only present on success)
    pub result: Option<serde_json::Value>,

    /// Error object (only present on error)
    pub error: Option<JsonRpcError>,

    /// Request identifier
    #[schema(example = 1)]
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 Error structure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "code": -32602,
    "message": "Invalid params: Invalid user operation format",
    "data": {
        "field": "sender",
        "reason": "Invalid address format"
    }
}))]
pub struct JsonRpcError {
    /// Error code
    #[schema(example = -32602)]
    pub code: i32,

    /// Error message
    #[schema(example = "Invalid params")]
    pub message: String,

    /// Additional error data
    pub data: Option<serde_json::Value>,
}

/// Standard error response structure
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "code": -32602,
    "message": "Invalid params: Invalid user operation format",
    "data": null
}))]
pub struct ErrorResponse {
    /// JSON-RPC error code
    #[schema(example = -32602)]
    pub code: i32,

    /// Human-readable error message
    #[schema(example = "Invalid params")]
    pub message: String,

    /// Additional error context
    pub data: Option<serde_json::Value>,
}

/// API error details
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiError {
    /// JSON-RPC error code
    #[schema(example = -32602)]
    pub code: i32,

    /// Human-readable error message
    #[schema(example = "Invalid params")]
    pub message: String,

    /// Additional error context
    pub data: Option<serde_json::Value>,
}

/// Error codes used by the API
pub mod error_codes {
    /// Invalid method parameters
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal server error
    pub const INTERNAL_ERROR: i32 = -32603;
    /// Policy rejected the operation
    pub const POLICY_REJECTED: i32 = -32001;
    /// Signer error
    pub const SIGNER_ERROR: i32 = -32002;
    /// Pool submission error
    pub const POOL_ERROR: i32 = -32003;
}

/// API examples for documentation
pub mod examples {
    use super::*;

    /// Example v0.6 UserOperation with realistic test data
    pub fn example_user_op_v06() -> serde_json::Value {
        serde_json::json!({
            "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "nonce": "0x0",
            "initCode": "0x",
            "callData": "0xb61d27f6000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000",
            "callGasLimit": "0x30D40",
            "verificationGasLimit": "0x186A0",
            "preVerificationGas": "0xC350",
            "maxFeePerGas": "0x59682F00",
            "maxPriorityFeePerGas": "0x59682F00",
            "paymasterAndData": "0x",
            "signature": "0xfffffffffffffffffffffffffffffff0000000000000000000000000000000007aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1c"
        })
    }

    /// Example v0.7 UserOperation with realistic test data
    pub fn example_user_op_v07() -> serde_json::Value {
        serde_json::json!({
            "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "nonce": "0x0",
            "callData": "0xb61d27f6000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000",
            "callGasLimit": "0x30D40",
            "verificationGasLimit": "0x186A0",
            "preVerificationGas": "0xC350",
            "maxFeePerGas": "0x59682F00",
            "maxPriorityFeePerGas": "0x59682F00",
            "signature": "0xfffffffffffffffffffffffffffffff0000000000000000000000000000000007aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1c",
            "factory": "0x9406Cc6185a346906296840746125a0E44976454",
            "factoryData": "0x5fbfb9cf000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266",
            "paymaster": "0x0000000000000000000000000000000000000000",
            "paymasterVerificationGasLimit": "0x186A0",
            "paymasterPostOpGasLimit": "0x4E20",
            "paymasterData": "0x"
        })
    }

    /// Example successful response with realistic paymaster data
    pub fn example_success_response() -> serde_json::Value {
        serde_json::json!({
            "paymasterAndData": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        })
    }

    /// Example error response
    pub fn example_error_response() -> ErrorResponse {
        ErrorResponse {
            code: error_codes::INVALID_PARAMS,
            message: "Invalid user operation format".to_string(),
            data: Some(serde_json::json!({
                "field": "sender",
                "reason": "Invalid address format"
            })),
        }
    }
}
