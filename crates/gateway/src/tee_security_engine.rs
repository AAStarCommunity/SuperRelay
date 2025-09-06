use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use alloy_primitives::Address;
use anyhow::{anyhow, Result};
use rundler_types::{UserOperation, UserOperationVariant};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::{
    error::GatewayResult,
    security::{SecurityChecker, SecurityResult},
};

/// TEE Security Engine - Core security validation within TEE environment
#[derive(Debug)]
pub struct TeeSecurityEngine {
    /// Main security checker
    security_checker: SecurityChecker,
    /// TEE-specific configuration
    config: TeeSecurityConfig,
    /// Blacklist cache (updated from TEE secure storage)
    blacklist_cache: Arc<RwLock<BlacklistCache>>,
    /// Phishing detection database
    phishing_db: Arc<RwLock<PhishingDatabase>>,
    /// Anomaly detection system
    anomaly_detector: AnomalyDetector,
    /// Security metrics
    metrics: Arc<RwLock<SecurityMetrics>>,
}

/// TEE Security Engine configuration
#[derive(Debug, Clone)]
pub struct TeeSecurityConfig {
    /// Enable TEE hardware attestation
    pub enable_tee_attestation: bool,
    /// Enable real-time threat intelligence updates
    pub enable_realtime_updates: bool,
    /// Maximum processing time per security check (ms)
    pub max_processing_time_ms: u64,
    /// Blacklist cache refresh interval (seconds)
    pub blacklist_refresh_interval: u64,
    /// Phishing database update interval (seconds)  
    pub phishing_db_update_interval: u64,
    /// Enable advanced anomaly detection
    pub enable_advanced_anomaly_detection: bool,
    /// TEE secure storage path for configuration
    pub tee_secure_config_path: String,
    /// Risk tolerance level (0-100, higher = more restrictive)
    pub risk_tolerance_level: u8,
}

impl Default for TeeSecurityConfig {
    fn default() -> Self {
        Self {
            enable_tee_attestation: true,
            enable_realtime_updates: false,    // Disabled for Phase 1
            max_processing_time_ms: 5000,      // 5 second timeout
            blacklist_refresh_interval: 3600,  // 1 hour
            phishing_db_update_interval: 1800, // 30 minutes
            enable_advanced_anomaly_detection: true,
            tee_secure_config_path: "/secure/tee/security_config.json".to_string(),
            risk_tolerance_level: 75, // Conservative by default
        }
    }
}

/// Blacklist cache for known malicious addresses and patterns
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BlacklistCache {
    /// Known malicious addresses
    addresses: HashMap<Address, MaliciousAddressInfo>,
    /// Known malicious contract patterns
    contract_patterns: Vec<MaliciousPattern>,
    /// Last update timestamp
    last_updated: SystemTime,
    /// Cache version for integrity checking
    cache_version: u64,
}

