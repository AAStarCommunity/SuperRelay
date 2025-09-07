//! SuperRelay Integration Tests
//!
//! 完整的端到端测试，验证SuperRelay的核心功能
//! 包括API测试、签名测试、策略测试等

use std::{
    error::Error,
    future::Future,
    pin::Pin,
    process::Command,
    time::{Duration, Instant},
};

use serde_json::{json, Value};
use tokio::time::sleep;

/// 测试配置 - 使用环境变量或默认值
fn get_anvil_url() -> String {
    std::env::var("ANVIL_URL")
        .or_else(|_| std::env::var("NODE_HTTP"))
        .unwrap_or_else(|_| "http://localhost:8545".to_string())
}

fn get_rundler_url() -> String {
    std::env::var("RUNDLER_URL")
        .or_else(|_| std::env::var("SUPER_RELAY_URL"))
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
}

fn get_dashboard_url() -> String {
    std::env::var("DASHBOARD_URL").unwrap_or_else(|_| "http://localhost:8082".to_string())
}
const ENTRYPOINT_ADDRESS: &str = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";

/// 测试结果统计
#[derive(Debug, Default)]
struct TestStats {
    passed: u32,
    failed: u32,
    skipped: u32,
}

/// 示例UserOperation
fn create_test_user_operation() -> Value {
    json!({
        "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "nonce": "0x0",
        "callData": "0x",
        "callGasLimit": "0x10000",
        "verificationGasLimit": "0x10000",
        "preVerificationGas": "0x5000",
        "maxFeePerGas": "0x3b9aca00",
        "maxPriorityFeePerGas": "0x3b9aca00",
        "signature": "0x",
        "initCode": "0x",
        "paymasterAndData": "0x"
    })
}

/// RPC调用辅助函数
async fn make_rpc_call(
    url: &str,
    method: &str,
    params: Value,
    id: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id
    });

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    Ok(response.json().await?)
}

type TestResult = Pin<Box<dyn Future<Output = Result<bool, Box<dyn Error>>> + Send>>;

/// 测试1: 基础连接性测试
fn test_basic_connectivity() -> TestResult {
    Box::pin(async move {
        println!("🔗 Testing basic connectivity...");

        // Test Anvil
        let anvil_response = make_rpc_call(&get_anvil_url(), "eth_chainId", json!([]), 1).await?;
        assert!(anvil_response.get("result").is_some());
        println!("  ✅ Anvil connection OK");

        // Test Rundler health
        let health_response = reqwest::get(&format!("{}/health", get_rundler_url())).await?;
        assert!(health_response.status().is_success());
        println!("  ✅ Rundler health OK");

        // Test Dashboard
        let dashboard_response = reqwest::get(&get_dashboard_url()).await?;
        assert!(dashboard_response.status().is_success());
        println!("  ✅ Dashboard accessibility OK");

        Ok(true)
    })
}

/// 测试2: 支持的RPC方法验证
fn test_supported_rpc_methods() -> TestResult {
    Box::pin(async move {
        println!("📋 Testing supported RPC methods...");

        let methods_to_test = vec![
            ("eth_chainId", json!([])),
            ("eth_supportedEntryPoints", json!([])),
            (
                "pm_sponsorUserOperation",
                json!([create_test_user_operation(), ENTRYPOINT_ADDRESS]),
            ),
        ];

        for (method, params) in methods_to_test {
            let response = make_rpc_call(&get_rundler_url(), method, params, 1).await?;

            if response.get("error").is_some() {
                if method == "pm_sponsorUserOperation" {
                    // 这个方法预期会有业务逻辑错误，但不应该是"Method not found"
                    let error_msg = response["error"]["message"].as_str().unwrap_or("");
                    assert!(
                        !error_msg.contains("Method not found"),
                        "pm_sponsorUserOperation should not return 'Method not found'"
                    );
                    println!(
                        "  ✅ {} - API registered (business logic error: {})",
                        method, error_msg
                    );
                } else {
                    return Err(format!("Method {} failed: {}", method, response["error"]).into());
                }
            } else {
                println!("  ✅ {} - Success", method);
            }
        }

        Ok(true)
    })
}

/// 测试3: EntryPoint配置验证
fn test_entrypoint_configuration() -> TestResult {
    Box::pin(async move {
        println!("🎯 Testing EntryPoint configuration...");

        let response =
            make_rpc_call(&get_rundler_url(), "eth_supportedEntryPoints", json!([]), 1).await?;

        let supported_entries = response["result"]
            .as_array()
            .ok_or("Expected array of supported EntryPoints")?;

        // 验证至少有一个EntryPoint被支持
        assert!(
            !supported_entries.is_empty(),
            "Should have at least one supported EntryPoint"
        );

        // 验证标准EntryPoint v0.6是否在支持列表中
        let standard_v06 = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";
        let mut found_standard = false;
        for entry in supported_entries {
            if let Some(addr) = entry.as_str() {
                println!("  📍 Supported EntryPoint: {}", addr);
                if addr.to_lowercase() == standard_v06.to_lowercase() {
                    found_standard = true;
                }
            }
        }

        assert!(
            found_standard,
            "Standard EntryPoint v0.6 should be supported"
        );
        println!(
            "  ✅ Standard EntryPoint v0.6 {} is supported",
            standard_v06
        );

        Ok(true)
    })
}

