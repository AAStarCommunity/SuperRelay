//! ECDSA签名标准化验证模块
//!
//! 实现标准化的ECDSA签名格式验证，包括：
//! - 签名格式标准化（65字节 r+s+v 或 64字节 r+s）
//! - 防止malleable签名攻击
//! - r, s 组件范围验证
//! - recovery id (v) 有效性检查
//! - 兼容EIP-155扩展v值

use alloy_primitives::U256;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::error::GatewayResult;
use crate::validation::ValidationSeverity;

/// ECDSA签名验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureValidationResult {
    /// 签名是否有效
    pub is_valid: bool,
    /// 签名格式
    pub signature_format: SignatureFormat,
    /// 验证消息
    pub message: String,
    /// 验证严重程度
    pub severity: ValidationSeverity,
    /// 签名组件（如果解析成功）
    pub components: Option<SignatureComponents>,
    /// 发现的安全问题
    pub security_issues: Vec<String>,
}

/// 签名格式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureFormat {
    /// 标准65字节格式 (r + s + v)
    Standard65Bytes,
    /// 压缩64字节格式 (r + s, v在其他地方提供)
    Compact64Bytes,
    /// EIP-2098紧凑签名格式
    Eip2098Compact,
    /// 无效格式
    Invalid,
}

/// ECDSA签名组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureComponents {
    /// r 组件 (32字节)
    pub r: U256,
    /// s 组件 (32字节)
    pub s: U256,
    /// v 组件 (recovery id)
    pub v: u8,
    /// 是否为高s值（存在malleable攻击风险）
    pub is_high_s: bool,
    /// 是否为规范化s值
    pub is_canonical_s: bool,
}

/// ECDSA签名验证器
pub struct SignatureValidator {
    /// 是否严格要求标准格式
    strict_format: bool,
    /// 是否允许malleable签名
    allow_malleable: bool,
    /// 是否要求压缩s值
    require_low_s: bool,
}

impl Default for SignatureValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl SignatureValidator {
    /// 创建默认签名验证器
    pub fn new() -> Self {
        Self {
            strict_format: true,
            allow_malleable: false,
            require_low_s: true,
        }
    }

    /// 创建宽松模式签名验证器
    pub fn lenient() -> Self {
        Self {
            strict_format: false,
            allow_malleable: true,
            require_low_s: false,
        }
    }

    /// 验证ECDSA签名格式和安全性
    pub async fn validate_signature(&self, signature: &[u8]) -> GatewayResult<SignatureValidationResult> {
        debug!("🔐 Validating ECDSA signature format (length: {})", signature.len());

        let mut security_issues = Vec::new();
        
        // 1. 基础长度检查
        let signature_format = self.determine_signature_format(signature);
        if signature_format == SignatureFormat::Invalid {
            return Ok(SignatureValidationResult {
                is_valid: false,
                signature_format,
                message: format!("Invalid signature length: {} bytes (expected 64 or 65)", signature.len()),
                severity: ValidationSeverity::Critical,
                components: None,
                security_issues,
            });
        }

        // 2. 解析签名组件
        let components = match self.parse_signature_components(signature) {
            Ok(components) => components,
            Err(e) => {
                return Ok(SignatureValidationResult {
                    is_valid: false,
                    signature_format,
                    message: format!("Failed to parse signature components: {}", e),
                    severity: ValidationSeverity::Critical,
                    components: None,
                    security_issues,
                });
            }
        };

        // 3. 验证r组件
        if !self.is_valid_r_component(components.r) {
            security_issues.push("Invalid r component: must be in range [1, secp256k1_order)".to_string());
        }

        // 4. 验证s组件和malleable检查
        if !self.is_valid_s_component(components.s) {
            security_issues.push("Invalid s component: must be in range [1, secp256k1_order)".to_string());
        }

        if components.is_high_s {
            if !self.allow_malleable {
                security_issues.push("High s value detected: signature is malleable (vulnerability to signature replay attacks)".to_string());
            } else {
                warn!("⚠️ High s value in signature - malleable signature allowed by configuration");
            }
        }

        // 5. 验证v组件
        if !self.is_valid_v_component(components.v) {
            security_issues.push(format!("Invalid recovery id (v): {} (expected 27/28 or 0/1)", components.v));
        }

        // 6. 确定最终验证结果
        let has_critical_issues = !security_issues.is_empty() && self.strict_format;
        let is_valid = !has_critical_issues;
        
        let severity = if has_critical_issues {
            ValidationSeverity::Critical
        } else if !security_issues.is_empty() {
            ValidationSeverity::Warning
        } else {
            ValidationSeverity::Info
        };

        let message = if is_valid {
            if security_issues.is_empty() {
                "✅ Signature format is valid and secure".to_string()
            } else {
                format!("⚠️ Signature valid but has {} security warnings", security_issues.len())
            }
        } else {
            format!("❌ Signature validation failed: {} security issues", security_issues.len())
        };

        debug!("Signature validation completed: {}", message);

        Ok(SignatureValidationResult {
            is_valid,
            signature_format,
            message,
            severity,
            components: Some(components),
            security_issues,
        })
    }

