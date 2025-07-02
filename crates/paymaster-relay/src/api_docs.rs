use ethers::types::{Address, Bytes, U256};
use utoipa::{OpenApi, ToSchema};

// --- Proxy types for external crates to implement ToSchema ---

#[derive(ToSchema)]
pub struct UserOperation {
    #[schema(example = "0x...")]
    pub sender: Address,
    #[schema(example = "0x0")]
    pub nonce: U256,
    #[schema(example = "0x")]
    pub init_code: Bytes,
    #[schema(example = "0x")]
    pub call_data: Bytes,
    #[schema(example = "0x5F5E100")]
    pub call_gas_limit: U256,
    #[schema(example = "0x5F5E100")]
    pub verification_gas_limit: U256,
    #[schema(example = "0x5F5E100")]
    pub pre_verification_gas: U256,
    #[schema(example = "0x1")]
    pub max_fee_per_gas: U256,
    #[schema(example = "0x1")]
    pub max_priority_fee_per_gas: U256,
    #[schema(example = "0x")]
    pub paymaster_and_data: Bytes,
    #[schema(example = "0x")]
    pub signature: Bytes,
}

#[derive(ToSchema)]
pub struct SponsorRequest {
    #[schema(value_type = UserOperation)]
    pub user_operation: UserOperation,
    #[schema(example = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789")]
    pub entry_point: Address,
}

#[derive(ToSchema)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::rpc::sponsor_user_operation_doc,
    ),
    components(
        schemas(SponsorRequest, UserOperation, ErrorResponse)
    ),
    tags(
        (name = "super-relay", description = "Super Relay Paymaster API")
    ),
    info(
        title = "Super-Relay API",
        version = "0.1.0",
        description = "API for the Super-Relay Paymaster Service"
    )
)]
pub struct ApiDoc; 