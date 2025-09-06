/**
 * EntryPoint Version Selector for SuperRelay Gateway
 *
 * Handles version detection and routing for different ERC-4337 EntryPoint versions:
 * - v0.6: Traditional UserOperation structure
 * - v0.7: PackedUserOperation structure  
 * - v0.8: Enhanced PackedUserOperation with EIP-7702 support
 *
 * Cross-chain EntryPoint address management based on config.md
 */
use std::collections::HashMap;

use anyhow::{anyhow, Result};
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Supported ERC-4337 EntryPoint versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntryPointVersion {
    /// ERC-4337 v0.6 - Traditional UserOperation structure
    V0_6,
    /// ERC-4337 v0.7 - PackedUserOperation structure (production ready)
    V0_7,
    /// ERC-4337 v0.8 - Enhanced PackedUserOperation with EIP-7702
    V0_8,
}

impl EntryPointVersion {
    /// Get version string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryPointVersion::V0_6 => "0.6",
            EntryPointVersion::V0_7 => "0.7",
            EntryPointVersion::V0_8 => "0.8",
        }
    }

    /// Parse version from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "0.6" | "v0.6" => Ok(EntryPointVersion::V0_6),
            "0.7" | "v0.7" => Ok(EntryPointVersion::V0_7),
            "0.8" | "v0.8" => Ok(EntryPointVersion::V0_8),
            _ => Err(anyhow!("Unsupported EntryPoint version: {}", s)),
        }
    }

    /// Get all supported versions
    pub fn all() -> Vec<EntryPointVersion> {
        vec![
            EntryPointVersion::V0_6,
            EntryPointVersion::V0_7,
            EntryPointVersion::V0_8,
        ]
    }

    /// Check if this version supports packed UserOperation structure
    pub fn uses_packed_user_operation(&self) -> bool {
        match self {
            EntryPointVersion::V0_6 => false,
            EntryPointVersion::V0_7 | EntryPointVersion::V0_8 => true,
        }
    }

    /// Check if this version supports EIP-7702 authorization
    pub fn supports_eip7702(&self) -> bool {
        match self {
            EntryPointVersion::V0_6 | EntryPointVersion::V0_7 => false,
            EntryPointVersion::V0_8 => true,
        }
    }
}

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Network {
    /// Ethereum Mainnet (Chain ID: 1)
    EthereumMainnet,
    /// Ethereum Sepolia Testnet (Chain ID: 11155111)
    EthereumSepolia,
    /// Optimism Mainnet (Chain ID: 10)
    OptimismMainnet,
    /// Optimism Sepolia Testnet (Chain ID: 11155420)
    OptimismSepolia,
}

impl Network {
    /// Get chain ID for this network
    pub fn chain_id(&self) -> u64 {
        match self {
            Network::EthereumMainnet => 1,
            Network::EthereumSepolia => 11155111,
            Network::OptimismMainnet => 10,
            Network::OptimismSepolia => 11155420,
        }
    }

    /// Parse network from chain ID
    pub fn from_chain_id(chain_id: u64) -> Result<Self> {
        match chain_id {
            1 => Ok(Network::EthereumMainnet),
            11155111 => Ok(Network::EthereumSepolia),
            10 => Ok(Network::OptimismMainnet),
            11155420 => Ok(Network::OptimismSepolia),
            _ => Err(anyhow!("Unsupported chain ID: {}", chain_id)),
        }
    }

    /// Get network name
    pub fn name(&self) -> &'static str {
        match self {
            Network::EthereumMainnet => "ethereum-mainnet",
            Network::EthereumSepolia => "ethereum-sepolia",
            Network::OptimismMainnet => "optimism-mainnet",
            Network::OptimismSepolia => "optimism-sepolia",
        }
    }
}

/// EntryPoint version configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSelectorConfig {
    /// Default EntryPoint version to use
    pub default_version: EntryPointVersion,
    /// Supported versions for this deployment
    pub supported_versions: Vec<EntryPointVersion>,
    /// Network-specific EntryPoint contract addresses
    pub entry_point_addresses: HashMap<(Network, EntryPointVersion), Address>,
    /// Enable automatic version detection from UserOperation structure
    pub auto_detect_version: bool,
    /// Fallback to default version if detection fails
    pub fallback_to_default: bool,
}

impl VersionSelectorConfig {
    /// Create default configuration for Sepolia testnet
    pub fn sepolia_testnet() -> Self {
        let mut entry_point_addresses = HashMap::new();

        // Sepolia EntryPoint addresses from config.md
        entry_point_addresses.insert(
            (Network::EthereumSepolia, EntryPointVersion::V0_6),
            "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                .parse()
                .unwrap(),
        );
        entry_point_addresses.insert(
            (Network::EthereumSepolia, EntryPointVersion::V0_7),
            "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
                .parse()
                .unwrap(),
        );
        entry_point_addresses.insert(
            (Network::EthereumSepolia, EntryPointVersion::V0_8),
            "0x4337084d9e255ff0702461cf8895ce9e3b5ff108"
                .parse()
                .unwrap(),
        );

        Self {
            default_version: EntryPointVersion::V0_7, // Production ready
            supported_versions: vec![
                EntryPointVersion::V0_6,
                EntryPointVersion::V0_7,
                EntryPointVersion::V0_8,
            ],
            entry_point_addresses,
            auto_detect_version: true,
            fallback_to_default: true,
        }
    }

