use std::sync::Arc;

use rundler_paymaster_relay::PaymasterRelayService;
use serde_json::Value;
use tracing::{debug, warn};

use crate::{
    error::{GatewayError, GatewayResult},
    gateway::JsonRpcRequest,
};

/// Router that handles request routing to appropriate rundler components
#[derive(Clone)]
pub struct GatewayRouter {
    // Future: Add rundler component references here
}

impl GatewayRouter {
    /// Create a new router
    pub fn new() -> Self {
        Self {}
    }

    /// Route request to paymaster service
    pub async fn route_to_paymaster(
        &self,
        paymaster_service: &Arc<PaymasterRelayService>,
        request: &JsonRpcRequest,
    ) -> GatewayResult<Value> {
        debug!("Routing to paymaster: {}", request.method);

        match request.method.as_str() {
            "pm_sponsorUserOperation" => {
                self.handle_sponsor_user_operation(paymaster_service, &request.params)
                    .await
            }
            _ => Err(GatewayError::InvalidRequest(format!(
                "Unknown paymaster method: {}",
                request.method
            ))),
        }
    }

    /// Route request to rundler components
    pub async fn route_to_rundler(&self, request: &JsonRpcRequest) -> GatewayResult<Value> {
        debug!("Routing to rundler: {}", request.method);

        // For now, return a placeholder response
        // TODO: Implement actual routing to rundler RPC components
        match request.method.as_str() {
            "eth_supportedEntryPoints" => Ok(serde_json::json!([
                "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
            ])),
            "eth_estimateUserOperationGas" => Ok(serde_json::json!({
                "preVerificationGas": "0x5208",
                "verificationGasLimit": "0x186A0",
                "callGasLimit": "0x186A0"
            })),
            "eth_sendUserOperation" => Ok(serde_json::json!("0x1234567890abcdef")),
            "eth_getUserOperationByHash" => Ok(Value::Null),
            "eth_getUserOperationReceipt" => Ok(Value::Null),
            _ => {
                warn!("Unhandled rundler method: {}", request.method);
                Err(GatewayError::InvalidRequest(format!(
                    "Method not implemented: {}",
                    request.method
                )))
            }
        }
    }

    /// Handle pm_sponsorUserOperation method
    async fn handle_sponsor_user_operation(
        &self,
        paymaster_service: &Arc<PaymasterRelayService>,
        params: &[Value],
    ) -> GatewayResult<Value> {
        if params.len() != 2 {
            return Err(GatewayError::InvalidRequest(
                "pm_sponsorUserOperation requires exactly 2 parameters".to_string(),
            ));
        }

        let _user_operation = &params[0];
        let _entry_point = &params[1];

        debug!(
            "Sponsoring UserOperation for entry point: {:?}",
            _entry_point
        );

        // Convert parameters to appropriate types and call paymaster service
        // TODO: Implement actual conversion and service call
        // For now, return a mock response

        Ok(serde_json::json!({
            "paymasterAndData": "0x1234567890abcdef",
            "preVerificationGas": "0x5208",
            "verificationGasLimit": "0x186A0",
            "callGasLimit": "0x186A0"
        }))
    }
}

impl Default for GatewayRouter {
    fn default() -> Self {
        Self::new()
    }
}