/// Information about a malicious address
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaliciousAddressInfo {
    /// Address
    pub address: Address,
    /// Threat type
    pub threat_type: ThreatType,
    /// Risk score (0-100)
    pub risk_score: u8,
    /// Source of intelligence
    pub source: String,
    /// First detected timestamp
    pub first_detected: i64,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Types of threats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatType {
    /// Known scammer address
    Scammer,
    /// Phishing contract
    PhishingContract,
    /// MEV exploit contract
    MevExploit,
    /// Rugpull project
    Rugpull,
    /// Sandwich attack bot
    SandwichBot,
    /// Malicious paymaster
    MaliciousPaymaster,
    /// Other threat type
    Other(String),
}

/// Malicious pattern for contract detection
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MaliciousPattern {
    /// Pattern name
    name: String,
    /// Pattern type
    pattern_type: PatternType,
    /// Pattern data (bytecode hash, function selector, etc.)
    pattern_data: Vec<u8>,
    /// Risk score
    risk_score: u8,
}

/// Pattern matching types
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum PatternType {
    /// Bytecode hash pattern
    BytecodeHash,
    /// Function selector pattern
    FunctionSelector,
    /// Event signature pattern
    EventSignature,
    /// Custom pattern
    Custom,
}

/// Phishing detection database
#[derive(Debug)]
#[allow(dead_code)]
struct PhishingDatabase {
    /// Known phishing domains
    domains: HashMap<String, PhishingDomainInfo>,
    /// Phishing contract signatures
    contract_signatures: Vec<PhishingSignature>,
    /// URL patterns for phishing detection
    url_patterns: Vec<PhishingUrlPattern>,
    /// Last update timestamp
    last_updated: SystemTime,
}

/// Phishing domain information
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PhishingDomainInfo {
    /// Domain name
    domain: String,
    /// First detected timestamp
    first_detected: i64,
    /// Confidence score (0-100)
    confidence_score: u8,
    /// Source of detection
    source: String,
}

/// Phishing smart contract signature
#[derive(Debug, Clone)]
struct PhishingSignature {
    /// Function selector
    selector: [u8; 4],
    /// Description
    description: String,
    /// Risk score
    risk_score: u8,
}

/// Phishing URL pattern
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PhishingUrlPattern {
    /// Pattern regex
    pattern: String,
    /// Description
    description: String,
    /// Confidence level
    confidence: f64,
}

/// Anomaly detection system
#[derive(Debug)]
#[allow(dead_code)]
struct AnomalyDetector {
    /// Transaction frequency analyzer
    frequency_analyzer: FrequencyAnalyzer,
    /// Value pattern analyzer
    value_analyzer: ValuePatternAnalyzer,
    /// Gas usage analyzer
    gas_analyzer: GasUsageAnalyzer,
}

/// Transaction frequency analysis
#[derive(Debug)]
struct FrequencyAnalyzer {
    /// Per-address transaction counts (sliding window)
    address_counts: HashMap<Address, TransactionWindow>,
}

/// Sliding window for transaction counting
#[derive(Debug)]
struct TransactionWindow {
    /// Transaction timestamps in current window
    timestamps: Vec<SystemTime>,
    /// Window duration in seconds
    window_duration: u64,
}

/// Value pattern analyzer for detecting unusual amounts
#[derive(Debug)]
#[allow(dead_code)]
struct ValuePatternAnalyzer {
    /// Historical value patterns per address
    value_history: HashMap<Address, ValueHistory>,
}

/// Value history for pattern analysis
#[derive(Debug)]
#[allow(dead_code)]
struct ValueHistory {
    /// Recent transaction values
    recent_values: Vec<u128>,
    /// Average transaction value
    avg_value: u128,
    /// Standard deviation
    std_deviation: f64,
}

/// Gas usage pattern analyzer
#[derive(Debug)]
#[allow(dead_code)]
struct GasUsageAnalyzer {
    /// Gas usage patterns per contract type
    gas_patterns: HashMap<String, GasPattern>,
}

/// Gas usage pattern
#[derive(Debug)]
#[allow(dead_code)]
struct GasPattern {
    /// Average gas usage
    avg_gas: u128,
    /// Gas usage variance
    variance: f64,
}

/// Security metrics for monitoring
#[derive(Debug, Clone)]
pub struct SecurityMetrics {
    /// Total security checks performed
    total_checks: u64,
    /// Number of threats blocked
    threats_blocked: u64,
    /// Number of warnings issued
    warnings_issued: u64,
    /// Processing time statistics
    avg_processing_time_ms: f64,
    /// Last reset timestamp
    metrics_reset_time: SystemTime,
}

impl Default for SecurityMetrics {
    fn default() -> Self {
        Self {
            total_checks: 0,
            threats_blocked: 0,
            warnings_issued: 0,
            avg_processing_time_ms: 0.0,
            metrics_reset_time: SystemTime::now(),
        }
    }
}

/// TEE Security Engine result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeeSecurityResult {
    /// Basic security result
    pub security_result: SecurityResult,
    /// TEE attestation information
    pub tee_attestation: Option<TeeAttestation>,
    /// Advanced threat intelligence
    pub threat_intelligence: ThreatIntelligence,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// TEE engine version
    pub engine_version: String,
}

/// TEE attestation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeeAttestation {
    /// Attestation signature
    pub signature: Vec<u8>,
    /// TEE hardware identifier
    pub hardware_id: String,
    /// Attestation timestamp
    pub timestamp: i64,
    /// TEE software version
    pub software_version: String,
}