    /// Create configuration for Ethereum Mainnet
    pub fn ethereum_mainnet() -> Self {
        let mut entry_point_addresses = HashMap::new();

        // Mainnet EntryPoint addresses from config.md
        entry_point_addresses.insert(
            (Network::EthereumMainnet, EntryPointVersion::V0_6),
            "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                .parse()
                .unwrap(),
        );
        entry_point_addresses.insert(
            (Network::EthereumMainnet, EntryPointVersion::V0_7),
            "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
                .parse()
                .unwrap(),
        );
        entry_point_addresses.insert(
            (Network::EthereumMainnet, EntryPointVersion::V0_8),
            "0x4337084d9e255ff0702461cf8895ce9e3b5ff108"
                .parse()
                .unwrap(),
        );

        Self {
            default_version: EntryPointVersion::V0_7,
            supported_versions: vec![
                EntryPointVersion::V0_6,
                EntryPointVersion::V0_7,
                EntryPointVersion::V0_8,
            ],
            entry_point_addresses,
            auto_detect_version: true,
            fallback_to_default: true,
        }
    }
}

/// Version selection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSelection {
    /// Selected EntryPoint version
    pub selected_version: EntryPointVersion,
    /// EntryPoint contract address
    pub entry_point_address: Address,
    /// Blockchain network
    pub network: Network,
    /// How the version was detected
    pub detection_method: DetectionMethod,
}

/// How the version was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionMethod {
    /// Explicitly specified by client
    Explicit,
    /// Auto-detected from UserOperation structure
    AutoDetected,
    /// Fell back to default configuration
    Default,
}

/// EntryPoint version selector
pub struct VersionSelector {
    config: VersionSelectorConfig,
    current_network: Network,
}

impl VersionSelector {
    /// Create new version selector
    pub fn new(config: VersionSelectorConfig, chain_id: u64) -> Result<Self> {
        let current_network = Network::from_chain_id(chain_id)?;

        info!("🔄 Initializing EntryPoint version selector");
        info!(
            "   Network: {} (Chain ID: {})",
            current_network.name(),
            chain_id
        );
        info!("   Default version: {}", config.default_version.as_str());
        info!(
            "   Supported versions: {:?}",
            config
                .supported_versions
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>()
        );

        Ok(Self {
            config,
            current_network,
        })
    }

    /// Select EntryPoint version for a UserOperation
    pub fn select_version(
        &self,
        explicit_version: Option<&str>,
        user_operation_json: Option<&serde_json::Value>,
    ) -> Result<VersionSelection> {
        // Method 1: Explicit version specification
        if let Some(version_str) = explicit_version {
            let version = EntryPointVersion::from_str(version_str)?;

            if !self.config.supported_versions.contains(&version) {
                return Err(anyhow!(
                    "Version {} not supported. Supported: {:?}",
                    version_str,
                    self.config
                        .supported_versions
                        .iter()
                        .map(|v| v.as_str())
                        .collect::<Vec<_>>()
                ));
            }

            let entry_point_address = self.get_entry_point_address(version)?;

            debug!("✅ Using explicitly specified version: {}", version_str);

            return Ok(VersionSelection {
                selected_version: version,
                entry_point_address,
                network: self.current_network,
                detection_method: DetectionMethod::Explicit,
            });
        }

        // Method 2: Auto-detect from UserOperation structure
        if self.config.auto_detect_version {
            if let Some(user_op) = user_operation_json {
                if let Ok(detected_version) = self.detect_version_from_user_operation(user_op) {
                    let entry_point_address = self.get_entry_point_address(detected_version)?;

                    debug!("✅ Auto-detected version: {}", detected_version.as_str());

                    return Ok(VersionSelection {
                        selected_version: detected_version,
                        entry_point_address,
                        network: self.current_network,
                        detection_method: DetectionMethod::AutoDetected,
                    });
                }
            }
        }

        // Method 3: Use default version
        if self.config.fallback_to_default {
            let entry_point_address = self.get_entry_point_address(self.config.default_version)?;

            debug!(
                "✅ Using default version: {}",
                self.config.default_version.as_str()
            );

            Ok(VersionSelection {
                selected_version: self.config.default_version,
                entry_point_address,
                network: self.current_network,
                detection_method: DetectionMethod::Default,
            })
        } else {
            Err(anyhow!(
                "Unable to determine EntryPoint version and fallback is disabled"
            ))
        }
    }

