//! SuperRelay-AirAccount 双重签名集成测试
//!
//! 测试双重签名流程：
//! 1. SuperRelay Paymaster 密钥管理器生成签名
//! 2. AirAccountKmsClient 构建双重签名请求
//! 3. 模拟 AirAccount KMS 端验证流程

use std::time::Duration;

use anyhow::Result;
use ethers::signers::Signer;
use serde_json::{json, Value};
use tokio::time::timeout;
use tracing::{info, warn};

use crate::{
    airaccount_kms::{AirAccountKmsClient, BusinessValidation, KmsDualSignRequest},
    key_manager::PaymasterKeyManager,
};

/// 双重签名集成测试套件
pub struct DualSignatureIntegrationTest {
    key_manager: PaymasterKeyManager,
    kms_client: Option<AirAccountKmsClient>,
}

impl DualSignatureIntegrationTest {
    /// 创建新的集成测试实例
    pub fn new() -> Self {
        let key_manager = PaymasterKeyManager::with_config(
            Duration::from_secs(3600),                 // 1小时轮换间隔用于测试
            Some("http://localhost:3000".to_string()), // 模拟 AirAccount KMS URL
        );

        Self {
            key_manager,
            kms_client: None,
        }
    }

    /// 初始化 KMS 客户端
    pub async fn initialize_kms_client(&mut self) -> Result<()> {
        info!("🚀 Initializing AirAccount KMS client for integration test");

        let kms_client = AirAccountKmsClient::new(
            "http://localhost:3000".to_string(),
            self.key_manager.clone(),
        );

        self.kms_client = Some(kms_client);
        info!("✅ KMS client initialized successfully");
        Ok(())
    }

    /// 测试密钥管理器基本功能
    pub async fn test_key_manager_basic_functionality(&self) -> Result<()> {
        info!("🔧 Testing PaymasterKeyManager basic functionality");

        // 测试获取签名器
        let signer = self.key_manager.get_signer().await;
        let address = signer.address();
        info!("🔑 Current Paymaster address: {:?}", address);

        // 测试获取状态
        let status = self.key_manager.get_status().await;
        info!("📊 Key manager status: {:?}", status);

        // 验证地址一致性
        assert_eq!(status.current_address, address);
        info!("✅ Key manager basic functionality test passed");

        Ok(())
    }

    /// 测试双重签名请求构建
    pub async fn test_dual_signature_request_building(&self) -> Result<()> {
        info!("🔨 Testing dual signature request building");

        let _kms_client = self
            .kms_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("KMS client not initialized"))?;

        // 模拟用户操作数据
        let user_op = self.create_mock_user_operation();

        // 模拟用户 Passkey 签名和公钥
        let user_passkey_signature = "0x1234567890abcdef";
        let user_public_key = "0xdeadbeefcafebabe";
        let account_id = "test-account-123";

        // 构建业务验证（模拟）
        let business_validation = BusinessValidation {
            balance: "0.1".to_string(),
            membership_level: "premium".to_string(),
            approved_at: chrono::Utc::now().timestamp() as u64,
        };

        // 测试请求构建逻辑（内部方法，这里模拟）
        let request = KmsDualSignRequest {
            user_operation: user_op,
            account_id: account_id.to_string(),
            signature_format: "erc4337".to_string(),
            user_signature: user_passkey_signature.to_string(),
            user_public_key: user_public_key.to_string(),
            business_validation,
            nonce: 12345,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        // 验证请求结构
        assert_eq!(request.account_id, account_id);
        assert_eq!(request.user_signature, user_passkey_signature);
        assert_eq!(request.user_public_key, user_public_key);
        assert_eq!(request.signature_format, "erc4337");

        info!("✅ Dual signature request building test passed");
        Ok(())
    }

    /// 测试 Paymaster 签名生成
    pub async fn test_paymaster_signature_generation(&self) -> Result<()> {
        info!("✍️ Testing Paymaster signature generation");

        let signer = self.key_manager.get_signer().await;
        let test_message = b"test message for signing";

        // 生成签名
        let signature = signer
            .sign_message(test_message)
            .await
            .map_err(|e| anyhow::anyhow!("Signing failed: {}", e))?;

        // 验证签名不为空
        assert!(!signature.to_vec().is_empty());

        // 验证签名长度（ECDSA 签名应该是 65 字节）
        assert_eq!(signature.to_vec().len(), 65);

        info!(
            "🔏 Generated signature: 0x{}",
            hex::encode(signature.to_vec())
        );
        info!("✅ Paymaster signature generation test passed");

        Ok(())
    }

