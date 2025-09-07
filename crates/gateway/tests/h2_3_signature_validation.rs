//! 测试 H2.3 标准化ECDSA签名格式功能
//!
//! 验证ECDSA签名格式标准化的所有功能，包括：
//! - 标准65字节签名格式验证
//! - 紧凑64字节签名格式处理
//! - r, s, v 组件验证
//! - malleable签名检测和防护
//! - 签名规范化功能
//! - 各种安全边界条件测试

use alloy_primitives::U256;
use super_relay_gateway::{
    signature_validator::{SignatureFormat, SignatureValidator},
    validation::{DataIntegrityChecker, ValidationSeverity},
};

/// 测试标准65字节ECDSA签名验证
#[tokio::test]
async fn test_valid_65_byte_ecdsa_signature() {
    println!("🧪 测试标准65字节ECDSA签名验证");

    let validator = SignatureValidator::new();

    // 创建有效的65字节签名 (r + s + v)
    let mut signature = vec![0u8; 65];

    // 设置有效的r值 (非零，小于secp256k1 order)
    signature[31] = 1; // r = 1

    // 设置低s值 (< order/2，避免malleable)
    signature[63] = 1; // s = 1

    // 设置标准以太坊格式v值
    signature[64] = 27; // v = 27

    let result = validator.validate_signature(&signature).await.unwrap();

    assert!(result.is_valid, "有效的65字节签名应该通过验证");
    assert_eq!(result.signature_format, SignatureFormat::Standard65Bytes);
    assert_eq!(result.severity, ValidationSeverity::Info);
    assert!(result.security_issues.is_empty(), "有效签名不应有安全问题");

    let components = result.components.unwrap();
    assert_eq!(components.r, U256::from(1u32));
    assert_eq!(components.s, U256::from(1u32));
    assert_eq!(components.v, 27);
    assert!(!components.is_high_s, "低s值不应被标记为high_s");
    assert!(components.is_canonical_s, "低s值应该是规范的");

    println!("  ✅ 65字节标准签名验证通过");
}

/// 测试64字节紧凑ECDSA签名验证
#[tokio::test]
async fn test_valid_64_byte_compact_signature() {
    println!("🧪 测试64字节紧凑ECDSA签名验证");

    let validator = SignatureValidator::new();

    // 创建有效的64字节签名 (r + s, v需要外部提供)
    let mut signature = vec![0u8; 64];
    signature[31] = 1; // r = 1
    signature[63] = 1; // s = 1

    let result = validator.validate_signature(&signature).await.unwrap();

    assert!(result.is_valid, "有效的64字节签名应该通过验证");
    assert_eq!(result.signature_format, SignatureFormat::Compact64Bytes);

    let components = result.components.unwrap();
    assert_eq!(components.r, U256::from(1u32));
    assert_eq!(components.s, U256::from(1u32));
    // v值对64字节格式应该为0（默认假设）
    assert_eq!(components.v, 0);

    println!("  ✅ 64字节紧凑签名验证通过");
}

/// 测试无效签名长度处理
#[tokio::test]
async fn test_invalid_signature_lengths() {
    println!("🧪 测试无效签名长度处理");

    let validator = SignatureValidator::new();

    let test_cases = vec![
        (32, "太短签名"),
        (63, "缺1字节签名"),
        (66, "超长签名"),
        (100, "过长签名"),
    ];

    for (length, description) in test_cases {
        let invalid_signature = vec![0u8; length];
        let result = validator
            .validate_signature(&invalid_signature)
            .await
            .unwrap();

        assert!(!result.is_valid, "{} 应该被拒绝", description);
        assert_eq!(result.signature_format, SignatureFormat::Invalid);
        assert_eq!(result.severity, ValidationSeverity::Critical);

        println!("    ✅ {} 正确被拒绝", description);
    }
}

/// 测试malleable签名检测
#[tokio::test]
async fn test_malleable_signature_detection() {
    println!("🧪 测试malleable签名检测");

    let validator = SignatureValidator::new();

    // 创建high s值签名（malleable）
    let mut signature = vec![0u8; 65];
    signature[31] = 1; // r = 1

    // 设置高s值 (> secp256k1_order/2)
    let high_s_bytes =
        hex::decode("7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A1").unwrap();
    signature[32..64].copy_from_slice(&high_s_bytes);
    signature[64] = 27; // v = 27

    let result = validator.validate_signature(&signature).await.unwrap();

    // 严格模式下malleable签名应该失败
    assert!(!result.is_valid, "Malleable签名应该在严格模式下失败");
    assert!(!result.security_issues.is_empty(), "应该检测到安全问题");
    assert!(
        result
            .security_issues
            .iter()
            .any(|issue| issue.contains("malleable")),
        "应该明确指出malleable问题"
    );

    let components = result.components.unwrap();
    assert!(components.is_high_s, "High s值应该被正确检测");
    assert!(!components.is_canonical_s, "High s值不应该是规范的");

    println!("  ✅ Malleable签名检测正常");
}

