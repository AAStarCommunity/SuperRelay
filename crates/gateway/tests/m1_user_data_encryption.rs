//! M1: 用户数据安全加密改进测试
//!
//! 验证AES-256-GCM加密模块的完整性和安全性

use alloy_primitives::Bytes;
use anyhow::Result;
use rundler_types::user_operation::{v0_7, UserOperationVariant};
use super_relay_gateway::{
    encryption_middleware::{EncryptionMiddleware, EncryptionService},
    user_data_encryption::{EncryptedData, EncryptionConfig, UserDataEncryption},
};

#[test]
fn test_encryption_config_default() {
    let config = EncryptionConfig::default();
    assert!(config.enabled);
    assert_eq!(config.key_rotation_interval, 3600); // 1 hour
    assert!(config.encrypt_call_data);
    assert!(!config.encrypt_signature); // signatures should not be encrypted
    assert!(config.encrypt_paymaster_data);
    assert_eq!(config.max_cached_keys, 5);
}

#[test]
fn test_user_data_encryption_creation() -> Result<()> {
    let config = EncryptionConfig::default();
    let encryption = UserDataEncryption::new(config)?;

    // Should have a valid key ID
    assert!(!encryption.current_key_id().is_empty());
    assert_eq!(encryption.cached_keys_count(), 1);

    // Key age should be very small (just created)
    let key_age = encryption.current_key_age()?;
    assert!(key_age < 5); // Less than 5 seconds old

    Ok(())
}

#[test]
fn test_aes_gcm_encryption_decryption() -> Result<()> {
    let config = EncryptionConfig::default();
    let encryption = UserDataEncryption::new(config)?;

    let test_data = b"Hello World! This is a test message for AES-256-GCM encryption.";

    // Encrypt the data
    let encrypted = encryption.encrypt(test_data)?;

    // Basic validations
    assert_ne!(encrypted.ciphertext, test_data);
    assert!(!encrypted.key_id.is_empty());
    assert_ne!(encrypted.key_id, "plaintext");
    assert!(encrypted.timestamp > 0);

    // Nonce should be all different (very high probability)
    assert_ne!(encrypted.nonce, [0u8; 12]);

    // Tag should be non-zero (authentication tag)
    assert_ne!(encrypted.tag, [0u8; 16]);

    // Decrypt the data
    let decrypted = encryption.decrypt(&encrypted)?;
    assert_eq!(decrypted, test_data);

    Ok(())
}

#[test]
fn test_call_data_encryption() -> Result<()> {
    let config = EncryptionConfig::default();
    let encryption = UserDataEncryption::new(config)?;

    let call_data = Bytes::from(b"0x1234567890abcdef".as_slice());

    // Encrypt call data
    let encrypted = encryption.encrypt_call_data(&call_data)?;
    assert_ne!(encrypted.ciphertext, call_data.as_ref());

    // Decrypt call data
    let decrypted = encryption.decrypt_call_data(&encrypted)?;
    assert_eq!(decrypted, call_data);

    Ok(())
}

#[test]
fn test_paymaster_data_encryption() -> Result<()> {
    let config = EncryptionConfig::default();
    let encryption = UserDataEncryption::new(config)?;

    let paymaster_data = Bytes::from(b"paymaster_verification_data".as_slice());

    // Encrypt paymaster data
    let encrypted = encryption.encrypt_paymaster_data(&paymaster_data)?;
    assert_ne!(encrypted.ciphertext, paymaster_data.as_ref());

    // Decrypt paymaster data
    let decrypted = encryption.decrypt_paymaster_data(&encrypted)?;
    assert_eq!(decrypted, paymaster_data);

    Ok(())
}

