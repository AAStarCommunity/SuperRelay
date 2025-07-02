// paymaster-relay/src/service.rs
// This file will contain the core business logic of the PaymasterRelayService. 

use crate::{
    error::PaymasterError, policy::PolicyEngine, signer::SignerManager,
};
use alloy_primitives::{Bytes, B256};
use ethers::types::Address;
use rundler_pool::LocalPoolHandle;
use rundler_types::{
    pool::Pool,
    user_operation::{UserOperation, UserOperationVariant},
    UserOperationPermissions,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct PaymasterRelayService {
    signer_manager: SignerManager,
    policy_engine: PolicyEngine,
    pool: Arc<LocalPoolHandle>,
}

impl PaymasterRelayService {
    pub fn new(
        signer_manager: SignerManager,
        policy_engine: PolicyEngine,
        pool: Arc<LocalPoolHandle>,
    ) -> Self {
        Self {
            signer_manager,
            policy_engine,
            pool,
        }
    }

    pub async fn sponsor_user_operation(
        &self,
        user_op: UserOperationVariant,
        _entry_point: Address, // Note: entry_point is part of UserOperationVariant now
    ) -> Result<B256, PaymasterError> {
        // 1. Check policy
        self.policy_engine.check_policy(&user_op)?;

        // 2. Sign the hash
        let user_op_hash = user_op.hash();
        let signature = self
            .signer_manager
            .sign_hash(user_op_hash.into())
            .await?;

        // 3. Construct paymasterAndData and create the new sponsored UserOperation
        let paymaster_address = self.signer_manager.address();
        let sponsored_user_op = match user_op {
            UserOperationVariant::V0_6(mut op) => {
                let paymaster_and_data =
                    [paymaster_address.as_bytes(), &signature.to_vec()].concat();
                op.paymaster_and_data = Bytes::from(paymaster_and_data);
                UserOperationVariant::V0_6(op)
            }
            UserOperationVariant::V0_7(mut op) => {
                // For v0.7, paymaster, paymaster_verification_gas_limit, paymaster_post_op_gas_limit, and paymaster_data are separate fields.
                // Here we are creating a simple sponsoring paymaster.
                // A more advanced implementation would get gas limits from a simulation.
                // For now, we'll use some reasonable defaults, assuming the paymaster contract
                // will be able to handle it.
                op.paymaster = Some(alloy_primitives::Address::from(paymaster_address.as_fixed_bytes()));
                op.paymaster_data = Bytes::from(signature.to_vec());
                // These should be estimated, but for now we'll use values from the user_op.
                if op.paymaster_verification_gas_limit == 0 {
                    op.paymaster_verification_gas_limit = 100_000;
                }
                if op.paymaster_post_op_gas_limit == 0 {
                    op.paymaster_post_op_gas_limit = 20_000;
                }
                UserOperationVariant::V0_7(op)
            }
        };

        // 4. Add to mempool
        self.pool
            .add_op(sponsored_user_op, UserOperationPermissions::default())
            .await?;

        Ok(user_op_hash)
    }
} 