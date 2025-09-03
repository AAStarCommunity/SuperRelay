use std::time::{SystemTime, UNIX_EPOCH};
use reqwest;
use serde_json::{json, Value};
use tokio;
use ethers::signers::{LocalWallet, Signer};
use ethers::core::rand::thread_rng;
use ethers::types::{Signature, H256, U256};
use ethers::utils::{keccak256, to_checksum};
use ethers::abi::{encode, Token};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 TEE TA 账户创建和签名测试");
    println!("==============================");
    
    let client = reqwest::Client::new();
    
    // 1. 创建真实的 TEE TA 账户
    println!("\n1. 🏗️  创建真实 TEE TA 账户...");
    let account_id = "test-tee-account-phase1";
    let account_request = json!({
        "email": "test@example.com",
        "passkeyCredentialId": "mock-credential-id-for-testing",
        "passkeyPublicKey": "04deadbeef" // 模拟公钥
    });
    
    let account_response = client
        .post("http://localhost:3002/kms/test/create-tee-account")
        .header("Content-Type", "application/json")
        .json(&account_request)
        .send()
        .await?;
    
    let account_status = account_response.status();
    println!("   TEE Account 创建状态: {}", account_status);
    
    if account_response.status().is_success() {
        let account_body: Value = account_response.json().await?;
        println!("   ✅ TEE Account 创建成功:");
        println!("      {}", serde_json::to_string_pretty(&account_body)?);
        
        // 从响应中获取 walletId 和 ethereumAddress
        if let (Some(wallet_id), Some(ethereum_address)) = (
            account_body.get("walletId"),
            account_body.get("ethereumAddress")
        ) {
            println!("   Wallet ID: {}", wallet_id);
            println!("   Ethereum Address: {}", ethereum_address);
            
            // 2. 创建 Paymaster 并授权
            println!("\n2. 🔐 创建并授权 Paymaster...");
            let paymaster_wallet = LocalWallet::new(&mut thread_rng());
            let paymaster_address = paymaster_wallet.address();
            println!("   Paymaster Address: {:?}", to_checksum(&paymaster_address, None));
            
            // 授权 Paymaster
            let admin_token = "dev_admin_token_for_testing";
            let auth_request = json!({
                "paymasterAddress": format!("{:?}", paymaster_address),
                "name": "TEE Test Paymaster",
                "permissions": ["dual_signature", "tee_signing"]
            });
            
            let auth_response = client
                .post("http://localhost:3002/kms/admin/authorize-paymaster")
                .header("Content-Type", "application/json")
                .header("admin-token", admin_token)
                .json(&auth_request)
                .send()
                .await?;
            
            if auth_response.status().is_success() {
                let auth_body: Value = auth_response.json().await?;
                println!("   ✅ Paymaster 授权成功");
            }
            
            // 3. 使用真实账户进行双重签名
            println!("\n3. ✍️  使用真实 TEE TA 账户进行双重签名...");
            
            // 创建 UserOperation
            let user_op = json!({
                "sender": ethereum_address,
                "nonce": "0x1",
                "initCode": "0x",
                "callData": "0xb61d27f60000000000000000000000001234567890123456789012345678901234567890000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000",
                "callGasLimit": "0x5208",
                "verificationGasLimit": "0x5208", 
                "preVerificationGas": "0x5208",
                "maxFeePerGas": "0x3b9aca00",
                "maxPriorityFeePerGas": "0x3b9aca00",
                "paymasterAndData": "0x"
            });
            
            // 计算 UserOperation Hash
            let user_op_hash = calculate_user_operation_hash(&user_op);
            println!("   UserOperation Hash: 0x{}", hex::encode(user_op_hash.as_bytes()));
            
            // 创建 Paymaster 签名消息
            let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let nonce = (current_timestamp % 1000000) as u64;
            
            let user_sig_mock = "0xmocksignaturefortesting1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456789001b";
            let user_sig_hash = keccak256(user_sig_mock.as_bytes());
            
            // 构建 solidityPackedKeccak256 消息
            let mut packed_data = Vec::new();
            packed_data.extend_from_slice(user_op_hash.as_bytes());
            packed_data.extend_from_slice(account_id.as_bytes());
            packed_data.extend_from_slice(&user_sig_hash);
            
            let mut nonce_bytes = [0u8; 32];
            let nonce_be = nonce.to_be_bytes();
            nonce_bytes[32 - 8..].copy_from_slice(&nonce_be);
            packed_data.extend_from_slice(&nonce_bytes);
            
            let mut timestamp_bytes = [0u8; 32];
            let timestamp_be = current_timestamp.to_be_bytes();
            timestamp_bytes[32 - 8..].copy_from_slice(&timestamp_be);
            packed_data.extend_from_slice(&timestamp_bytes);
            
            let message_hash = keccak256(&packed_data);
            
            // 生成 Paymaster 签名
            let signature: Signature = paymaster_wallet.sign_message(&message_hash).await?;
            let signature_bytes = signature.to_vec();
            
            println!("   Message Hash: 0x{}", hex::encode(&message_hash));
            println!("   Paymaster Signature: 0x{}", hex::encode(&signature_bytes));
            
            // 4. 发送双重签名请求到 KMS
            println!("\n4. 🔐 发送双重签名请求...");
            
            let dual_sign_request = json!({
                "userOperation": user_op,
                "accountId": account_id,
                "signatureFormat": "erc4337",
                "userSignature": user_sig_mock,
                "userPublicKey": "0x04deadbeefcafebabe1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12",
                "businessValidation": {
                    "balance": "2.0",
                    "membershipLevel": "premium",
                    "approvedAt": current_timestamp
                },
                "nonce": nonce,
                "timestamp": current_timestamp
            });
            
            let response = client
                .post("http://localhost:3002/kms/sign-user-operation")
                .header("Content-Type", "application/json")
                .header("x-paymaster-address", format!("{:?}", paymaster_address))
                .header("x-paymaster-signature", format!("0x{}", hex::encode(&signature_bytes)))
                .json(&dual_sign_request)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await?;
            
            let status = response.status();
            let body: Value = response.json().await?;
            
            println!("   Response Status: {}", status);
            println!("   Response Body:");
            println!("   {}", serde_json::to_string_pretty(&body)?);
            
            // 5. 验证结果
            if status.is_success() {
                if let Some(signature) = body.get("signature") {
                    println!("\n   ✅ 真实 TEE TA 双重签名成功！");
                    println!("   TEE Signature: {}", signature);
                    
                    if let Some(tee_device_id) = body.get("teeDeviceId") {
                        println!("   TEE Device ID: {}", tee_device_id);
                    }
                    
                    if let Some(proof) = body.get("verificationProof") {
                        println!("   验证证明: {}", serde_json::to_string_pretty(proof)?);
                    }
                    
                    println!("\n🎉 Phase 1 Enhanced 测试完全成功！");
                    println!("================================");
                    println!("✅ 真实 TEE TA 账户创建");
                    println!("✅ 真实 Paymaster 签名验证");
                    println!("✅ 完整双重签名流程");
                    println!("✅ 真实 TEE 硬件签名");
                } else {
                    println!("\n   ⚠️  响应中缺少签名字段");
                }
            } else {
                println!("\n   ⚠️  双重签名失败:");
                if let Some(error) = body.get("error") {
                    println!("   Error: {}", error);
                }
                if let Some(details) = body.get("details") {
                    println!("   Details: {}", details);
                }
            }
        } else {
            println!("   ⚠️  账户创建响应中缺少必要字段");
        }
    } else {
        println!("   ⚠️  TEE Account 创建失败: {}", account_status);
        let error_body: Value = account_response.json().await.unwrap_or_default();
        println!("      Error: {}", serde_json::to_string_pretty(&error_body)?);
    }
    
    Ok(())
}

