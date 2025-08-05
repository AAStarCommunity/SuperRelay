use alloy_primitives::{Address, Bytes, U256};
use rundler_types::{chain::ChainSpec, v0_6, v0_7, UserOperationVariant};
use super_relay_gateway::{SecurityChecker, SecurityConfig};

/// Create a test UserOperation v0.6
fn create_test_user_operation_v06() -> UserOperationVariant {
    let sender: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        .parse()
        .unwrap();
    let chain_spec = ChainSpec::default();

    let user_op = v0_6::UserOperationBuilder::new(
        &chain_spec,
        v0_6::UserOperationRequiredFields {
            sender,
            nonce: U256::from(1),
            init_code: Bytes::new(),
            call_data: Bytes::from(hex::decode("a9059cbb000000000000000000000000000000000000000000000000000000000000dead0000000000000000000000000000000000000000000000000de0b6b3a7640000").unwrap()),
            call_gas_limit: 100_000,
            verification_gas_limit: 100_000,
            pre_verification_gas: 21_000,
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
            paymaster_and_data: Bytes::new(),
            signature: Bytes::new(),
        },
    ).build();

    UserOperationVariant::V0_6(user_op)
}

/// Create a test UserOperation v0.7
fn create_test_user_operation_v07() -> UserOperationVariant {
    let sender: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        .parse()
        .unwrap();
    let chain_spec = ChainSpec::default();

    let user_op = v0_7::UserOperationBuilder::new(
        &chain_spec,
        v0_7::UserOperationRequiredFields {
            sender,
            nonce: U256::from(1),
            call_data: Bytes::from(hex::decode("a9059cbb000000000000000000000000000000000000000000000000000000000000dead0000000000000000000000000000000000000000000000000de0b6b3a7640000").unwrap()),
            call_gas_limit: 100_000,
            verification_gas_limit: 100_000,
            pre_verification_gas: 21_000,
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
            signature: Bytes::new(),
        },
    ).build();

    UserOperationVariant::V0_7(user_op)
}

/// Create a malicious UserOperation with known bad address
fn create_malicious_user_operation() -> UserOperationVariant {
    let malicious_sender: Address = "0x0000000000000000000000000000000000000bad"
        .parse()
        .unwrap();
    let chain_spec = ChainSpec::default();

    let user_op = v0_6::UserOperationBuilder::new(
        &chain_spec,
        v0_6::UserOperationRequiredFields {
            sender: malicious_sender,
            nonce: U256::from(1),
            init_code: Bytes::new(),
            call_data: Bytes::new(),
            call_gas_limit: 100_000,
            verification_gas_limit: 100_000,
            pre_verification_gas: 21_000,
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
            paymaster_and_data: Bytes::new(),
            signature: Bytes::new(),
        },
    )
    .build();

    UserOperationVariant::V0_6(user_op)
}

/// Create a UserOperation with excessive gas limits
fn create_high_gas_user_operation() -> UserOperationVariant {
    let sender: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        .parse()
        .unwrap();
    let chain_spec = ChainSpec::default();

    let user_op = v0_6::UserOperationBuilder::new(
        &chain_spec,
        v0_6::UserOperationRequiredFields {
            sender,
            nonce: U256::from(1),
            init_code: Bytes::new(),
            call_data: Bytes::new(),
            call_gas_limit: 50_000_000,         // Excessive gas limit
            verification_gas_limit: 20_000_000, // Excessive gas limit
            pre_verification_gas: 21_000,
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
            paymaster_and_data: Bytes::new(),
            signature: Bytes::new(),
        },
    )
    .build();

    UserOperationVariant::V0_6(user_op)
}

#[tokio::test]
async fn test_security_check_normal_operation_v06() {
    let mut checker = SecurityChecker::new();
    checker.load_threat_intelligence().await.unwrap();

    let user_op = create_test_user_operation_v06();
    let entry_point: Address = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
        .parse()
        .unwrap();

    let result = checker
        .check_security(&user_op, &entry_point, None)
        .await
        .unwrap();

    assert!(
        result.is_secure,
        "Normal operation should pass security checks"
    );
    assert!(
        result.security_score > 50,
        "Security score should be reasonable"
    );
    assert!(
        result.critical_violations.is_empty(),
        "Should have no critical violations"
    );
    assert!(result.check_results.contains_key("malicious_address"));
    assert!(result.check_results.contains_key("gas_limits"));
    assert!(result.check_results.contains_key("calldata_security"));
}

#[tokio::test]
async fn test_security_check_normal_operation_v07() {
    let mut checker = SecurityChecker::new();
    checker.load_threat_intelligence().await.unwrap();

    let user_op = create_test_user_operation_v07();
    let entry_point: Address = "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
        .parse()
        .unwrap();

    let result = checker
        .check_security(&user_op, &entry_point, None)
        .await
        .unwrap();

    assert!(
        result.is_secure,
        "Normal v0.7 operation should pass security checks"
    );
    assert!(
        result.security_score > 50,
        "Security score should be reasonable"
    );
    assert!(
        result.critical_violations.is_empty(),
        "Should have no critical violations"
    );
}

