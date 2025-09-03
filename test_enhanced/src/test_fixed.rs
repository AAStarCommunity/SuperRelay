use ethers::utils::keccak256;
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 固定值测试 solidityPackedKeccak256 实现");
    
    // 使用与 JavaScript 完全相同的固定值
    let user_op_hash_hex = "0x8dfca86d8053ca45deb4661f4dd97500536aa0ce31f2c03aa73e573b515173af";
    let account_id = "test-account-phase1-real";
    let user_signature = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1b";
    let nonce = 805841u64;  // 0x000c4bd1
    let timestamp = 1756805857u64;  // 0x68b6bae1
    
    println!("参数:");
    println!("  UserOp Hash: {}", user_op_hash_hex);
    println!("  Account ID: {}", account_id);
    println!("  User Signature: {}", user_signature);
    println!("  Nonce: {} (0x{:x})", nonce, nonce);
    println!("  Timestamp: {} (0x{:x})", timestamp, timestamp);
    
    // 解析 UserOp Hash
    let user_op_hash = hex::decode(&user_op_hash_hex[2..])
        .map_err(|e| format!("Failed to decode user_op_hash: {}", e))?;
    
    // 计算 user signature 的 keccak256
    let user_sig_hash = keccak256(user_signature.as_bytes());
    
    println!("\n计算:");
    println!("  User Sig Hash: 0x{}", hex::encode(&user_sig_hash));
    
    // 构建 solidityPackedKeccak256 的打包数据
    let mut packed_data = Vec::new();
    
    // bytes32: UserOp Hash (32字节)
    packed_data.extend_from_slice(&user_op_hash);
    println!("  Added UserOp Hash: {} bytes", user_op_hash.len());
    
    // string: Account ID (UTF-8 bytes)
    packed_data.extend_from_slice(account_id.as_bytes());
    println!("  Added Account ID: {} bytes", account_id.len());
    
    // bytes32: User signature hash (32字节)
    packed_data.extend_from_slice(&user_sig_hash);
    println!("  Added User Sig Hash: {} bytes", user_sig_hash.len());
    
    // uint256: Nonce (32字节大端序)
    let nonce_bytes = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x4b, 0xd1, // 注意：手动确保正确
    ];
    packed_data.extend_from_slice(&nonce_bytes);
    println!("  Added Nonce: 0x{}", hex::encode(&nonce_bytes));
    
    // uint256: Timestamp (32字节大端序)
    let timestamp_bytes = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x68, 0xb6, 0xba, 0xe1, // 注意：手动确保正确
    ];
    packed_data.extend_from_slice(&timestamp_bytes);
    println!("  Added Timestamp: 0x{}", hex::encode(&timestamp_bytes));
    
    println!("\n打包数据:");
    println!("  Total length: {} bytes", packed_data.len());
    println!("  Data: 0x{}", hex::encode(&packed_data[0..64.min(packed_data.len())]));
    if packed_data.len() > 64 {
        println!("  ... (remaining {} bytes)", packed_data.len() - 64);
    }
    
    // 计算最终哈希
    let message_hash = keccak256(&packed_data);
    println!("\n结果:");
    println!("  Message Hash: 0x{}", hex::encode(&message_hash));
    println!("  Expected: 0x372ba18acff4d1024973db46794b0489e88105852b3f2d949954b185e733c4aa");
    
    let expected = hex::decode("372ba18acff4d1024973db46794b0489e88105852b3f2d949954b185e733c4aa")?;
    let matches = message_hash.as_slice() == expected.as_slice();
    println!("  Matches: {}", if matches { "✅ YES" } else { "❌ NO" });
    
    Ok(())
}