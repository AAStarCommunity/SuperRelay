// This file is part of Rundler.
//
// Rundler is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later version.
//
// Rundler is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Rundler.
// If not, see https://www.gnu.org/licenses/.

//! ERC-4337 v0.8 UserOperation support
//!
//! v0.8 uses the same PackedUserOperation format as v0.7, but with enhanced EIP-7702 support
//! and some optimizations. This implementation reuses v0.7 structures while providing v0.8-specific
//! entry point version and addresses.

use serde::{Deserialize, Serialize};

// Re-export v0.7 UserOperation as the base for v0.8
use super::v0_7;
use super::{UserOperation as UserOperationTrait, UserOperationId, UserOperationVariant};
use crate::EntryPointVersion;

/// User Operation for Entry Point v0.8
///
/// v0.8 uses the same format as v0.7 but with enhanced EIP-7702 support
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "test-utils", derive(Default))]
pub struct UserOperation(pub v0_7::UserOperation);

impl UserOperationTrait for UserOperation {
    type OptionalGas = v0_7::UserOperationOptionalGas;

    fn entry_point_version() -> EntryPointVersion {
        EntryPointVersion::V0_8
    }

    // Delegate all other methods to the wrapped v0.7 UserOperation
    fn entry_point(&self) -> alloy_primitives::Address {
        self.0.entry_point()
    }
    fn chain_id(&self) -> u64 {
        self.0.chain_id()
    }
    fn hash(&self) -> alloy_primitives::B256 {
        self.0.hash()
    }
    fn id(&self) -> UserOperationId {
        self.0.id()
    }
    fn sender(&self) -> alloy_primitives::Address {
        self.0.sender()
    }
    fn nonce(&self) -> alloy_primitives::U256 {
        self.0.nonce()
    }
    fn paymaster(&self) -> Option<alloy_primitives::Address> {
        self.0.paymaster()
    }
    fn factory(&self) -> Option<alloy_primitives::Address> {
        self.0.factory()
    }
    fn aggregator(&self) -> Option<alloy_primitives::Address> {
        self.0.aggregator()
    }
    fn call_data(&self) -> &alloy_primitives::Bytes {
        self.0.call_data()
    }
    fn max_gas_cost(&self) -> alloy_primitives::U256 {
        self.0.max_gas_cost()
    }
    fn entities(&self) -> Vec<crate::Entity> {
        self.0.entities()
    }
    fn heap_size(&self) -> usize {
        self.0.heap_size()
    }
    fn max_fee_per_gas(&self) -> u128 {
        self.0.max_fee_per_gas()
    }
    fn max_priority_fee_per_gas(&self) -> u128 {
        self.0.max_priority_fee_per_gas()
    }
    fn signature(&self) -> &alloy_primitives::Bytes {
        self.0.signature()
    }
    fn pre_verification_gas(&self) -> u128 {
        self.0.pre_verification_gas()
    }
    fn call_gas_limit(&self) -> u128 {
        self.0.call_gas_limit()
    }
    fn verification_gas_limit(&self) -> u128 {
        self.0.verification_gas_limit()
    }
    fn total_verification_gas_limit(&self) -> u128 {
        self.0.total_verification_gas_limit()
    }
    fn paymaster_post_op_gas_limit(&self) -> u128 {
        self.0.paymaster_post_op_gas_limit()
    }

    fn static_pre_verification_gas(&self, chain_spec: &crate::chain::ChainSpec) -> u128 {
        // v0.8 uses the same gas calculation as v0.7
        self.0.static_pre_verification_gas(chain_spec)
    }

    fn calldata_floor_gas_limit(&self) -> u128 {
        self.0.calldata_floor_gas_limit()
    }
    fn required_pre_execution_buffer(&self) -> u128 {
        self.0.required_pre_execution_buffer()
    }
    fn aggregator_gas_limit(
        &self,
        chain_spec: &crate::chain::ChainSpec,
        bundle_size: Option<usize>,
    ) -> u128 {
        self.0.aggregator_gas_limit(chain_spec, bundle_size)
    }

    fn transform_for_aggregator(
        self,
        chain_spec: &crate::chain::ChainSpec,
        aggregator: alloy_primitives::Address,
        aggregator_costs: crate::aggregator::AggregatorCosts,
        new_signature: alloy_primitives::Bytes,
    ) -> Self {
        UserOperation(self.0.transform_for_aggregator(
            chain_spec,
            aggregator,
            aggregator_costs,
            new_signature,
        ))
    }

