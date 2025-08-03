// paymaster-relay/src/rpc.rs
// This file will contain the JSON-RPC API definition and implementation.

use std::str::FromStr;

use alloy_primitives::{Address as AlloyAddress, Bytes, U256};
use async_trait::async_trait;
use jsonrpsee::{proc_macros::rpc, types::ErrorObjectOwned};
use rundler_types::{chain::ChainSpec, v0_6, v0_7, UserOperationVariant};
use serde::{Deserialize, Serialize};

use crate::{
    service::PaymasterRelayService,
    validation::{InputValidator, ValidationLimits},
};

/// Parse a number string that can be either hex (0x prefix) or decimal
fn parse_hex_or_decimal(s: &str) -> Result<u128, String> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u128::from_str_radix(&s[2..], 16).map_err(|e| format!("Invalid hex number: {}", e))
    } else {
        s.parse::<u128>()
            .map_err(|e| format!("Invalid decimal number: {}", e))
    }
}

/// Simplified UserOperation structure for JSON-RPC deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonUserOperation {
    pub sender: String,
    pub nonce: String,
    pub call_data: String,
    pub call_gas_limit: String,
    pub verification_gas_limit: String,
    pub pre_verification_gas: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub signature: String,
    // v0.6 fields
    pub init_code: Option<String>,
    pub paymaster_and_data: Option<String>,
    // v0.7 fields
    pub factory: Option<String>,
    pub factory_data: Option<String>,
    pub paymaster: Option<String>,
    pub paymaster_verification_gas_limit: Option<String>,
    pub paymaster_post_op_gas_limit: Option<String>,
    pub paymaster_data: Option<String>,
}

impl TryInto<UserOperationVariant> for JsonUserOperation {
    type Error = String;

    fn try_into(self) -> Result<UserOperationVariant, Self::Error> {
        let chain_spec = ChainSpec::default(); // Use default for now

        // Parse common fields
        let sender = AlloyAddress::from_str(&self.sender)
            .map_err(|e| format!("Invalid sender address: {}", e))?;
        let nonce = U256::from_str(&self.nonce).map_err(|e| format!("Invalid nonce: {}", e))?;
        let call_data =
            Bytes::from_str(&self.call_data).map_err(|e| format!("Invalid call_data: {}", e))?;
        let call_gas_limit = parse_hex_or_decimal(&self.call_gas_limit)
            .map_err(|e| format!("Invalid call_gas_limit: {}", e))?;
        let verification_gas_limit = parse_hex_or_decimal(&self.verification_gas_limit)
            .map_err(|e| format!("Invalid verification_gas_limit: {}", e))?;
        let pre_verification_gas = parse_hex_or_decimal(&self.pre_verification_gas)
            .map_err(|e| format!("Invalid pre_verification_gas: {}", e))?;
        let max_fee_per_gas = parse_hex_or_decimal(&self.max_fee_per_gas)
            .map_err(|e| format!("Invalid max_fee_per_gas: {}", e))?;
        let max_priority_fee_per_gas = parse_hex_or_decimal(&self.max_priority_fee_per_gas)
            .map_err(|e| format!("Invalid max_priority_fee_per_gas: {}", e))?;
        let signature =
            Bytes::from_str(&self.signature).map_err(|e| format!("Invalid signature: {}", e))?;

        // Check if this is v0.6 or v0.7 based on field presence
        if self.init_code.is_some() || self.paymaster_and_data.is_some() {
            // v0.6 format
            let init_code = if let Some(ic) = self.init_code {
                Bytes::from_str(&ic).map_err(|e| format!("Invalid init_code: {}", e))?
            } else {
                Bytes::new()
            };
            let paymaster_and_data = if let Some(pad) = self.paymaster_and_data {
                Bytes::from_str(&pad).map_err(|e| format!("Invalid paymaster_and_data: {}", e))?
            } else {
                Bytes::new()
            };

            let uo = v0_6::UserOperationBuilder::new(
                &chain_spec,
                v0_6::UserOperationRequiredFields {
                    sender,
                    nonce,
                    init_code,
                    call_data,
                    call_gas_limit,
                    verification_gas_limit,
                    pre_verification_gas,
                    max_fee_per_gas,
                    max_priority_fee_per_gas,
                    paymaster_and_data,
                    signature,
                },
            )
            .build();

            Ok(UserOperationVariant::V0_6(uo))
        } else {
            // v0.7 format
            let mut builder = v0_7::UserOperationBuilder::new(
                &chain_spec,
                v0_7::UserOperationRequiredFields {
                    sender,
                    nonce,
                    call_data,
                    call_gas_limit,
                    verification_gas_limit,
                    pre_verification_gas,
                    max_fee_per_gas,
                    max_priority_fee_per_gas,
                    signature,
                },
            );

            // Handle optional v0.7 fields
            if let Some(factory) = self.factory {
                let factory_addr = AlloyAddress::from_str(&factory)
                    .map_err(|e| format!("Invalid factory address: {}", e))?;
                let factory_data = if let Some(fd) = self.factory_data {
                    Bytes::from_str(&fd).map_err(|e| format!("Invalid factory_data: {}", e))?
                } else {
                    Bytes::new()
                };
                builder = builder.factory(factory_addr, factory_data);
            }

            if let Some(paymaster) = self.paymaster {
                let paymaster_addr = AlloyAddress::from_str(&paymaster)
                    .map_err(|e| format!("Invalid paymaster address: {}", e))?;
                let paymaster_verification_gas_limit = if let Some(pvgl) =
                    self.paymaster_verification_gas_limit
                {
                    parse_hex_or_decimal(&pvgl)
                        .map_err(|e| format!("Invalid paymaster_verification_gas_limit: {}", e))?
                } else {
                    0
                };
                let paymaster_post_op_gas_limit =
                    if let Some(ppogl) = self.paymaster_post_op_gas_limit {
                        parse_hex_or_decimal(&ppogl)
                            .map_err(|e| format!("Invalid paymaster_post_op_gas_limit: {}", e))?
                    } else {
                        0
                    };
                let paymaster_data = if let Some(pd) = self.paymaster_data {
                    Bytes::from_str(&pd).map_err(|e| format!("Invalid paymaster_data: {}", e))?
                } else {
                    Bytes::new()
                };
                builder = builder.paymaster(
                    paymaster_addr,
                    paymaster_verification_gas_limit,
                    paymaster_post_op_gas_limit,
                    paymaster_data,
                );
            }

            let uo = builder.build();
            Ok(UserOperationVariant::V0_7(uo))
        }
    }
}

