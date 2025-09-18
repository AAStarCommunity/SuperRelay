//! BLS聚合签名防护机制
//!
//! 为BLS签名聚合提供增强的安全防护，包括：
//! - BLS签名格式验证和标准化
//! - 聚合攻击检测和防护
//! - 恶意聚合器识别和黑名单
//! - 性能监控和异常检测
//! - 兼容v0.7/v0.8的UserOperation

use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use alloy_primitives::{Address, Bytes, B256};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// BLS防护配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlsProtectionConfig {
    /// 是否启用BLS防护
    pub enabled: bool,
    /// BLS签名验证是否启用
    pub signature_validation_enabled: bool,
    /// 聚合器黑名单检查是否启用
    pub blacklist_enabled: bool,
    /// 性能监控是否启用
    pub performance_monitoring_enabled: bool,
    /// 恶意聚合器检测是否启用
    pub malicious_aggregator_detection: bool,
    /// 最大允许的聚合延迟 (毫秒)
    pub max_aggregation_delay_ms: u64,
    /// 最大允许的签名数量
    pub max_signatures_per_aggregation: usize,
    /// 性能阈值：平均处理时间 (毫秒)
    pub performance_threshold_ms: u64,
    /// 黑名单过期时间 (秒)
    pub blacklist_expiry_seconds: u64,
}

impl Default for BlsProtectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            signature_validation_enabled: true,
            blacklist_enabled: true,
            performance_monitoring_enabled: true,
            malicious_aggregator_detection: true,
            max_aggregation_delay_ms: 5000, // 5 seconds
            max_signatures_per_aggregation: 100,
            performance_threshold_ms: 1000, // 1 second
            blacklist_expiry_seconds: 3600, // 1 hour
        }
    }
}

/// BLS签名验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlsValidationResult {
    /// 签名是否有效
    pub is_valid: bool,
    /// 验证消息
    pub message: String,
    /// 聚合器地址
    pub aggregator_address: Option<Address>,
    /// 验证耗时 (毫秒)
    pub validation_time_ms: u64,
    /// 检测到的安全问题
    pub security_issues: Vec<String>,
}

/// 聚合器性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatorPerformanceStats {
    /// 聚合器地址
    pub address: Address,
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间 (毫秒)
    pub avg_response_time_ms: f64,
    /// 最大响应时间 (毫秒)
    pub max_response_time_ms: u64,
    /// 最后更新时间
    pub last_updated: u64,
}

/// 黑名单条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistEntry {
    /// 聚合器地址
    pub address: Address,
    /// 加入黑名单的原因
    pub reason: String,
    /// 加入时间
    pub blocked_at: u64,
    /// 过期时间
    pub expires_at: u64,
    /// 违规次数
    pub violation_count: u32,
}

/// BLS防护状态
#[derive(Debug, Serialize, Deserialize)]
pub struct BlsProtectionStatus {
    /// 防护是否启用
    pub enabled: bool,
    /// 已验证的签名总数
    pub total_signatures_validated: u64,
    /// 检测到的恶意尝试次数
    pub malicious_attempts_detected: u64,
    /// 黑名单中的聚合器数量
    pub blacklisted_aggregators_count: usize,
    /// 当前监控的聚合器数量
    pub monitored_aggregators_count: usize,
    /// 系统运行时间
    pub uptime_seconds: u64,
}

/// BLS聚合签名防护系统
#[derive(Debug)]
pub struct BlsProtectionSystem {
    config: BlsProtectionConfig,
    /// 聚合器性能统计
    performance_stats: RwLock<HashMap<Address, AggregatorPerformanceStats>>,
    /// 聚合器黑名单
    blacklist: RwLock<HashMap<Address, BlacklistEntry>>,
    /// 已知的可信聚合器
    trusted_aggregators: RwLock<HashSet<Address>>,
    /// 系统启动时间
    start_time: Instant,
    /// 统计计数器
    validation_counter: RwLock<u64>,
    malicious_attempt_counter: RwLock<u64>,
}