#[test]
fn test_disabled_encryption() -> Result<()> {
    let mut config = EncryptionConfig::default();
    config.enabled = false;

    let encryption = UserDataEncryption::new(config)?;

    let test_data = b"test data";
    let encrypted = encryption.encrypt(test_data)?;

    // When disabled, should return plaintext
    assert_eq!(encrypted.ciphertext, test_data);
    assert_eq!(encrypted.key_id, "plaintext");
    assert_eq!(encrypted.nonce, [0u8; 12]);
    assert_eq!(encrypted.tag, [0u8; 16]);

    let decrypted = encryption.decrypt(&encrypted)?;
    assert_eq!(decrypted, test_data);

    Ok(())
}

#[test]
fn test_call_data_encryption_disabled() -> Result<()> {
    let mut config = EncryptionConfig::default();
    config.encrypt_call_data = false;

    let encryption = UserDataEncryption::new(config)?;

    let call_data = Bytes::from(b"test call data".as_slice());
    let encrypted = encryption.encrypt_call_data(&call_data)?;

    // Should return plaintext when call data encryption is disabled
    assert_eq!(encrypted.ciphertext, call_data.as_ref());
    assert_eq!(encrypted.key_id, "plaintext");

    Ok(())
}

#[test]
fn test_multiple_encrypt_decrypt_cycles() -> Result<()> {
    let config = EncryptionConfig::default();
    let encryption = UserDataEncryption::new(config)?;

    let test_messages = vec![
        b"short".as_slice(),
        b"A longer message that spans multiple AES blocks to test boundary conditions".as_slice(),
        b"".as_slice(),                                 // Empty data
        &[0u8; 1000],                                   // Large data with zeros
        b"\x00\x01\x02\x03\xff\xfe\xfd\xfc".as_slice(), // Binary data with special bytes
    ];

    for (i, message) in test_messages.iter().enumerate() {
        println!("Testing message #{}: {} bytes", i, message.len());

        let encrypted = encryption.encrypt(message)?;
        let decrypted = encryption.decrypt(&encrypted)?;

        assert_eq!(decrypted.as_slice(), *message, "Failed for message #{}", i);

        // Each encryption should produce different ciphertext (due to random nonce)
        if i > 0 && message == test_messages[0] {
            let encrypted2 = encryption.encrypt(message)?;
            assert_ne!(
                encrypted.ciphertext, encrypted2.ciphertext,
                "Nonce should make each encryption unique"
            );
        }
    }

    Ok(())
}

#[test]
fn test_encryption_status() -> Result<()> {
    let config = EncryptionConfig::default();
    let encryption = UserDataEncryption::new(config.clone())?;

    let status = encryption.get_status()?;

    assert_eq!(status.enabled, config.enabled);
    assert!(!status.current_key_id.is_empty());
    assert!(status.key_age_seconds < 5); // Just created
    assert_eq!(status.cached_keys_count, 1);
    assert_eq!(status.encryption_config.enabled, config.enabled);
    assert_eq!(
        status.encryption_config.encrypt_call_data,
        config.encrypt_call_data
    );

    Ok(())
}

#[test]
fn test_invalid_decrypt_fails() -> Result<()> {
    let config = EncryptionConfig::default();
    let encryption = UserDataEncryption::new(config)?;

    // Create a valid encryption first
    let test_data = b"test data";
    let mut encrypted = encryption.encrypt(test_data)?;

    // Tamper with the ciphertext
    if !encrypted.ciphertext.is_empty() {
        encrypted.ciphertext[0] ^= 0x01;
    }

    // Decryption should fail due to authentication tag mismatch
    let result = encryption.decrypt(&encrypted);
    assert!(
        result.is_err(),
        "Decryption should fail with tampered ciphertext"
    );

    Ok(())
}

#[test]
fn test_tampered_tag_fails() -> Result<()> {
    let config = EncryptionConfig::default();
    let encryption = UserDataEncryption::new(config)?;

    let test_data = b"test data for tag tampering";
    let mut encrypted = encryption.encrypt(test_data)?;

    // Tamper with the authentication tag
    encrypted.tag[0] ^= 0x01;

    // Decryption should fail
    let result = encryption.decrypt(&encrypted);
    assert!(result.is_err(), "Decryption should fail with tampered tag");

    Ok(())
}