    /// Detect EntryPoint version from UserOperation JSON structure
    fn detect_version_from_user_operation(
        &self,
        user_op: &serde_json::Value,
    ) -> Result<EntryPointVersion> {
        // v0.6 has separate factory/factoryData fields
        // v0.7/v0.8 use PackedUserOperation with accountGasLimits, gasFees fields

        if user_op.get("factory").is_some() && user_op.get("factoryData").is_some() {
            // v0.6 structure detected
            debug!("🔍 Detected v0.6 structure: factory/factoryData fields present");
            return Ok(EntryPointVersion::V0_6);
        }

        if user_op.get("accountGasLimits").is_some() && user_op.get("gasFees").is_some() {
            // v0.7/v0.8 packed structure detected

            // Check for EIP-7702 authorization (v0.8 specific)
            if user_op.get("authorization").is_some() {
                debug!("🔍 Detected v0.8 structure: EIP-7702 authorization present");
                return Ok(EntryPointVersion::V0_8);
            } else {
                debug!("🔍 Detected v0.7 structure: PackedUserOperation without EIP-7702");
                return Ok(EntryPointVersion::V0_7);
            }
        }

        // Legacy detection: check for v0.6 specific fields
        if user_op.get("initCode").is_some() && user_op.get("paymasterAndData").is_some() {
            debug!("🔍 Detected legacy v0.6 structure: initCode/paymasterAndData fields");
            return Ok(EntryPointVersion::V0_6);
        }

        Err(anyhow!(
            "Unable to detect EntryPoint version from UserOperation structure"
        ))
    }

    /// Get EntryPoint contract address for version and network
    fn get_entry_point_address(&self, version: EntryPointVersion) -> Result<Address> {
        let key = (self.current_network, version);

        self.config
            .entry_point_addresses
            .get(&key)
            .copied()
            .ok_or_else(|| {
                anyhow!(
                    "No EntryPoint address configured for {} version {} on network {}",
                    version.as_str(),
                    version.as_str(),
                    self.current_network.name()
                )
            })
    }

    /// Get current network
    pub fn get_network(&self) -> Network {
        self.current_network
    }

    /// Get all supported versions
    pub fn get_supported_versions(&self) -> &[EntryPointVersion] {
        &self.config.supported_versions
    }

    /// Check if version is supported
    pub fn is_version_supported(&self, version: EntryPointVersion) -> bool {
        self.config.supported_versions.contains(&version)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_entry_point_version_parsing() {
        assert_eq!(
            EntryPointVersion::from_str("0.6").unwrap(),
            EntryPointVersion::V0_6
        );
        assert_eq!(
            EntryPointVersion::from_str("v0.7").unwrap(),
            EntryPointVersion::V0_7
        );
        assert_eq!(
            EntryPointVersion::from_str("0.8").unwrap(),
            EntryPointVersion::V0_8
        );

        assert!(EntryPointVersion::from_str("1.0").is_err());
    }

    #[test]
    fn test_network_from_chain_id() {
        assert_eq!(Network::from_chain_id(1).unwrap(), Network::EthereumMainnet);
        assert_eq!(
            Network::from_chain_id(11155111).unwrap(),
            Network::EthereumSepolia
        );
        assert_eq!(
            Network::from_chain_id(10).unwrap(),
            Network::OptimismMainnet
        );

        assert!(Network::from_chain_id(999999).is_err());
    }

    #[test]
    fn test_version_detection_from_user_operation() {
        let config = VersionSelectorConfig::sepolia_testnet();
        let selector = VersionSelector::new(config, 11155111).unwrap();

        // Test v0.6 detection
        let v06_user_op = json!({
            "sender": "0x1234567890123456789012345678901234567890",
            "nonce": "0x0",
            "factory": "0x9406cc6185a346906296840746125a0e44976454",
            "factoryData": "0x1234",
            "callData": "0x",
            "signature": "0x"
        });

        let detected = selector
            .detect_version_from_user_operation(&v06_user_op)
            .unwrap();
        assert_eq!(detected, EntryPointVersion::V0_6);

        // Test v0.7 detection
        let v07_user_op = json!({
            "sender": "0x1234567890123456789012345678901234567890",
            "nonce": "0x0",
            "accountGasLimits": "0x1234567890123456",
            "gasFees": "0x1234567890123456",
            "callData": "0x",
            "signature": "0x"
        });

        let detected = selector
            .detect_version_from_user_operation(&v07_user_op)
            .unwrap();
        assert_eq!(detected, EntryPointVersion::V0_7);
    }

    #[test]
    fn test_version_selection_explicit() {
        let config = VersionSelectorConfig::sepolia_testnet();
        let selector = VersionSelector::new(config, 11155111).unwrap();

        let result = selector.select_version(Some("0.6"), None).unwrap();
        assert_eq!(result.selected_version, EntryPointVersion::V0_6);
        assert!(matches!(result.detection_method, DetectionMethod::Explicit));
    }

    #[test]
    fn test_packed_user_operation_detection() {
        assert!(!EntryPointVersion::V0_6.uses_packed_user_operation());
        assert!(EntryPointVersion::V0_7.uses_packed_user_operation());
        assert!(EntryPointVersion::V0_8.uses_packed_user_operation());
    }

    #[test]
    fn test_eip7702_support() {
        assert!(!EntryPointVersion::V0_6.supports_eip7702());
        assert!(!EntryPointVersion::V0_7.supports_eip7702());
        assert!(EntryPointVersion::V0_8.supports_eip7702());
    }
}
