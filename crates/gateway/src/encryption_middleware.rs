//! 用户数据加密中间件
//!
//! 提供自动的UserOperation数据加密/解密功能：
//! - 入站请求：加密敏感的UserOperation数据
//! - 出站响应：解密数据用于客户端
//! - 透明的密钥轮换支持
//! - 配置驱动的选择性加密

use std::sync::Arc;

use alloy_primitives::Bytes;
use anyhow::{Context, Result};
use rundler_types::user_operation::UserOperationVariant;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::user_data_encryption::{EncryptedData, EncryptionConfig, UserDataEncryption};

/// 加密的UserOperation数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedUserOperationData {
    /// 原始的UserOperation（非敏感字段保持原样）
    pub base_operation: UserOperationVariant,
    /// 加密的callData（如果启用）
    pub encrypted_call_data: Option<EncryptedData>,
    /// 加密的paymaster数据（如果启用）
    pub encrypted_paymaster_data: Option<EncryptedData>,
    /// 加密的factory数据（如果启用）
    pub encrypted_factory_data: Option<EncryptedData>,
    /// 加密标记
    pub is_encrypted: bool,
}

/// 加密中间件
#[derive(Debug)]
pub struct EncryptionMiddleware {
    encryption: Arc<RwLock<UserDataEncryption>>,
    config: EncryptionConfig,
}

impl EncryptionMiddleware {
    /// 创建新的加密中间件
    pub fn new(config: EncryptionConfig) -> Result<Self> {
        let encryption = UserDataEncryption::new(config.clone())?;
        Ok(Self {
            encryption: Arc::new(RwLock::new(encryption)),
            config,
        })
    }

    /// 检查并执行密钥轮换
    pub async fn check_key_rotation(&self) -> Result<bool> {
        let mut encryption = self.encryption.write().await;
        encryption.check_key_rotation()
    }

    /// 加密UserOperation的敏感数据
    pub async fn encrypt_user_operation(
        &self,
        user_op: UserOperationVariant,
    ) -> Result<EncryptedUserOperationData> {
        if !self.config.enabled {
            debug!("Encryption disabled, returning original operation");
            return Ok(EncryptedUserOperationData {
                base_operation: user_op,
                encrypted_call_data: None,
                encrypted_paymaster_data: None,
                encrypted_factory_data: None,
                is_encrypted: false,
            });
        }

        let encryption = self.encryption.read().await;
        let mut encrypted_data = EncryptedUserOperationData {
            base_operation: user_op.clone(),
            encrypted_call_data: None,
            encrypted_paymaster_data: None,
            encrypted_factory_data: None,
            is_encrypted: true,
        };

        // 加密callData
        if self.config.encrypt_call_data {
            let call_data = user_op.call_data();
            if !call_data.is_empty() {
                let encrypted = encryption.encrypt_call_data(call_data)?;
                encrypted_data.encrypted_call_data = Some(encrypted);
                debug!("Encrypted callData: {} bytes", call_data.len());
            }
        }

        // 加密paymaster数据
        if self.config.encrypt_paymaster_data {
            if let Some(paymaster_data) = self.extract_paymaster_data(&user_op) {
                if !paymaster_data.is_empty() {
                    let encrypted = encryption.encrypt_paymaster_data(&paymaster_data)?;
                    encrypted_data.encrypted_paymaster_data = Some(encrypted);
                    debug!("Encrypted paymaster data: {} bytes", paymaster_data.len());
                }
            }
        }

        // 加密factory数据
        if let Some(factory_data) = self.extract_factory_data(&user_op) {
            if !factory_data.is_empty() {
                let encrypted = encryption.encrypt(&factory_data)?;
                encrypted_data.encrypted_factory_data = Some(encrypted);
                debug!("Encrypted factory data: {} bytes", factory_data.len());
            }
        }

        info!(
            "UserOperation encryption completed: callData={}, paymaster={}, factory={}",
            encrypted_data.encrypted_call_data.is_some(),
            encrypted_data.encrypted_paymaster_data.is_some(),
            encrypted_data.encrypted_factory_data.is_some()
        );

        Ok(encrypted_data)
    }