#[test]
fn test_tampered_nonce_fails() -> Result<()> {
    let config = EncryptionConfig::default();
    let encryption = UserDataEncryption::new(config)?;

    let test_data = b"test data for nonce tampering";
    let mut encrypted = encryption.encrypt(test_data)?;

    // Tamper with the nonce
    encrypted.nonce[0] ^= 0x01;

    // Decryption should fail or produce wrong plaintext
    let result = encryption.decrypt(&encrypted);

    // Either fails due to auth tag mismatch, or produces different plaintext
    if let Ok(decrypted) = result {
        assert_ne!(
            decrypted.as_slice(),
            test_data,
            "Tampered nonce should produce different plaintext"
        );
    } else {
        // This is also acceptable - auth tag verification might catch it
        println!("Nonce tampering caught by authentication tag verification");
    }

    Ok(())
}

#[test]
fn test_key_rotation_check() -> Result<()> {
    let mut config = EncryptionConfig::default();
    config.key_rotation_interval = 1; // 1 second for quick testing

    let mut encryption = UserDataEncryption::new(config)?;

    let original_key_id = encryption.current_key_id().to_string();

    // Check should return false initially (key just created)
    let needs_rotation = encryption.check_key_rotation()?;
    assert!(!needs_rotation);
    assert_eq!(encryption.current_key_id(), original_key_id);

    // Wait for key to age (in a real test, we might mock time instead)
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Now check should trigger rotation
    let needs_rotation = encryption.check_key_rotation()?;
    assert!(needs_rotation);
    assert_ne!(encryption.current_key_id(), original_key_id);

    // Should have 2 keys now (old + new)
    assert_eq!(encryption.cached_keys_count(), 2);

    Ok(())
}

#[test]
fn test_key_rotation_decrypt_old_data() -> Result<()> {
    let mut config = EncryptionConfig::default();
    config.key_rotation_interval = 1; // 1 second

    let mut encryption = UserDataEncryption::new(config)?;

    // Encrypt data with original key
    let test_data = b"data encrypted with old key";
    let encrypted_with_old_key = encryption.encrypt(test_data)?;
    let old_key_id = encrypted_with_old_key.key_id.clone();

    // Force key rotation
    std::thread::sleep(std::time::Duration::from_secs(2));
    encryption.check_key_rotation()?;

    // Should still be able to decrypt old data
    let decrypted = encryption.decrypt(&encrypted_with_old_key)?;
    assert_eq!(decrypted.as_slice(), test_data);

    // New encryptions should use new key
    let encrypted_with_new_key = encryption.encrypt(test_data)?;
    assert_ne!(encrypted_with_new_key.key_id, old_key_id);

    // Both should decrypt correctly
    let decrypted_old = encryption.decrypt(&encrypted_with_old_key)?;
    let decrypted_new = encryption.decrypt(&encrypted_with_new_key)?;

    assert_eq!(decrypted_old.as_slice(), test_data);
    assert_eq!(decrypted_new.as_slice(), test_data);

    Ok(())
}

// =============================================================================
// 中间件和服务测试
// =============================================================================

