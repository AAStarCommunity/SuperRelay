// paymaster-relay/src/policy.rs
// This file will implement the PolicyEngine for sponsorship rules. 

use crate::error::PaymasterError;
use alloy_primitives::Address;
use rundler_types::user_operation::{UserOperation, UserOperationVariant};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

#[derive(Clone, Debug, Deserialize)]
pub struct Policy {
    pub senders: Vec<Address>,
    // We can add more policy rules here later, e.g.,
    // target_contracts: Vec<Address>,
    // max_gas_limit: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PolicyConfig {
    #[serde(flatten)]
    pub policies: HashMap<String, Policy>,
}

#[derive(Clone)]
pub struct PolicyEngine {
    config: PolicyConfig,
}

impl PolicyEngine {
    pub fn new(config_path: &Path) -> Result<Self, PaymasterError> {
        let config_str = std::fs::read_to_string(config_path)
            .map_err(|e| PaymasterError::PolicyRejected(format!("Failed to read policy file: {}", e)))?;
        let config: PolicyConfig = toml::from_str(&config_str)
            .map_err(|e| PaymasterError::PolicyRejected(format!("Failed to parse policy file: {}", e)))?;
        Ok(Self { config })
    }

    pub fn check_policy(&self, user_op: &UserOperationVariant) -> Result<(), PaymasterError> {
        // For now, we use a single, hardcoded "default" policy.
        // This can be extended to select a policy based on the RPC input.
        if let Some(policy) = self.config.policies.get("default") {
            if !policy.senders.contains(&user_op.sender()) {
                return Err(PaymasterError::PolicyRejected(format!(
                    "Sender {} is not in the allowlist.",
                    user_op.sender()
                )));
            }
        } else {
            return Err(PaymasterError::PolicyRejected(
                "Default policy not found.".to_string(),
            ));
        }

        // Add more policy checks here as needed.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Write, str::FromStr};
    use tempfile::tempdir;
    use rundler_types::user_operation::v0_6::UserOperation as UserOperationV6;

    fn create_test_user_op(sender: Address) -> UserOperationVariant {
        let mut op = UserOperationV6::default();
        op.sender = sender;
        UserOperationVariant::V0_6(op)
    }

    #[test]
    fn test_policy_engine_loading() {
        let dir = tempdir().unwrap();

        // 1. Test with a valid policy file
        let file_path = dir.path().join("policy.toml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            r#"[default]
senders = ["0x0000000000000000000000000000000000000001"]"#
        )
        .unwrap();

        let engine = PolicyEngine::new(&file_path);
        assert!(engine.is_ok());

        // 2. Test with a non-existent file
        let bad_path = dir.path().join("non_existent.toml");
        let engine = PolicyEngine::new(&bad_path);
        assert!(engine.is_err());

        // 3. Test with a malformed file
        let malformed_path = dir.path().join("malformed.toml");
        let mut malformed_file = File::create(&malformed_path).unwrap();
        writeln!(malformed_file, r#"senders = 123"#).unwrap();
        let engine = PolicyEngine::new(&malformed_path);
        assert!(engine.is_err());
    }

    #[test]
    fn test_check_policy() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("policy.toml");
        let mut file = File::create(&file_path).unwrap();

        let allowed_sender = Address::from_str("0x0000000000000000000000000000000000000001").unwrap();
        let disallowed_sender = Address::from_str("0x0000000000000000000000000000000000000002").unwrap();

        writeln!(
            file,
            r#"[default]
senders = ["{}"]"#,
            allowed_sender
        )
        .unwrap();

        let engine = PolicyEngine::new(&file_path).unwrap();

        // 1. Test with allowed sender
        let user_op_allowed = create_test_user_op(allowed_sender);
        assert!(engine.check_policy(&user_op_allowed).is_ok());

        // 2. Test with disallowed sender
        let user_op_disallowed = create_test_user_op(disallowed_sender);
        assert!(engine.check_policy(&user_op_disallowed).is_err());

        // 3. Test with no "default" policy in file
        let no_default_path = dir.path().join("no_default.toml");
        let mut no_default_file = File::create(&no_default_path).unwrap();
        writeln!(no_default_file, r#"[other_policy]
senders = ["{}"]"#, allowed_sender).unwrap();
        let engine_no_default = PolicyEngine::new(&no_default_path).unwrap();
        assert!(engine_no_default.check_policy(&user_op_allowed).is_err());
    }
}