    /// 解密UserOperation数据
    pub async fn decrypt_user_operation(
        &self,
        encrypted_data: &EncryptedUserOperationData,
    ) -> Result<UserOperationVariant> {
        if !encrypted_data.is_encrypted {
            debug!("Data not encrypted, returning original operation");
            return Ok(encrypted_data.base_operation.clone());
        }

        let encryption = self.encryption.read().await;
        let mut user_op = encrypted_data.base_operation.clone();

        // 解密callData
        if let Some(encrypted_call_data) = &encrypted_data.encrypted_call_data {
            let decrypted = encryption.decrypt_call_data(encrypted_call_data)?;
            user_op = self.set_call_data(user_op, decrypted)?;
            debug!(
                "Decrypted callData: {} bytes",
                encrypted_call_data.ciphertext.len()
            );
        }

        // 解密paymaster数据
        if let Some(encrypted_paymaster) = &encrypted_data.encrypted_paymaster_data {
            let decrypted = encryption.decrypt_paymaster_data(encrypted_paymaster)?;
            user_op = self.set_paymaster_data(user_op, decrypted)?;
            debug!(
                "Decrypted paymaster data: {} bytes",
                encrypted_paymaster.ciphertext.len()
            );
        }

        // 解密factory数据
        if let Some(encrypted_factory) = &encrypted_data.encrypted_factory_data {
            let decrypted_bytes = encryption.decrypt(encrypted_factory)?;
            let decrypted = Bytes::from(decrypted_bytes);
            user_op = self.set_factory_data(user_op, decrypted)?;
            debug!(
                "Decrypted factory data: {} bytes",
                encrypted_factory.ciphertext.len()
            );
        }

        info!("UserOperation decryption completed successfully");
        Ok(user_op)
    }

    /// 获取加密状态信息
    pub async fn encryption_status(&self) -> Result<crate::user_data_encryption::EncryptionStatus> {
        let encryption = self.encryption.read().await;
        encryption.get_status()
    }

    /// 提取paymaster数据（根据UserOperation版本）
    fn extract_paymaster_data(&self, user_op: &UserOperationVariant) -> Option<Bytes> {
        match user_op {
            UserOperationVariant::V0_6(op) => {
                if op.paymaster_and_data.is_empty() {
                    None
                } else {
                    Some(op.paymaster_and_data.clone())
                }
            }
            UserOperationVariant::V0_7(op) => {
                let paymaster_data = op.paymaster_data();
                if paymaster_data.is_empty() {
                    None
                } else {
                    Some(paymaster_data.clone())
                }
            }
            UserOperationVariant::V0_8(op) => {
                let paymaster_data = op.paymaster_data();
                if paymaster_data.is_empty() {
                    None
                } else {
                    Some(paymaster_data.clone())
                }
            }
        }
    }

    /// 提取factory数据（根据UserOperation版本）
    fn extract_factory_data(&self, user_op: &UserOperationVariant) -> Option<Vec<u8>> {
        match user_op {
            UserOperationVariant::V0_6(op) => {
                if op.init_code.is_empty() {
                    None
                } else {
                    Some(op.init_code.to_vec())
                }
            }
            UserOperationVariant::V0_7(op) => {
                let factory_data = op.factory_data();
                if factory_data.is_empty() {
                    None
                } else {
                    Some(factory_data.to_vec())
                }
            }
            UserOperationVariant::V0_8(op) => {
                let factory_data = op.factory_data();
                if factory_data.is_empty() {
                    None
                } else {
                    Some(factory_data.to_vec())
                }
            }
        }
    }

    /// 设置callData（根据UserOperation版本）
    fn set_call_data(
        &self,
        mut user_op: UserOperationVariant,
        call_data: Bytes,
    ) -> Result<UserOperationVariant> {
        match &mut user_op {
            UserOperationVariant::V0_6(op) => {
                op.call_data = call_data;
            }
            UserOperationVariant::V0_7(op) => {
                // V0.7 uses PackedUserOperation, need to access the inner structure
                op.inner_mut().call_data = call_data;
            }
            UserOperationVariant::V0_8(op) => {
                // V0.8 wraps V0.7, so access the wrapped operation
                op.inner_mut().inner_mut().call_data = call_data;
            }
        }
        Ok(user_op)
    }

    /// 设置paymaster数据（根据UserOperation版本）
    fn set_paymaster_data(
        &self,
        mut user_op: UserOperationVariant,
        paymaster_data: Bytes,
    ) -> Result<UserOperationVariant> {
        match &mut user_op {
            UserOperationVariant::V0_6(op) => {
                op.paymaster_and_data = paymaster_data;
            }
            UserOperationVariant::V0_7(op) => {
                // For V0.7, need to update the paymaster_and_data field in PackedUserOperation
                op.inner_mut().paymaster_and_data = paymaster_data;
            }
            UserOperationVariant::V0_8(op) => {
                // V0.8 wraps V0.7
                op.inner_mut().inner_mut().paymaster_and_data = paymaster_data;
            }
        }
        Ok(user_op)
    }

