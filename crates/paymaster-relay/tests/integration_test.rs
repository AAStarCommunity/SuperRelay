//! Integration tests for SuperRelay Paymaster Service

use std::time::Duration;

use jsonrpsee::server::ServerBuilder;
use rundler_paymaster_relay::{Config, PaymasterRelayApiServer, PaymasterRelayService};
use tokio::time::timeout;

#[tokio::test]
async fn test_service_creation() {
    let config = Config::default();
    let service = PaymasterRelayService::new(config).await;
    assert!(service.is_ok());
}

#[tokio::test]
async fn test_rpc_server_health() {
    let config = Config::default();
    let service = PaymasterRelayService::new(config)
        .await
        .expect("Failed to create service");

    // Start server on random port
    let server = ServerBuilder::default()
        .build("127.0.0.1:0")
        .await
        .expect("Failed to build server");

    let addr = server.local_addr().expect("Failed to get server address");
    let _handle = server.start(service.into_rpc());

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create HTTP client to test the health endpoint
    let client = reqwest::Client::new();
    let url = format!("http://{}", addr);

    // Test RPC health method
    let rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "pm_health",
        "id": 1
    });

    let response = timeout(Duration::from_secs(5), async {
        client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&rpc_request)
            .send()
            .await
    })
    .await;

    assert!(response.is_ok(), "Health check request timed out");
    let response = response
        .unwrap()
        .expect("Failed to send health check request");
    assert!(
        response.status().is_success(),
        "Health check returned error status: {}",
        response.status()
    );

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse JSON response");
    assert_eq!(
        body["result"], "ok",
        "Health check returned unexpected result: {:?}",
        body
    );
}

#[tokio::test]
async fn test_get_chain_id() {
    let config = Config::default();
    let service = PaymasterRelayService::new(config)
        .await
        .expect("Failed to create service");

    // Start server on random port
    let server = ServerBuilder::default()
        .build("127.0.0.1:0")
        .await
        .expect("Failed to build server");

    let addr = server.local_addr().expect("Failed to get server address");
    let _handle = server.start(service.into_rpc());

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create HTTP client to test the chain ID endpoint
    let client = reqwest::Client::new();
    let url = format!("http://{}", addr);

    // Test RPC chain ID method
    let rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "pm_getChainId",
        "id": 1
    });

    let response = timeout(Duration::from_secs(5), async {
        client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&rpc_request)
            .send()
            .await
    })
    .await;

    assert!(response.is_ok(), "Chain ID request timed out");
    let response = response.unwrap().expect("Failed to send chain ID request");
    assert!(
        response.status().is_success(),
        "Chain ID request returned error status: {}",
        response.status()
    );

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse JSON response");
    assert_eq!(
        body["result"], 31337,
        "Chain ID returned unexpected result: {:?}",
        body
    );
}

#[tokio::test]
async fn test_get_supported_entry_points() {
    let config = Config::default();
    let service = PaymasterRelayService::new(config)
        .await
        .expect("Failed to create service");

    // Start server on random port
    let server = ServerBuilder::default()
        .build("127.0.0.1:0")
        .await
        .expect("Failed to build server");

    let addr = server.local_addr().expect("Failed to get server address");
    let _handle = server.start(service.into_rpc());

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create HTTP client to test the supported entry points endpoint
    let client = reqwest::Client::new();
    let url = format!("http://{}", addr);

    // Test RPC supported entry points method
    let rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "pm_getSupportedEntryPoints",
        "id": 1
    });

    let response = timeout(Duration::from_secs(5), async {
        client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&rpc_request)
            .send()
            .await
    })
    .await;

    assert!(response.is_ok(), "Supported entry points request timed out");
    let response = response
        .unwrap()
        .expect("Failed to send supported entry points request");
    assert!(
        response.status().is_success(),
        "Supported entry points request returned error status: {}",
        response.status()
    );

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse JSON response");
    assert!(
        body["result"].is_array(),
        "Supported entry points should return an array: {:?}",
        body
    );

    let entry_points = body["result"].as_array().unwrap();
    assert_eq!(
        entry_points.len(),
        2,
        "Should have 2 supported entry points: {:?}",
        body
    );
}
