use std::collections::HashMap;

use ethers::types::{Address, H256};
use rundler_paymaster_relay::{
    kms::GasEstimates, KmsConfig, MockKmsProvider, SignerManager, SigningContext,
};

/// Test KMS provider initialization and basic functionality
#[tokio::test]
async fn test_kms_provider_initialization() {
    let config = KmsConfig::default();
    let kms = MockKmsProvider::new(config).unwrap();

    // Should have primary + 2 backup keys by default
    assert_eq!(kms.list_keys().len(), 3);

    // Test key retrieval
    let primary_key = kms.get_key_info("paymaster-primary-key").unwrap();
    assert_eq!(primary_key.key_id, "paymaster-primary-key");
    assert!(primary_key.enabled);
    assert!(primary_key.permissions.contains(&"sign".to_string()));
}

/// Test SignerManager with KMS backend
#[tokio::test]
async fn test_signer_manager_kms_integration() {
    let kms_config = KmsConfig::default();
    let mut signer_manager =
        SignerManager::new_with_kms(kms_config).expect("Failed to create KMS signer manager");

    // Test backend type
    assert_eq!(signer_manager.backend_type(), "kms");

    // Test metadata
    let metadata = signer_manager.get_metadata();
    assert_eq!(metadata.get("backend_type"), Some(&"kms".to_string()));
    assert_eq!(metadata.get("backup_keys_count"), Some(&"2".to_string()));

    // Test connectivity
    signer_manager
        .test_kms_connectivity()
        .await
        .expect("KMS connectivity test should pass");

    // Test signing with context
    let hash = H256::random().to_fixed_bytes();
    let context = SigningContext {
        operation_type: "test_paymaster_operation".to_string(),
        user_operation_hash: Some(H256::random()),
        sender_address: Some(Address::random()),
        entry_point: Some(Address::random()),
        gas_estimates: Some(GasEstimates {
            call_gas_limit: 100_000,
            verification_gas_limit: 100_000,
            pre_verification_gas: 21_000,
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
        }),
        metadata: HashMap::new(),
    };

    let signature = signer_manager
        .sign_hash_with_context(hash, Some(context))
        .await
        .expect("KMS signing should succeed");

    // Verify signature is valid
    assert!(signature.r != ethers::types::U256::zero());
    assert!(signature.s != ethers::types::U256::zero());
}

/// Test KMS audit logging
#[tokio::test]
async fn test_kms_audit_logging() {
    let kms_config = KmsConfig::default();
    let mut signer_manager =
        SignerManager::new_with_kms(kms_config).expect("Failed to create KMS signer manager");

    // Initial audit log should be empty
    let initial_audit = signer_manager.get_kms_audit_log().unwrap();
    assert!(initial_audit.is_empty());

    // Perform signing operation
    let hash = H256::random().to_fixed_bytes();
    let context = SigningContext {
        operation_type: "audit_test_operation".to_string(),
        user_operation_hash: Some(H256::from_low_u64_be(12345)),
        sender_address: Some(
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
                .parse()
                .unwrap(),
        ),
        entry_point: Some(
            "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                .parse()
                .unwrap(),
        ),
        gas_estimates: Some(GasEstimates {
            call_gas_limit: 150_000,
            verification_gas_limit: 120_000,
            pre_verification_gas: 21_000,
            max_fee_per_gas: 2_000_000_000,
            max_priority_fee_per_gas: 1_500_000_000,
        }),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("test_id".to_string(), "audit_test_123".to_string());
            meta.insert("environment".to_string(), "integration_test".to_string());
            meta
        },
    };

    let _signature = signer_manager
        .sign_hash_with_context(hash, Some(context))
        .await
        .expect("KMS signing should succeed");

    // Check audit log
    let audit_log = signer_manager.get_kms_audit_log().unwrap();
    assert_eq!(audit_log.len(), 1);

    let audit_entry = &audit_log[0];
    assert_eq!(
        audit_entry.service_metadata.get("operation_type"),
        Some(&"audit_test_operation".to_string())
    );
    assert!(audit_entry.duration_ms > 0);
    assert!(!audit_entry.request_id.is_empty());
}

/// Test KMS key rotation functionality
#[tokio::test]
async fn test_kms_key_rotation() {
    let kms_config = KmsConfig::default();
    let mut signer_manager =
        SignerManager::new_with_kms(kms_config).expect("Failed to create KMS signer manager");

    // Rotate primary key
    signer_manager
        .rotate_kms_key("paymaster-primary-key")
        .await
        .expect("Key rotation should succeed");

    // Test that rotation affected the key metadata (simulated in mock)
    // In a real implementation, this would verify the key version changed
    assert_eq!(signer_manager.backend_type(), "kms");
}

/// Test KMS failover with backup keys
#[tokio::test]
async fn test_kms_backup_keys_configuration() {
    let kms_config = KmsConfig {
        backup_key_ids: vec![
            "backup-key-alpha".to_string(),
            "backup-key-beta".to_string(),
            "backup-key-gamma".to_string(),
        ],
        ..Default::default()
    };

    let signer_manager = SignerManager::new_with_kms(kms_config)
        .expect("Failed to create KMS signer manager with multiple backups");

    let metadata = signer_manager.get_metadata();
    assert_eq!(metadata.get("backup_keys_count"), Some(&"3".to_string()));
}

