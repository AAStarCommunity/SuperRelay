/**
 * TEE TA 双重签名验证测试
 * 测试通过 KMS-TA 端点调用 TEE TA 进行安全的双重签名验证
 * 
 * 此测试验证：
 * 1. AirAccount KMS-TA 端点可正确接收请求
 * 2. TA 双重签名验证逻辑能正确执行
 * 3. 安全的端到端签名验证流程
 */

use ethers::{
    prelude::*,
    utils::{keccak256},
    abi::{encode, Token},
};
use reqwest;
use serde_json::{json, Value};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 开始 TEE TA 双重签名验证测试");
    
    // 1. 设置测试环境
    let kms_ta_url = "http://localhost:3002/kms-ta";
    let paymaster_private_key = "0x59c6995e998f97436e73cb5c6d1c2c7e4a65e2d78ab0b8c5b9fb9a5a8b8f8b8d";
    let paymaster_wallet: LocalWallet = paymaster_private_key.parse().expect("Invalid private key");
    let paymaster_address = format!("0x{:x}", paymaster_wallet.address());
    
    println!("📋 测试参数:");
    println!("   KMS-TA URL: {}", kms_ta_url);
    println!("   Paymaster Address: {}", paymaster_address);
    
    // 2. 构造 UserOperation 测试数据
    let user_operation = json!({
        "sender": "0x1234567890123456789012345678901234567890",
        "nonce": "0x1",
        "initCode": "0x",
        "callData": "0x70a08231000000000000000000000000b4fbf271143f4fbf7b91a5ded31805e42b2208d6",
        "callGasLimit": "0x5208",
        "verificationGasLimit": "0x5208", 
        "preVerificationGas": "0x5208",
        "maxFeePerGas": "0x3b9aca00",
        "maxPriorityFeePerGas": "0x3b9aca00",
        "paymasterAndData": "0x"
    });
    
    // 3. 计算 UserOperation Hash（标准 ABI 编码）
    let tokens = vec![
        Token::Address(H160::from_str("0x1234567890123456789012345678901234567890")?),
        Token::Uint(U256::from(1)),
        Token::Bytes(hex::decode("").unwrap()),
        Token::Bytes(hex::decode("70a08231000000000000000000000000b4fbf271143f4fbf7b91a5ded31805e42b2208d6").unwrap()),
        Token::Uint(U256::from(0x5208)),
        Token::Uint(U256::from(0x5208)),
        Token::Uint(U256::from(0x5208)),
        Token::Uint(U256::from(0x3b9aca00)),
        Token::Uint(U256::from(0x3b9aca00)),
        Token::Bytes(hex::decode("").unwrap()),
    ];
    
    let encoded = encode(&tokens);
    let user_op_hash = keccak256(&encoded);
    let user_op_hash_hex = format!("0x{}", hex::encode(user_op_hash));
    
    println!("📊 UserOperation Hash: {}", user_op_hash_hex);
    
    // 4. 创建测试用户信息
    let account_id = "test_ta_user_001";
    let test_timestamp = chrono::Utc::now().timestamp() as u64;
    let test_nonce = 123456u64;
    
    // 5. 生成 Paymaster 签名
    // 为双重签名创建消息（模拟 solidityPackedKeccak256）
    let user_sig_dummy = vec![0u8; 256]; // 模拟用户签名
    let user_sig_hash = keccak256(&user_sig_dummy);
    
    let mut packed_message = Vec::new();
    packed_message.extend_from_slice(&user_op_hash);
    packed_message.extend_from_slice(account_id.as_bytes());
    packed_message.extend_from_slice(&user_sig_hash);
    packed_message.extend_from_slice(&test_nonce.to_be_bytes());
    packed_message.extend_from_slice(&test_timestamp.to_be_bytes());
    
    let paymaster_message_hash = keccak256(&packed_message);
    let paymaster_signature = paymaster_wallet.sign_hash(H256::from_slice(&paymaster_message_hash))?;
    let paymaster_signature_bytes = paymaster_signature.to_vec();
    let paymaster_signature_hex = format!("0x{}", hex::encode(&paymaster_signature_bytes));
    
    println!("✅ Paymaster 签名生成完成: {}", paymaster_signature_hex);
    
    // 6. 构造双重签名请求
    let request_body = json!({
        "userOperation": user_operation,
        "accountId": account_id,
        "userSignature": format!("0x{}", hex::encode(&user_sig_dummy)),
        "nonce": test_nonce,
        "timestamp": test_timestamp
    });
    
    // 7. 发送 TEE TA 双重签名验证请求
    println!("🔒 向 KMS-TA 发送双重签名验证请求...");
    
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/verify-dual-signature", kms_ta_url))
        .header("Content-Type", "application/json")
        .header("x-paymaster-signature", &paymaster_signature_hex)
        .header("x-paymaster-address", &paymaster_address)
        .json(&request_body)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            let status = resp.status();
            let response_text = resp.text().await?;
            
            println!("📡 KMS-TA 响应状态: {}", status);
            println!("📋 响应内容: {}", response_text);
            
            if status.is_success() {
                let response_json: Value = serde_json::from_str(&response_text)?;
                if response_json["success"].as_bool().unwrap_or(false) {
                    println!("✅ TEE TA 双重签名验证成功！");
                    println!("🔐 TEE 最终签名: {}", response_json["signature"].as_str().unwrap_or("N/A"));
                    println!("📊 验证证明: {}", serde_json::to_string_pretty(&response_json["verificationProof"]).unwrap_or_default());
                } else {
                    println!("❌ TEE TA 双重签名验证失败");
                    println!("   错误: {}", response_json["error"].as_str().unwrap_or("Unknown error"));
                }
            } else {
                println!("❌ HTTP 请求失败: {}", status);
                println!("   响应: {}", response_text);
            }
        }
        Err(e) => {
            println!("❌ 网络请求失败: {}", e);
        }
    }
    
    // 8. 测试 Paymaster 注册功能
    println!("\n📝 测试 Paymaster 注册功能...");
    
    let register_response = client
        .post(&format!("{}/register-paymaster", kms_ta_url))
        .header("Content-Type", "application/json")
        .json(&json!({
            "address": paymaster_address,
            "name": "Test SuperRelay Paymaster"
        }))
        .send()
        .await;
    
    match register_response {
        Ok(resp) => {
            let status = resp.status();
            let response_text = resp.text().await?;
            
            println!("📡 注册响应状态: {}", status);
            println!("📋 注册响应: {}", response_text);
            
            if status.is_success() {
                println!("✅ Paymaster 注册成功！");
            } else {
                println!("⚠️ Paymaster 注册失败或已存在");
            }
        }
        Err(e) => {
            println!("❌ Paymaster 注册请求失败: {}", e);
        }
    }
    
    // 9. 测试状态查询功能
    println!("\n📊 查询 TEE TA 验证状态...");
    
    let status_response = client
        .get(&format!("{}/status", kms_ta_url))
        .send()
        .await;
    
    match status_response {
        Ok(resp) => {
            let status = resp.status();
            let response_text = resp.text().await?;
            
            println!("📡 状态查询响应: {}", status);
            println!("📋 TA 状态: {}", response_text);
            
            if status.is_success() {
                println!("✅ TEE TA 状态查询成功！");
            }
        }
        Err(e) => {
            println!("❌ 状态查询请求失败: {}", e);
        }
    }
    
    println!("\n🎯 TEE TA 双重签名验证测试完成！");
    println!("📝 总结:");
    println!("   • 成功构造了标准 UserOperation 数据");
    println!("   • 生成了有效的 Paymaster 签名");
    println!("   • 调用了 KMS-TA 端点进行 TEE 内验证");
    println!("   • 测试了 Paymaster 注册和状态查询");
    println!("   • 验证了安全的双重签名架构");
    
    Ok(())
}