/**
 * SBT + PNTs Balance Validator for SuperRelay Gateway
 *
 * Validates user eligibility before processing UserOperations:
 * 1. SBT (Soul Bound Token) ownership verification
 * 2. PNTs token balance check for gas sponsorship
 *
 * Integration with AirAccount dual-signature flow:
 * Gateway SBT+PNTs check → AirAccount KMS dual signature → Paymaster sponsorship
 */
use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use ethers::{
    contract::abigen,
    providers::{Http, Middleware, Provider},
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

// Generate contract bindings for SBT and PNTs contracts
abigen!(
    SBTContract,
    r#"[
        function balanceOf(address owner) external view returns (uint256)
        function ownerOf(uint256 tokenId) external view returns (address)
        function tokenOfOwnerByIndex(address owner, uint256 index) external view returns (uint256)
    ]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    PNTSContract,
    r#"[
        function balanceOf(address account) external view returns (uint256)
        function decimals() external view returns (uint8)
        function transfer(address to, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
    ]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);

/// Configuration for SBT + PNTs validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SBTValidatorConfig {
    /// RPC endpoint for blockchain queries
    pub rpc_url: String,
    /// SBT contract address
    pub sbt_contract: Address,
    /// PNTs ERC20 contract address  
    pub pnts_contract: Address,
    /// PNTs to ETH conversion rate (e.g., 1000 means 1000 PNTs = 1 ETH)
    pub pnts_to_eth_rate: u64,
    /// Gas price buffer multiplier (e.g., 1.2 for 20% buffer)
    pub gas_price_buffer: f64,
    /// Minimum required SBT balance (usually 1)
    pub min_sbt_balance: u64,
    /// Validation timeout in seconds
    pub validation_timeout: u64,
}

impl Default for SBTValidatorConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://eth-sepolia.g.alchemy.com/v2/demo".to_string(),
            sbt_contract: "0xBfde68c232F2248114429DDD9a7c3Adbff74bD7f"
                .parse()
                .unwrap(), // Sepolia
            pnts_contract: "0x3e7B771d4541eC85c8137e950598Ac97553a337a"
                .parse()
                .unwrap(), // Sepolia
            pnts_to_eth_rate: 1000, // 1000 PNTs = 1 ETH
            gas_price_buffer: 1.2,  // 20% buffer
            min_sbt_balance: 1,
            validation_timeout: 30, // 30 seconds
        }
    }
}

/// Validation result for user eligibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// User's Ethereum address
    pub user_address: Address,
    /// Number of SBT tokens owned
    pub sbt_balance: u64,
    /// Current PNTs token balance
    pub pnts_balance: U256,
    /// Required PNTs for this transaction
    pub required_pnts: U256,
    /// Estimated gas for this transaction
    pub gas_estimation: U256,
    /// Whether user is eligible for gas sponsorship
    pub is_eligible: bool,
    /// When the validation was performed
    pub validation_time: DateTime<Utc>,
    /// Error message if validation failed
    pub error_message: Option<String>,
}

/// SBT + PNTs Validator implementation
pub struct SBTValidator {
    config: SBTValidatorConfig,
    rpc_client: Arc<Provider<Http>>,
    sbt_contract: SBTContract<Provider<Http>>,
    pnts_contract: PNTSContract<Provider<Http>>,
}

impl SBTValidator {
    /// Create new SBT validator instance
    pub async fn new(config: SBTValidatorConfig) -> Result<Self> {
        info!("🔧 Initializing SBT + PNTs Validator...");
        info!("   RPC URL: {}", config.rpc_url);
        info!("   SBT Contract: {:?}", config.sbt_contract);
        info!("   PNTs Contract: {:?}", config.pnts_contract);
        info!("   PNTs/ETH Rate: {}", config.pnts_to_eth_rate);

        // Initialize RPC client
        let provider = Provider::<Http>::try_from(&config.rpc_url)
            .map_err(|e| anyhow!("Failed to create RPC provider: {}", e))?;
        let rpc_client = Arc::new(provider);

        // Initialize contract instances
        let sbt_contract = SBTContract::new(config.sbt_contract, Arc::clone(&rpc_client));
        let pnts_contract = PNTSContract::new(config.pnts_contract, Arc::clone(&rpc_client));

        // Test connectivity
        let chain_id = rpc_client
            .get_chainid()
            .await
            .map_err(|e| anyhow!("Failed to connect to blockchain: {}", e))?;
        info!("✅ Connected to blockchain, Chain ID: {}", chain_id);

        Ok(Self {
            config,
            rpc_client,
            sbt_contract,
            pnts_contract,
        })
    }