/// Advanced threat intelligence result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelligence {
    /// Detected threat types
    pub threat_types: Vec<ThreatType>,
    /// Anomaly scores
    pub anomaly_scores: HashMap<String, f64>,
    /// Confidence levels
    pub confidence_levels: HashMap<String, f64>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

impl TeeSecurityEngine {
    /// Create new TEE Security Engine
    pub async fn new(config: TeeSecurityConfig) -> Result<Self> {
        info!("🔐 Initializing TEE Security Engine v1.0.0");

        let mut security_checker = SecurityChecker::new();

        // Initialize security checker with threat intelligence
        security_checker
            .load_threat_intelligence()
            .await
            .map_err(|e| anyhow!("Failed to load threat intelligence: {}", e))?;

        let blacklist_cache = Arc::new(RwLock::new(BlacklistCache {
            addresses: HashMap::new(),
            contract_patterns: Vec::new(),
            last_updated: SystemTime::now(),
            cache_version: 1,
        }));

        let phishing_db = Arc::new(RwLock::new(PhishingDatabase {
            domains: HashMap::new(),
            contract_signatures: Vec::new(),
            url_patterns: Vec::new(),
            last_updated: SystemTime::now(),
        }));

        let anomaly_detector = AnomalyDetector {
            frequency_analyzer: FrequencyAnalyzer {
                address_counts: HashMap::new(),
            },
            value_analyzer: ValuePatternAnalyzer {
                value_history: HashMap::new(),
            },
            gas_analyzer: GasUsageAnalyzer {
                gas_patterns: HashMap::new(),
            },
        };

        let metrics = Arc::new(RwLock::new(SecurityMetrics {
            metrics_reset_time: SystemTime::now(),
            ..Default::default()
        }));

        let mut engine = Self {
            security_checker,
            config,
            blacklist_cache,
            phishing_db,
            anomaly_detector,
            metrics,
        };

        // Initialize threat databases
        engine.initialize_threat_databases().await?;

        info!("✅ TEE Security Engine initialized successfully");
        Ok(engine)
    }

    /// Perform comprehensive security analysis within TEE
    pub async fn analyze_security(
        &mut self,
        user_op: &UserOperationVariant,
        entry_point: &Address,
        client_ip: Option<&str>,
    ) -> GatewayResult<TeeSecurityResult> {
        let start_time = SystemTime::now();
        debug!("🔍 Starting TEE security analysis");

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_checks += 1;
        }

        // 1. Basic security checks (using existing SecurityChecker)
        let security_result = self
            .security_checker
            .check_security(user_op, entry_point, client_ip)
            .await?;

        // 2. TEE-specific advanced checks
        let mut threat_intelligence = ThreatIntelligence {
            threat_types: Vec::new(),
            anomaly_scores: HashMap::new(),
            confidence_levels: HashMap::new(),
            recommendations: Vec::new(),
        };

        // Enhanced blacklist checking
        self.enhanced_blacklist_check(user_op, &mut threat_intelligence)
            .await?;

        // Advanced phishing detection
        self.advanced_phishing_detection(user_op, &mut threat_intelligence)
            .await?;

        // Anomaly detection
        if self.config.enable_advanced_anomaly_detection {
            self.advanced_anomaly_detection(user_op, &mut threat_intelligence)
                .await?;
        }

        // 3. Generate TEE attestation (if enabled)
        let tee_attestation = if self.config.enable_tee_attestation {
            Some(self.generate_tee_attestation().await?)
        } else {
            None
        };

        // Calculate processing time
        let processing_time_ms = start_time
            .elapsed()
            .unwrap_or(Duration::from_millis(0))
            .as_millis() as u64;

        // Update metrics with processing time
        {
            let mut metrics = self.metrics.write().await;
            // Simple moving average
            metrics.avg_processing_time_ms =
                (metrics.avg_processing_time_ms * 0.9) + (processing_time_ms as f64 * 0.1);

            if !security_result.is_secure {
                metrics.threats_blocked += 1;
            }
            if !security_result.warnings.is_empty() {
                metrics.warnings_issued += 1;
            }
        }

        // Check processing time limit
        if processing_time_ms > self.config.max_processing_time_ms {
            warn!(
                "⚠️  Security analysis took {}ms, exceeds limit of {}ms",
                processing_time_ms, self.config.max_processing_time_ms
            );
        }

