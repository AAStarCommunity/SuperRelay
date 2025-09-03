use ethers::utils::keccak256;
use ethers::abi::{encode, Token};
use ethers::types::{Address, U256, H256};
use hex;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 UserOperation Hash 一致性测试");
    
    // 使用完全相同的 UserOperation 数据  
    let user_op_json = r#"{
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
    }"#;
    
    let user_op: Value = serde_json::from_str(user_op_json)?;
    
    println!("UserOperation 数据:");
    println!("  Sender: {}", user_op["sender"]);
    println!("  Nonce: {}", user_op["nonce"]);
    println!("  CallData: {}...", user_op["callData"].as_str().unwrap_or("").chars().take(20).collect::<String>());
    
    // 使用正确的 ERC-4337 标准 ABI 编码（不是 encode_packed）
    let user_op_hash = calculate_user_operation_hash_correct(&user_op);
    println!("\n✅ 正确的 UserOperation Hash: 0x{}", hex::encode(user_op_hash.as_bytes()));
    
    // 预期的 JavaScript 计算结果（需要验证）
    println!("📋 预期匹配 JavaScript ethers.AbiCoder 的结果");
    
    Ok(())
}

// 正确的 ERC-4337 UserOperation Hash 计算（使用标准 ABI 编码）
fn calculate_user_operation_hash_correct(user_op: &Value) -> H256 {
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
    
    println!("\n🔧 计算各字段哈希:");
    
    // 计算各字段的哈希
    let init_code_hash = if init_code == "0x" || init_code.is_empty() {
        keccak256(&[])
    } else {
        keccak256(hex::decode(&init_code[2..]).unwrap_or_default())
    };
    println!("  initCode hash: 0x{}", hex::encode(&init_code_hash));
    
    let call_data_hash = if call_data == "0x" || call_data.is_empty() {
        keccak256(&[])
    } else {
        keccak256(hex::decode(&call_data[2..]).unwrap_or_default())
    };
    println!("  callData hash: 0x{}", hex::encode(&call_data_hash));
    
    let paymaster_hash = if paymaster_and_data == "0x" || paymaster_and_data.is_empty() {
        keccak256(&[])
    } else {
        keccak256(hex::decode(&paymaster_and_data[2..]).unwrap_or_default())
    };
    println!("  paymasterAndData hash: 0x{}", hex::encode(&paymaster_hash));
    
    // 使用标准 ABI 编码（不是 encode_packed）
    let tokens = vec![
        Token::Address(sender.parse::<Address>().unwrap_or_default()),
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
    
    // 使用标准 ABI 编码（关键修复！）
    let encoded = encode(&tokens);
    let user_op_hash = keccak256(&encoded);
    
    println!("\n🔧 第一步 ABI 编码:");
    println!("  Encoded length: {} bytes", encoded.len());
    println!("  UserOp hash: 0x{}", hex::encode(&user_op_hash));
    
    // 第二步：与 entry point 和 chain id 一起编码
    let final_tokens = vec![
        Token::FixedBytes(user_op_hash.to_vec()),
        Token::Address(entry_point_address.parse::<Address>().unwrap_or_default()),
        Token::Uint(U256::from(chain_id)),
    ];
    
    let final_encoded = encode(&final_tokens);
    let final_hash = keccak256(&final_encoded);
    
    println!("\n🔧 第二步最终编码:");
    println!("  Final encoded length: {} bytes", final_encoded.len());
    println!("  Final hash: 0x{}", hex::encode(&final_hash));
    
    H256::from(final_hash)
}