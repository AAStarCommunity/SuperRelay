// api_schemas.rs
// OpenAPI schemas and documentation for the Paymaster Relay API

use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

/// Main OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::swagger::sponsor_user_operation_endpoint
    ),
    components(
        schemas(
            SponsorUserOperationRequest,
            SponsorUserOperationResponse,
            JsonUserOperation,
            ErrorResponse,
            ApiError
        )
    ),
    tags(
        (name = "paymaster", description = "Paymaster Relay API endpoints for sponsoring user operations")
    ),
    info(
        title = "SuperPaymaster Relay API",
        version = "0.2.0",
        description = "Enterprise-grade Paymaster Relay service for ERC-4337 user operation sponsorship",
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
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "http://localhost:9000", description = "Swagger UI server")
    )
)]
pub struct ApiDoc;

/// Request structure for sponsoring a user operation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "user_op": {
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
    "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
}))]
pub struct SponsorUserOperationRequest {
    /// The user operation to sponsor (ERC-4337 v0.6 or v0.7 format)
    pub user_op: serde_json::Value,

    /// The EntryPoint contract address
    #[schema(example = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789")]
    pub entry_point: String,
}

/// Response structure for successful sponsorship
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "user_op_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
}))]
pub struct SponsorUserOperationResponse {
    /// Hash of the user operation after sponsorship
    #[schema(example = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")]
    pub user_op_hash: String,
}

/// Unified UserOperation structure supporting both v0.6 and v0.7 formats
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    "nonce": "0x0",
    "callData": "0x",
    "callGasLimit": "0x186A0",
    "verificationGasLimit": "0x186A0",
    "preVerificationGas": "0x5208", 
    "maxFeePerGas": "0x3B9ACA00",
    "maxPriorityFeePerGas": "0x3B9ACA00",
    "signature": "0x"
}))]
pub struct JsonUserOperation {
    /// The account making the operation
    #[schema(example = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")]
    pub sender: String,

    /// Anti-replay parameter (hex or decimal)
    #[schema(example = "0x0")]
    pub nonce: String,

    /// Data to pass to the account's execute function
    #[schema(example = "0x")]
    pub call_data: String,

    /// Gas limit for the account's execution phase
    #[schema(example = "0x186A0")]
    pub call_gas_limit: String,

    /// Gas limit for the account's verification phase  
    #[schema(example = "0x186A0")]
    pub verification_gas_limit: String,

    /// Gas overhead of this UserOperation
    #[schema(example = "0x5208")]
    pub pre_verification_gas: String,

    /// Maximum fee per gas unit (hex or decimal)
    #[schema(example = "0x3B9ACA00")]
    pub max_fee_per_gas: String,

    /// Maximum priority fee per gas unit (hex or decimal)
    #[schema(example = "0x3B9ACA00")]
    pub max_priority_fee_per_gas: String,

    /// Account signature
    #[schema(example = "0x")]
    pub signature: String,

    // v0.6 specific fields
    /// Account initialization code (v0.6 format)
    #[schema(example = "0x")]
    pub init_code: Option<String>,

    /// Paymaster address and data (v0.6 format)
    #[schema(example = "0x")]
    pub paymaster_and_data: Option<String>,

    // v0.7 specific fields
    /// Account factory address (v0.7 format)
    pub factory: Option<String>,

    /// Factory initialization data (v0.7 format)
    pub factory_data: Option<String>,

    /// Paymaster address (v0.7 format)
    pub paymaster: Option<String>,

    /// Paymaster verification gas limit (v0.7 format)
    pub paymaster_verification_gas_limit: Option<String>,

    /// Paymaster post-operation gas limit (v0.7 format)
    pub paymaster_post_op_gas_limit: Option<String>,

    /// Paymaster data (v0.7 format)
    pub paymaster_data: Option<String>,
}

/// Standard error response structure
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "error": {
        "code": -32602,
        "message": "Invalid params: Invalid user operation format",
        "data": null
    }
}))]
pub struct ErrorResponse {
    /// Error details
    pub error: ApiError,
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

    /// Example v0.6 UserOperation
    pub fn example_user_op_v06() -> serde_json::Value {
        serde_json::json!({
            "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "nonce": "0x0",
            "initCode": "0x",
            "callData": "0x",
            "callGasLimit": "0x186A0",
            "verificationGasLimit": "0x186A0",
            "preVerificationGas": "0x5208",
            "maxFeePerGas": "0x3B9ACA00",
            "maxPriorityFeePerGas": "0x3B9ACA00",
            "paymasterAndData": "0x",
            "signature": "0x"
        })
    }

    /// Example v0.7 UserOperation
    pub fn example_user_op_v07() -> serde_json::Value {
        serde_json::json!({
            "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "nonce": "0x0",
            "callData": "0x",
            "callGasLimit": "0x186A0",
            "verificationGasLimit": "0x186A0",
            "preVerificationGas": "0x5208",
            "maxFeePerGas": "0x3B9ACA00",
            "maxPriorityFeePerGas": "0x3B9ACA00",
            "signature": "0x",
            "factory": "0x9406Cc6185a346906296840746125a0E44976454",
            "factoryData": "0x5fbfb9cf000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266",
            "paymaster": null,
            "paymasterVerificationGasLimit": null,
            "paymasterPostOpGasLimit": null,
            "paymasterData": null
        })
    }

    /// Example successful response
    pub fn example_success_response() -> SponsorUserOperationResponse {
        SponsorUserOperationResponse {
            user_op_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                .to_string(),
        }
    }

    /// Example error response
    pub fn example_error_response() -> ErrorResponse {
        ErrorResponse {
            error: ApiError {
                code: error_codes::INVALID_PARAMS,
                message: "Invalid user operation format".to_string(),
                data: Some(serde_json::json!({
                    "field": "sender",
                    "reason": "Invalid address format"
                })),
            },
        }
    }
}