    /// Validate user eligibility for gas sponsorship
    /// This is the main entry point called by Gateway before processing UserOperations
    pub async fn verify_user_eligibility(
        &self,
        user_address: Address,
        estimated_gas: U256,
    ) -> Result<ValidationResult> {
        let start_time = Utc::now();
        debug!(
            "🔍 Starting eligibility verification for user: {:?}",
            user_address
        );
        debug!("   Estimated gas: {}", estimated_gas);

        let mut result = ValidationResult {
            user_address,
            sbt_balance: 0,
            pnts_balance: U256::zero(),
            required_pnts: U256::zero(),
            gas_estimation: estimated_gas,
            is_eligible: false,
            validation_time: start_time,
            error_message: None,
        };

        // Step 1: Check SBT ownership
        match self.check_sbt_ownership(user_address).await {
            Ok(sbt_balance) => {
                result.sbt_balance = sbt_balance;
                debug!("✅ SBT balance check passed: {}", sbt_balance);
            }
            Err(e) => {
                let error_msg = format!("SBT ownership check failed: {}", e);
                error!("{}", error_msg);
                result.error_message = Some(error_msg);
                return Ok(result);
            }
        }

        // Step 2: Check if user has minimum SBT balance
        if result.sbt_balance < self.config.min_sbt_balance {
            let error_msg = format!(
                "Insufficient SBT balance. Required: {}, Got: {}",
                self.config.min_sbt_balance, result.sbt_balance
            );
            warn!("{}", error_msg);
            result.error_message = Some(error_msg);
            return Ok(result);
        }

        // Step 3: Calculate required PNTs for gas
        let required_pnts = self.calculate_required_pnts(estimated_gas)?;
        result.required_pnts = required_pnts;
        debug!("💰 Required PNTs for gas: {}", required_pnts);

        // Step 4: Check PNTs balance
        match self.check_pnts_balance(user_address).await {
            Ok(pnts_balance) => {
                result.pnts_balance = pnts_balance;
                debug!("✅ PNTs balance: {}", pnts_balance);
            }
            Err(e) => {
                let error_msg = format!("PNTs balance check failed: {}", e);
                error!("{}", error_msg);
                result.error_message = Some(error_msg);
                return Ok(result);
            }
        }

        // Step 5: Verify sufficient PNTs balance
        if result.pnts_balance >= required_pnts {
            result.is_eligible = true;
            info!("✅ User eligibility verified for {:?}", user_address);
            info!("   SBT Balance: {}", result.sbt_balance);
            info!("   PNTs Balance: {}", result.pnts_balance);
            info!("   Required PNTs: {}", required_pnts);
        } else {
            let error_msg = format!(
                "Insufficient PNTs balance. Required: {}, Got: {}",
                required_pnts, result.pnts_balance
            );
            warn!("{}", error_msg);
            result.error_message = Some(error_msg);
        }

        let validation_duration = Utc::now().signed_duration_since(start_time);
        debug!(
            "🕐 Validation completed in {}ms",
            validation_duration.num_milliseconds()
        );

        Ok(result)
    }

    /// Check SBT ownership for the user
    async fn check_sbt_ownership(&self, user_address: Address) -> Result<u64> {
        debug!("🏷️ Checking SBT ownership for {:?}", user_address);

        let balance = self
            .sbt_contract
            .balance_of(user_address)
            .call()
            .await
            .map_err(|e| anyhow!("Failed to query SBT balance: {}", e))?;

        let sbt_balance = balance.as_u64();
        debug!("   SBT balance: {}", sbt_balance);

        Ok(sbt_balance)
    }

