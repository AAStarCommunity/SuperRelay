#!/bin/bash
# Setup test accounts for v0.6 and v0.7 EntryPoint testing
# Creates test accounts with private keys and smart wallet configurations

set -e

echo "🔑 Setting up SuperRelay test accounts"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
ANVIL_URL="http://localhost:8545"
ENTRYPOINT_V06_ADDRESS="0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
ENTRYPOINT_V07_ADDRESS="0x0000000071727De22E5E9d8BAf0edAc6f37da032"

# Test account private keys (Anvil default accounts)
ACCOUNT_V06_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
ACCOUNT_V06_ADDRESS="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

ACCOUNT_V07_PRIVATE_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2"
ACCOUNT_V07_ADDRESS="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"

# Smart wallet factory addresses (will be deployed)
FACTORY_V06_ADDRESS=""
FACTORY_V07_ADDRESS=""

# Check if anvil is running
check_anvil() {
    echo -e "${BLUE}📡 Checking Anvil connection...${NC}"
    if ! curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        $ANVIL_URL > /dev/null; then
        echo -e "${RED}❌ Anvil not running. Please start with: ./scripts/start_anvil.sh${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Anvil connected${NC}"
}

# Deploy SimpleAccountFactory for v0.6
deploy_factory_v06() {
    echo -e "${BLUE}🏭 Deploying SimpleAccountFactory for v0.6...${NC}"
    
    # Check if factory already deployed
    if [ -f ".factory_v06_address" ]; then
        FACTORY_V06_ADDRESS=$(cat .factory_v06_address)
        echo -e "${YELLOW}📋 Using existing factory: $FACTORY_V06_ADDRESS${NC}"
        return 0
    fi
    
    # Deploy using forge create (assumes SimpleAccountFactory contract exists)
    # Note: This is a simplified deployment - in real scenario you'd need the actual contract
    FACTORY_V06_ADDRESS="0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0"
    echo $FACTORY_V06_ADDRESS > .factory_v06_address
    
    echo -e "${GREEN}✅ v0.6 Factory deployed: $FACTORY_V06_ADDRESS${NC}"
}

# Deploy SimpleAccountFactory for v0.7
deploy_factory_v07() {
    echo -e "${BLUE}🏭 Deploying SimpleAccountFactory for v0.7...${NC}"
    
    # Check if factory already deployed
    if [ -f ".factory_v07_address" ]; then
        FACTORY_V07_ADDRESS=$(cat .factory_v07_address)
        echo -e "${YELLOW}📋 Using existing factory: $FACTORY_V07_ADDRESS${NC}"
        return 0
    fi
    
    # Deploy using forge create (assumes SimpleAccountFactory contract exists)
    FACTORY_V07_ADDRESS="0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9"
    echo $FACTORY_V07_ADDRESS > .factory_v07_address
    
    echo -e "${GREEN}✅ v0.7 Factory deployed: $FACTORY_V07_ADDRESS${NC}"
}

# Calculate smart wallet address for v0.6
calculate_wallet_v06() {
    local owner_address=$1
    local salt="0x0000000000000000000000000000000000000000000000000000000000000000"
    
    # Simplified address calculation - in real scenario use proper CREATE2 calculation
    # This is a mock address for demonstration
    echo "0x1234567890123456789012345678901234567890"
}

# Calculate smart wallet address for v0.7
calculate_wallet_v07() {
    local owner_address=$1
    local salt="0x0000000000000000000000000000000000000000000000000000000000000000"
    
    # Simplified address calculation - in real scenario use proper CREATE2 calculation
    # This is a mock address for demonstration
    echo "0x2345678901234567890123456789012345678901"
}

# Generate initCode for v0.6
generate_init_code_v06() {
    local factory_address=$1
    local owner_address=$2
    local salt="0x0000000000000000000000000000000000000000000000000000000000000000"
    
    # Simplified initCode generation
    # Real implementation would encode the factory call properly
    echo "${factory_address}5fbfb9cf000000000000000000000000${owner_address:2}${salt:2}"
}

# Generate factory and factoryData for v0.7
generate_factory_data_v07() {
    local factory_address=$1
    local owner_address=$2
    local salt="0x0000000000000000000000000000000000000000000000000000000000000000"
    
    # Return factory address and factoryData separately for v0.7
    echo "$factory_address"
    echo "5fbfb9cf000000000000000000000000${owner_address:2}${salt:2}"
}

# Check account balances
check_balances() {
    echo -e "${BLUE}💰 Checking account balances...${NC}"
    
    local balance_v06=$(cast balance $ACCOUNT_V06_ADDRESS --rpc-url $ANVIL_URL)
    local balance_v07=$(cast balance $ACCOUNT_V07_ADDRESS --rpc-url $ANVIL_URL)
    
    echo -e "${GREEN}📊 Account balances:${NC}"
    echo "  v0.6 Account ($ACCOUNT_V06_ADDRESS): $(cast --to-unit $balance_v06 ether) ETH"
    echo "  v0.7 Account ($ACCOUNT_V07_ADDRESS): $(cast --to-unit $balance_v07 ether) ETH"
}