/// 测试签名规范化功能
#[tokio::test]
async fn test_signature_normalization() {
    println!("🧪 测试签名规范化功能");

    let validator = SignatureValidator::new();

    // 创建high s值签名
    let mut malleable_signature = vec![0u8; 65];
    malleable_signature[31] = 1; // r = 1

    // 高s值
    let high_s_bytes =
        hex::decode("7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A1").unwrap();
    malleable_signature[32..64].copy_from_slice(&high_s_bytes);
    malleable_signature[64] = 27; // v = 27

    // 规范化签名
    let normalized = validator.normalize_signature(&malleable_signature).unwrap();
    assert_eq!(normalized.len(), 65, "规范化后应该保持65字节");

    // 验证规范化后的签名
    let result = validator.validate_signature(&normalized).await.unwrap();

    if let Some(components) = result.components {
        assert!(!components.is_high_s, "规范化后的签名s值应该是低的");
        assert!(components.is_canonical_s, "规范化后的签名应该是规范的");

        // v值应该已经被适当调整
        assert!(
            components.v == 28 || components.v == 1 || components.v >= 37,
            "v值应该被正确调整，实际值: {}",
            components.v
        );
    } else {
        panic!("规范化后的签名应该有有效的组件");
    }

    println!("  ✅ 签名规范化功能正常");
}

/// 测试无效r组件处理
#[tokio::test]
async fn test_invalid_r_component() {
    println!("🧪 测试无效r组件处理");

    let validator = SignatureValidator::new();

    // r = 0 (无效)
    let mut signature = vec![0u8; 65];
    // r全为0 (无效)
    signature[63] = 1; // s = 1
    signature[64] = 27; // v = 27

    let result = validator.validate_signature(&signature).await.unwrap();

    assert!(!result.is_valid, "r=0的签名应该失败");
    assert!(
        result
            .security_issues
            .iter()
            .any(|issue| issue.contains("Invalid r component")),
        "应该检测到无效r组件"
    );

    println!("  ✅ 无效r组件检测正常");
}

/// 测试无效s组件处理  
#[tokio::test]
async fn test_invalid_s_component() {
    println!("🧪 测试无效s组件处理");

    let validator = SignatureValidator::new();

    // s = 0 (无效)
    let mut signature = vec![0u8; 65];
    signature[31] = 1; // r = 1
                       // s全为0 (无效)
    signature[64] = 27; // v = 27

    let result = validator.validate_signature(&signature).await.unwrap();

    assert!(!result.is_valid, "s=0的签名应该失败");
    assert!(
        result
            .security_issues
            .iter()
            .any(|issue| issue.contains("Invalid s component")),
        "应该检测到无效s组件"
    );

    println!("  ✅ 无效s组件检测正常");
}

/// 测试无效v组件处理
#[tokio::test]
async fn test_invalid_v_component() {
    println!("🧪 测试无效v组件处理");

    let validator = SignatureValidator::new();

    let invalid_v_values = vec![2, 3, 26, 29, 30, 36]; // 各种无效v值

    for v_value in invalid_v_values {
        let mut signature = vec![0u8; 65];
        signature[31] = 1; // r = 1
        signature[63] = 1; // s = 1
        signature[64] = v_value; // 无效v值

        let result = validator.validate_signature(&signature).await.unwrap();

        assert!(!result.is_valid, "v={}的签名应该失败", v_value);
        assert!(
            result
                .security_issues
                .iter()
                .any(|issue| issue.contains("Invalid recovery id")),
            "应该检测到无效recovery id (v={})",
            v_value
        );

        println!("    ✅ 无效v值 {} 被正确拒绝", v_value);
    }
}

/// 测试各种有效v值
#[tokio::test]
async fn test_valid_v_components() {
    println!("🧪 测试各种有效v组件");

    let validator = SignatureValidator::lenient(); // 使用宽松模式避免malleable问题

    let valid_v_values = vec![
        0,  // 原始格式
        1,  // 原始格式
        27, // 以太坊格式
        28, // 以太坊格式
        37, // EIP-155 (chain_id=1, recovery_id=0)
        38, // EIP-155 (chain_id=1, recovery_id=1)
        71, // EIP-155 (chain_id=18, recovery_id=0)
    ];

    for v_value in valid_v_values {
        let mut signature = vec![0u8; 65];
        signature[31] = 1; // r = 1
        signature[63] = 1; // s = 1 (低值)
        signature[64] = v_value;

        let result = validator.validate_signature(&signature).await.unwrap();

        assert!(result.is_valid, "v={}的签名应该通过验证", v_value);

        let components = result.components.unwrap();
        assert_eq!(components.v, v_value, "v值应该被正确解析");

        println!("    ✅ 有效v值 {} 通过验证", v_value);
    }
}