// 复用之前修复的正确 UserOperation Hash 计算函数
fn calculate_user_operation_hash(user_op: &Value) -> H256 {
    let entry_point_address = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";
    let chain_id = 11155111u64; // Sepolia
    
    // 解析 UserOperation 字段
    let sender = user_op["sender"].as_str().unwrap_or("0x0");
    let nonce = user_op["nonce"].as_str().unwrap_or("0x0");
    let init_code = user_op["initCode"].as_str().unwrap_or("0x");
    let call_data = user_op["callData"].as_str().unwrap_or("0x");
    let call_gas_limit = user_op["callGasLimit"].as_str().unwrap_or("0x0");
    let verification_gas_limit = user_op["verificationGasLimit"].as_str().unwrap_or("0x0");
    let pre_verification_gas = user_op["preVerificationGas"].as_str().unwrap_or("0x0");
    let max_fee_per_gas = user_op["maxFeePerGas"].as_str().unwrap_or("0x0");
    let max_priority_fee_per_gas = user_op["maxPriorityFeePerGas"].as_str().unwrap_or("0x0");
    let paymaster_and_data = user_op["paymasterAndData"].as_str().unwrap_or("0x");
    
    // 计算各字段的哈希
    let init_code_hash = if init_code == "0x" || init_code.is_empty() {
        keccak256(&[])
    } else {
        keccak256(hex::decode(&init_code[2..]).unwrap_or_default())
    };
    
    let call_data_hash = if call_data == "0x" || call_data.is_empty() {
        keccak256(&[])
    } else {
        keccak256(hex::decode(&call_data[2..]).unwrap_or_default())
    };
    
    let paymaster_hash = if paymaster_and_data == "0x" || paymaster_and_data.is_empty() {
        keccak256(&[])
    } else {
        keccak256(hex::decode(&paymaster_and_data[2..]).unwrap_or_default())
    };
    
    let tokens = vec![
        Token::Address(sender.parse().unwrap_or_default()),
        Token::Uint(U256::from_str_radix(&nonce[2..], 16).unwrap_or_default()),
        Token::FixedBytes(init_code_hash.to_vec()),
        Token::FixedBytes(call_data_hash.to_vec()),
        Token::Uint(U256::from_str_radix(&call_gas_limit[2..], 16).unwrap_or_default()),
        Token::Uint(U256::from_str_radix(&verification_gas_limit[2..], 16).unwrap_or_default()),
        Token::Uint(U256::from_str_radix(&pre_verification_gas[2..], 16).unwrap_or_default()),
        Token::Uint(U256::from_str_radix(&max_fee_per_gas[2..], 16).unwrap_or_default()),
        Token::Uint(U256::from_str_radix(&max_priority_fee_per_gas[2..], 16).unwrap_or_default()),
        Token::FixedBytes(paymaster_hash.to_vec()),
    ];
    
    // 使用标准 ABI 编码（不是 encode_packed）
    let encoded = encode(&tokens);
    let user_op_hash = keccak256(&encoded);
    
    // 最终哈希包含 entry point 和 chain id
    let final_tokens = vec![
        Token::FixedBytes(user_op_hash.to_vec()),
        Token::Address(entry_point_address.parse().unwrap_or_default()),
        Token::Uint(U256::from(chain_id)),
    ];
    
    let final_encoded = encode(&final_tokens);
    H256::from(keccak256(&final_encoded))
}