#[tokio::test]
async fn test_security_check_malicious_address() {
    let mut checker = SecurityChecker::new();
    checker.load_threat_intelligence().await.unwrap();

    let user_op = create_malicious_user_operation();
    let entry_point: Address = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
        .parse()
        .unwrap();

    let result = checker
        .check_security(&user_op, &entry_point, None)
        .await
        .unwrap();

    assert!(
        !result.is_secure,
        "Malicious operation should fail security checks"
    );
    assert!(
        !result.critical_violations.is_empty(),
        "Should have critical violations"
    );
    assert!(
        result.security_score < 100,
        "Security score should be reduced"
    );

    // Check that malicious address check failed
    let malicious_check = result.check_results.get("malicious_address").unwrap();
    assert!(
        !malicious_check.passed,
        "Malicious address check should fail"
    );
}

#[tokio::test]
async fn test_security_check_high_gas_limits() {
    let mut checker = SecurityChecker::new();
    checker.load_threat_intelligence().await.unwrap();

    let user_op = create_high_gas_user_operation();
    let entry_point: Address = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
        .parse()
        .unwrap();

    let result = checker
        .check_security(&user_op, &entry_point, None)
        .await
        .unwrap();

    assert!(
        !result.is_secure,
        "High gas operation should fail security checks"
    );
    assert!(
        !result.critical_violations.is_empty(),
        "Should have critical violations for high gas"
    );

    // Check that gas limits check failed
    let gas_check = result.check_results.get("gas_limits").unwrap();
    assert!(!gas_check.passed, "Gas limits check should fail");
}

#[tokio::test]
async fn test_security_config_customization() {
    let custom_config = SecurityConfig {
        enable_contract_verification: false,
        enable_pattern_analysis: false,
        enable_phishing_detection: false,
        enable_anomaly_detection: false,
        enable_mev_protection: false,
        enable_calldata_analysis: true,
        max_call_gas_limit: 1_000_000,
        max_verification_gas_limit: 500_000,
        max_calldata_size: 1000,
        max_init_code_size: 10000,
        min_contract_reputation: 80,
    };

    let mut checker = SecurityChecker::with_config(custom_config);
    checker.load_threat_intelligence().await.unwrap();

    let user_op = create_test_user_operation_v06();
    let entry_point: Address = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
        .parse()
        .unwrap();

    let result = checker
        .check_security(&user_op, &entry_point, None)
        .await
        .unwrap();

    // With custom config, fewer checks should be enabled
    assert!(result.is_secure, "Should pass with limited checks");
    assert!(
        !result.check_results.contains_key("phishing_detection"),
        "Phishing detection should be disabled"
    );
    assert!(
        !result.check_results.contains_key("mev_protection"),
        "MEV protection should be disabled"
    );
}

#[tokio::test]
async fn test_security_threat_intelligence_management() {
    let mut checker = SecurityChecker::new();

    // Test adding custom threat intelligence
    let test_malicious_addr: Address = "0x1234567890123456789012345678901234567890"
        .parse()
        .unwrap();
    checker.add_malicious_address(test_malicious_addr);
    checker.add_phishing_pattern("drain.*wallet".to_string());
    checker.set_contract_reputation(test_malicious_addr, 10);

    // Load default threat intelligence
    checker.load_threat_intelligence().await.unwrap();

    // Create a test UserOperation from the malicious address
    let sender = test_malicious_addr;
    let chain_spec = ChainSpec::default();

    let malicious_user_op = v0_6::UserOperationBuilder::new(
        &chain_spec,
        v0_6::UserOperationRequiredFields {
            sender,
            nonce: U256::from(1),
            init_code: Bytes::new(),
            call_data: Bytes::new(),
            call_gas_limit: 100_000,
            verification_gas_limit: 100_000,
            pre_verification_gas: 21_000,
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
            paymaster_and_data: Bytes::new(),
            signature: Bytes::new(),
        },
    )
    .build();

    let user_op = UserOperationVariant::V0_6(malicious_user_op);
    let entry_point: Address = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
        .parse()
        .unwrap();

    // Test that the malicious address is detected
    let result = checker
        .check_security(&user_op, &entry_point, None)
        .await
        .unwrap();
    assert!(!result.is_secure, "Should detect custom malicious address");
}

#[tokio::test]
async fn test_security_check_performance() {
    let mut checker = SecurityChecker::new();
    checker.load_threat_intelligence().await.unwrap();

    let user_op = create_test_user_operation_v06();
    let entry_point: Address = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
        .parse()
        .unwrap();

    let start = std::time::Instant::now();

    // Run security check multiple times to test performance
    for _ in 0..100 {
        let _result = checker
            .check_security(&user_op, &entry_point, None)
            .await
            .unwrap();
    }

    let duration = start.elapsed();

    // Security checks should complete quickly (less than 1 second for 100 operations)
    assert!(duration.as_secs() < 1, "Security checks should be fast");
}

#[tokio::test]
async fn test_security_metadata_generation() {
    let mut checker = SecurityChecker::new();
    checker.load_threat_intelligence().await.unwrap();

    let user_op = create_test_user_operation_v06();
    let entry_point: Address = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
        .parse()
        .unwrap();

    let result = checker
        .check_security(&user_op, &entry_point, Some("127.0.0.1"))
        .await
        .unwrap();

    // Check that metadata is populated
    assert!(result.metadata.timestamp > 0, "Timestamp should be set");
    assert!(
        result.metadata.phishing_risk_level.is_some(),
        "Phishing risk level should be set"
    );
    assert!(
        result.metadata.contract_risk_score.is_some(),
        "Contract risk score should be set"
    );
    assert!(
        result.metadata.pattern_analysis.is_some(),
        "Pattern analysis should be set"
    );
    // Check that basic metadata is present
    assert!(!result.summary.is_empty(), "Summary should be present");
}