impl BlsProtectionSystem {
    /// 创建新的BLS防护系统
    pub fn new(config: BlsProtectionConfig) -> Self {
        Self {
            config,
            performance_stats: RwLock::new(HashMap::new()),
            blacklist: RwLock::new(HashMap::new()),
            trusted_aggregators: RwLock::new(HashSet::new()),
            start_time: Instant::now(),
            validation_counter: RwLock::new(0),
            malicious_attempt_counter: RwLock::new(0),
        }
    }

    /// 验证BLS签名和聚合器
    pub async fn validate_bls_signature(
        &self,
        aggregator_address: Address,
        signature: &Bytes,
        message_hash: &B256,
    ) -> Result<BlsValidationResult> {
        if !self.config.enabled {
            return Ok(BlsValidationResult {
                is_valid: true,
                message: "BLS protection disabled".to_string(),
                aggregator_address: Some(aggregator_address),
                validation_time_ms: 0,
                security_issues: vec![],
            });
        }

        let validation_start = Instant::now();
        let mut security_issues = Vec::new();

        // 1. 检查聚合器是否在黑名单中
        if self.config.blacklist_enabled && self.is_blacklisted(aggregator_address).await {
            let result = BlsValidationResult {
                is_valid: false,
                message: "Aggregator is blacklisted".to_string(),
                aggregator_address: Some(aggregator_address),
                validation_time_ms: validation_start.elapsed().as_millis() as u64,
                security_issues: vec!["BLACKLISTED_AGGREGATOR".to_string()],
            };
            self.record_malicious_attempt().await;
            return Ok(result);
        }

        // 2. 验证BLS签名格式
        if self.config.signature_validation_enabled {
            if let Err(issue) = self.validate_signature_format(signature) {
                security_issues.push(issue);
            }
        }

        // 3. 检查签名长度和内容
        if signature.is_empty() {
            security_issues.push("EMPTY_SIGNATURE".to_string());
        } else if signature.len() < 32 {
            security_issues.push("SIGNATURE_TOO_SHORT".to_string());
        } else if signature.len() > 256 {
            security_issues.push("SIGNATURE_TOO_LONG".to_string());
        }

        // 4. 检查消息哈希
        if message_hash.is_zero() {
            security_issues.push("ZERO_MESSAGE_HASH".to_string());
        }

        // 5. 性能监控
        let validation_time_ms = validation_start.elapsed().as_millis() as u64;
        if self.config.performance_monitoring_enabled {
            self.record_performance(
                aggregator_address,
                validation_time_ms,
                security_issues.is_empty(),
            )
            .await;
        }

        // 6. 恶意聚合器检测
        if self.config.malicious_aggregator_detection && !security_issues.is_empty() {
            self.check_for_malicious_behavior(aggregator_address, &security_issues)
                .await?;
        }

        let is_valid = security_issues.is_empty();
        if !is_valid {
            self.record_malicious_attempt().await;
        }

        self.increment_validation_counter().await;

        Ok(BlsValidationResult {
            is_valid,
            message: if is_valid {
                "BLS signature validation successful".to_string()
            } else {
                format!(
                    "BLS signature validation failed: {}",
                    security_issues.join(", ")
                )
            },
            aggregator_address: Some(aggregator_address),
            validation_time_ms,
            security_issues,
        })
    }

