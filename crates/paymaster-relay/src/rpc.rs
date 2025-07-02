// paymaster-relay/src/rpc.rs
// This file will contain the JSON-RPC API definition and implementation. 

use crate::service::PaymasterRelayService;
use async_trait::async_trait;
use ethers::types::Address;
use jsonrpsee::proc_macros::rpc;
use rundler_types::user_operation::UserOperationVariant;
use alloy_primitives::B256;

#[rpc(server, client, namespace = "pm")]
pub trait PaymasterRelayApi {
    #[method(name = "sponsorUserOperation")]
    async fn sponsor_user_operation(
        &self,
        user_op: UserOperationVariant,
        entry_point: Address,
    ) -> Result<B256, jsonrpsee::types::ErrorObjectOwned>;
}

/// A dummy function for utoipa to generate OpenAPI documentation.
#[utoipa::path(
    post,
    path = "/pm/sponsorUserOperation",
    request_body = crate::api_docs::SponsorRequest,
    responses(
        (status = 200, description = "Successfully sponsored UserOperation", body = String),
        (status = 500, description = "Internal Server Error", body = crate::api_docs::ErrorResponse)
    )
)]
pub async fn sponsor_user_operation_doc() {}

pub struct PaymasterRelayApiServerImpl {
    pub service: PaymasterRelayService,
}

#[async_trait]
impl PaymasterRelayApiServer for PaymasterRelayApiServerImpl {
    async fn sponsor_user_operation(
        &self,
        user_op: UserOperationVariant,
        entry_point: Address,
    ) -> Result<B256, jsonrpsee::types::ErrorObjectOwned> {
        self.service
            .sponsor_user_operation(user_op, entry_point)
            .await
            .map_err(|e| e.into())
    }
} 