// Simple RPC request/response types for JSON handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SponsorUserOperationRequest {
    pub user_op: serde_json::Value,
    pub entry_point: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SponsorUserOperationResponse {
    pub user_op_hash: String,
}

#[rpc(client, server, namespace = "pm")]
#[async_trait]
pub trait PaymasterRelayApi {
    #[method(name = "sponsorUserOperation")]
    async fn sponsor_user_operation(
        &self,
        user_op: serde_json::Value,
        entry_point: String,
    ) -> Result<String, ErrorObjectOwned>;
}

pub struct PaymasterRelayApiServerImpl {
    pub service: std::sync::Arc<PaymasterRelayService>,
    pub validator: InputValidator,
}

impl PaymasterRelayApiServerImpl {
    pub fn new(service: std::sync::Arc<PaymasterRelayService>) -> Self {
        Self {
            service,
            validator: InputValidator::new(ValidationLimits::default()),
        }
    }

    pub fn new_with_limits(
        service: std::sync::Arc<PaymasterRelayService>,
        limits: ValidationLimits,
    ) -> Self {
        Self {
            service,
            validator: InputValidator::new(limits),
        }
    }
}

#[async_trait]
impl PaymasterRelayApiServer for PaymasterRelayApiServerImpl {
    async fn sponsor_user_operation(
        &self,
        user_op: serde_json::Value,
        entry_point: String,
    ) -> Result<String, ErrorObjectOwned> {
        // Enhanced input validation
        self.validator
            .validate_user_operation_json(&user_op)
            .map_err(|e| {
                ErrorObjectOwned::owned(
                    -32602,
                    "Input validation failed",
                    Some(format!("Validation error: {}", e)),
                )
            })?;

        let entry_point_addr = self
            .validator
            .validate_entry_point(&entry_point)
            .map_err(|e| {
                ErrorObjectOwned::owned(
                    -32602,
                    "Invalid entry point address",
                    Some(format!("Entry point validation error: {}", e)),
                )
            })?;

        // Convert JSON to UserOperation
        let json_user_op: JsonUserOperation = serde_json::from_value(user_op).map_err(|e| {
            ErrorObjectOwned::owned(
                -32602,
                "Invalid user operation format",
                Some(format!("JSON parsing error: {}", e)),
            )
        })?;

        let user_op_variant: UserOperationVariant = json_user_op
            .try_into()
            .map_err(|e| ErrorObjectOwned::owned(-32602, "Invalid user operation data", Some(e)))?;

        // Additional validation on converted UserOperation
        self.validator
            .validate_user_operation(&user_op_variant)
            .map_err(|e| {
                ErrorObjectOwned::owned(
                    -32602,
                    "User operation validation failed",
                    Some(format!("Operation validation error: {}", e)),
                )
            })?;

        // Call the service
        match self
            .service
            .sponsor_user_operation(user_op_variant, entry_point_addr)
            .await
        {
            Ok(sponsor_result) => {
                // Return JSON response with paymaster data
                let response = serde_json::json!({
                    "paymasterAndData": format!("0x{}", hex::encode(&sponsor_result.paymaster_and_data)),
                    "preVerificationGas": sponsor_result.pre_verification_gas.map(|g| format!("0x{:x}", g)),
                    "verificationGasLimit": sponsor_result.verification_gas_limit_uo.map(|g| format!("0x{:x}", g)),
                    "callGasLimit": sponsor_result.call_gas_limit.map(|g| format!("0x{:x}", g)),
                });
                Ok(response.to_string())
            }
            Err(e) => Err(e.into()),
        }
    }
}

/// Helper function to convert JsonUserOperation to UserOperationVariant
/// This is used by both RPC and REST API endpoints
pub fn json_user_operation_to_user_operation_variant(
    json_user_op: JsonUserOperation,
) -> Result<UserOperationVariant, String> {
    json_user_op.try_into()
}