    /// 确定签名格式
    fn determine_signature_format(&self, signature: &[u8]) -> SignatureFormat {
        match signature.len() {
            65 => SignatureFormat::Standard65Bytes,
            64 => {
                // 检查是否为EIP-2098格式（最后一个字节的最高位可能包含v信息）
                if signature.len() == 64 {
                    // 暂时假设为compact格式，实际可能需要更复杂的检测
                    SignatureFormat::Compact64Bytes
                } else {
                    SignatureFormat::Invalid
                }
            }
            _ => SignatureFormat::Invalid,
        }
    }

    /// 解析签名组件
    fn parse_signature_components(&self, signature: &[u8]) -> Result<SignatureComponents, String> {
        match signature.len() {
            65 => {
                // 标准格式: r(32) + s(32) + v(1)
                let r_bytes = &signature[0..32];
                let s_bytes = &signature[32..64];
                let v = signature[64];

                let r = U256::from_be_slice(r_bytes);
                let s = U256::from_be_slice(s_bytes);

                Ok(self.create_signature_components(r, s, v))
            }
            64 => {
                // 紧凑格式: r(32) + s(32), v需要从其他地方获取或假设
                let r_bytes = &signature[0..32];
                let s_bytes = &signature[32..64];

                let r = U256::from_be_slice(r_bytes);
                let s = U256::from_be_slice(s_bytes);

                // 对于64字节格式，我们无法确定v值，假设为0(这在实际使用时需要外部提供)
                warn!("⚠️ 64-byte signature detected - recovery id (v) unknown, assuming 0");
                Ok(self.create_signature_components(r, s, 0))
            }
            _ => Err(format!("Unsupported signature length: {}", signature.len())),
        }
    }

    /// 创建签名组件结构
    fn create_signature_components(&self, r: U256, s: U256, v: u8) -> SignatureComponents {
        let is_high_s = self.is_high_s_value(s);
        let is_canonical_s = !is_high_s;

        SignatureComponents {
            r,
            s,
            v,
            is_high_s,
            is_canonical_s,
        }
    }

    /// 验证r组件是否有效
    fn is_valid_r_component(&self, r: U256) -> bool {
        // r must be in range [1, secp256k1_order)
        // secp256k1 curve order: n = FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141  
        let secp256k1_order = U256::from_be_bytes([
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
            0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41
        ]);
        
        r != U256::ZERO && r < secp256k1_order
    }

    /// 验证s组件是否有效
    fn is_valid_s_component(&self, s: U256) -> bool {
        // s must be in range [1, secp256k1_order)
        // secp256k1 curve order: n = FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141  
        let secp256k1_order = U256::from_be_bytes([
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
            0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41
        ]);
        
        s != U256::ZERO && s < secp256k1_order
    }

    /// 检查s值是否为高值（malleable）
    fn is_high_s_value(&self, s: U256) -> bool {
        // s is "high" if s > secp256k1_order / 2
        // secp256k1 curve order: n = FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141  
        let secp256k1_order = U256::from_be_bytes([
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
            0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41
        ]);
        let half_order = secp256k1_order / U256::from(2u32);
        
        s > half_order
    }

    /// 验证v组件是否有效
    fn is_valid_v_component(&self, v: u8) -> bool {
        // 标准recovery id值
        match v {
            0 | 1 => true,         // 原始格式
            27 | 28 => true,       // 以太坊格式
            v if v >= 37 => {      // EIP-155格式 (chain_id * 2 + 35 + recovery_id)
                (v - 35) % 2 <= 1
            }
            _ => false,
        }
    }

