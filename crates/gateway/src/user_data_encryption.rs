//! 用户数据安全加密模块
//!
//! 为UserOperation中的敏感数据提供AES-256-GCM加密保护，包括：
//! - 对称加密/解密操作
//! - 密钥派生和管理
//! - 安全随机数生成
//! - 加密数据完整性验证
//! - 多级密钥轮换支持

use std::collections::HashMap;

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng as AeadOsRng},
    Aes256Gcm, Key, Nonce,
};
use alloy_primitives::Bytes;
use anyhow::{Context, Result};
use rand::{rngs::OsRng, RngCore};
use secrecy::{ExposeSecret, Secret, SecretVec};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// AES-256-GCM key size in bytes
const AES_256_KEY_SIZE: usize = 32;
/// AES-GCM nonce/IV size in bytes
const AES_GCM_NONCE_SIZE: usize = 12;
/// AES-GCM tag size in bytes
const AES_GCM_TAG_SIZE: usize = 16;
/// Default key rotation interval in seconds
const DEFAULT_KEY_ROTATION_INTERVAL: u64 = 3600; // 1 hour

/// 加密后的数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// 加密后的数据
    pub ciphertext: Vec<u8>,
    /// 随机数/初始化向量
    pub nonce: [u8; AES_GCM_NONCE_SIZE],
    /// 认证标签
    pub tag: [u8; AES_GCM_TAG_SIZE],
    /// 密钥ID（用于密钥轮换）
    pub key_id: String,
    /// 加密时间戳
    pub timestamp: u64,
}

/// 加密配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// 是否启用加密
    pub enabled: bool,
    /// 密钥轮换间隔（秒）
    pub key_rotation_interval: u64,
    /// 最大密钥缓存数量
    pub max_cached_keys: usize,
    /// 是否对callData加密
    pub encrypt_call_data: bool,
    /// 是否对signature加密
    pub encrypt_signature: bool,
    /// 是否对paymaster数据加密
    pub encrypt_paymaster_data: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            key_rotation_interval: DEFAULT_KEY_ROTATION_INTERVAL,
            max_cached_keys: 5,
            encrypt_call_data: true,
            encrypt_signature: false, // signatures need to be verifiable
            encrypt_paymaster_data: true,
        }
    }
}

/// 密钥信息
#[derive(Debug, Clone)]
struct KeyInfo {
    key: Secret<[u8; AES_256_KEY_SIZE]>,
    created_at: u64,
    key_id: String,
}

/// 用户数据加密管理器
#[derive(Debug)]
pub struct UserDataEncryption {
    config: EncryptionConfig,
    /// 当前活跃密钥
    current_key: KeyInfo,
    /// 历史密钥缓存（用于解密旧数据）
    key_cache: HashMap<String, KeyInfo>,
}

impl UserDataEncryption {
    /// 创建新的加密管理器
    pub fn new(config: EncryptionConfig) -> Result<Self> {
        let current_key = Self::generate_key_info()?;
        let mut key_cache = HashMap::new();
        key_cache.insert(current_key.key_id.clone(), current_key.clone());

        Ok(Self {
            config,
            current_key,
            key_cache,
        })
    }

    /// 生成新的密钥信息
    fn generate_key_info() -> Result<KeyInfo> {
        let mut key_bytes = [0u8; AES_256_KEY_SIZE];
        OsRng.fill_bytes(&mut key_bytes);

        let key_id = uuid::Uuid::new_v4().to_string();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        Ok(KeyInfo {
            key: Secret::new(key_bytes),
            created_at,
            key_id,
        })
    }

