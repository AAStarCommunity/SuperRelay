#!/bin/bash
# Setup test accounts for Sepolia testnet
# Creates test accounts with private keys and smart wallet configurations for Sepolia

set -e

echo "🌍 Setting up SuperRelay test accounts for Sepolia"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Sepolia Configuration
SEPOLIA_RPC_URL="${SEPOLIA_RPC_URL:-https://ethereum-sepolia-rpc.publicnode.com}"
SEPOLIA_CHAIN_ID="11155111"
ENTRYPOINT_V06_ADDRESS="0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
ENTRYPOINT_V07_ADDRESS="0x0000000071727De22E5E9d8BAf0edAc6f37da032"

# You need to provide your own test accounts for Sepolia
# NEVER use these keys with real funds on mainnet
TEST_ACCOUNT_V06_PRIVATE_KEY="${TEST_ACCOUNT_V06_PRIVATE_KEY:-}"
TEST_ACCOUNT_V07_PRIVATE_KEY="${TEST_ACCOUNT_V07_PRIVATE_KEY:-}"

# Smart wallet factory addresses on Sepolia (example addresses)
FACTORY_V06_ADDRESS="0x9406Cc6185a346906296840746125a0E44976454"
FACTORY_V07_ADDRESS="0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67"

# Check if private keys are provided
check_private_keys() {
    echo -e "${BLUE}🔐 Checking private key configuration...${NC}"

    if [ -z "$TEST_ACCOUNT_V06_PRIVATE_KEY" ] || [ -z "$TEST_ACCOUNT_V07_PRIVATE_KEY" ]; then
        echo -e "${RED}❌ Private keys not configured for Sepolia${NC}"
        echo -e "${YELLOW}🔧 Please set environment variables:${NC}"
        echo "  export TEST_ACCOUNT_V06_PRIVATE_KEY=0x..."
        echo "  export TEST_ACCOUNT_V07_PRIVATE_KEY=0x..."
        echo ""
        echo -e "${YELLOW}⚠️ Security Notice:${NC}"
        echo "  - Use test accounts with small amounts of Sepolia ETH only"
        echo "  - NEVER use mainnet accounts or accounts with real funds"
        echo "  - Consider using a separate wallet for testing"
        echo ""
        echo -e "${BLUE}💡 To get Sepolia test ETH:${NC}"
        echo "  - https://sepoliafaucet.com/"
        echo "  - https://www.alchemy.com/faucets/ethereum-sepolia"
        echo "  - https://sepolia-faucet.pk910.de/"
        exit 1
    fi

    echo -e "${GREEN}✅ Private keys configured${NC}"
}

# Check Sepolia network connectivity
check_sepolia_connection() {
    echo -e "${BLUE}🌐 Checking Sepolia network connection...${NC}"

    local response=$(curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        "$SEPOLIA_RPC_URL")

    local chain_id=$(echo "$response" | jq -r '.result // "null"')
    local expected_chain_id="0x$(printf '%x' $SEPOLIA_CHAIN_ID)"

    if [ "$chain_id" = "$expected_chain_id" ]; then
        echo -e "${GREEN}✅ Connected to Sepolia (Chain ID: $chain_id)${NC}"
    else
        echo -e "${RED}❌ Failed to connect to Sepolia${NC}"
        echo "  Expected Chain ID: $expected_chain_id"
        echo "  Received: $chain_id"
        echo "  RPC URL: $SEPOLIA_RPC_URL"
        exit 1
    fi
}

# Get account address from private key
get_address_from_private_key() {
    local private_key=$1
    # Use cast to derive address from private key
    cast wallet address "$private_key"
}

# Check account balances on Sepolia
check_sepolia_balances() {
    echo -e "${BLUE}💰 Checking Sepolia account balances...${NC}"

    local v06_address=$(get_address_from_private_key "$TEST_ACCOUNT_V06_PRIVATE_KEY")
    local v07_address=$(get_address_from_private_key "$TEST_ACCOUNT_V07_PRIVATE_KEY")

    local balance_v06=$(cast balance "$v06_address" --rpc-url "$SEPOLIA_RPC_URL")
    local balance_v07=$(cast balance "$v07_address" --rpc-url "$SEPOLIA_RPC_URL")

    echo -e "${GREEN}📊 Account balances:${NC}"
    echo "  v0.6 Account ($v06_address): $(cast --to-unit $balance_v06 ether) ETH"
    echo "  v0.7 Account ($v07_address): $(cast --to-unit $balance_v07 ether) ETH"

    # Check minimum balance (0.01 ETH for testing)
    local min_balance_wei="10000000000000000" # 0.01 ETH in wei

    if [ "$balance_v06" -lt "$min_balance_wei" ]; then
        echo -e "${YELLOW}⚠️ v0.6 account balance low. Consider funding from faucet.${NC}"
    fi

    if [ "$balance_v07" -lt "$min_balance_wei" ]; then
        echo -e "${YELLOW}⚠️ v0.7 account balance low. Consider funding from faucet.${NC}"
    fi
}