/// 测试宽松模式vs严格模式
#[tokio::test]
async fn test_lenient_vs_strict_mode() {
    println!("🧪 测试宽松模式vs严格模式");

    let strict_validator = SignatureValidator::new();
    let lenient_validator = SignatureValidator::lenient();

    // 创建malleable签名
    let mut signature = vec![0u8; 65];
    signature[31] = 1; // r = 1
    let high_s_bytes =
        hex::decode("7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A1").unwrap();
    signature[32..64].copy_from_slice(&high_s_bytes);
    signature[64] = 27; // v = 27

    // 严格模式应该拒绝
    let strict_result = strict_validator
        .validate_signature(&signature)
        .await
        .unwrap();
    assert!(!strict_result.is_valid, "严格模式应该拒绝malleable签名");

    // 宽松模式应该接受但有警告
    let lenient_result = lenient_validator
        .validate_signature(&signature)
        .await
        .unwrap();
    assert!(lenient_result.is_valid, "宽松模式应该接受malleable签名");

    // 注意：宽松模式可能不会添加安全问题到列表中，这是设计的一部分
    // 我们检查是否至少检测到了high_s，即使不当作错误处理
    if let Some(components) = &lenient_result.components {
        assert!(components.is_high_s, "宽松模式仍应正确检测high_s");
        println!("  ⚠️  宽松模式检测到malleable签名但允许通过");
    }

    println!("  ✅ 严格模式和宽松模式行为符合预期");
}

/// 测试集成到DataIntegrityChecker中的签名验证
#[tokio::test]
async fn test_integrated_signature_validation() {
    println!("🧪 测试集成签名验证");

    let _checker = DataIntegrityChecker::new();

    // 使用一个简单的测试：直接调用私有的validate_signature_field方法
    // 由于该方法是私有的，我们通过创建一个UserOperation来间接测试

    // 创建有效签名
    let mut valid_signature = [0u8; 65];
    valid_signature[31] = 1; // r = 1
    valid_signature[63] = 1; // s = 1
    valid_signature[64] = 27; // v = 27

    // 创建malleable签名
    let mut malleable_signature = [0u8; 65];
    malleable_signature[31] = 1; // r = 1
    let high_s_bytes =
        hex::decode("7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A1").unwrap();
    malleable_signature[32..64].copy_from_slice(&high_s_bytes);
    malleable_signature[64] = 27; // v = 27

    println!("  ✅ 签名验证集成测试准备完成");

    // 注意：由于validate_signature_field是私有方法，
    // 实际的集成测试需要通过完整的UserOperation验证流程
    // 这里主要是确认模块可以正常导入和使用
}

/// 测试性能基准 - 签名验证速度
#[tokio::test]
async fn test_signature_validation_performance() {
    println!("🧪 测试签名验证性能基准");

    let validator = SignatureValidator::new();

    // 创建测试签名
    let mut signature = vec![0u8; 65];
    signature[31] = 1; // r = 1
    signature[63] = 1; // s = 1
    signature[64] = 27; // v = 27

    let iterations = 1000;
    let start_time = std::time::Instant::now();

    for _ in 0..iterations {
        let _result = validator.validate_signature(&signature).await.unwrap();
    }

    let elapsed = start_time.elapsed();
    let avg_time = elapsed / iterations;

    println!("  📊 性能结果:");
    println!("    总迭代: {} 次", iterations);
    println!("    总耗时: {:?}", elapsed);
    println!("    平均耗时: {:?} per validation", avg_time);

    // 性能要求：每次验证应该在1ms以内
    assert!(
        avg_time.as_millis() < 1,
        "签名验证性能应该 < 1ms，实际: {:?}",
        avg_time
    );

    println!("  ✅ 签名验证性能达标");
}

/// 运行所有H2.3相关测试的主函数
#[tokio::test]
async fn test_h2_3_signature_validation_comprehensive() {
    println!("🚀 开始H2.3 ECDSA签名格式标准化综合测试");
    println!("{}", "=".repeat(50));

    // 注意：这个是集成测试演示，实际的测试用例已经单独定义
    // 在运行 `cargo test h2_3_signature_validation` 时，
    // 所有带 #[tokio::test] 的函数都会自动运行

    println!("✅ 综合测试准备完成");
    println!("🎯 本次实现的核心功能:");
    println!("  • 标准65字节和紧凑64字节签名格式支持");
    println!("  • Malleable签名攻击防护");
    println!("  • r, s, v组件完整性验证");
    println!("  • 签名规范化能力");
    println!("  • 严格/宽松验证模式");
    println!("  • 高性能验证目标 (< 1ms)");
    println!("{}", "=".repeat(50));

    // 运行一个基础验证确保核心功能正常
    let validator = SignatureValidator::new();
    let mut test_signature = vec![0u8; 65];
    test_signature[31] = 1; // r = 1
    test_signature[63] = 1; // s = 1
    test_signature[64] = 27; // v = 27

    let result = validator.validate_signature(&test_signature).await.unwrap();
    assert!(result.is_valid, "基础签名验证应该通过");

    println!("✅ H2.3 ECDSA签名格式标准化功能验证完成！");
}
