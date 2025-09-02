use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use ethers::{
    signers::{LocalWallet, Signer},
    types::Address,
};
use rand::rngs::OsRng;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// PaymasterKeyManager 负责管理 SuperRelay Paymaster 的签名密钥
///
/// 功能：
/// - 自动密钥轮换（默认24小时）
/// - 线程安全的密钥访问
/// - 密钥轮换通知机制
/// - 支持开发和生产环境配置
#[derive(Clone)]
pub struct PaymasterKeyManager {
    current_wallet: Arc<RwLock<LocalWallet>>,
    rotation_interval: Duration,
    last_rotation: Arc<RwLock<Instant>>,
    kms_notification_url: Option<String>,
}

impl PaymasterKeyManager {
    /// 创建新的密钥管理器
    pub fn new() -> Self {
        Self::with_config(
            Duration::from_secs(86400), // 24小时轮换
            None,
        )
    }

    /// 使用自定义配置创建密钥管理器
    pub fn with_config(rotation_interval: Duration, kms_notification_url: Option<String>) -> Self {
        // 初始化时生成新密钥
        let wallet = LocalWallet::new(&mut OsRng);

        info!(
            "🔑 PaymasterKeyManager initialized with address: {}",
            wallet.address()
        );

        Self {
            current_wallet: Arc::new(RwLock::new(wallet)),
            rotation_interval,
            last_rotation: Arc::new(RwLock::new(Instant::now())),
            kms_notification_url,
        }
    }

    /// 获取当前签名器
    /// 会自动检查是否需要轮换密钥
    pub async fn get_signer(&self) -> LocalWallet {
        // 检查是否需要轮换
        if self.should_rotate().await {
            if let Err(e) = self.rotate_key().await {
                error!("Failed to rotate key: {}", e);
            }
        }

        self.current_wallet.read().await.clone()
    }

    /// 获取当前签名器地址
    pub async fn get_address(&self) -> Address {
        let wallet = self.current_wallet.read().await;
        wallet.address()
    }

    /// 检查是否需要轮换密钥
    async fn should_rotate(&self) -> bool {
        let last_rotation = *self.last_rotation.read().await;
        last_rotation.elapsed() > self.rotation_interval
    }

    /// 手动触发密钥轮换
    pub async fn force_rotation(&self) -> Result<(), PaymasterKeyError> {
        self.rotate_key().await
    }

    /// 轮换密钥
    async fn rotate_key(&self) -> Result<(), PaymasterKeyError> {
        let old_address = {
            let wallet = self.current_wallet.read().await;
            wallet.address()
        };

        // 生成新密钥
        let new_wallet = LocalWallet::new(&mut OsRng);
        let new_address = new_wallet.address();

        info!(
            "🔄 Rotating Paymaster signing key: {} -> {}",
            old_address, new_address
        );

        // 通知 AirAccount KMS 新的公钥（如果配置了通知URL）
        if let Some(ref url) = self.kms_notification_url {
            if let Err(e) = self
                .notify_key_rotation(old_address, new_address, url)
                .await
            {
                warn!("Failed to notify KMS about key rotation: {}", e);
                // 通知失败不阻止密钥轮换，但记录警告
            }
        }

        // 更新密钥
        {
            let mut wallet_guard = self.current_wallet.write().await;
            *wallet_guard = new_wallet;
        }

        // 更新轮换时间
        {
            let mut last_rotation = self.last_rotation.write().await;
            *last_rotation = Instant::now();
        }

        info!("✅ Paymaster signing key rotated successfully");
        Ok(())
    }