        let result = TeeSecurityResult {
            security_result,
            tee_attestation,
            threat_intelligence,
            processing_time_ms,
            engine_version: "1.0.0".to_string(),
        };

        debug!(
            "✅ TEE security analysis completed in {}ms",
            processing_time_ms
        );
        Ok(result)
    }

    /// Enhanced blacklist checking with TEE secure storage
    async fn enhanced_blacklist_check(
        &self,
        user_op: &UserOperationVariant,
        threat_intel: &mut ThreatIntelligence,
    ) -> Result<()> {
        let blacklist = self.blacklist_cache.read().await;

        let sender = match user_op {
            UserOperationVariant::V0_6(op) => op.sender(),
            UserOperationVariant::V0_7(op) => op.sender(),
        };

        // Check sender against blacklist
        if let Some(malicious_info) = blacklist.addresses.get(&sender) {
            threat_intel
                .threat_types
                .push(malicious_info.threat_type.clone());
            threat_intel.confidence_levels.insert(
                "blacklist_match".to_string(),
                malicious_info.risk_score as f64 / 100.0,
            );
            threat_intel.recommendations.push(format!(
                "Block transaction from blacklisted address {:?} ({:?})",
                sender, malicious_info.threat_type
            ));
        }

        Ok(())
    }

    /// Advanced phishing detection with pattern matching
    async fn advanced_phishing_detection(
        &self,
        user_op: &UserOperationVariant,
        threat_intel: &mut ThreatIntelligence,
    ) -> Result<()> {
        let phishing_db = self.phishing_db.read().await;

        let call_data = match user_op {
            UserOperationVariant::V0_6(op) => op.call_data(),
            UserOperationVariant::V0_7(op) => op.call_data(),
        };

        // Check function selectors against known phishing patterns
        if call_data.len() >= 4 {
            let selector = &call_data[0..4];
            for signature in &phishing_db.contract_signatures {
                if selector == signature.selector {
                    threat_intel.threat_types.push(ThreatType::PhishingContract);
                    threat_intel.confidence_levels.insert(
                        "phishing_signature".to_string(),
                        signature.risk_score as f64 / 100.0,
                    );
                    threat_intel.recommendations.push(format!(
                        "Potential phishing detected: {}",
                        signature.description
                    ));
                }
            }
        }

        Ok(())
    }

    /// Advanced anomaly detection
    async fn advanced_anomaly_detection(
        &mut self,
        user_op: &UserOperationVariant,
        threat_intel: &mut ThreatIntelligence,
    ) -> Result<()> {
        let sender = match user_op {
            UserOperationVariant::V0_6(op) => op.sender(),
            UserOperationVariant::V0_7(op) => op.sender(),
        };

        // Frequency analysis
        let frequency_score = self
            .anomaly_detector
            .frequency_analyzer
            .analyze_frequency(&sender, SystemTime::now());

        threat_intel
            .anomaly_scores
            .insert("transaction_frequency".to_string(), frequency_score);

        if frequency_score > 0.8 {
            threat_intel.recommendations.push(
                "High transaction frequency detected - potential automated abuse".to_string(),
            );
        }

        // Gas pattern analysis
        let (call_gas, verification_gas) = match user_op {
            UserOperationVariant::V0_6(op) => {
                (op.call_gas_limit(), op.total_verification_gas_limit())
            }
            UserOperationVariant::V0_7(op) => {
                (op.call_gas_limit(), op.total_verification_gas_limit())
            }
        };

        let gas_anomaly_score = self
            .anomaly_detector
            .gas_analyzer
            .analyze_gas_patterns(call_gas, verification_gas);

        threat_intel
            .anomaly_scores
            .insert("gas_usage_pattern".to_string(), gas_anomaly_score);

        Ok(())
    }

    /// Generate TEE attestation
    async fn generate_tee_attestation(&self) -> Result<TeeAttestation> {
        // TODO: In production, integrate with actual TEE attestation APIs
        // For now, generate mock attestation
        Ok(TeeAttestation {
            signature: vec![0u8; 64], // Mock signature
            hardware_id: "mock_tee_device_001".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            software_version: "TEE_Security_Engine_1.0.0".to_string(),
        })
    }

    /// Initialize threat intelligence databases
    async fn initialize_threat_databases(&mut self) -> Result<()> {
        debug!("📚 Initializing threat intelligence databases");

        // Initialize blacklist cache with known threats
        {
            let mut blacklist = self.blacklist_cache.write().await;

            // Add known malicious addresses (examples for testing)
            blacklist.addresses.insert(
                "0x0000000000000000000000000000000000000bad"
                    .parse()
                    .unwrap(),
                MaliciousAddressInfo {
                    address: "0x0000000000000000000000000000000000000bad"
                        .parse()
                        .unwrap(),
                    threat_type: ThreatType::Scammer,
                    risk_score: 95,
                    source: "Manual_Blacklist".to_string(),
                    first_detected: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                    metadata: None,
                },
            );
        }

        // Initialize phishing database
        {
            let mut phishing_db = self.phishing_db.write().await;

            // Add known phishing function selectors
            phishing_db.contract_signatures.push(PhishingSignature {
                selector: [0xa9, 0x05, 0x9c, 0xbb], // transfer function selector
                description: "Suspicious transfer pattern".to_string(),
                risk_score: 70,
            });
        }

        debug!("✅ Threat intelligence databases initialized");
        Ok(())
    }

    /// Update threat intelligence from external sources
    pub async fn update_threat_intelligence(&mut self) -> Result<()> {
        if !self.config.enable_realtime_updates {
            debug!("Real-time threat intelligence updates disabled");
            return Ok(());
        }

        info!("🔄 Updating threat intelligence from external sources");

        // TODO: In production, integrate with threat intelligence feeds
        // - Query threat intelligence APIs
        // - Update blacklist cache
        // - Update phishing database
        // - Validate data integrity

        Ok(())
    }

    /// Get security engine metrics
    pub async fn get_metrics(&self) -> SecurityMetrics {
        (*self.metrics.read().await).clone()
    }

    /// Reset security metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = SecurityMetrics {
            metrics_reset_time: SystemTime::now(),
            ..Default::default()
        };
        info!("🔄 Security metrics reset");
    }
}