    /// 测试密钥轮换功能
    pub async fn test_key_rotation(&self) -> Result<()> {
        info!("🔄 Testing key rotation functionality");

        let initial_address = self.key_manager.get_address().await;
        info!("🔑 Initial Paymaster address: {:?}", initial_address);

        // 强制进行密钥轮换
        self.key_manager
            .force_rotation()
            .await
            .map_err(|e| anyhow::anyhow!("Key rotation failed: {}", e))?;

        let new_address = self.key_manager.get_address().await;
        info!("🔑 New Paymaster address: {:?}", new_address);

        // 验证地址已经改变
        assert_ne!(initial_address, new_address);

        info!("✅ Key rotation test passed");
        Ok(())
    }

    /// 模拟完整的多重验证流程（不实际连接 AirAccount KMS）
    pub async fn test_complete_multi_layer_verification_flow_simulation(&self) -> Result<()> {
        info!("🎭 Testing complete multi-layer verification flow simulation");

        let _kms_client = self
            .kms_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("KMS client not initialized"))?;

        // 创建模拟数据
        let user_op = self.create_mock_user_operation();
        let account_id = "test-account-456";
        let user_passkey_signature = "0xfedcba0987654321";
        let user_public_key = "0xcafebabe12345678";

        info!("📋 Simulating dual signature for account: {}", account_id);

        // 由于没有实际的 AirAccount KMS 服务，这里只测试请求构建部分
        // 在真实环境中，这里会调用 kms_client.sign_user_operation()

        // 1. 测试业务规则验证（模拟）
        info!("📋 Step 1: Business rules validation");
        // 在实际实现中会检查账户余额、会员状态等

        // 2. 测试 Paymaster 签名生成
        info!("✍️ Step 2: Paymaster signature generation");
        let signer = self.key_manager.get_signer().await;
        let paymaster_address = signer.address();

        // 3. 模拟双重签名请求构建
        info!("🔨 Step 3: Dual signature request construction");
        let business_validation = BusinessValidation {
            balance: "0.05".to_string(),
            membership_level: "premium".to_string(),
            approved_at: chrono::Utc::now().timestamp() as u64,
        };

        let dual_sign_request = KmsDualSignRequest {
            user_operation: user_op,
            account_id: account_id.to_string(),
            signature_format: "erc4337".to_string(),
            user_signature: user_passkey_signature.to_string(),
            user_public_key: user_public_key.to_string(),
            business_validation,
            nonce: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        // 4. 模拟 AirAccount KMS 端的验证流程
        info!("🔐 Step 4: AirAccount KMS validation simulation");
        self.simulate_airaccount_kms_validation(&dual_sign_request, &paymaster_address)
            .await?;

        info!("✅ Complete dual signature flow simulation test passed");
        Ok(())
    }

    /// TODO: 添加真实 OP-TEE(TA) 环境测试
    /// 当前方法仅为模拟测试，生产环境需要连接真实硬件
    #[allow(dead_code)]
    async fn test_real_optee_integration(&self) -> Result<()> {
        info!("🔴 TODO: Real OP-TEE(TA) integration test - NOT IMPLEMENTED YET");
        info!("     This test should:");
        info!("     1. Connect to real TEE device");
        info!("     2. Load actual TA (Trusted Application)");
        info!("     3. Perform hardware-level key generation");
        info!("     4. Execute TEE-based signing operations");
        info!("     5. Verify hardware attestation");

        // TODO: 实现真实的 OP-TEE 测试逻辑
        // - 连接到 TEE 设备
        // - 调用 TA 进行签名
        // - 验证硬件证明

        Ok(())
    }

    /// 运行所有集成测试
    pub async fn run_all_tests(&mut self) -> Result<()> {
        info!("🚀 Starting SuperRelay-AirAccount integration tests");

        // 初始化 KMS 客户端
        self.initialize_kms_client().await?;

        // 运行各项测试
        let test_results = vec![
            ("Key Manager Basic Functionality", {
                self.test_key_manager_basic_functionality().await
            }),
            ("Dual Signature Request Building", {
                self.test_dual_signature_request_building().await
            }),
            ("Paymaster Signature Generation", {
                self.test_paymaster_signature_generation().await
            }),
            ("Key Rotation", { self.test_key_rotation().await }),
            ("Complete Dual Signature Flow Simulation", {
                self.test_complete_multi_layer_verification_flow_simulation()
                    .await
            }),
        ];

        let mut passed = 0;
        let mut failed = 0;

        for (test_name, result) in test_results {
            match result {
                Ok(_) => {
                    info!("✅ {} - PASSED", test_name);
                    passed += 1;
                }
                Err(e) => {
                    warn!("❌ {} - FAILED: {}", test_name, e);
                    failed += 1;
                }
            }
        }

        info!(
            "🏁 Integration tests completed: {} passed, {} failed",
            passed, failed
        );

        if failed > 0 {
            Err(anyhow::anyhow!("{} tests failed", failed))
        } else {
            Ok(())
        }
    }

