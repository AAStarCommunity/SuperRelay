use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ethers::{
    signers::Signer,
    types::{Address, U256},
    utils::{keccak256, to_checksum},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    key_manager::PaymasterKeyManager,
    kms::{
        KmsError, KmsKeyInfo, KmsKeyType, KmsProvider, KmsSigningRequest, KmsSigningResponse,
        SigningAuditInfo,
    },
};

/// AirAccount KMS 客户端
/// 实现双重签名验证机制，与 AirAccount TEE-KMS 服务通信
pub struct AirAccountKmsClient {
    base_url: String,
    http_client: Client,
    key_manager: PaymasterKeyManager,
    timeout: Duration,
}

/// KMS 双重签名请求
#[derive(Debug, Serialize)]
pub struct KmsDualSignRequest {
    #[serde(rename = "userOperation")]
    pub user_operation: Value,
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "signatureFormat")]
    pub signature_format: String,
    #[serde(rename = "userSignature")]
    pub user_signature: String,
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
    #[serde(rename = "businessValidation")]
    pub business_validation: BusinessValidation,
    pub nonce: u64,
    pub timestamp: u64,
}

/// 业务验证信息
#[derive(Debug, Serialize)]
pub struct BusinessValidation {
    pub balance: String,
    #[serde(rename = "membershipLevel")]
    pub membership_level: String,
    #[serde(rename = "approvedAt")]
    pub approved_at: u64,
}

/// KMS 签名响应
#[derive(Debug, Deserialize)]
pub struct KmsSignResponse {
    pub success: bool,
    pub signature: String,
    #[serde(rename = "userOpHash")]
    pub user_op_hash: String,
    #[serde(rename = "teeDeviceId")]
    pub tee_device_id: String,
    #[serde(rename = "verificationProof")]
    pub verification_proof: VerificationProof,
}

/// 验证证明
#[derive(Debug, Deserialize)]
pub struct VerificationProof {
    #[serde(rename = "paymasterVerified")]
    pub paymaster_verified: bool,
    #[serde(rename = "userPasskeyVerified")]
    pub user_passkey_verified: bool,
    #[serde(rename = "dualSignatureMode")]
    pub dual_signature_mode: bool,
    pub timestamp: String,
}

/// KMS 状态响应
#[derive(Debug, Deserialize)]
pub struct KmsStatusResponse {
    pub success: bool,
    pub status: KmsStatus,
    pub timestamp: String,
}

/// KMS 状态信息
#[derive(Debug, Deserialize)]
pub struct KmsStatus {
    pub service: String,
    pub mode: String,
    #[serde(rename = "teeConnection")]
    pub tee_connection: String,
    #[serde(rename = "authorizedPaymastersCount")]
    pub authorized_paymasters_count: u32,
    #[serde(rename = "activeNoncesCount")]
    pub active_nonces_count: u32,
    pub features: Vec<String>,
}

impl AirAccountKmsClient {
    /// 创建新的 KMS 客户端
    pub fn new(base_url: String, key_manager: PaymasterKeyManager) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("SuperRelay-Paymaster/1.0")
            .build()
            .unwrap();