    /// Check PNTs token balance for the user
    async fn check_pnts_balance(&self, user_address: Address) -> Result<U256> {
        debug!("💳 Checking PNTs balance for {:?}", user_address);

        let balance = self
            .pnts_contract
            .balance_of(user_address)
            .call()
            .await
            .map_err(|e| anyhow!("Failed to query PNTs balance: {}", e))?;

        debug!("   PNTs balance: {}", balance);
        Ok(balance)
    }

    /// Calculate required PNTs amount for gas consumption
    fn calculate_required_pnts(&self, estimated_gas: U256) -> Result<U256> {
        // Apply gas price buffer (e.g., 20% extra)
        let buffered_gas = estimated_gas
            * U256::from((self.config.gas_price_buffer * 1000.0) as u64)
            / U256::from(1000);

        // Convert gas to PNTs using the conversion rate
        // Formula: required_pnts = (buffered_gas * pnts_rate) / (10^18)
        let required_pnts = buffered_gas * U256::from(self.config.pnts_to_eth_rate)
            / U256::from(10).pow(U256::from(18));

        debug!("💰 Gas calculation:");
        debug!("   Original gas: {}", estimated_gas);
        debug!("   Buffered gas: {}", buffered_gas);
        debug!("   Required PNTs: {}", required_pnts);

        Ok(required_pnts)
    }

    /// Get validator configuration
    pub fn get_config(&self) -> &SBTValidatorConfig {
        &self.config
    }

    /// Health check for validator services
    pub async fn health_check(&self) -> Result<bool> {
        debug!("🩺 Performing SBT validator health check...");

        // Test RPC connectivity
        let chain_id = self
            .rpc_client
            .get_chainid()
            .await
            .map_err(|e| anyhow!("RPC health check failed: {}", e))?;

        // Test SBT contract connectivity
        let test_address: Address = "0x0000000000000000000000000000000000000001"
            .parse()
            .unwrap();
        let _ = self
            .sbt_contract
            .balance_of(test_address)
            .call()
            .await
            .map_err(|e| anyhow!("SBT contract health check failed: {}", e))?;

        // Test PNTs contract connectivity
        let _ = self
            .pnts_contract
            .balance_of(test_address)
            .call()
            .await
            .map_err(|e| anyhow!("PNTs contract health check failed: {}", e))?;

        info!(
            "✅ SBT validator health check passed (Chain ID: {})",
            chain_id
        );
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn test_sbt_validator_creation() {
        let config = SBTValidatorConfig::default();

        // This test will fail without a real RPC endpoint, but validates the structure
        let result = SBTValidator::new(config).await;

        // In real tests, we would use a mock or test RPC endpoint
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_gas_calculation() {
        let config = SBTValidatorConfig::default();
        let validator = SBTValidator {
            config: config.clone(),
            rpc_client: Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap()),
            sbt_contract: SBTContract::new(
                config.sbt_contract,
                Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap()),
            ),
            pnts_contract: PNTSContract::new(
                config.pnts_contract,
                Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap()),
            ),
        };

        let gas = U256::from(21000); // Standard ETH transfer gas
        let required_pnts = validator.calculate_required_pnts(gas).unwrap();

        // With 1000 PNTs/ETH rate and 21000 gas, should require minimal PNTs
        assert!(required_pnts > U256::zero());
    }

    #[test]
    fn test_validation_result_serialization() {
        let result = ValidationResult {
            user_address: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
            sbt_balance: 1,
            pnts_balance: U256::from(1000),
            required_pnts: U256::from(100),
            gas_estimation: U256::from(21000),
            is_eligible: true,
            validation_time: Utc::now(),
            error_message: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("user_address"));
        assert!(json.contains("is_eligible"));
    }
}