    /// 设置factory数据（根据UserOperation版本）  
    fn set_factory_data(
        &self,
        mut user_op: UserOperationVariant,
        factory_data: Bytes,
    ) -> Result<UserOperationVariant> {
        match &mut user_op {
            UserOperationVariant::V0_6(op) => {
                op.init_code = factory_data;
            }
            UserOperationVariant::V0_7(op) => {
                // For V0.7, need to update the init_code field in PackedUserOperation
                op.inner_mut().init_code = factory_data;
            }
            UserOperationVariant::V0_8(op) => {
                // V0.8 wraps V0.7
                op.inner_mut().inner_mut().init_code = factory_data;
            }
        }
        Ok(user_op)
    }
}

/// 加密中间件服务
#[derive(Debug)]
pub struct EncryptionService {
    middleware: EncryptionMiddleware,
}

impl EncryptionService {
    /// 创建加密服务
    pub fn new(config: EncryptionConfig) -> Result<Self> {
        let middleware = EncryptionMiddleware::new(config)?;
        Ok(Self { middleware })
    }

    /// 处理入站UserOperation（加密敏感数据）
    pub async fn process_inbound_user_operation(
        &self,
        user_op: UserOperationVariant,
    ) -> Result<EncryptedUserOperationData> {
        self.middleware.encrypt_user_operation(user_op).await
    }

    /// 处理出站UserOperation（解密数据供客户端使用）
    pub async fn process_outbound_user_operation(
        &self,
        encrypted_data: &EncryptedUserOperationData,
    ) -> Result<UserOperationVariant> {
        self.middleware.decrypt_user_operation(encrypted_data).await
    }

    /// 启动后台密钥轮换任务
    pub async fn start_key_rotation_task(self: Arc<Self>) -> Result<()> {
        let service = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(60), // Check every minute
            );

            loop {
                interval.tick().await;

                match service.middleware.check_key_rotation().await {
                    Ok(rotated) => {
                        if rotated {
                            info!("Encryption key rotated successfully");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to check key rotation: {}", e);
                    }
                }
            }
        });

        info!("Encryption key rotation task started");
        Ok(())
    }

    /// 获取加密状态
    pub async fn status(&self) -> Result<crate::user_data_encryption::EncryptionStatus> {
        self.middleware.encryption_status().await
    }
}

#[cfg(test)]
mod tests {
    use rundler_types::user_operation::v0_7;

    use super::*;

    #[tokio::test]
    async fn test_encryption_middleware_creation() -> Result<()> {
        let config = EncryptionConfig::default();
        let middleware = EncryptionMiddleware::new(config)?;

        let status = middleware.encryption_status().await?;
        assert!(status.enabled);

        Ok(())
    }

    #[tokio::test]
    async fn test_disabled_encryption() -> Result<()> {
        let mut config = EncryptionConfig::default();
        config.enabled = false;

        let middleware = EncryptionMiddleware::new(config)?;
        let user_op = UserOperationVariant::V0_7(v0_7::UserOperation::default());

        let encrypted = middleware.encrypt_user_operation(user_op.clone()).await?;

        assert!(!encrypted.is_encrypted);
        assert!(encrypted.encrypted_call_data.is_none());
        assert!(encrypted.encrypted_paymaster_data.is_none());

        let decrypted = middleware.decrypt_user_operation(&encrypted).await?;

        // Should be identical when encryption is disabled
        match (&user_op, &decrypted) {
            (UserOperationVariant::V0_7(orig), UserOperationVariant::V0_7(dec)) => {
                assert_eq!(orig.call_data(), dec.call_data());
            }
            _ => panic!("Unexpected UserOperation variant"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_encryption_service() -> Result<()> {
        let config = EncryptionConfig::default();
        let service = EncryptionService::new(config)?;

        let user_op = UserOperationVariant::V0_7(v0_7::UserOperation::default());

        // Process inbound (encrypt)
        let encrypted = service
            .process_inbound_user_operation(user_op.clone())
            .await?;
        assert!(encrypted.is_encrypted);

        // Process outbound (decrypt)
        let decrypted = service.process_outbound_user_operation(&encrypted).await?;

        // Should be equivalent after round-trip
        match (&user_op, &decrypted) {
            (UserOperationVariant::V0_7(orig), UserOperationVariant::V0_7(dec)) => {
                assert_eq!(orig.call_data(), dec.call_data());
            }
            _ => panic!("Unexpected UserOperation variant"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_key_rotation_check() -> Result<()> {
        let mut config = EncryptionConfig::default();
        config.key_rotation_interval = 1; // 1 second for testing

        let middleware = EncryptionMiddleware::new(config)?;

        // Should not need rotation immediately
        let rotated = middleware.check_key_rotation().await?;
        assert!(!rotated);

        // Wait for key to age
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Should trigger rotation now
        let rotated = middleware.check_key_rotation().await?;
        assert!(rotated);

        Ok(())
    }
}