    fn original_signature(&self) -> &alloy_primitives::Bytes {
        self.0.original_signature()
    }
    fn with_original_signature(self) -> Self {
        UserOperation(self.0.with_original_signature())
    }
    fn extra_data_len(&self, bundle_size: usize) -> usize {
        self.0.extra_data_len(bundle_size)
    }
    fn abi_encoded_size(&self) -> usize {
        self.0.abi_encoded_size()
    }
    fn authorization_tuple(&self) -> Option<&crate::authorization::Eip7702Auth> {
        self.0.authorization_tuple()
    }

    fn effective_verification_gas_limit_efficiency_reject_threshold(
        &self,
        verification_gas_limit_efficiency_reject_threshold: f64,
    ) -> f64 {
        self.0
            .effective_verification_gas_limit_efficiency_reject_threshold(
                verification_gas_limit_efficiency_reject_threshold,
            )
    }
}

impl UserOperation {
    /// Create a new v0.8 UserOperation from a v0.7 UserOperation
    pub fn from_v0_7(v0_7_op: v0_7::UserOperation) -> Self {
        UserOperation(v0_7_op)
    }

    /// Convert to v0.7 UserOperation
    pub fn into_v0_7(self) -> v0_7::UserOperation {
        self.0
    }

    /// Access the wrapped v0.7 operation
    pub fn inner(&self) -> &v0_7::UserOperation {
        &self.0
    }

    /// Access the wrapped v0.7 operation mutably
    pub fn inner_mut(&mut self) -> &mut v0_7::UserOperation {
        &mut self.0
    }

    /// Packs the user operation to its offchain representation
    pub fn pack(self) -> rundler_contracts::v0_7::PackedUserOperation {
        self.0.pack()
    }

    /// Returns a reference to the packed user operation
    pub fn packed(&self) -> &rundler_contracts::v0_7::PackedUserOperation {
        self.0.packed()
    }

    /// Get the paymaster data
    pub fn paymaster_data(&self) -> &alloy_primitives::Bytes {
        self.0.paymaster_data()
    }

    /// Get the factory data  
    pub fn factory_data(&self) -> &alloy_primitives::Bytes {
        self.0.factory_data()
    }
}

impl From<UserOperationVariant> for UserOperation {
    /// Converts a UserOperationVariant to a UserOperation 0.8
    ///
    /// # Panics
    ///
    /// Panics if the variant is not v0.8. This is for use in contexts
    /// where the variant is known to be v0.8.
    fn from(value: UserOperationVariant) -> Self {
        value.into_v0_8().expect("Expected UserOperationV0_8")
    }
}

impl From<UserOperation> for super::UserOperationVariant {
    fn from(op: UserOperation) -> Self {
        super::UserOperationVariant::V0_8(op)
    }
}

impl AsRef<UserOperation> for super::UserOperationVariant {
    /// # Panics
    ///
    /// Panics if the variant is not v0.8. This is for use in contexts
    /// where the variant is known to be v0.8.
    fn as_ref(&self) -> &UserOperation {
        match self {
            super::UserOperationVariant::V0_8(op) => op,
            _ => panic!("Expected UserOperationV0_8"),
        }
    }
}

impl AsMut<UserOperation> for super::UserOperationVariant {
    /// # Panics
    ///
    /// Panics if the variant is not v0.8. This is for use in contexts
    /// where the variant is known to be v0.8.
    fn as_mut(&mut self) -> &mut UserOperation {
        match self {
            super::UserOperationVariant::V0_8(op) => op,
            _ => panic!("Expected UserOperationV0_8"),
        }
    }
}

/// Re-export v0.7 types for convenience, since v0.8 uses the same format
pub use v0_7::{
    UnstructuredUserOperation, UserOperationBuilder, UserOperationOptionalGas,
    UserOperationRequiredFields,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::ChainSpec;

    #[test]
    fn test_v0_8_entry_point_version() {
        assert_eq!(
            UserOperation::entry_point_version(),
            EntryPointVersion::V0_8
        );
    }

    #[test]
    fn test_v0_8_wrapping() {
        let v0_7_op = v0_7::UserOperation::default();
        let v0_8_op = UserOperation::from_v0_7(v0_7_op.clone());

        assert_eq!(v0_8_op.into_v0_7(), v0_7_op);
    }

    #[test]
    fn test_v0_8_static_pre_verification_gas() {
        let cs = ChainSpec::default();
        let v0_8_op = UserOperation::default();
        let v0_7_op = v0_7::UserOperation::default();

        // v0.8 delegates to v0.7 implementation, so should return same value
        let v0_8_gas = v0_8_op.static_pre_verification_gas(&cs);
        let v0_7_gas = v0_7_op.static_pre_verification_gas(&cs);
        assert_eq!(v0_8_gas, v0_7_gas);
    }
}