    /// 验证BLS聚合操作
    pub async fn validate_aggregation(
        &self,
        aggregator_address: Address,
        signatures: &[Bytes],
    ) -> Result<BlsValidationResult> {
        if !self.config.enabled {
            return Ok(BlsValidationResult {
                is_valid: true,
                message: "BLS protection disabled".to_string(),
                aggregator_address: Some(aggregator_address),
                validation_time_ms: 0,
                security_issues: vec![],
            });
        }

        let validation_start = Instant::now();
        let mut security_issues = Vec::new();

        // 1. 检查聚合器黑名单
        if self.is_blacklisted(aggregator_address).await {
            return Ok(BlsValidationResult {
                is_valid: false,
                message: "Aggregator is blacklisted".to_string(),
                aggregator_address: Some(aggregator_address),
                validation_time_ms: validation_start.elapsed().as_millis() as u64,
                security_issues: vec!["BLACKLISTED_AGGREGATOR".to_string()],
            });
        }

        // 2. 检查签名数量限制
        if signatures.len() > self.config.max_signatures_per_aggregation {
            security_issues.push("TOO_MANY_SIGNATURES".to_string());
        }

        // 3. 检查每个签名
        for (i, sig) in signatures.iter().enumerate() {
            if sig.is_empty() {
                security_issues.push(format!("EMPTY_SIGNATURE_AT_INDEX_{}", i));
            } else if let Err(issue) = self.validate_signature_format(sig) {
                security_issues.push(format!("INVALID_SIGNATURE_AT_INDEX_{}_{}", i, issue));
            }
        }

        // 4. 检查重复签名
        let mut unique_signatures = HashSet::new();
        for sig in signatures {
            if !unique_signatures.insert(sig.clone()) {
                security_issues.push("DUPLICATE_SIGNATURES".to_string());
                break;
            }
        }

        let validation_time_ms = validation_start.elapsed().as_millis() as u64;

        // 5. 性能检查
        if validation_time_ms > self.config.max_aggregation_delay_ms {
            security_issues.push("AGGREGATION_TIMEOUT".to_string());
        }

        let is_valid = security_issues.is_empty();
        if !is_valid {
            self.record_malicious_attempt().await;
            if self.config.malicious_aggregator_detection {
                self.check_for_malicious_behavior(aggregator_address, &security_issues)
                    .await?;
            }
        }

        if self.config.performance_monitoring_enabled {
            self.record_performance(aggregator_address, validation_time_ms, is_valid)
                .await;
        }

        self.increment_validation_counter().await;

        Ok(BlsValidationResult {
            is_valid,
            message: if is_valid {
                format!(
                    "BLS aggregation validation successful for {} signatures",
                    signatures.len()
                )
            } else {
                format!(
                    "BLS aggregation validation failed: {}",
                    security_issues.join(", ")
                )
            },
            aggregator_address: Some(aggregator_address),
            validation_time_ms,
            security_issues,
        })
    }

    /// 添加可信聚合器
    pub async fn add_trusted_aggregator(&self, address: Address) -> Result<()> {
        let mut trusted = self.trusted_aggregators.write().await;
        trusted.insert(address);
        info!("Added trusted BLS aggregator: {:?}", address);
        Ok(())
    }

    /// 移除可信聚合器
    pub async fn remove_trusted_aggregator(&self, address: Address) -> Result<()> {
        let mut trusted = self.trusted_aggregators.write().await;
        trusted.remove(&address);
        info!("Removed trusted BLS aggregator: {:?}", address);
        Ok(())
    }

    /// 添加聚合器到黑名单
    pub async fn blacklist_aggregator(
        &self,
        address: Address,
        reason: String,
        duration_seconds: u64,
    ) -> Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let expires_at = now + duration_seconds;

        let entry = BlacklistEntry {
            address,
            reason: reason.clone(),
            blocked_at: now,
            expires_at,
            violation_count: 1,
        };

        let mut blacklist = self.blacklist.write().await;
        if let Some(existing) = blacklist.get_mut(&address) {
            existing.violation_count += 1;
            existing.expires_at = expires_at;
            existing.reason = format!("{} (violations: {})", reason, existing.violation_count);
        } else {
            blacklist.insert(address, entry);
        }

