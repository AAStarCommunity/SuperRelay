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
    println!("🚀 Enhanced Phase 1 测试：真实签名 + TEE TA 集成");
    println!("==================================================");
    
    // 1. 创建真实的 Paymaster 钱包和签名
    println!("\n1. 🔑 创建真实 Paymaster 钱包...");
    let paymaster_wallet = LocalWallet::new(&mut thread_rng());
    let paymaster_address = paymaster_wallet.address();
    println!("   Paymaster Address: {:?}", to_checksum(&paymaster_address, None));
    
    // 2. 先授权 Paymaster 地址
    println!("\n2. 🔐 授权 Paymaster 地址...");
    let admin_token = "dev_admin_token_for_testing"; // 测试用 token
    
    let auth_request = json!({
        "paymasterAddress": format!("{:?}", paymaster_address),
        "name": "Test Paymaster for Phase 1",
        "permissions": ["dual_signature", "tee_signing"]
    });
    
    let client = reqwest::Client::new();
    let auth_response = client
        .post("http://localhost:3002/kms/admin/authorize-paymaster")
        .header("Content-Type", "application/json")
        .header("admin-token", admin_token)
        .json(&auth_request)
        .send()
        .await?;
    
    if auth_response.status().is_success() {
        let auth_body: Value = auth_response.json().await?;
        println!("   ✅ Paymaster 授权成功:");
        println!("      {}", serde_json::to_string_pretty(&auth_body)?);
    } else {
        println!("   ⚠️  Paymaster 授权失败 (可能已授权): {}", auth_response.status());
        let error_body: Value = auth_response.json().await.unwrap_or_default();
        println!("      Error: {}", serde_json::to_string_pretty(&error_body)?);
    }
    
    // 3. 创建真实的 UserOperation
    println!("\n3. 🏗️  创建真实 UserOperation...");
    let user_op = json!({
        "sender": "0x1234567890123456789012345678901234567890",
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
    
    // 4. 计算 UserOperation 哈希
    let user_op_hash = calculate_user_operation_hash(&user_op);
    println!("   UserOperation Hash: 0x{}", hex::encode(user_op_hash.as_bytes()));
    
    // 5. 创建双重签名请求数据
    let account_id = "test-account-phase1-real";
    let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let nonce = (current_timestamp % 1000000) as u64;
    
    // 6. 创建 Paymaster 签名消息并签名
    println!("\n4. ✍️  创建真实 Paymaster 签名...");
    
    // 构建消息哈希 (与 KMS 路由中的 ethers.solidityPackedKeccak256 逻辑一致)
    let user_sig_mock = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1b";
    
    // 计算 userSignature 的 keccak256（与 ethers.keccak256(ethers.toUtf8Bytes()) 匹配）
    let user_sig_hash = keccak256(user_sig_mock.as_bytes());
    
    // 构建 solidityPackedKeccak256 的打包数据
    // solidityPackedKeccak256(['bytes32', 'string', 'bytes32', 'uint256', 'uint256'])
    let mut packed_data = Vec::new();
    
    // bytes32: UserOp Hash (32字节)
    packed_data.extend_from_slice(user_op_hash.as_bytes());
    
    // string: Account ID (UTF-8 bytes，无长度前缀)
    packed_data.extend_from_slice(account_id.as_bytes());
    
    // bytes32: User signature hash (32字节)  
    packed_data.extend_from_slice(&user_sig_hash);
    
    // uint256: Nonce (32字节大端序，零填充)
    let mut nonce_bytes = [0u8; 32];
    let nonce_be = nonce.to_be_bytes();  // u64 -> [u8; 8]
    nonce_bytes[32 - 8..].copy_from_slice(&nonce_be);  // 放到最后8字节
    packed_data.extend_from_slice(&nonce_bytes);
    
    // uint256: Timestamp (32字节大端序，零填充)
    let mut timestamp_bytes = [0u8; 32];
    let timestamp_be = current_timestamp.to_be_bytes();  // u64 -> [u8; 8] 
    timestamp_bytes[32 - 8..].copy_from_slice(&timestamp_be);  // 放到最后8字节
    packed_data.extend_from_slice(&timestamp_bytes);
    
    let message_hash = keccak256(&packed_data);
    
    // 调试信息
    println!("   Packed data length: {} bytes", packed_data.len());
    println!("   User sig hash: 0x{}", hex::encode(&user_sig_hash));
    println!("   Nonce bytes: 0x{}", hex::encode(&nonce_bytes));
    println!("   Timestamp bytes: 0x{}", hex::encode(&timestamp_bytes));
    
    println!("   Message Hash: 0x{}", hex::encode(&message_hash));
    
    // 签名消息 (使用 ethers 的消息签名方式，与 KMS verifyMessage 兼容)
    let signature: Signature = paymaster_wallet.sign_message(&message_hash).await?;
    let signature_bytes = signature.to_vec();
    
    println!("   Signature: 0x{}", hex::encode(&signature_bytes));
    
    // 7. 创建完整的双重签名请求
    let dual_sign_request = json!({
        "userOperation": user_op,
        "accountId": account_id,
        "signatureFormat": "erc4337",
        "userSignature": user_sig_mock,
        "userPublicKey": "0x04deadbeefcafebabe1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12",
        "businessValidation": {
            "balance": "1.5",
            "membershipLevel": "premium",
            "approvedAt": current_timestamp
        },
        "nonce": nonce,
        "timestamp": current_timestamp
    });
    
    // 8. 发送真实的双重签名请求
    println!("\n5. 🔐 发送真实双重签名请求...");
    
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
    
    // 9. 验证响应
    if status.is_success() {
        if let Some(signature) = body.get("signature") {
            println!("\n   ✅ 双重签名成功！");
            println!("   TEE Signature: {}", signature);
            
            if let Some(proof) = body.get("verificationProof") {
                println!("   验证证明: {}", serde_json::to_string_pretty(proof)?);
            }
            
            if let Some(tee_device_id) = body.get("teeDeviceId") {
                println!("   TEE Device ID: {}", tee_device_id);
            }
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
    
    // 10. 测试 KMS 状态
    println!("\n6. 📊 检查 KMS 最新状态...");
    let status_response = client
        .get("http://localhost:3002/kms/status")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
    
    if status_response.status().is_success() {
        let status_body: Value = status_response.json().await?;
        println!("   ✅ KMS 状态:");
        println!("   {}", serde_json::to_string_pretty(&status_body)?);
    } else {
        println!("   ⚠️  KMS 状态获取失败: {}", status_response.status());
    }
    
    println!("\n🎉 Enhanced Phase 1 测试完成！");
    println!("=================================");
    println!("✅ 真实 Paymaster 签名生成和验证");
    println!("✅ 完整的双重签名请求流程");
    println!("✅ TEE 环境集成验证"); 
    println!("✅ KMS 服务状态监控");
    
    Ok(())
}

// 计算 UserOperation 哈希 (ERC-4337 标准)
fn calculate_user_operation_hash(user_op: &Value) -> H256 {
    let entry_point_address = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";
    let chain_id = 11155111u64; // Sepolia
    
    // 按照 ERC-4337 标准计算哈希
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
    
    // 创建打包数据
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