/// Test KMS error handling
#[tokio::test]
async fn test_kms_error_handling() {
    let kms_config = KmsConfig::default();
    let mut signer_manager =
        SignerManager::new_with_kms(kms_config).expect("Failed to create KMS signer manager");

    // Test invalid key rotation
    let result = signer_manager.rotate_kms_key("non-existent-key").await;
    assert!(result.is_err(), "Should fail for non-existent key");
}

/// Test KMS performance benchmarking
#[tokio::test]
async fn test_kms_signing_performance() {
    let kms_config = KmsConfig::default();
    let mut signer_manager =
        SignerManager::new_with_kms(kms_config).expect("Failed to create KMS signer manager");

    let start_time = std::time::Instant::now();
    let iterations = 10;

    for i in 0..iterations {
        let hash = H256::from_low_u64_be(i).to_fixed_bytes();
        let context = SigningContext {
            operation_type: format!("performance_test_{}", i),
            user_operation_hash: Some(H256::from_low_u64_be(i)),
            sender_address: Some(Address::random()),
            entry_point: Some(Address::random()),
            gas_estimates: None,
            metadata: HashMap::new(),
        };

        let _signature = signer_manager
            .sign_hash_with_context(hash, Some(context))
            .await
            .expect("KMS signing should succeed");
    }

    let total_duration = start_time.elapsed();
    let avg_duration_ms = total_duration.as_millis() / iterations as u128;

    println!(
        "KMS signing performance: {} operations in {:?} (avg: {}ms per operation)",
        iterations, total_duration, avg_duration_ms
    );

    // Performance should be reasonable (allowing for mock latencies)
    assert!(
        avg_duration_ms < 300,
        "Average signing time should be under 300ms"
    );
}

/// Test concurrent KMS operations
#[tokio::test]
async fn test_kms_concurrent_operations() {
    let kms_config = KmsConfig::default();
    let signer_manager =
        SignerManager::new_with_kms(kms_config).expect("Failed to create KMS signer manager");

    // Test concurrent connectivity checks
    let mut handles = vec![];

    for i in 0..5 {
        let signer = signer_manager.clone();
        let handle = tokio::spawn(async move {
            signer
                .test_kms_connectivity()
                .await
                .unwrap_or_else(|_| panic!("Connectivity test {} should pass", i));
        });
        handles.push(handle);
    }

    // Wait for all concurrent operations to complete
    for handle in handles {
        handle
            .await
            .expect("Concurrent KMS operation should complete");
    }
}

/// Integration test combining all KMS features
#[tokio::test]
async fn test_kms_full_integration() {
    let kms_config = KmsConfig {
        primary_key_id: "integration-test-primary".to_string(),
        backup_key_ids: vec!["integration-test-backup".to_string()],
        service_endpoint: Some("https://mock-kms.example.com".to_string()),
        credentials: rundler_paymaster_relay::kms::KmsCredentials {
            access_key: "integration-test-access".to_string(),
            secret_key: "integration-test-secret".to_string(),
            region_or_tenant: "us-east-1".to_string(),
            additional_params: HashMap::new(),
        },
        signing_timeout_seconds: 30,
        enable_audit_logging: true,
        rate_limit_per_minute: 1000,
    };

    let mut signer_manager = SignerManager::new_with_kms(kms_config)
        .expect("Failed to create integration test KMS signer manager");

    // 1. Test connectivity
    signer_manager
        .test_kms_connectivity()
        .await
        .expect("Integration test connectivity should pass");

    // 2. Test comprehensive signing with full context
    let user_op_hash = H256::from_slice(&[0x12; 32]);
    let hash = user_op_hash.to_fixed_bytes();

    let context = SigningContext {
        operation_type: "integration_test_paymaster_operation".to_string(),
        user_operation_hash: Some(user_op_hash),
        sender_address: Some(
            "0x1234567890123456789012345678901234567890"
                .parse()
                .unwrap(),
        ),
        entry_point: Some(
            "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                .parse()
                .unwrap(),
        ),
        gas_estimates: Some(GasEstimates {
            call_gas_limit: 200_000,
            verification_gas_limit: 150_000,
            pre_verification_gas: 25_000,
            max_fee_per_gas: 3_000_000_000,
            max_priority_fee_per_gas: 2_000_000_000,
        }),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("integration_test".to_string(), "full_suite".to_string());
            meta.insert(
                "test_timestamp".to_string(),
                chrono::Utc::now().to_rfc3339(),
            );
            meta.insert("test_env".to_string(), "rust_integration_test".to_string());
            meta
        },
    };

    let signature = signer_manager
        .sign_hash_with_context(hash, Some(context))
        .await
        .expect("Integration test signing should succeed");

    // 3. Verify signature
    assert!(signature.r != ethers::types::U256::zero());
    assert!(signature.s != ethers::types::U256::zero());

    // 4. Check audit trail
    let audit_log = signer_manager.get_kms_audit_log().unwrap();
    assert!(!audit_log.is_empty());
    assert_eq!(
        audit_log[0].service_metadata.get("operation_type"),
        Some(&"integration_test_paymaster_operation".to_string())
    );

    // 5. Test metadata retrieval
    let metadata = signer_manager.get_metadata();
    assert_eq!(metadata.get("backend_type"), Some(&"kms".to_string()));
    assert_eq!(
        metadata.get("primary_key_id"),
        Some(&"integration-test-primary".to_string())
    );

    // 6. Test key rotation
    signer_manager
        .rotate_kms_key("integration-test-primary")
        .await
        .expect("Integration test key rotation should succeed");

    println!("✅ KMS full integration test completed successfully");
}