    /// 检查是否需要轮换密钥
    pub fn check_key_rotation(&mut self) -> Result<bool> {
        if !self.config.enabled {
            return Ok(false);
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        if now - self.current_key.created_at > self.config.key_rotation_interval {
            self.rotate_key()?;
            return Ok(true);
        }

        Ok(false)
    }

    /// 轮换密钥
    fn rotate_key(&mut self) -> Result<()> {
        info!(
            "Rotating encryption key, old key_id: {}",
            self.current_key.key_id
        );

        // 生成新密钥
        let new_key = Self::generate_key_info()?;

        // 将当前密钥移到缓存中
        self.key_cache
            .insert(self.current_key.key_id.clone(), self.current_key.clone());

        // 清理过期密钥
        if self.key_cache.len() > self.config.max_cached_keys {
            let oldest_key_id = self
                .key_cache
                .iter()
                .min_by_key(|(_, info)| info.created_at)
                .map(|(id, _)| id.clone());

            if let Some(id) = oldest_key_id {
                self.key_cache.remove(&id);
                debug!("Removed expired key: {}", id);
            }
        }

        self.current_key = new_key;
        info!(
            "Key rotation completed, new key_id: {}",
            self.current_key.key_id
        );

        Ok(())
    }

    /// 加密数据
    pub fn encrypt(&self, data: &[u8]) -> Result<EncryptedData> {
        if !self.config.enabled {
            return Ok(EncryptedData {
                ciphertext: data.to_vec(),
                nonce: [0u8; AES_GCM_NONCE_SIZE],
                tag: [0u8; AES_GCM_TAG_SIZE],
                key_id: "plaintext".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
            });
        }

        // 生成随机nonce
        let mut nonce = [0u8; AES_GCM_NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);

        // 实际的AES-GCM加密会在这里实现
        // 由于需要添加crypto依赖，这里先提供接口
        let ciphertext = self.aes_gcm_encrypt(data, &nonce)?;

        Ok(EncryptedData {
            ciphertext: ciphertext.0,
            nonce,
            tag: ciphertext.1,
            key_id: self.current_key.key_id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        })
    }

    /// 解密数据
    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>> {
        if !self.config.enabled || encrypted_data.key_id == "plaintext" {
            return Ok(encrypted_data.ciphertext.clone());
        }

        // 查找对应的密钥
        let key_info = self
            .key_cache
            .get(&encrypted_data.key_id)
            .or_else(|| {
                if encrypted_data.key_id == self.current_key.key_id {
                    Some(&self.current_key)
                } else {
                    None
                }
            })
            .with_context(|| format!("Key not found: {}", encrypted_data.key_id))?;

        // 实际的AES-GCM解密会在这里实现
        self.aes_gcm_decrypt(
            &encrypted_data.ciphertext,
            &encrypted_data.nonce,
            &encrypted_data.tag,
            key_info,
        )
    }

    /// AES-GCM加密实现
    /// 返回 (ciphertext, tag)
    fn aes_gcm_encrypt(
        &self,
        data: &[u8],
        nonce_bytes: &[u8],
    ) -> Result<(Vec<u8>, [u8; AES_GCM_TAG_SIZE])> {
        let key_bytes = self.current_key.key.expose_secret();
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);

        // Convert nonce bytes to proper Nonce type
        let nonce = Nonce::from_slice(&nonce_bytes[..AES_GCM_NONCE_SIZE]);

        // Encrypt the data
        let ciphertext_with_tag = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("AES-GCM encryption failed: {}", e))?;

        // Split ciphertext and tag
        if ciphertext_with_tag.len() < AES_GCM_TAG_SIZE {
            anyhow::bail!("Invalid ciphertext length");
        }

        let (ciphertext, tag_slice) =
            ciphertext_with_tag.split_at(ciphertext_with_tag.len() - AES_GCM_TAG_SIZE);
        let mut tag = [0u8; AES_GCM_TAG_SIZE];
        tag.copy_from_slice(tag_slice);

        debug!(
            "AES-GCM encryption completed, plaintext: {} bytes, ciphertext: {} bytes",
            data.len(),
            ciphertext.len()
        );