# Calculate smart wallet address (simplified)
calculate_wallet_address() {
    local owner_address=$1
    local factory_address=$2
    local salt="0x0000000000000000000000000000000000000000000000000000000000000000"

    # This is a simplified calculation for demo purposes
    # Real implementation would use CREATE2 with proper factory bytecode
    local combined="${factory_address}${owner_address:2}${salt:2}"
    local hash=$(echo -n "$combined" | xxd -r -p | sha256sum | cut -d' ' -f1)
    echo "0x${hash:24:40}"
}

# Generate Sepolia test configuration
generate_sepolia_config() {
    echo -e "${BLUE}📝 Generating Sepolia test configuration...${NC}"

    local v06_address=$(get_address_from_private_key "$TEST_ACCOUNT_V06_PRIVATE_KEY")
    local v07_address=$(get_address_from_private_key "$TEST_ACCOUNT_V07_PRIVATE_KEY")

    local wallet_v06_address=$(calculate_wallet_address "$v06_address" "$FACTORY_V06_ADDRESS")
    local wallet_v07_address=$(calculate_wallet_address "$v07_address" "$FACTORY_V07_ADDRESS")

    # Generate initCode for v0.6
    local init_code_v06="${FACTORY_V06_ADDRESS}5fbfb9cf000000000000000000000000${v06_address:2}0000000000000000000000000000000000000000000000000000000000000000"

    # Generate factory data for v0.7
    local factory_data_v07="5fbfb9cf000000000000000000000000${v07_address:2}0000000000000000000000000000000000000000000000000000000000000000"

    # Create configuration file
    cat > .test_accounts_sepolia.json << EOF
{
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "network": "sepolia",
  "chain_id": "$SEPOLIA_CHAIN_ID",
  "rpc_url": "$SEPOLIA_RPC_URL",
  "v0_6": {
    "entrypoint": "$ENTRYPOINT_V06_ADDRESS",
    "account_owner": {
      "address": "$v06_address",
      "private_key": "$TEST_ACCOUNT_V06_PRIVATE_KEY"
    },
    "smart_wallet": {
      "factory": "$FACTORY_V06_ADDRESS",
      "account_address": "$wallet_v06_address",
      "init_code": "$init_code_v06"
    }
  },
  "v0_7": {
    "entrypoint": "$ENTRYPOINT_V07_ADDRESS",
    "account_owner": {
      "address": "$v07_address",
      "private_key": "$TEST_ACCOUNT_V07_PRIVATE_KEY"
    },
    "smart_wallet": {
      "factory": "$FACTORY_V07_ADDRESS",
      "account_address": "$wallet_v07_address",
      "factory_data": "$factory_data_v07"
    }
  }
}
EOF

    echo -e "${GREEN}✅ Sepolia configuration saved to .test_accounts_sepolia.json${NC}"
}

# Create Sepolia environment file
create_sepolia_env() {
    echo -e "${BLUE}⚙️ Creating Sepolia environment file...${NC}"

    local v06_address=$(get_address_from_private_key "$TEST_ACCOUNT_V06_PRIVATE_KEY")
    local v07_address=$(get_address_from_private_key "$TEST_ACCOUNT_V07_PRIVATE_KEY")

    cat > .env.sepolia << EOF
# Sepolia testnet configuration for SuperRelay
# Generated by setup_test_accounts_sepolia.sh

# Network Configuration
RPC_URL=$SEPOLIA_RPC_URL
CHAIN_ID=$SEPOLIA_CHAIN_ID
NETWORK=sepolia

# EntryPoint Addresses
ENTRY_POINT_V06_ADDRESS=$ENTRYPOINT_V06_ADDRESS
ENTRY_POINT_V07_ADDRESS=$ENTRYPOINT_V07_ADDRESS

# Test Account Configuration
TEST_ACCOUNT_V06_ADDRESS=$v06_address
TEST_ACCOUNT_V06_PRIVATE_KEY=$TEST_ACCOUNT_V06_PRIVATE_KEY
TEST_ACCOUNT_V07_ADDRESS=$v07_address
TEST_ACCOUNT_V07_PRIVATE_KEY=$TEST_ACCOUNT_V07_PRIVATE_KEY

# Smart Wallet Factories
FACTORY_V06_ADDRESS=$FACTORY_V06_ADDRESS
FACTORY_V07_ADDRESS=$FACTORY_V07_ADDRESS

# API Configuration
HTTP_API_HOST=0.0.0.0
HTTP_API_PORT=3000

# Logging
RUST_LOG=info

# SuperRelay Configuration
SUPER_RELAY_URL=http://localhost:3000
EOF

    echo -e "${GREEN}✅ Sepolia environment saved to .env.sepolia${NC}"
}

