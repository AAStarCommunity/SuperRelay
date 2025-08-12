// proxy_client.rs
// HTTP client for proxying requests to external SuperRelay service

use std::time::Duration;

use reqwest::Client;
use serde_json::Value;

/// HTTP client for connecting to external SuperRelay service
#[derive(Clone)]
pub struct SuperRelayProxyClient {
    client: Client,
    base_url: String,
}

impl SuperRelayProxyClient {
    /// Create a new proxy client
    pub fn new(super_relay_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: super_relay_url.trim_end_matches('/').to_string(),
        }
    }

    /// Forward JSON-RPC request to SuperRelay service
    pub async fn forward_json_rpc(&self, request: Value) -> Result<Value, reqwest::Error> {
        let response = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        response.json().await
    }

    /// Test connection to SuperRelay service
    pub async fn health_check(&self) -> Result<bool, reqwest::Error> {
        let health_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health",
            "params": [],
            "id": 1
        });

        match self.forward_json_rpc(health_request).await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("SuperRelay health check failed: {}", e);
                Err(e)
            }
        }
    }
}