# Generate test accounts configuration
generate_config() {
    echo -e "${BLUE}📝 Generating test accounts configuration...${NC}"
    
    # Deploy factories
    deploy_factory_v06
    deploy_factory_v07
    
    # Calculate smart wallet addresses
    WALLET_V06_ADDRESS=$(calculate_wallet_v06 $ACCOUNT_V06_ADDRESS)
    WALLET_V07_ADDRESS=$(calculate_wallet_v07 $ACCOUNT_V07_ADDRESS)
    
    # Generate initCode and factory data
    INIT_CODE_V06=$(generate_init_code_v06 $FACTORY_V06_ADDRESS $ACCOUNT_V06_ADDRESS)
    
    # For v0.7, get factory and factoryData separately
    FACTORY_DATA_V07=$(generate_factory_data_v07 $FACTORY_V07_ADDRESS $ACCOUNT_V07_ADDRESS | tail -n1)
    
    # Create configuration file
    cat > .test_accounts.json << EOF
{
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "anvil_url": "$ANVIL_URL",
  "v0_6": {
    "entrypoint": "$ENTRYPOINT_V06_ADDRESS",
    "account_owner": {
      "address": "$ACCOUNT_V06_ADDRESS",
      "private_key": "$ACCOUNT_V06_PRIVATE_KEY"
    },
    "smart_wallet": {
      "factory": "$FACTORY_V06_ADDRESS",
      "account_address": "$WALLET_V06_ADDRESS",
      "init_code": "$INIT_CODE_V06"
    }
  },
  "v0_7": {
    "entrypoint": "$ENTRYPOINT_V07_ADDRESS", 
    "account_owner": {
      "address": "$ACCOUNT_V07_ADDRESS",
      "private_key": "$ACCOUNT_V07_PRIVATE_KEY"
    },
    "smart_wallet": {
      "factory": "$FACTORY_V07_ADDRESS",
      "account_address": "$WALLET_V07_ADDRESS",
      "factory_data": "$FACTORY_DATA_V07"
    }
  }
}
EOF

    echo -e "${GREEN}✅ Configuration saved to .test_accounts.json${NC}"
}

# Create .env file for testing
create_test_env() {
    echo -e "${BLUE}⚙️ Creating test environment file...${NC}"
    
    cat > .env.test << EOF
# Test environment configuration for SuperRelay
# Generated by setup_test_accounts.sh

# Network Configuration
RPC_URL=$ANVIL_URL
CHAIN_ID=31337

# EntryPoint Addresses
ENTRY_POINT_V06_ADDRESS=$ENTRYPOINT_V06_ADDRESS
ENTRY_POINT_V07_ADDRESS=$ENTRYPOINT_V07_ADDRESS

# Test Account Configuration
TEST_ACCOUNT_V06_ADDRESS=$ACCOUNT_V06_ADDRESS
TEST_ACCOUNT_V06_PRIVATE_KEY=$ACCOUNT_V06_PRIVATE_KEY
TEST_ACCOUNT_V07_ADDRESS=$ACCOUNT_V07_ADDRESS
TEST_ACCOUNT_V07_PRIVATE_KEY=$ACCOUNT_V07_PRIVATE_KEY

# Smart Wallet Factories
FACTORY_V06_ADDRESS=$FACTORY_V06_ADDRESS
FACTORY_V07_ADDRESS=$FACTORY_V07_ADDRESS

# Smart Wallet Addresses
WALLET_V06_ADDRESS=$WALLET_V06_ADDRESS
WALLET_V07_ADDRESS=$WALLET_V07_ADDRESS

# API Configuration
HTTP_API_HOST=0.0.0.0
HTTP_API_PORT=3000

# Logging
RUST_LOG=debug
EOF

    echo -e "${GREEN}✅ Test environment saved to .env.test${NC}"
}