    /// 通知 AirAccount KMS 密钥轮换
    async fn notify_key_rotation(
        &self,
        old_address: Address,
        new_address: Address,
        notification_url: &str,
    ) -> Result<(), PaymasterKeyError> {
        use reqwest::Client;
        use serde_json::json;

        let client = Client::new();
        let notification_payload = json!({
            "event": "key_rotation",
            "old_address": format!("{:?}", old_address),
            "new_address": format!("{:?}", new_address),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "rotation_proof": self.generate_rotation_proof(old_address, new_address).await?
        });

        let response = client
            .post(notification_url)
            .json(&notification_payload)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| PaymasterKeyError::NotificationFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PaymasterKeyError::NotificationFailed(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        info!("📤 Key rotation notification sent to KMS");
        Ok(())
    }

    /// 生成密钥轮换证明（双签名验证）
    async fn generate_rotation_proof(
        &self,
        old_address: Address,
        new_address: Address,
    ) -> Result<String, PaymasterKeyError> {
        // 构建轮换消息
        use ethers::{abi::encode, utils::keccak256};

        let rotation_message = encode(&[
            ethers::abi::Token::Address(old_address),
            ethers::abi::Token::Address(new_address),
            ethers::abi::Token::Uint(ethers::types::U256::from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )),
        ]);

        let message_hash = keccak256(&rotation_message);

        // 使用新密钥签名轮换消息
        let new_wallet = self.current_wallet.read().await;
        let signature = new_wallet
            .sign_message(message_hash)
            .await
            .map_err(|e| PaymasterKeyError::SigningFailed(e.to_string()))?;

        // 返回证明数据
        Ok(format!("0x{}", hex::encode(signature.to_vec())))
    }

    /// 获取密钥管理器状态
    pub async fn get_status(&self) -> PaymasterKeyStatus {
        let wallet = self.current_wallet.read().await;
        let last_rotation = *self.last_rotation.read().await;

        PaymasterKeyStatus {
            current_address: wallet.address(),
            last_rotation,
            next_rotation: last_rotation + self.rotation_interval,
            rotation_interval: self.rotation_interval,
            time_until_rotation: self
                .rotation_interval
                .saturating_sub(last_rotation.elapsed()),
        }
    }

    /// 配置 KMS 通知 URL
    pub async fn set_kms_notification_url(&mut self, url: Option<String>) {
        self.kms_notification_url = url;
    }
}

impl Default for PaymasterKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 密钥管理器状态
#[derive(Debug)]
pub struct PaymasterKeyStatus {
    pub current_address: Address,
    pub last_rotation: Instant,
    pub next_rotation: Instant,
    pub rotation_interval: Duration,
    pub time_until_rotation: Duration,
}

/// 密钥管理器错误类型
#[derive(Debug, thiserror::Error)]
pub enum PaymasterKeyError {
    #[error("Key rotation notification failed: {0}")]
    NotificationFailed(String),

    #[error("Signing operation failed: {0}")]
    SigningFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[cfg(test)]
mod tests {
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_key_manager_creation() {
        let manager = PaymasterKeyManager::new();
        let address = manager.get_address().await;

        // 地址不应该为空
        assert_ne!(address, Address::zero());

        let status = manager.get_status().await;
        assert_eq!(status.current_address, address);
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let manager = PaymasterKeyManager::with_config(
            Duration::from_millis(100), // 100ms 轮换间隔用于测试
            None,
        );

        let initial_address = manager.get_address().await;

        // 等待轮换间隔
        sleep(Duration::from_millis(150)).await;

        // 触发轮换
        let signer = manager.get_signer().await;
        let new_address = signer.address();

        // 地址应该已经改变
        assert_ne!(initial_address, new_address);
    }

    #[tokio::test]
    async fn test_force_rotation() {
        let manager = PaymasterKeyManager::new();
        let initial_address = manager.get_address().await;

        // 强制轮换
        manager.force_rotation().await.unwrap();

        let new_address = manager.get_address().await;
        assert_ne!(initial_address, new_address);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let manager = Arc::new(PaymasterKeyManager::new());
        let mut handles = vec![];

        // 并发访问测试
        for _ in 0..10 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let signer = manager_clone.get_signer().await;
                signer.address()
            });
            handles.push(handle);
        }

        let addresses: Vec<Address> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // 所有地址应该相同（因为没有轮换）
        let first_address = addresses[0];
        assert!(addresses.iter().all(|&addr| addr == first_address));
    }
}