# Validate Sepolia setup
validate_sepolia_setup() {
    echo -e "${BLUE}🔍 Validating Sepolia setup...${NC}"

    local errors=0

    # Check configuration file
    if [ -f ".test_accounts_sepolia.json" ]; then
        if jq empty .test_accounts_sepolia.json 2>/dev/null; then
            echo -e "${GREEN}✅ Sepolia configuration file is valid JSON${NC}"
        else
            echo -e "${RED}❌ Sepolia configuration file is invalid JSON${NC}"
            ((errors++))
        fi
    else
        echo -e "${RED}❌ Sepolia configuration file not found${NC}"
        ((errors++))
    fi

    # Check EntryPoint contracts on Sepolia
    echo -e "${BLUE}🔍 Checking EntryPoint contracts on Sepolia...${NC}"

    local ep_v06_code=$(cast code "$ENTRYPOINT_V06_ADDRESS" --rpc-url "$SEPOLIA_RPC_URL")
    local ep_v07_code=$(cast code "$ENTRYPOINT_V07_ADDRESS" --rpc-url "$SEPOLIA_RPC_URL")

    if [ "$ep_v06_code" != "0x" ] && [ ${#ep_v06_code} -gt 10 ]; then
        echo -e "${GREEN}✅ EntryPoint v0.6 deployed on Sepolia${NC}"
    else
        echo -e "${YELLOW}⚠️ EntryPoint v0.6 not found on Sepolia${NC}"
    fi

    if [ "$ep_v07_code" != "0x" ] && [ ${#ep_v07_code} -gt 10 ]; then
        echo -e "${GREEN}✅ EntryPoint v0.7 deployed on Sepolia${NC}"
    else
        echo -e "${YELLOW}⚠️ EntryPoint v0.7 not found on Sepolia${NC}"
    fi

    return $errors
}

# Display Sepolia setup summary
display_sepolia_summary() {
    echo -e "\n${BLUE}📋 Sepolia Test Account Setup Summary${NC}"
    echo "======================================"

    local v06_address=$(get_address_from_private_key "$TEST_ACCOUNT_V06_PRIVATE_KEY")
    local v07_address=$(get_address_from_private_key "$TEST_ACCOUNT_V07_PRIVATE_KEY")

    echo -e "\n${GREEN}🌍 Network Configuration:${NC}"
    echo "  Network: Sepolia Testnet"
    echo "  Chain ID: $SEPOLIA_CHAIN_ID"
    echo "  RPC URL: $SEPOLIA_RPC_URL"

    echo -e "\n${GREEN}📍 EntryPoint Addresses:${NC}"
    echo "  v0.6: $ENTRYPOINT_V06_ADDRESS"
    echo "  v0.7: $ENTRYPOINT_V07_ADDRESS"

    echo -e "\n${GREEN}👤 Test Accounts:${NC}"
    echo "  v0.6 Owner: $v06_address"
    echo "  v0.7 Owner: $v07_address"

    echo -e "\n${GREEN}🏭 Smart Wallet Factories:${NC}"
    echo "  v0.6 Factory: $FACTORY_V06_ADDRESS"
    echo "  v0.7 Factory: $FACTORY_V07_ADDRESS"

    echo -e "\n${GREEN}📂 Generated Files:${NC}"
    echo "  .test_accounts_sepolia.json - Complete account configuration"
    echo "  .env.sepolia - Environment variables for Sepolia testing"

    echo -e "\n${BLUE}🚀 Next Steps:${NC}"
    echo "1. Start SuperRelay with Sepolia config:"
    echo "   source .env.sepolia && ./scripts/start_dev_server.sh"
    echo "2. Run Sepolia tests:"
    echo "   ./scripts/test_userop_construction_sepolia.sh"
    echo "3. Monitor test results and balances"

    echo -e "\n${YELLOW}🔐 Security Reminders:${NC}"
    echo "• These are test keys for Sepolia testnet only"
    echo "• Keep private keys secure and never commit to git"
    echo "• Use separate accounts for production deployments"
    echo "• Monitor account balances and refill from faucets as needed"

    echo -e "\n${BLUE}💰 Faucet Links:${NC}"
    echo "• https://sepoliafaucet.com/"
    echo "• https://www.alchemy.com/faucets/ethereum-sepolia"
    echo "• https://sepolia-faucet.pk910.de/"
}

# Main execution
main() {
    echo -e "${BLUE}🌍 Starting Sepolia test account setup...${NC}"

    # Check required tools
    if ! command -v cast &> /dev/null; then
        echo -e "${RED}❌ 'cast' command not found. Please install Foundry.${NC}"
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        echo -e "${RED}❌ 'jq' command not found. Please install jq.${NC}"
        exit 1
    fi

    if ! command -v curl &> /dev/null; then
        echo -e "${RED}❌ 'curl' command not found. Please install curl.${NC}"
        exit 1
    fi

    # Check prerequisites
    check_private_keys
    check_sepolia_connection

    # Check account balances
    check_sepolia_balances

    # Generate configuration
    generate_sepolia_config

    # Create environment file
    create_sepolia_env

    # Validate setup
    if validate_sepolia_setup; then
        echo -e "\n${GREEN}🎉 Sepolia test account setup completed successfully!${NC}"
    else
        echo -e "\n${YELLOW}⚠️ Setup completed with warnings. Please review the output above.${NC}"
    fi

    # Display summary
    display_sepolia_summary
}

# Run main function
main "$@"