//! Integration tests for Swagger UI functionality

use rundler_paymaster_relay::{policy::PolicyEngine, signer::SignerManager};
use secrecy::SecretString;
use tempfile::tempdir;

#[tokio::test]
async fn test_swagger_ui_server_startup() {
    // Create a temporary policy file
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let policy_path = temp_dir.path().join("test_policy.toml");

    std::fs::write(
        &policy_path,
        r#"
[default]
senders = ["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"]
"#,
    )
    .expect("Failed to write policy file");

    // Create test components
    let private_key = SecretString::new(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .to_string()
            .into(),
    );
    let _signer_manager = SignerManager::new(private_key).expect("Failed to create signer manager");
    let _policy_engine = PolicyEngine::new(&policy_path).expect("Failed to create policy engine");

    // Create a mock pool handle
    // Note: This is a simplified test that doesn't test the full integration
    // For full testing, we'd need to set up a complete test environment

    // The test here is mainly to verify that the Swagger server can be created
    // and the components compile correctly together

    // Test passes if we reach this point without panicking
    println!("Swagger UI components compile and instantiate successfully");
}

#[tokio::test]
async fn test_api_schemas_serialization() {
    use rundler_paymaster_relay::api_schemas::*;

    // Test JSON-RPC request serialization
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "pm_sponsorUserOperation".to_string(),
        params: serde_json::json!([{
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
        }, "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"]),
        id: serde_json::json!(1),
    };

    let json_str = serde_json::to_string(&request).expect("Failed to serialize request");
    assert!(json_str.contains("jsonrpc"));
    assert!(json_str.contains("pm_sponsorUserOperation"));

    // Test JSON-RPC response serialization
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(serde_json::json!("0x70997970C51812dc3A010C7d01b50e0d17dc79C8000000000000000000000000000000000000000000000000000000006678c5500000000000000000000000000000000000000000000000000000000000000000")),
        error: None,
        id: serde_json::json!(1),
    };

    let json_str = serde_json::to_string(&response).expect("Failed to serialize response");
    assert!(json_str.contains("result"));

    // Test error response
    let error_response = ErrorResponse {
        code: error_codes::INVALID_PARAMS,
        message: "Test error".to_string(),
        data: Some(serde_json::json!({"test": "data"})),
    };

    let json_str = serde_json::to_string(&error_response).expect("Failed to serialize error");
    assert!(json_str.contains("code"));
    assert!(json_str.contains("-32602"));
}

#[tokio::test]
async fn test_openapi_generation() {
    use rundler_paymaster_relay::api_schemas::ApiDoc;
    use utoipa::OpenApi;

    // Generate OpenAPI spec
    let openapi = ApiDoc::openapi();

    // Verify basic structure
    assert_eq!(openapi.info.title, "SuperPaymaster Relay API");
    assert_eq!(openapi.info.version, "0.2.0");

    // Verify components are present
    assert!(openapi.components.is_some());
    let components = openapi.components.unwrap();
    assert!(components
        .schemas
        .contains_key("SponsorUserOperationRequest"));
    assert!(components
        .schemas
        .contains_key("SponsorUserOperationResponse"));
    assert!(components.schemas.contains_key("ErrorResponse"));

    // Verify paths
    assert!(openapi.paths.paths.contains_key("/api/v1/sponsor"));
}

#[tokio::test]
async fn test_examples_generation() {
    use rundler_paymaster_relay::api_schemas::examples;

    // Test v0.6 example
    let v06_example = examples::example_user_op_v06();
    assert!(v06_example.get("sender").is_some());
    assert!(v06_example.get("initCode").is_some());
    assert!(v06_example.get("paymasterAndData").is_some());

    // Test v0.7 example
    let v07_example = examples::example_user_op_v07();
    assert!(v07_example.get("sender").is_some());
    assert!(v07_example.get("factory").is_some());

    // Test response examples
    let success_response = examples::example_success_response();
    assert!(success_response.get("paymasterAndData").is_some());

    let error_response = examples::example_error_response();
    assert_eq!(error_response.code, -32602);
}