        Ok((ciphertext.to_vec(), tag))
    }

    /// AES-GCM解密实现
    fn aes_gcm_decrypt(
        &self,
        ciphertext: &[u8],
        nonce_bytes: &[u8],
        expected_tag: &[u8; AES_GCM_TAG_SIZE],
        key_info: &KeyInfo,
    ) -> Result<Vec<u8>> {
        let key_bytes = key_info.key.expose_secret();
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);

        // Convert nonce bytes to proper Nonce type
        let nonce = Nonce::from_slice(&nonce_bytes[..AES_GCM_NONCE_SIZE]);

        // Reconstruct the ciphertext with tag for decryption
        let mut ciphertext_with_tag = ciphertext.to_vec();
        ciphertext_with_tag.extend_from_slice(expected_tag);

        // Decrypt the data
        let plaintext = cipher
            .decrypt(nonce, ciphertext_with_tag.as_slice())
            .map_err(|e| anyhow::anyhow!("AES-GCM decryption failed: {}", e))?;

        debug!(
            "AES-GCM decryption completed, ciphertext: {} bytes, plaintext: {} bytes",
            ciphertext.len(),
            plaintext.len()
        );

        Ok(plaintext)
    }

    /// 加密UserOperation的callData
    pub fn encrypt_call_data(&self, call_data: &Bytes) -> Result<EncryptedData> {
        if !self.config.encrypt_call_data {
            debug!("CallData encryption disabled");
            return Ok(EncryptedData {
                ciphertext: call_data.to_vec(),
                nonce: [0u8; AES_GCM_NONCE_SIZE],
                tag: [0u8; AES_GCM_TAG_SIZE],
                key_id: "plaintext".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
            });
        }

        debug!("Encrypting callData, size: {} bytes", call_data.len());
        self.encrypt(call_data)
    }

    /// 解密UserOperation的callData
    pub fn decrypt_call_data(&self, encrypted: &EncryptedData) -> Result<Bytes> {
        let decrypted = self.decrypt(encrypted)?;
        Ok(Bytes::from(decrypted))
    }

    /// 加密paymaster数据
    pub fn encrypt_paymaster_data(&self, paymaster_data: &Bytes) -> Result<EncryptedData> {
        if !self.config.encrypt_paymaster_data {
            debug!("Paymaster data encryption disabled");
            return Ok(EncryptedData {
                ciphertext: paymaster_data.to_vec(),
                nonce: [0u8; AES_GCM_NONCE_SIZE],
                tag: [0u8; AES_GCM_TAG_SIZE],
                key_id: "plaintext".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
            });
        }

        debug!(
            "Encrypting paymaster data, size: {} bytes",
            paymaster_data.len()
        );
        self.encrypt(paymaster_data)
    }

    /// 解密paymaster数据
    pub fn decrypt_paymaster_data(&self, encrypted: &EncryptedData) -> Result<Bytes> {
        let decrypted = self.decrypt(encrypted)?;
        Ok(Bytes::from(decrypted))
    }

    /// 获取当前密钥ID（用于监控）
    pub fn current_key_id(&self) -> &str {
        &self.current_key.key_id
    }

    /// 获取缓存的密钥数量
    pub fn cached_keys_count(&self) -> usize {
        self.key_cache.len()
    }

    /// 获取密钥年龄（秒）
    pub fn current_key_age(&self) -> Result<u64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        Ok(now - self.current_key.created_at)
    }
}

/// 加密状态信息
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptionStatus {
    pub enabled: bool,
    pub current_key_id: String,
    pub key_age_seconds: u64,
    pub cached_keys_count: usize,
    pub encryption_config: EncryptionConfig,
}

impl UserDataEncryption {
    /// 获取加密状态信息
    pub fn get_status(&self) -> Result<EncryptionStatus> {
        Ok(EncryptionStatus {
            enabled: self.config.enabled,
            current_key_id: self.current_key.key_id.clone(),
            key_age_seconds: self.current_key_age()?,
            cached_keys_count: self.cached_keys_count(),
            encryption_config: self.config.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_config_default() {
        let config = EncryptionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.key_rotation_interval, DEFAULT_KEY_ROTATION_INTERVAL);
        assert!(config.encrypt_call_data);
        assert!(!config.encrypt_signature);
        assert!(config.encrypt_paymaster_data);
    }

    #[test]
    fn test_user_data_encryption_creation() -> Result<()> {
        let config = EncryptionConfig::default();
        let encryption = UserDataEncryption::new(config)?;

        assert!(!encryption.current_key_id().is_empty());
        assert_eq!(encryption.cached_keys_count(), 1);

        Ok(())
    }

    #[test]
    fn test_basic_encryption_decryption() -> Result<()> {
        let config = EncryptionConfig::default();
        let encryption = UserDataEncryption::new(config)?;

        let test_data = b"hello world, this is test data";
        let encrypted = encryption.encrypt(test_data)?;

        assert_ne!(encrypted.ciphertext, test_data);
        assert!(!encrypted.key_id.is_empty());

        let decrypted = encryption.decrypt(&encrypted)?;
        assert_eq!(decrypted, test_data);

        Ok(())
    }

    #[test]
    fn test_call_data_encryption() -> Result<()> {
        let config = EncryptionConfig::default();
        let encryption = UserDataEncryption::new(config)?;

        let call_data = Bytes::from(b"test call data".as_slice());
        let encrypted = encryption.encrypt_call_data(&call_data)?;
        let decrypted = encryption.decrypt_call_data(&encrypted)?;

        assert_eq!(decrypted, call_data);

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

        Ok(())
    }

    #[test]
    fn test_encryption_status() -> Result<()> {
        let config = EncryptionConfig::default();
        let encryption = UserDataEncryption::new(config)?;

        let status = encryption.get_status()?;
        assert!(status.enabled);
        assert!(!status.current_key_id.is_empty());
        assert_eq!(status.cached_keys_count, 1);

        Ok(())
    }
}