        Self {
            base_url,
            http_client,
            key_manager,
            timeout: Duration::from_secs(30),
        }
    }

    /// 使用双重签名机制签名 UserOperation
    pub async fn sign_user_operation(
        &self,
        user_op: &Value,
        account_id: &str,
        user_passkey_signature: &str,
        user_public_key: &str,
    ) -> Result<KmsSignResponse> {
        info!(
            "🔐 Initiating dual-signature UserOperation signing for account: {}",
            account_id
        );

        // 1. 验证业务规则
        let business_validation = self.validate_business_rules(account_id).await?;

        // 2. 构建请求数据
        let request_data = self
            .build_dual_sign_request(
                user_op,
                account_id,
                user_passkey_signature,
                user_public_key,
                business_validation,
            )
            .await?;

        // 3. 使用 Paymaster 私钥签名请求
        let (paymaster_signature, paymaster_address) = self.sign_request(&request_data).await?;

        // 4. 发送双重签名请求
        let response = self
            .send_kms_request(&request_data, &paymaster_signature, &paymaster_address)
            .await?;

        info!("✅ Dual-signature UserOperation signed successfully");
        Ok(response)
    }

    /// 验证业务规则
    async fn validate_business_rules(&self, account_id: &str) -> Result<BusinessValidation> {
        debug!("📋 Validating business rules for account: {}", account_id);

        // TODO: 实现真实的业务规则验证
        // 这里应该检查：
        // - 账户余额
        // - 会员状态
        // - 使用限额
        // - 黑名单检查

        // 模拟业务验证
        let business_validation = BusinessValidation {
            balance: "0.05".to_string(), // 模拟余额 0.05 ETH
            membership_level: "premium".to_string(),
            approved_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        debug!("✅ Business validation passed for account: {}", account_id);
        Ok(business_validation)
    }

    /// 构建双重签名请求
    async fn build_dual_sign_request(
        &self,
        user_op: &Value,
        account_id: &str,
        user_signature: &str,
        user_public_key: &str,
        business_validation: BusinessValidation,
    ) -> Result<KmsDualSignRequest> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(KmsDualSignRequest {
            user_operation: user_op.clone(),
            account_id: account_id.to_string(),
            signature_format: "erc4337".to_string(),
            user_signature: user_signature.to_string(),
            user_public_key: user_public_key.to_string(),
            business_validation,
            nonce,
            timestamp,
        })
    }

    /// 使用 Paymaster 私钥签名请求
    async fn sign_request(&self, request_data: &KmsDualSignRequest) -> Result<(String, String)> {
        debug!("🖋️ Signing request with Paymaster key");

        let signer = self.key_manager.get_signer().await;

        // 计算签名消息（与 AirAccount KMS 端的验证逻辑一致）
        let user_op_hash = Self::get_user_operation_hash(&request_data.user_operation)?;
        let user_signature_hash = keccak256(request_data.user_signature.as_bytes());

        let message_to_sign = keccak256(ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(user_op_hash.to_vec()),
            ethers::abi::Token::String(request_data.account_id.clone()),
            ethers::abi::Token::FixedBytes(user_signature_hash.to_vec()),
            ethers::abi::Token::Uint(U256::from(request_data.nonce)),
            ethers::abi::Token::Uint(U256::from(request_data.timestamp)),
        ]));

        let signature = signer
            .sign_message(message_to_sign)
            .await
            .map_err(|e| anyhow!("Failed to sign request: {}", e))?;

        let paymaster_address = to_checksum(&signer.address(), None);
        let paymaster_signature = format!("0x{}", hex::encode(signature.to_vec()));

        debug!("✅ Request signed by Paymaster: {}", paymaster_address);
        Ok((paymaster_signature, paymaster_address))
    }

    /// 发送 KMS 请求
    async fn send_kms_request(
        &self,
        request_data: &KmsDualSignRequest,
        paymaster_signature: &str,
        paymaster_address: &str,
    ) -> Result<KmsSignResponse> {
        let url = format!("{}/kms/sign-user-operation", self.base_url);

        debug!("📤 Sending KMS request to: {}", url);

        let response = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Paymaster-Signature", paymaster_signature)
            .header("X-Paymaster-Address", paymaster_address)
            .json(request_data)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send KMS request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("KMS request failed: {} - {}", status, error_text));
        }

        let kms_response: KmsSignResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse KMS response: {}", e))?;

        if !kms_response.success {
            return Err(anyhow!("KMS signing failed"));
        }

        debug!("✅ KMS request successful");
        Ok(kms_response)
    }

    /// 检查 KMS 服务状态
    pub async fn check_status(&self) -> Result<KmsStatusResponse> {
        let url = format!("{}/kms/status", self.base_url);

        debug!("📊 Checking KMS status: {}", url);

        let response = self
            .http_client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to check KMS status: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "KMS status check failed: {} - {}",
                status,
                error_text
            ));
        }

        let status_response: KmsStatusResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse KMS status: {}", e))?;

        debug!("✅ KMS status retrieved");
        Ok(status_response)
    }

    /// 计算 UserOperation 哈希（ERC-4337 标准）
    fn get_user_operation_hash(user_op: &Value) -> Result<[u8; 32]> {
        // 提取 UserOperation 字段
        let sender = user_op["sender"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing sender in UserOperation"))?;
        let nonce = user_op["nonce"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing nonce in UserOperation"))?;
        let init_code = user_op["initCode"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing initCode in UserOperation"))?;
        let call_data = user_op["callData"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing callData in UserOperation"))?;
        let call_gas_limit = user_op["callGasLimit"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing callGasLimit in UserOperation"))?;
        let verification_gas_limit = user_op["verificationGasLimit"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing verificationGasLimit in UserOperation"))?;
        let pre_verification_gas = user_op["preVerificationGas"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing preVerificationGas in UserOperation"))?;
        let max_fee_per_gas = user_op["maxFeePerGas"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing maxFeePerGas in UserOperation"))?;
        let max_priority_fee_per_gas = user_op["maxPriorityFeePerGas"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing maxPriorityFeePerGas in UserOperation"))?;
        let paymaster_and_data = user_op["paymasterAndData"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing paymasterAndData in UserOperation"))?;

        // 解析地址和数值
        let sender_addr: ethers::types::Address = sender.parse()?;
        let nonce_u256 = ethers::types::U256::from_str_radix(&nonce[2..], 16)?;
        let init_code_hash = keccak256(hex::decode(&init_code[2..])?);
        let call_data_hash = keccak256(hex::decode(&call_data[2..])?);
        let call_gas_limit_u256 = ethers::types::U256::from_str_radix(&call_gas_limit[2..], 16)?;
        let verification_gas_limit_u256 =
            ethers::types::U256::from_str_radix(&verification_gas_limit[2..], 16)?;
        let pre_verification_gas_u256 =
            ethers::types::U256::from_str_radix(&pre_verification_gas[2..], 16)?;
        let max_fee_per_gas_u256 = ethers::types::U256::from_str_radix(&max_fee_per_gas[2..], 16)?;
        let max_priority_fee_per_gas_u256 =
            ethers::types::U256::from_str_radix(&max_priority_fee_per_gas[2..], 16)?;
        let paymaster_and_data_hash = keccak256(hex::decode(&paymaster_and_data[2..])?);

        // ERC-4337 UserOperation 哈希计算
        let entry_point: ethers::types::Address =
            "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789".parse()?;
        let chain_id = U256::from(11155111u64); // Sepolia

        // 编码第一层
        let encoded = ethers::abi::encode(&[
            ethers::abi::Token::Address(sender_addr),
            ethers::abi::Token::Uint(nonce_u256),
            ethers::abi::Token::FixedBytes(init_code_hash.to_vec()),
            ethers::abi::Token::FixedBytes(call_data_hash.to_vec()),
            ethers::abi::Token::Uint(call_gas_limit_u256),
            ethers::abi::Token::Uint(verification_gas_limit_u256),
            ethers::abi::Token::Uint(pre_verification_gas_u256),
            ethers::abi::Token::Uint(max_fee_per_gas_u256),
            ethers::abi::Token::Uint(max_priority_fee_per_gas_u256),
            ethers::abi::Token::FixedBytes(paymaster_and_data_hash.to_vec()),
        ]);

        // 计算最终哈希
        let final_encoded = ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(keccak256(&encoded).to_vec()),
            ethers::abi::Token::Address(entry_point),
            ethers::abi::Token::Uint(chain_id),
        ]);

        Ok(keccak256(&final_encoded))
    }

    /// 设置请求超时
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// 获取 KMS 基础 URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[async_trait]
impl KmsProvider for AirAccountKmsClient {
    async fn sign(
        &mut self,
        request: KmsSigningRequest,
        context: crate::kms::SigningContext,
    ) -> Result<KmsSigningResponse, KmsError> {
        let start_time = SystemTime::now();
        let request_id = Uuid::new_v4().to_string();

        info!("🔐 AirAccount KMS signing request: {}", request_id);
        debug!("   Key ID: {}", request.key_id);
        debug!("   Message hash: {:?}", request.message_hash);

        // Convert SigningContext to UserOperation JSON for AirAccount KMS
        let user_op_json = self.convert_context_to_userop(&context).map_err(|e| {
            KmsError::InvalidConfiguration {
                reason: format!("Failed to convert context: {}", e),
            }
        })?;

        // Extract account_id from context metadata
        let account_id =
            context
                .metadata
                .get("account_id")
                .ok_or_else(|| KmsError::InvalidConfiguration {
                    reason: "Missing account_id in signing context".to_string(),
                })?;

        // Extract user signature and public key from metadata
        let user_signature = context.metadata.get("user_signature").ok_or_else(|| {
            KmsError::InvalidConfiguration {
                reason: "Missing user_signature in signing context".to_string(),
            }
        })?;

        let user_public_key = context.metadata.get("user_public_key").ok_or_else(|| {
            KmsError::InvalidConfiguration {
                reason: "Missing user_public_key in signing context".to_string(),
            }
        })?;

        // Use dual-signature mechanism
        let kms_response = self
            .sign_user_operation(&user_op_json, account_id, user_signature, user_public_key)
            .await
            .map_err(|e| KmsError::SignatureFailed {
                reason: format!("AirAccount KMS signing failed: {}", e),
            })?;

        // Parse signature from response
        let signature_bytes = hex::decode(kms_response.signature.trim_start_matches("0x"))
            .map_err(|e| KmsError::SignatureFailed {
                reason: format!("Invalid signature format: {}", e),
            })?;

        let signature =
            ethers::types::Signature::try_from(signature_bytes.as_slice()).map_err(|e| {
                KmsError::SignatureFailed {
                    reason: format!("Failed to parse signature: {}", e),
                }
            })?;

        let duration_ms = start_time.elapsed().unwrap_or_default().as_millis() as u64;

        // Create audit information
        let mut service_metadata = std::collections::HashMap::new();
        service_metadata.insert("kms_provider".to_string(), "airaccount_kms".to_string());
        service_metadata.insert("key_id".to_string(), request.key_id.clone());
        service_metadata.insert("tee_device_id".to_string(), kms_response.tee_device_id);
        service_metadata.insert(
            "dual_signature_mode".to_string(),
            kms_response
                .verification_proof
                .dual_signature_mode
                .to_string(),
        );
        service_metadata.insert(
            "paymaster_verified".to_string(),
            kms_response
                .verification_proof
                .paymaster_verified
                .to_string(),
        );
        service_metadata.insert(
            "user_passkey_verified".to_string(),
            kms_response
                .verification_proof
                .user_passkey_verified
                .to_string(),
        );

        let audit_info = SigningAuditInfo {
            request_id,
            service_metadata,
            duration_ms,
            hardware_validated: true, // AirAccount uses TEE hardware
        };

        info!("✅ AirAccount KMS signing completed successfully");

        Ok(KmsSigningResponse {
            signature,
            key_id: request.key_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            audit_info,
        })
    }

    async fn get_key_address(&self, key_id: &str) -> Result<Address, KmsError> {
        debug!("📍 Getting address for AirAccount KMS key: {}", key_id);

        // For AirAccount KMS, return the paymaster address from key manager
        let signer = self.key_manager.get_signer().await;
        Ok(signer.address())
    }

    async fn list_keys(&self) -> Result<Vec<KmsKeyInfo>, KmsError> {
        debug!("📋 Listing available AirAccount KMS keys");

        // Get the single paymaster key info
        let signer = self.key_manager.get_signer().await;
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("environment".to_string(), "airaccount_kms".to_string());
        metadata.insert("service".to_string(), "super-relay-airaccount".to_string());

        let key_info = KmsKeyInfo {
            key_id: "airaccount-paymaster-key".to_string(),
            key_type: KmsKeyType::HardwareSecurityModule, // TEE-based
            address: signer.address(),
            description: "AirAccount KMS Paymaster Key with TEE support".to_string(),
            enabled: true,
            permissions: vec!["sign".to_string(), "dual_signature".to_string()],
            metadata,
        };

        Ok(vec![key_info])
    }

    async fn health_check(&self) -> Result<bool, KmsError> {
        debug!("🩺 Performing AirAccount KMS health check");

        match self.check_status().await {
            Ok(status) => {
                if status.success && status.status.tee_connection == "connected" {
                    info!("✅ AirAccount KMS health check passed");
                    Ok(true)
                } else {
                    Err(KmsError::ServiceUnavailable {
                        reason: format!("KMS status check failed: {:?}", status),
                    })
                }
            }
            Err(e) => {
                error!("❌ AirAccount KMS health check failed: {}", e);
                Err(KmsError::ServiceUnavailable {
                    reason: format!("Health check failed: {}", e),
                })
            }
        }
    }
}

impl AirAccountKmsClient {
    /// Convert SigningContext to UserOperation JSON format
    fn convert_context_to_userop(&self, context: &crate::kms::SigningContext) -> Result<Value> {
        // For AirAccount KMS, we need a full UserOperation structure
        // This is a simplified conversion - in practice, you'd need the full UserOp data
        let user_op = serde_json::json!({
            "sender": context.sender_address.unwrap_or_default().to_string(),
            "nonce": "0x0", // Should come from context
            "initCode": "0x",
            "callData": "0x",
            "callGasLimit": context.gas_estimates
                .as_ref()
                .map(|g| format!("0x{:x}", g.call_gas_limit))
                .unwrap_or_else(|| "0x186a0".to_string()),
            "verificationGasLimit": context.gas_estimates
                .as_ref()
                .map(|g| format!("0x{:x}", g.verification_gas_limit))
                .unwrap_or_else(|| "0x186a0".to_string()),
            "preVerificationGas": context.gas_estimates
                .as_ref()
                .map(|g| format!("0x{:x}", g.pre_verification_gas))
                .unwrap_or_else(|| "0x5208".to_string()),
            "maxFeePerGas": context.gas_estimates
                .as_ref()
                .map(|g| format!("0x{:x}", g.max_fee_per_gas))
                .unwrap_or_else(|| "0x59682f00".to_string()),
            "maxPriorityFeePerGas": context.gas_estimates
                .as_ref()
                .map(|g| format!("0x{:x}", g.max_priority_fee_per_gas))
                .unwrap_or_else(|| "0x3b9aca00".to_string()),
            "paymasterAndData": "0x"
        });

        Ok(user_op)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_kms_client_creation() {
        let key_manager = PaymasterKeyManager::new();
        let client = AirAccountKmsClient::new("http://localhost:3002".to_string(), key_manager);

        assert_eq!(client.base_url(), "http://localhost:3002");
    }

    #[tokio::test]
    async fn test_business_validation() {
        let key_manager = PaymasterKeyManager::new();
        let client = AirAccountKmsClient::new("http://localhost:3002".to_string(), key_manager);

        let validation = client
            .validate_business_rules("test-account")
            .await
            .unwrap();
        assert_eq!(validation.membership_level, "premium");
        assert!(!validation.balance.is_empty());
    }

    #[tokio::test]
    async fn test_user_operation_hash() {
        let user_op = json!({
            "sender": "0x742D35Cc6634C0532925a3b8D6C18E3CB1EB98C1",
            "nonce": "0x0",
            "initCode": "0x",
            "callData": "0x",
            "callGasLimit": "0x186a0",
            "verificationGasLimit": "0x186a0",
            "preVerificationGas": "0x5208",
            "maxFeePerGas": "0x59682f00",
            "maxPriorityFeePerGas": "0x3b9aca00",
            "paymasterAndData": "0x"
        });

        let hash = AirAccountKmsClient::get_user_operation_hash(&user_op).unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[tokio::test]
    async fn test_request_signing() {
        let key_manager = PaymasterKeyManager::new();
        let client = AirAccountKmsClient::new("http://localhost:3002".to_string(), key_manager);

        let request = KmsDualSignRequest {
            user_operation: json!({}),
            account_id: "test-account".to_string(),
            signature_format: "erc4337".to_string(),
            user_signature: "0x1234567890".to_string(),
            user_public_key: "0xabcdef".to_string(),
            business_validation: BusinessValidation {
                balance: "0.05".to_string(),
                membership_level: "premium".to_string(),
                approved_at: 1234567890,
            },
            nonce: 123456789,
            timestamp: 1234567890,
        };

        let (signature, address) = client.sign_request(&request).await.unwrap();
        assert!(signature.starts_with("0x"));
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42); // Ethereum address length
    }
}