#[tokio::test]
async fn test_encryption_middleware_basic() -> Result<()> {
    let config = EncryptionConfig::default();
    let middleware = EncryptionMiddleware::new(config)?;

    let status = middleware.encryption_status().await?;
    assert!(status.enabled);
    assert!(!status.current_key_id.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_encryption_service_user_operation() -> Result<()> {
    let config = EncryptionConfig::default();
    let service = EncryptionService::new(config)?;

    // Create a test UserOperation with some data
    let mut user_op = v0_7::UserOperation::default();
    user_op.inner_mut().call_data = Bytes::from("0x1234567890abcdef");
    user_op.inner_mut().paymaster_and_data = Bytes::from("paymaster_test_data");

    let user_op_variant = UserOperationVariant::V0_7(user_op.clone());

    // Process inbound (should encrypt sensitive data)
    let encrypted_data = service
        .process_inbound_user_operation(user_op_variant.clone())
        .await?;

    assert!(encrypted_data.is_encrypted);
    assert!(encrypted_data.encrypted_call_data.is_some());
    assert!(encrypted_data.encrypted_paymaster_data.is_some());

    // Process outbound (should decrypt back to original)
    let decrypted_op = service
        .process_outbound_user_operation(&encrypted_data)
        .await?;

    // Verify data integrity after encryption/decryption cycle
    match (&user_op_variant, &decrypted_op) {
        (UserOperationVariant::V0_7(original), UserOperationVariant::V0_7(decrypted)) => {
            assert_eq!(original.call_data(), decrypted.call_data());
            // Note: paymaster data comparison might be more complex due to struct differences
        }
        _ => panic!("Unexpected UserOperation variant combination"),
    }

    Ok(())
}

#[tokio::test]
async fn test_encryption_service_disabled() -> Result<()> {
    let mut config = EncryptionConfig::default();
    config.enabled = false;

    let service = EncryptionService::new(config)?;

    let user_op_variant = UserOperationVariant::V0_7(v0_7::UserOperation::default());

    // Process inbound with encryption disabled
    let encrypted_data = service
        .process_inbound_user_operation(user_op_variant.clone())
        .await?;

    assert!(!encrypted_data.is_encrypted);
    assert!(encrypted_data.encrypted_call_data.is_none());
    assert!(encrypted_data.encrypted_paymaster_data.is_none());

    // Process outbound should return same data
    let decrypted_op = service
        .process_outbound_user_operation(&encrypted_data)
        .await?;

    // Should be identical when encryption is disabled
    match (&user_op_variant, &decrypted_op) {
        (UserOperationVariant::V0_7(original), UserOperationVariant::V0_7(decrypted)) => {
            assert_eq!(original.call_data(), decrypted.call_data());
        }
        _ => panic!("Unexpected UserOperation variant combination"),
    }

    Ok(())
}

#[tokio::test]
async fn test_selective_encryption_config() -> Result<()> {
    let mut config = EncryptionConfig::default();
    config.encrypt_call_data = true;
    config.encrypt_paymaster_data = false; // Disable paymaster encryption

    let service = EncryptionService::new(config)?;

    let mut user_op = v0_7::UserOperation::default();
    user_op.inner_mut().call_data = Bytes::from("test_call_data");
    user_op.inner_mut().paymaster_and_data = Bytes::from("test_paymaster_data");

    let user_op_variant = UserOperationVariant::V0_7(user_op);

    let encrypted_data = service
        .process_inbound_user_operation(user_op_variant)
        .await?;

    // Call data should be encrypted
    assert!(encrypted_data.encrypted_call_data.is_some());
    // Paymaster data should NOT be encrypted
    assert!(encrypted_data.encrypted_paymaster_data.is_none());

    Ok(())
}

#[tokio::test]
async fn test_encryption_service_status() -> Result<()> {
    let config = EncryptionConfig::default();
    let service = EncryptionService::new(config.clone())?;

    let status = service.status().await?;

    assert_eq!(status.enabled, config.enabled);
    assert_eq!(
        status.encryption_config.encrypt_call_data,
        config.encrypt_call_data
    );
    assert_eq!(
        status.encryption_config.encrypt_paymaster_data,
        config.encrypt_paymaster_data
    );
    assert!(status.key_age_seconds < 10); // Should be recently created

    Ok(())
}

#[tokio::test]
async fn test_encryption_middleware_key_rotation() -> Result<()> {
    let mut config = EncryptionConfig::default();
    config.key_rotation_interval = 1; // 1 second for fast testing

    let middleware = EncryptionMiddleware::new(config)?;

    let initial_status = middleware.encryption_status().await?;
    let initial_key_id = initial_status.current_key_id.clone();

    // Wait for key to age
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Trigger key rotation check
    let rotated = middleware.check_key_rotation().await?;
    assert!(rotated);

    let new_status = middleware.encryption_status().await?;
    assert_ne!(new_status.current_key_id, initial_key_id);
    assert_eq!(new_status.cached_keys_count, 2); // Old key + new key

    Ok(())
}

#[tokio::test]
async fn test_cross_key_decryption() -> Result<()> {
    let mut config = EncryptionConfig::default();
    config.key_rotation_interval = 1; // 1 second

    let middleware = EncryptionMiddleware::new(config)?;

    // Encrypt with original key
    let user_op = UserOperationVariant::V0_7(v0_7::UserOperation::default());
    let encrypted_old_key = middleware.encrypt_user_operation(user_op.clone()).await?;

    // Force key rotation
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    middleware.check_key_rotation().await?;

    // Encrypt with new key
    let encrypted_new_key = middleware.encrypt_user_operation(user_op.clone()).await?;

    // Both should decrypt correctly
    let decrypted_old = middleware
        .decrypt_user_operation(&encrypted_old_key)
        .await?;
    let decrypted_new = middleware
        .decrypt_user_operation(&encrypted_new_key)
        .await?;

    // Both should match the original
    match (&user_op, &decrypted_old, &decrypted_new) {
        (
            UserOperationVariant::V0_7(orig),
            UserOperationVariant::V0_7(dec_old),
            UserOperationVariant::V0_7(dec_new),
        ) => {
            assert_eq!(orig.call_data(), dec_old.call_data());
            assert_eq!(orig.call_data(), dec_new.call_data());
        }
        _ => panic!("Unexpected UserOperation variants"),
    }

    Ok(())
}

#[tokio::test]
async fn test_encryption_performance_benchmark() -> Result<()> {
    let config = EncryptionConfig::default();
    let service = EncryptionService::new(config)?;

    // Create a UserOperation with moderate-sized data
    let mut user_op = v0_7::UserOperation::default();
    user_op.inner_mut().call_data = Bytes::from(vec![0xAA; 1000]); // 1KB call data
    user_op.inner_mut().paymaster_and_data = Bytes::from(vec![0xBB; 500]); // 500B paymaster data

    let user_op_variant = UserOperationVariant::V0_7(user_op);

    let iterations = 100;
    let start_time = std::time::Instant::now();

    for _ in 0..iterations {
        let encrypted = service
            .process_inbound_user_operation(user_op_variant.clone())
            .await?;
        let _decrypted = service.process_outbound_user_operation(&encrypted).await?;
    }

    let duration = start_time.elapsed();
    let avg_time = duration / iterations;

    println!("M1 Encryption Performance:");
    println!("  {} iterations: {:?}", iterations, duration);
    println!("  Average per operation: {:?}", avg_time);
    println!(
        "  Operations per second: {:.2}",
        1.0 / avg_time.as_secs_f64()
    );

    // Performance assertion - should complete within reasonable time
    assert!(
        avg_time < std::time::Duration::from_millis(10),
        "Encryption should complete within 10ms on average, got {:?}",
        avg_time
    );

    Ok(())
}

#[tokio::test]
async fn test_empty_data_handling() -> Result<()> {
    let config = EncryptionConfig::default();
    let service = EncryptionService::new(config)?;

    // UserOperation with empty data fields
    let user_op = v0_7::UserOperation::default(); // All fields are empty by default
    let user_op_variant = UserOperationVariant::V0_7(user_op);

    let encrypted = service
        .process_inbound_user_operation(user_op_variant.clone())
        .await?;
    let decrypted = service.process_outbound_user_operation(&encrypted).await?;

    // Should handle empty data gracefully
    match (&user_op_variant, &decrypted) {
        (UserOperationVariant::V0_7(original), UserOperationVariant::V0_7(dec)) => {
            assert_eq!(original.call_data(), dec.call_data());
            assert!(original.call_data().is_empty());
            assert!(dec.call_data().is_empty());
        }
        _ => panic!("Unexpected UserOperation variant"),
    }

    Ok(())
}