/// 测试4: Paymaster余额状态检查
fn test_paymaster_balance_status() -> TestResult {
    Box::pin(async move {
        println!("💰 Testing paymaster balance status...");

        // 运行fund_paymaster脚本状态检查
        let output = Command::new("../../scripts/fund_paymaster.sh").output()?;

        let status_output = String::from_utf8(output.stdout)?;

        // 验证关键信息是否存在
        assert!(status_output.contains("SuperPaymaster Financial Status Report"));
        assert!(status_output.contains(ENTRYPOINT_ADDRESS));

        // 检查是否有余额信息
        let has_balance_info = status_output.contains("Funder Account:")
            && status_output.contains("Paymaster Account:")
            && status_output.contains("EntryPoint Deposit:");

        assert!(has_balance_info, "Balance information should be displayed");
        println!("  ✅ Paymaster balance status reporting works");

        Ok(true)
    })
}

/// 测试5: API性能基准测试
fn test_api_performance() -> TestResult {
    Box::pin(async move {
        println!("⚡ Testing API performance...");

        let test_cases = 10;
        let mut response_times = Vec::new();

        for i in 0..test_cases {
            let start = Instant::now();

            let _response = make_rpc_call(
                &get_rundler_url(),
                "pm_sponsorUserOperation",
                json!([create_test_user_operation(), ENTRYPOINT_ADDRESS]),
                i + 1,
            )
            .await?;

            let duration = start.elapsed();
            response_times.push(duration);

            // 避免过快请求
            sleep(Duration::from_millis(100)).await;
        }

        let avg_response_time = response_times.iter().sum::<Duration>() / test_cases as u32;
        let max_response_time = response_times.iter().max().unwrap();

        println!("  📊 Performance metrics:");
        println!("    Average response time: {:?}", avg_response_time);
        println!("    Maximum response time: {:?}", max_response_time);

        // 性能要求：平均响应时间应小于1秒
        assert!(
            avg_response_time < Duration::from_secs(1),
            "Average response time should be less than 1 second"
        );

        println!("  ✅ API performance meets requirements");

        Ok(true)
    })
}

/// 测试6: Dashboard功能验证
fn test_dashboard_functionality() -> TestResult {
    Box::pin(async move {
        println!("📊 Testing dashboard functionality...");

        let client = reqwest::Client::new();

        // 测试主页面
        let main_page = client.get(get_dashboard_url()).send().await?;
        assert!(main_page.status().is_success());

        let content = main_page.text().await?;
        assert!(content.contains("SuperPaymaster"));
        assert!(content.contains("Operations Dashboard"));

        println!("  ✅ Dashboard main page loads correctly");

        // 测试API endpoints (如果存在)
        let api_endpoints = vec![
            "/dashboard/api/balance",
            "/dashboard/api/policies",
            "/dashboard/api/metrics",
        ];

        for endpoint in api_endpoints {
            let url = format!("{}{}", get_dashboard_url(), endpoint);
            let response = client.get(&url).send().await;

            if let Ok(resp) = response {
                if resp.status().is_success() {
                    println!("  ✅ API endpoint {} works", endpoint);
                } else {
                    println!("  ⚠️  API endpoint {} returned {}", endpoint, resp.status());
                }
            } else {
                println!("  ⚠️  API endpoint {} not available", endpoint);
            }
        }

        Ok(true)
    })
}

/// 主测试入口
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 SuperRelay Integration Test Suite");
    println!("=====================================");

    let mut stats = TestStats::default();

    // 定义测试用例
    let test_cases: Vec<(&str, TestResult)> = vec![
        ("Basic Connectivity", test_basic_connectivity()),
        ("Supported RPC Methods", test_supported_rpc_methods()),
        ("EntryPoint Configuration", test_entrypoint_configuration()),
        ("Paymaster Balance Status", test_paymaster_balance_status()),
        ("API Performance", test_api_performance()),
        ("Dashboard Functionality", test_dashboard_functionality()),
    ];

    // 执行测试
    for (test_name, test_fn) in test_cases {
        println!("\n🧪 Running test: {}", test_name);
        match test_fn.await {
            Ok(_) => {
                println!("✅ PASSED: {}", test_name);
                stats.passed += 1;
            }
            Err(e) => {
                println!("❌ FAILED: {} - Error: {}", test_name, e);
                stats.failed += 1;
            }
        }
    }

    // 输出测试结果
    println!("\n📊 Test Results Summary");
    println!("========================");
    println!("✅ Passed: {}", stats.passed);
    println!("❌ Failed: {}", stats.failed);
    println!("⏭️  Skipped: {}", stats.skipped);
    println!(
        "📈 Success Rate: {:.1}%",
        (stats.passed as f64 / (stats.passed + stats.failed) as f64) * 100.0
    );

    if stats.failed > 0 {
        std::process::exit(1);
    }

    println!("\n🎉 All tests passed! SuperRelay is ready for production.");
    Ok(())
}
