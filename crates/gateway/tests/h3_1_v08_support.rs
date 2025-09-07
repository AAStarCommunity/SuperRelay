//! H3.1: PackedUserOperation v0.7/v0.8 Support Tests
//!
//! This test suite validates the extended support for both v0.7 and v0.8
//! PackedUserOperation formats, ensuring proper handling of entry point versions.

use rundler_types::{
    chain::ChainSpec,
    user_operation::{v0_7, v0_8, UserOperationVariant},
    EntryPointVersion,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v08_entry_point_version() {
        let v0_8_op = v0_8::UserOperation::default();
        assert_eq!(v0_8_op.entry_point_version(), EntryPointVersion::V0_8);
    }

    #[test]
    fn test_v07_to_v08_wrapper() {
        let v0_7_op = v0_7::UserOperation::default();
        let v0_8_op = v0_8::UserOperation::from_v0_7(v0_7_op.clone());

        // v0.8 wraps v0.7, should maintain same core data
        assert_eq!(v0_8_op.into_v0_7(), v0_7_op);
        assert_eq!(v0_8_op.sender(), v0_7_op.sender());
        assert_eq!(v0_8_op.nonce(), v0_7_op.nonce());
    }

    #[test]
    fn test_user_operation_variant_v08() {
        let v0_8_op = v0_8::UserOperation::default();
        let variant = UserOperationVariant::V0_8(v0_8_op);

        // Test type checking methods
        assert!(variant.is_v0_8());
        assert!(!variant.is_v0_7());
        assert!(!variant.is_v0_6());
        assert_eq!(variant.uo_type(), EntryPointVersion::V0_8);
    }

    #[test]
    fn test_v08_variant_conversion() {
        let v0_8_op = v0_8::UserOperation::default();
        let variant = UserOperationVariant::V0_8(v0_8_op.clone());

        // Test conversion back to v0.8
        let converted = variant.into_v0_8().expect("Should convert to v0.8");
        assert_eq!(converted.entry_point_version(), EntryPointVersion::V0_8);
    }

    #[test]
    fn test_v08_trait_implementation() {
        let chain_spec = ChainSpec::default();
        let v0_8_op = v0_8::UserOperation::default();

        // Test that all UserOperation trait methods work
        let _entry_point = v0_8_op.entry_point();
        let _chain_id = v0_8_op.chain_id();
        let _hash = v0_8_op.hash();
        let _sender = v0_8_op.sender();
        let _nonce = v0_8_op.nonce();
        let _signature = v0_8_op.signature();
        let _gas_limit = v0_8_op.call_gas_limit();
        let _verification_gas = v0_8_op.verification_gas_limit();
        let _max_fee = v0_8_op.max_fee_per_gas();
        let _max_priority_fee = v0_8_op.max_priority_fee_per_gas();
        let _pvg = v0_8_op.static_pre_verification_gas(&chain_spec);
    }

    #[test]
    fn test_v08_gas_calculation_delegation() {
        let chain_spec = ChainSpec::default();
        let v0_7_op = v0_7::UserOperation::default();
        let v0_8_op = v0_8::UserOperation::from_v0_7(v0_7_op.clone());

        // v0.8 should delegate gas calculation to wrapped v0.7
        assert_eq!(
            v0_8_op.static_pre_verification_gas(&chain_spec),
            v0_7_op.static_pre_verification_gas(&chain_spec)
        );

        assert_eq!(v0_8_op.call_gas_limit(), v0_7_op.call_gas_limit());
        assert_eq!(
            v0_8_op.verification_gas_limit(),
            v0_7_op.verification_gas_limit()
        );
    }

    #[test]
    fn test_v08_packed_user_operation_compatibility() {
        let v0_8_op = v0_8::UserOperation::default();

        // v0.8 should use same PackedUserOperation format as v0.7
        let _packed = v0_8_op.pack();
        let _packed_ref = v0_8_op.packed();

        // Should be able to access v0.7 specific methods
        let _paymaster_data = v0_8_op.paymaster_data();
        let _factory_data = v0_8_op.factory_data();
    }

    #[test]
    fn test_v08_variant_match_coverage() {
        let v0_8_op = v0_8::UserOperation::default();
        let variant = UserOperationVariant::V0_8(v0_8_op);

        // Test that all variant match arms work with v0.8
        let _entry_point = variant.entry_point();
        let _sender = variant.sender();
        let _nonce = variant.nonce();
        let _signature = variant.signature();
        let _entities = variant.entities();
        let _max_gas_cost = variant.max_gas_cost();
    }

    #[test]
    fn test_v08_eip7702_support() {
        let v0_8_op = v0_8::UserOperation::default();

        // v0.8 should support EIP-7702 authorization tuples
        let _auth_tuple = v0_8_op.authorization_tuple();

        // Should delegate to v0.7 implementation
        assert_eq!(
            v0_8_op.authorization_tuple(),
            v0_8_op.inner().authorization_tuple()
        );
    }

    #[test]
    fn test_comprehensive_v08_feature_coverage() {
        // This test ensures all major v0.8 features work together
        let chain_spec = ChainSpec::default();
        let base_v0_7 = v0_7::UserOperation::default();
        let v0_8_op = v0_8::UserOperation::from_v0_7(base_v0_7);
        let variant = UserOperationVariant::V0_8(v0_8_op.clone());

        // Version identification
        assert_eq!(v0_8_op.entry_point_version(), EntryPointVersion::V0_8);
        assert!(variant.is_v0_8());
        assert_eq!(variant.uo_type(), EntryPointVersion::V0_8);

        // Core functionality
        assert!(v0_8_op.hash() != alloy_primitives::B256::ZERO);
        assert_eq!(v0_8_op.max_gas_cost(), variant.max_gas_cost());

        // Gas calculations
        let pvg = v0_8_op.static_pre_verification_gas(&chain_spec);
        assert!(pvg > 0);
        assert_eq!(pvg, variant.static_pre_verification_gas(&chain_spec));

        // Format compatibility
        let _packed = v0_8_op.pack();
        let _size = v0_8_op.abi_encoded_size();

        // Entry point version consistency
        assert_eq!(
            v0_8::UserOperation::entry_point_version(),
            EntryPointVersion::V0_8
        );
    }
}