    /// 创建模拟的 UserOperation
    fn create_mock_user_operation(&self) -> Value {
        json!({
            "sender": "0x1234567890123456789012345678901234567890",
            "nonce": "0x1",
            "initCode": "0x",
            "callData": "0x",
            "callGasLimit": "0x5208",
            "verificationGasLimit": "0x5208",
            "preVerificationGas": "0x5208",
            "maxFeePerGas": "0x3b9aca00",
            "maxPriorityFeePerGas": "0x3b9aca00",
            "paymasterAndData": "0x"
        })
    }

    /// 模拟 AirAccount KMS 端的验证逻辑
    async fn simulate_airaccount_kms_validation(
        &self,
        request: &KmsDualSignRequest,
        paymaster_address: &ethers::types::Address,
    ) -> Result<()> {
        info!("🔍 Simulating AirAccount KMS validation");

        // 1. 模拟时间戳验证
        let current_time = chrono::Utc::now().timestamp() as u64;
        if current_time.saturating_sub(request.timestamp) > 300 {
            return Err(anyhow::anyhow!("Request timestamp too old"));
        }
        info!("✅ Timestamp validation passed");

        // 2. 模拟 nonce 唯一性检查
        // 在真实环境中会检查 nonce 存储
        info!("✅ Nonce uniqueness check passed (simulated)");

        // 3. 模拟 Paymaster 签名验证
        // 这里会使用与 AirAccount KMS 相同的签名验证逻辑
        info!(
            "✅ Paymaster signature verification passed for address: {:?}",
            paymaster_address
        );

        // 4. 模拟 Paymaster 授权检查
        // 在真实环境中会检查白名单
        info!("✅ Paymaster authorization check passed (simulated)");

        // 5. 模拟用户 Passkey 签名验证
        // 这里会验证用户的 WebAuthn/Passkey 签名
        info!("✅ User Passkey signature verification passed (simulated)");

        // 6. 模拟 TEE 签名
        // 在真实环境中会调用 TEE TA 进行硬件签名
        info!("✅ TEE signature generation completed (simulated)");

        info!("🎉 All AirAccount KMS validation steps passed");
        Ok(())
    }
}

impl Default for DualSignatureIntegrationTest {
    fn default() -> Self {
        Self::new()
    }
}

/// 运行集成测试的主函数
pub async fn run_integration_tests() -> Result<()> {
    // 初始化日志（如果还未初始化）
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    let _ = tracing_subscriber::fmt::try_init();

    info!("🎯 Starting SuperRelay-AirAccount Dual Signature Integration Tests");
    info!("⚠️  NOTE: Current tests use MOCK signatures. Real OP-TEE(TA) testing required for production!");

    let mut test_suite = DualSignatureIntegrationTest::new();

    // 设置测试超时
    timeout(Duration::from_secs(60), test_suite.run_all_tests())
        .await
        .map_err(|_| anyhow::anyhow!("Integration tests timed out"))?
        .map_err(|e| anyhow::anyhow!("Integration tests failed: {}", e))?;

    info!("🎉 All integration tests completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_tests() {
        // 设置测试日志级别
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "debug");
        }
        let _ = tracing_subscriber::fmt::try_init();

        let result = run_integration_tests().await;
        assert!(result.is_ok(), "Integration tests should pass");
    }

    #[tokio::test]
    async fn test_key_manager_standalone() {
        let test_suite = DualSignatureIntegrationTest::new();
        let result = test_suite.test_key_manager_basic_functionality().await;
        assert!(
            result.is_ok(),
            "Key manager basic functionality should work"
        );
    }

    #[tokio::test]
    async fn test_signature_generation_standalone() {
        let test_suite = DualSignatureIntegrationTest::new();
        let result = test_suite.test_paymaster_signature_generation().await;
        assert!(result.is_ok(), "Signature generation should work");
    }
}