// Helper implementations for anomaly detection

impl FrequencyAnalyzer {
    fn analyze_frequency(&mut self, address: &Address, current_time: SystemTime) -> f64 {
        let window = self
            .address_counts
            .entry(*address)
            .or_insert_with(|| TransactionWindow {
                timestamps: Vec::new(),
                window_duration: 300, // 5 minutes
            });

        // Add current timestamp
        window.timestamps.push(current_time);

        // Remove old timestamps outside the window
        let cutoff_time = current_time - Duration::from_secs(window.window_duration);
        window.timestamps.retain(|&ts| ts > cutoff_time);

        // Calculate frequency score (0.0 = normal, 1.0 = extremely high)
        let tx_count = window.timestamps.len();
        match tx_count {
            0..=5 => 0.0,
            6..=15 => 0.3,
            16..=30 => 0.6,
            31..=50 => 0.8,
            _ => 1.0,
        }
    }
}

impl GasUsageAnalyzer {
    fn analyze_gas_patterns(&self, call_gas: u128, verification_gas: u128) -> f64 {
        // Simple anomaly detection based on gas usage
        let total_gas = call_gas + verification_gas;

        // Flag extremely high gas usage as suspicious
        if total_gas > 50_000_000 {
            0.9 // High anomaly score
        } else if total_gas > 20_000_000 {
            0.5 // Medium anomaly score
        } else {
            0.1 // Low anomaly score
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tee_security_engine_initialization() {
        let config = TeeSecurityConfig::default();
        let engine = TeeSecurityEngine::new(config).await;
        assert!(engine.is_ok());
    }

    #[test]
    fn test_threat_type_serialization() {
        let threat = ThreatType::Scammer;
        let serialized = serde_json::to_string(&threat).unwrap();
        assert!(serialized.contains("scammer"));
    }

    #[test]
    fn test_frequency_analyzer() {
        let mut analyzer = FrequencyAnalyzer {
            address_counts: HashMap::new(),
        };

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let score = analyzer.analyze_frequency(&address, SystemTime::now());

        // First transaction should have low score
        assert!(score < 0.5);
    }
}