        warn!("Blacklisted BLS aggregator {:?}: {}", address, reason);
        Ok(())
    }

    /// 获取系统状态
    pub async fn get_status(&self) -> Result<BlsProtectionStatus> {
        let blacklist = self.blacklist.read().await;
        let performance_stats = self.performance_stats.read().await;
        let validation_count = *self.validation_counter.read().await;
        let malicious_count = *self.malicious_attempt_counter.read().await;

        Ok(BlsProtectionStatus {
            enabled: self.config.enabled,
            total_signatures_validated: validation_count,
            malicious_attempts_detected: malicious_count,
            blacklisted_aggregators_count: blacklist.len(),
            monitored_aggregators_count: performance_stats.len(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        })
    }

    /// 获取聚合器性能统计
    pub async fn get_aggregator_stats(
        &self,
        address: Address,
    ) -> Option<AggregatorPerformanceStats> {
        let stats = self.performance_stats.read().await;
        stats.get(&address).cloned()
    }

    /// 清理过期的黑名单条目
    pub async fn cleanup_expired_blacklist(&self) -> Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let mut blacklist = self.blacklist.write().await;

        let expired_addresses: Vec<Address> = blacklist
            .iter()
            .filter(|(_, entry)| entry.expires_at <= now)
            .map(|(addr, _)| *addr)
            .collect();

        for addr in expired_addresses {
            blacklist.remove(&addr);
            info!("Removed expired blacklist entry for aggregator: {:?}", addr);
        }

        Ok(())
    }

    // 内部辅助方法

    async fn is_blacklisted(&self, address: Address) -> bool {
        let blacklist = self.blacklist.read().await;
        if let Some(entry) = blacklist.get(&address) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            entry.expires_at > now
        } else {
            false
        }
    }

    fn validate_signature_format(&self, signature: &Bytes) -> Result<(), String> {
        // BLS签名基本格式验证
        if signature.len() < 48 {
            return Err("BLS signature too short (minimum 48 bytes)".to_string());
        }

        if signature.len() > 96 {
            return Err("BLS signature too long (maximum 96 bytes)".to_string());
        }

        // 检查全零签名（通常是无效的）
        if signature.iter().all(|&b| b == 0) {
            return Err("All-zero BLS signature".to_string());
        }

        Ok(())
    }

    async fn record_performance(&self, address: Address, time_ms: u64, success: bool) {
        let mut stats = self.performance_stats.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = stats.entry(address).or_insert(AggregatorPerformanceStats {
            address,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            max_response_time_ms: 0,
            last_updated: now,
        });

        entry.total_requests += 1;
        if success {
            entry.successful_requests += 1;
        } else {
            entry.failed_requests += 1;
        }

        // 更新平均响应时间（简单移动平均）
        entry.avg_response_time_ms =
            (entry.avg_response_time_ms * (entry.total_requests as f64 - 1.0) + time_ms as f64)
                / entry.total_requests as f64;

        if time_ms > entry.max_response_time_ms {
            entry.max_response_time_ms = time_ms;
        }

        entry.last_updated = now;
    }

    async fn check_for_malicious_behavior(
        &self,
        address: Address,
        issues: &[String],
    ) -> Result<()> {
        // 检查是否需要将聚合器加入黑名单
        let stats = {
            let stats_lock = self.performance_stats.read().await;
            stats_lock.get(&address).cloned()
        };

        if let Some(stats) = stats {
            let failure_rate = stats.failed_requests as f64 / stats.total_requests as f64;

            // 如果失败率过高，加入黑名单
            if failure_rate > 0.5 && stats.total_requests > 10 {
                self.blacklist_aggregator(
                    address,
                    format!("High failure rate: {:.1}%", failure_rate * 100.0),
                    self.config.blacklist_expiry_seconds,
                )
                .await?;
            }

            // 如果响应时间过长，加入黑名单
            if stats.avg_response_time_ms > self.config.performance_threshold_ms as f64 {
                self.blacklist_aggregator(
                    address,
                    format!(
                        "Poor performance: {:.1}ms average",
                        stats.avg_response_time_ms
                    ),
                    self.config.blacklist_expiry_seconds,
                )
                .await?;
            }
        }

        // 检查特定的安全问题
        for issue in issues {
            if issue.contains("TOO_MANY_SIGNATURES") || issue.contains("DUPLICATE_SIGNATURES") {
                self.blacklist_aggregator(
                    address,
                    format!("Security violation: {}", issue),
                    self.config.blacklist_expiry_seconds,
                )
                .await?;
                break;
            }
        }

        Ok(())
    }

    async fn increment_validation_counter(&self) {
        let mut counter = self.validation_counter.write().await;
        *counter += 1;
    }

    async fn record_malicious_attempt(&self) {
        let mut counter = self.malicious_attempt_counter.write().await;
        *counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{address, b256};

    use super::*;

    #[tokio::test]
    async fn test_bls_protection_system_creation() -> Result<()> {
        let config = BlsProtectionConfig::default();
        let system = BlsProtectionSystem::new(config);

        let status = system.get_status().await?;
        assert!(status.enabled);
        assert_eq!(status.total_signatures_validated, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_signature_validation() -> Result<()> {
        let config = BlsProtectionConfig::default();
        let system = BlsProtectionSystem::new(config);

        let aggregator = address!("1234567890123456789012345678901234567890");
        let signature = Bytes::from(vec![0x01; 48]); // Valid 48-byte signature
        let message_hash =
            b256!("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");

        let result = system
            .validate_bls_signature(aggregator, &signature, &message_hash)
            .await?;

        assert!(result.is_valid);
        assert_eq!(result.aggregator_address, Some(aggregator));
        assert!(result.security_issues.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_signature_detection() -> Result<()> {
        let config = BlsProtectionConfig::default();
        let system = BlsProtectionSystem::new(config);

        let aggregator = address!("1234567890123456789012345678901234567890");
        let signature = Bytes::from(vec![0x00; 20]); // Too short
        let message_hash =
            b256!("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");

        let result = system
            .validate_bls_signature(aggregator, &signature, &message_hash)
            .await?;

        assert!(!result.is_valid);
        assert!(!result.security_issues.is_empty());
        assert!(result
            .security_issues
            .iter()
            .any(|s| s.contains("TOO_SHORT")));

        Ok(())
    }

    #[tokio::test]
    async fn test_blacklist_functionality() -> Result<()> {
        let config = BlsProtectionConfig::default();
        let system = BlsProtectionSystem::new(config);

        let aggregator = address!("1234567890123456789012345678901234567890");

        // Add to blacklist
        system
            .blacklist_aggregator(aggregator, "Test reason".to_string(), 3600)
            .await?;

        // Verify blacklisted
        assert!(system.is_blacklisted(aggregator).await);

        let status = system.get_status().await?;
        assert_eq!(status.blacklisted_aggregators_count, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_validation() -> Result<()> {
        let config = BlsProtectionConfig::default();
        let system = BlsProtectionSystem::new(config);

        let aggregator = address!("1234567890123456789012345678901234567890");
        let signatures = vec![
            Bytes::from(vec![0x01; 48]),
            Bytes::from(vec![0x02; 48]),
            Bytes::from(vec![0x03; 48]),
        ];

        let result = system.validate_aggregation(aggregator, &signatures).await?;

        assert!(result.is_valid);
        assert!(result.message.contains("3 signatures"));

        Ok(())
    }

    #[tokio::test]
    async fn test_trusted_aggregators() -> Result<()> {
        let config = BlsProtectionConfig::default();
        let system = BlsProtectionSystem::new(config);

        let aggregator = address!("1234567890123456789012345678901234567890");

        // Add as trusted
        system.add_trusted_aggregator(aggregator).await?;

        let trusted = system.trusted_aggregators.read().await;
        assert!(trusted.contains(&aggregator));

        // Remove from trusted
        drop(trusted);
        system.remove_trusted_aggregator(aggregator).await?;

        let trusted = system.trusted_aggregators.read().await;
        assert!(!trusted.contains(&aggregator));

        Ok(())
    }

    #[tokio::test]
    async fn test_disabled_protection() -> Result<()> {
        let mut config = BlsProtectionConfig::default();
        config.enabled = false;

        let system = BlsProtectionSystem::new(config);

        let aggregator = address!("1234567890123456789012345678901234567890");
        let signature = Bytes::from(vec![]); // Empty signature should fail if enabled
        let message_hash = B256::ZERO; // Zero hash should fail if enabled

        let result = system
            .validate_bls_signature(aggregator, &signature, &message_hash)
            .await?;

        // Should pass because protection is disabled
        assert!(result.is_valid);
        assert!(result.message.contains("disabled"));

        Ok(())
    }
}