    /// 规范化签名（如果可能的话）
    pub fn normalize_signature(&self, signature: &[u8]) -> GatewayResult<Vec<u8>> {
        if signature.len() != 65 {
            return Err(anyhow::anyhow!("Can only normalize 65-byte signatures").into());
        }

        let components = self.parse_signature_components(signature)
            .map_err(|e| anyhow::anyhow!("Failed to parse signature: {}", e))?;

        // 如果s值过高，将其规范化
        let normalized_s = if components.is_high_s && self.require_low_s {
            let secp256k1_order = U256::from_str_radix(
                "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 
                16
            ).unwrap();
            secp256k1_order - components.s
        } else {
            components.s
        };

        // 重新构建签名
        let mut normalized = Vec::with_capacity(65);
        normalized.extend_from_slice(&components.r.to_be_bytes::<32>());
        normalized.extend_from_slice(&normalized_s.to_be_bytes::<32>());
        
        // 如果s被规范化了，v值也需要调整
        let normalized_v = if components.is_high_s && self.require_low_s {
            match components.v {
                27 => 28,
                28 => 27,
                0 => 1,
                1 => 0,
                v if v >= 37 => {
                    let chain_id = (v - 35) / 2;
                    let recovery_id = (v - 35) % 2;
                    let flipped_recovery = 1 - recovery_id;
                    chain_id * 2 + 35 + flipped_recovery
                }
                v => v,
            }
        } else {
            components.v
        };

        normalized.push(normalized_v);

        debug!("🔧 Signature normalized: high_s={}, v_changed={}", 
               components.is_high_s, normalized_v != components.v);

        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid_65_byte_signature() {
        let validator = SignatureValidator::new();
        
        // 创建一个模拟的65字节签名 (r + s + v)
        let mut signature = vec![0u8; 65];
        // 设置有效的r值 (非零，小于secp256k1 order)
        signature[31] = 1; // r = 1
        // 设置有效的s值 (非零，小于order/2以避免malleable)
        signature[63] = 1; // s = 1
        signature[64] = 27; // v = 27 (标准以太坊格式)

        let result = validator.validate_signature(&signature).await.unwrap();
        assert!(result.is_valid);
        assert_eq!(result.signature_format, SignatureFormat::Standard65Bytes);
        assert!(result.components.is_some());
        
        let components = result.components.unwrap();
        assert_eq!(components.r, U256::from(1u32));
        assert_eq!(components.s, U256::from(1u32));
        assert_eq!(components.v, 27);
        assert!(!components.is_high_s);
    }

    #[tokio::test]
    async fn test_invalid_signature_length() {
        let validator = SignatureValidator::new();
        let invalid_signature = vec![0u8; 32]; // 太短

        let result = validator.validate_signature(&invalid_signature).await.unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.signature_format, SignatureFormat::Invalid);
        assert_eq!(result.severity, ValidationSeverity::Critical);
    }

    #[tokio::test]
    async fn test_malleable_signature_detection() {
        let validator = SignatureValidator::new();
        
        // 创建一个high s值的签名
        let mut signature = vec![0u8; 65];
        signature[31] = 1; // r = 1
        
        // 设置高s值 (> secp256k1_order/2)
        let high_s_bytes = hex::decode("7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A1").unwrap();
        signature[32..64].copy_from_slice(&high_s_bytes);
        signature[64] = 27; // v = 27

        let result = validator.validate_signature(&signature).await.unwrap();
        // 严格模式下应该失败
        assert!(!result.is_valid);
        assert!(!result.security_issues.is_empty());
        assert!(result.security_issues[0].contains("malleable"));
    }

    #[tokio::test]
    async fn test_signature_normalization() {
        let validator = SignatureValidator::new();
        
        // 创建一个high s值的签名
        let mut signature = vec![0u8; 65];
        signature[31] = 1; // r = 1
        
        // 设置高s值
        let high_s_bytes = hex::decode("7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A1").unwrap();
        signature[32..64].copy_from_slice(&high_s_bytes);
        signature[64] = 27; // v = 27

        let normalized = validator.normalize_signature(&signature).unwrap();
        assert_eq!(normalized.len(), 65);
        
        // 验证规范化后的签名
        let result = validator.validate_signature(&normalized).await.unwrap();
        if let Some(components) = result.components {
            assert!(!components.is_high_s, "Normalized signature should have low s");
        }
    }

    #[tokio::test]
    async fn test_invalid_v_component() {
        let validator = SignatureValidator::new();
        
        let mut signature = vec![0u8; 65];
        signature[31] = 1; // r = 1
        signature[63] = 1; // s = 1
        signature[64] = 26; // 无效的v值

        let result = validator.validate_signature(&signature).await.unwrap();
        assert!(!result.is_valid);
        assert!(result.security_issues.iter().any(|issue| issue.contains("Invalid recovery id")));
    }
}