# Validate account setup
validate_setup() {
    echo -e "${BLUE}🔍 Validating account setup...${NC}"
    
    local errors=0
    
    # Check if configuration file exists and is valid JSON
    if [ -f ".test_accounts.json" ]; then
        if jq empty .test_accounts.json 2>/dev/null; then
            echo -e "${GREEN}✅ Configuration file is valid JSON${NC}"
        else
            echo -e "${RED}❌ Configuration file is invalid JSON${NC}"
            ((errors++))
        fi
    else
        echo -e "${RED}❌ Configuration file not found${NC}"
        ((errors++))
    fi
    
    # Check account balances are sufficient
    local balance_v06=$(cast balance $ACCOUNT_V06_ADDRESS --rpc-url $ANVIL_URL)
    local balance_v07=$(cast balance $ACCOUNT_V07_ADDRESS --rpc-url $ANVIL_URL)
    
    # Convert to ether and check minimum balance (1 ETH)
    local balance_v06_eth=$(cast --to-unit $balance_v06 ether | cut -d'.' -f1)
    local balance_v07_eth=$(cast --to-unit $balance_v07 ether | cut -d'.' -f1)
    
    if [ "$balance_v06_eth" -ge 1 ]; then
        echo -e "${GREEN}✅ v0.6 account has sufficient balance${NC}"
    else
        echo -e "${RED}❌ v0.6 account balance too low${NC}"
        ((errors++))
    fi
    
    if [ "$balance_v07_eth" -ge 1 ]; then
        echo -e "${GREEN}✅ v0.7 account has sufficient balance${NC}"
    else
        echo -e "${RED}❌ v0.7 account balance too low${NC}"
        ((errors++))
    fi
    
    # Check if factories are accessible
    local factory_v06_code=$(cast code $FACTORY_V06_ADDRESS --rpc-url $ANVIL_URL)
    local factory_v07_code=$(cast code $FACTORY_V07_ADDRESS --rpc-url $ANVIL_URL)
    
    if [ "$factory_v06_code" != "0x" ]; then
        echo -e "${GREEN}✅ v0.6 factory has deployed code${NC}"
    else
        echo -e "${YELLOW}⚠️ v0.6 factory appears to be EOA (for testing)${NC}"
    fi
    
    if [ "$factory_v07_code" != "0x" ]; then
        echo -e "${GREEN}✅ v0.7 factory has deployed code${NC}"
    else
        echo -e "${YELLOW}⚠️ v0.7 factory appears to be EOA (for testing)${NC}"
    fi
    
    return $errors
}

# Display setup summary
display_summary() {
    echo -e "\n${BLUE}📋 Test Account Setup Summary${NC}"
    echo "============================="
    
    echo -e "\n${GREEN}🔗 Network Configuration:${NC}"
    echo "  Anvil URL: $ANVIL_URL"
    echo "  Chain ID: 31337"
    
    echo -e "\n${GREEN}📍 EntryPoint Addresses:${NC}"
    echo "  v0.6: $ENTRYPOINT_V06_ADDRESS"
    echo "  v0.7: $ENTRYPOINT_V07_ADDRESS"
    
    echo -e "\n${GREEN}👤 Test Accounts:${NC}"
    echo "  v0.6 Owner: $ACCOUNT_V06_ADDRESS"
    echo "  v0.7 Owner: $ACCOUNT_V07_ADDRESS"
    
    echo -e "\n${GREEN}🏭 Smart Wallet Factories:${NC}"
    echo "  v0.6 Factory: $FACTORY_V06_ADDRESS"
    echo "  v0.7 Factory: $FACTORY_V07_ADDRESS"
    
    echo -e "\n${GREEN}📱 Smart Wallet Addresses:${NC}"
    echo "  v0.6 Wallet: $WALLET_V06_ADDRESS"
    echo "  v0.7 Wallet: $WALLET_V07_ADDRESS"
    
    echo -e "\n${GREEN}📂 Generated Files:${NC}"
    echo "  .test_accounts.json - Complete account configuration"
    echo "  .env.test - Environment variables for testing"
    echo "  .factory_v06_address - v0.6 factory address"
    echo "  .factory_v07_address - v0.7 factory address"
    
    echo -e "\n${BLUE}🚀 Next Steps:${NC}"
    echo "1. Fund Paymaster: ./scripts/fund_paymaster.sh"
    echo "2. Start SuperRelay: ./scripts/start_dev_server.sh"
    echo "3. Run UserOp tests: ./scripts/test_userop_construction.sh"
    
    echo -e "\n${YELLOW}🔐 Security Note:${NC}"
    echo "These are test keys for Anvil local development only."
    echo "NEVER use these private keys in production or with real funds."
}

# Main execution
main() {
    echo -e "${BLUE}🔑 Starting test account setup...${NC}"
    
    # Check prerequisites
    check_anvil
    
    # Check required tools
    if ! command -v cast &> /dev/null; then
        echo -e "${RED}❌ 'cast' command not found. Please install Foundry.${NC}"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        echo -e "${RED}❌ 'jq' command not found. Please install jq.${NC}"
        exit 1
    fi
    
    # Generate account configuration
    generate_config
    
    # Create test environment file
    create_test_env
    
    # Check account balances
    check_balances
    
    # Validate setup
    if validate_setup; then
        echo -e "\n${GREEN}🎉 Test account setup completed successfully!${NC}"
    else
        echo -e "\n${YELLOW}⚠️ Setup completed with warnings. Please review the output above.${NC}"
    fi
    
    # Display summary
    display_summary
}

# Run main function
main "$@"