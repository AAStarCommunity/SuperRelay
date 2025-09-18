#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! aes-gcm = "0.10"
//! rand = "0.8"
//! secrecy = "0.8"
//! anyhow = "1.0"
//! serde = { version = "1.0", features = ["derive"] }
//! uuid = { version = "1.0", features = ["v4"] }
//! ```

//! 独立测试AES-256-GCM加密功能

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use rand::{rngs::OsRng, RngCore};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

const AES_256_KEY_SIZE: usize = 32;
const AES_GCM_NONCE_SIZE: usize = 12;
const AES_GCM_TAG_SIZE: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; AES_GCM_NONCE_SIZE],
    pub tag: [u8; AES_GCM_TAG_SIZE],
    pub key_id: String,
    pub timestamp: u64,
}

struct SimpleEncryption {
    key: Secret<[u8; AES_256_KEY_SIZE]>,
    key_id: String,
}

impl SimpleEncryption {
    fn new() -> Result<Self> {
        let mut key_bytes = [0u8; AES_256_KEY_SIZE];
        OsRng.fill_bytes(&mut key_bytes);
        
        Ok(Self {
            key: Secret::new(key_bytes),
            key_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    fn encrypt(&self, data: &[u8]) -> Result<EncryptedData> {
        let mut nonce_bytes = [0u8; AES_GCM_NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);

        let key_bytes = self.key.expose_secret();
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext_with_tag = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("AES-GCM encryption failed: {}", e))?;
        
        if ciphertext_with_tag.len() < AES_GCM_TAG_SIZE {
            anyhow::bail!("Invalid ciphertext length");
        }
        
        let (ciphertext, tag_slice) = ciphertext_with_tag.split_at(ciphertext_with_tag.len() - AES_GCM_TAG_SIZE);
        let mut tag = [0u8; AES_GCM_TAG_SIZE];
        tag.copy_from_slice(tag_slice);
        
        Ok(EncryptedData {
            ciphertext: ciphertext.to_vec(),
            nonce: nonce_bytes,
            tag,
            key_id: self.key_id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        })
    }

    fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        let key_bytes = self.key.expose_secret();
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        let mut ciphertext_with_tag = encrypted.ciphertext.clone();
        ciphertext_with_tag.extend_from_slice(&encrypted.tag);
        
        let plaintext = cipher
            .decrypt(nonce, ciphertext_with_tag.as_slice())
            .map_err(|e| anyhow::anyhow!("AES-GCM decryption failed: {}", e))?;
        
        Ok(plaintext)
    }
}

fn main() -> Result<()> {
    println!("🧪 M1: 用户数据安全加密改进 - 独立测试");
    println!("=======================================");

    // Test 1: Basic encryption/decryption
    println!("\n📋 测试1: 基础加密解密");
    let encryption = SimpleEncryption::new()?;
    let test_data = b"Hello World! This is a test message for AES-256-GCM encryption.";
    
    let encrypted = encryption.encrypt(test_data)?;
    println!("✅ 原始数据: {} 字节", test_data.len());
    println!("✅ 加密数据: {} 字节", encrypted.ciphertext.len());
    println!("✅ 密钥ID: {}", encrypted.key_id);
    
    let decrypted = encryption.decrypt(&encrypted)?;
    assert_eq!(decrypted, test_data);
    println!("✅ 解密验证: 成功");

    // Test 2: Different data sizes
    println!("\n📋 测试2: 不同数据大小");
    let test_cases = vec![
        (b"".as_slice(), "空数据"),
        (b"short".as_slice(), "短数据"),
        (b"A longer message that spans multiple AES blocks to test boundary conditions and ensure proper handling of data that exceeds single block size".as_slice(), "长数据"),
        (&[0u8; 1000], "1KB零字节数据"),
        (b"\x00\x01\x02\x03\xff\xfe\xfd\xfc".as_slice(), "二进制数据"),
    ];

    for (data, description) in test_cases {
        let encrypted = encryption.encrypt(data)?;
        let decrypted = encryption.decrypt(&encrypted)?;
        assert_eq!(decrypted.as_slice(), data);
        println!("✅ {}: {} 字节 -> {} 字节", description, data.len(), encrypted.ciphertext.len());
    }

    // Test 3: Nonce uniqueness
    println!("\n📋 测试3: 随机数唯一性验证");
    let test_message = b"test message for nonce uniqueness";
    let mut nonces = std::collections::HashSet::new();
    
    for i in 0..100 {
        let encrypted = encryption.encrypt(test_message)?;
        let nonce_hex = hex::encode(encrypted.nonce);
        
        if nonces.contains(&nonce_hex) {
            anyhow::bail!("发现重复的nonce在第{}次加密中", i + 1);
        }
        nonces.insert(nonce_hex);
        
        // Verify decryption still works
        let decrypted = encryption.decrypt(&encrypted)?;
        assert_eq!(decrypted.as_slice(), test_message);
    }
    println!("✅ 100次加密产生了100个唯一的nonce");

    // Test 4: Tampering detection
    println!("\n📋 测试4: 篡改检测");
    let test_data = b"data for tampering test";
    let mut encrypted = encryption.encrypt(test_data)?;
    
    // Tamper with ciphertext
    if !encrypted.ciphertext.is_empty() {
        encrypted.ciphertext[0] ^= 0x01;
    }
    
    let result = encryption.decrypt(&encrypted);
    assert!(result.is_err(), "篡改的密文应该解密失败");
    println!("✅ 密文篡改检测: 成功");
    
    // Restore and tamper with tag
    encrypted.ciphertext[0] ^= 0x01; // restore
    encrypted.tag[0] ^= 0x01; // tamper tag
    
    let result = encryption.decrypt(&encrypted);
    assert!(result.is_err(), "篡改的认证标签应该解密失败");
    println!("✅ 认证标签篡改检测: 成功");

    // Test 5: Performance test
    println!("\n📋 测试5: 性能测试");
    let large_data = vec![0xAAu8; 10000]; // 10KB data
    let iterations = 100;
    
    let start_time = std::time::Instant::now();
    for _ in 0..iterations {
        let encrypted = encryption.encrypt(&large_data)?;
        let _decrypted = encryption.decrypt(&encrypted)?;
    }
    let duration = start_time.elapsed();
    
    println!("✅ {}次 10KB 数据加密解密耗时: {:?}", iterations, duration);
    println!("✅ 平均每次: {:?}", duration / iterations);
    println!("✅ 吞吐量: {:.2} MB/s", (large_data.len() * iterations as usize * 2) as f64 / duration.as_secs_f64() / 1024.0 / 1024.0);

    println!("\n🎉 所有测试通过！AES-256-GCM加密模块工作正常");
    
    Ok(())
}

// Hex encoding helper (simplified)
mod hex {
    pub fn encode(data: impl AsRef<[u8]>) -> String {
        data.as_ref().iter()
            .map(|byte| format!("{:02x}", byte))
            .collect()
    }
}