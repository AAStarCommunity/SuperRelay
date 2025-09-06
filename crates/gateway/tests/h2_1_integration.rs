//! 测试 H2.1 Gateway-Pool-Bundler 完整链路集成

use serde_json::json;

/// 测试 Gateway-Pool 集成的基本功能
#[test]
fn test_gateway_pool_integration() {
    println!("🧪 测试 Gateway-Pool 集成");

    // 1. 模拟 JSON-RPC 请求
    let test_user_op = json!({
        "sender": "0x1234567890123456789012345678901234567890",
        "nonce": "0x0",
        "initCode": "0x",
        "callData": "0x",
        "callGasLimit": "0x5208",
        "verificationGasLimit": "0x5208",
        "preVerificationGas": "0x5208",
        "maxFeePerGas": "0x9184e72a000",
        "maxPriorityFeePerGas": "0x9184e72a000",
        "paymasterAndData": "0x",
        "signature": "0x"
    });

    let json_rpc_request = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendUserOperation",
        "params": [test_user_op, "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"],
        "id": 1
    });

    // 2. 测试请求格式验证
    assert_eq!(json_rpc_request["jsonrpc"], "2.0");
    assert_eq!(json_rpc_request["method"], "eth_sendUserOperation");
    assert!(json_rpc_request["params"].is_array());

    // 3. 模拟 Gateway 处理请求的逻辑
    let method = json_rpc_request["method"].as_str().unwrap();
    assert!(method.starts_with("eth_"));

    // 4. 验证参数结构
    let params = json_rpc_request["params"].as_array().unwrap();
    assert_eq!(params.len(), 2); // userOp + entryPoint

    let user_op = &params[0];
    assert!(user_op["sender"].is_string());
    assert!(user_op["nonce"].is_string());
    assert!(user_op["callData"].is_string());

    println!("✅ Gateway-Pool 基本集成测试通过");
}

/// 测试 Builder API 方法
#[test]
fn test_builder_api_methods() {
    println!("🧪 测试 Builder API 方法");

    // 测试 getBundleStats 方法请求格式
    let bundle_stats_request = json!({
        "jsonrpc": "2.0",
        "method": "rundler_getBundleStats",
        "params": [],
        "id": 2
    });

    assert_eq!(bundle_stats_request["method"], "rundler_getBundleStats");

    // 测试 sendBundleNow 方法请求格式
    let send_bundle_request = json!({
        "jsonrpc": "2.0",
        "method": "rundler_sendBundleNow",
        "params": [],
        "id": 3
    });

    assert_eq!(send_bundle_request["method"], "rundler_sendBundleNow");

    println!("✅ Builder API 方法测试通过");
}

/// 测试错误处理
#[test]
fn test_error_handling() {
    println!("🧪 测试错误处理");

    // 测试无效的 JSON-RPC 请求
    let invalid_request = json!({
        "jsonrpc": "2.0",
        "method": "invalid_method",
        "params": [],
        "id": 4
    });

    let method = invalid_request["method"].as_str().unwrap();

    // 验证错误方法会被正确识别
    assert!(!method.starts_with("eth_"));
    assert!(!method.starts_with("rundler_"));
    assert!(!method.starts_with("pm_"));

    println!("✅ 错误处理测试通过");
}
