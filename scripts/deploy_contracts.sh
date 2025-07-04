#!/bin/bash
# Deploy all required contracts for SuperPaymaster
# This script deploys EntryPoint and any other required contracts

set -e

# Load environment variables
if [ -f ".env.local" ]; then
    source .env.local
elif [ -f ".env" ]; then
    source .env
fi

# Set default values
RPC_URL=${RPC_URL:-http://localhost:8545}
DEPLOYER_KEY=${DEPLOYER_PRIVATE_KEY:-0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80}
CHAIN_ID=${CHAIN_ID:-31337}

echo "🚀 Deploying SuperPaymaster Contracts"
echo "====================================="
echo "🔗 RPC URL: $RPC_URL"
echo "⛓️  Chain ID: $CHAIN_ID"
echo "👤 Deployer: $(cast wallet address --private-key $DEPLOYER_KEY)"

# Check if blockchain is running
if ! curl -s $RPC_URL >/dev/null 2>&1; then
    echo "❌ Blockchain not accessible at $RPC_URL"
    echo "   Please start Anvil first: ./scripts/start_anvil.sh"
    exit 1
fi

# Check if cast is available
if ! command -v cast >/dev/null 2>&1; then
    echo "❌ Cast not found. Please install Foundry first:"
    echo "   curl -L https://foundry.paradigm.xyz | bash && foundryup"
    exit 1
fi

# Create contracts deployed file
CONTRACTS_FILE="data/deployed_contracts.json"
mkdir -p data

echo "📄 Contract Deployment Log" > data/deployment.log
echo "Deployment Time: $(date)" >> data/deployment.log
echo "RPC URL: $RPC_URL" >> data/deployment.log
echo "Chain ID: $CHAIN_ID" >> data/deployment.log
echo "=================================" >> data/deployment.log

# Initialize JSON file
echo '{}' > $CONTRACTS_FILE

# Function to update contracts file
update_contracts_file() {
    local name=$1
    local address=$2
    local temp_file=$(mktemp)

    jq --arg name "$name" --arg address "$address" \
       '.[$name] = $address' $CONTRACTS_FILE > $temp_file
    mv $temp_file $CONTRACTS_FILE

    echo "📝 $name: $address" >> data/deployment.log
}

# 1. Deploy EntryPoint v0.6
echo ""
echo "📍 Step 1: Deploying EntryPoint v0.6"
echo "=================================="

if [ -f "crates/contracts/contracts/bytecode/entrypoint/0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789.txt" ]; then
    ENTRYPOINT_BYTECODE=$(cat crates/contracts/contracts/bytecode/entrypoint/0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789.txt)

    echo "🚀 Deploying EntryPoint v0.6..."
    DEPLOY_RESULT=$(cast send --rpc-url $RPC_URL --private-key $DEPLOYER_KEY --value 0 --create $ENTRYPOINT_BYTECODE)

    ENTRYPOINT_ADDRESS=$(echo "$DEPLOY_RESULT" | grep "contractAddress" | awk '{print $2}')

    echo "✅ EntryPoint v0.6 deployed at: $ENTRYPOINT_ADDRESS"
    update_contracts_file "EntryPointV06" "$ENTRYPOINT_ADDRESS"

    # Save to legacy file for backward compatibility
    echo $ENTRYPOINT_ADDRESS > .entrypoint_address

    # Verify deployment
    CODE=$(cast code --rpc-url $RPC_URL $ENTRYPOINT_ADDRESS)
    if [ ${#CODE} -gt 2 ]; then
        echo "✅ EntryPoint deployment verified (code length: ${#CODE} chars)"
    else
        echo "❌ EntryPoint deployment failed - no code at address"
        exit 1
    fi
else
    echo "❌ EntryPoint bytecode not found at crates/contracts/contracts/bytecode/entrypoint/"
    echo "   Please make sure the contract bytecode is available"
    exit 1
fi

# 2. Deploy SimpleAccountFactory (for testing)
echo ""
echo "📍 Step 2: Deploying SimpleAccountFactory"
echo "======================================="

# Note: This is a simplified factory for testing
# In production, you might want to deploy the official SimpleAccountFactory
SIMPLE_ACCOUNT_FACTORY_BYTECODE="0x608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80634f1ef2861461003b5780635c60da1b14610050575b600080fd5b61004e6100493660046100a5565b610058565b005b610058610084565b6000819050806001600160a01b0316ff5b6040805160048152602481019091526020810180516001600160e01b03166329092d0e60e01b179052606090815b9392505050565b6000806040838503121561009257600080fd5b50508035926020909101359150565b6000806040838503121561009257600080fdfea2646970667358221220550c5b6b4e93326b3c2f9c8a9b9e1e8b7b7b1b1b1b1b1b1b1b1b1b1b1b1b1b1b64736f6c63430008110033"

echo "🚀 Deploying SimpleAccountFactory (test version)..."
FACTORY_DEPLOY=$(cast send --rpc-url $RPC_URL --private-key $DEPLOYER_KEY --value 0 --create $SIMPLE_ACCOUNT_FACTORY_BYTECODE)
FACTORY_ADDRESS=$(echo "$FACTORY_DEPLOY" | grep "contractAddress" | awk '{print $2}')

if [ -n "$FACTORY_ADDRESS" ]; then
    echo "✅ SimpleAccountFactory deployed at: $FACTORY_ADDRESS"
    update_contracts_file "SimpleAccountFactory" "$FACTORY_ADDRESS"
else
    echo "⚠️  SimpleAccountFactory deployment failed, but this is optional for basic testing"
fi

# 3. Deploy test paymaster (future step - placeholder)
echo ""
echo "📍 Step 3: Test Paymaster Contract (Placeholder)"
echo "=============================================="
echo "⚠️  SuperPaymaster contract will be deployed from a separate repository"
echo "   For now, using EOA as paymaster for testing: $(cast wallet address --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2)"

TEST_PAYMASTER_ADDRESS=$(cast wallet address --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2)
update_contracts_file "TestPaymaster" "$TEST_PAYMASTER_ADDRESS"

# 4. Fund paymaster account with ETH
echo ""
echo "📍 Step 4: Funding Paymaster Account"
echo "=================================="

FUNDING_AMOUNT="10" # 10 ETH
echo "💰 Sending $FUNDING_AMOUNT ETH to paymaster..."

cast send --rpc-url $RPC_URL --private-key $DEPLOYER_KEY --value "${FUNDING_AMOUNT}ether" $TEST_PAYMASTER_ADDRESS
BALANCE=$(cast balance --rpc-url $RPC_URL $TEST_PAYMASTER_ADDRESS)
echo "✅ Paymaster funded with $FUNDING_AMOUNT ETH (Balance: $(cast to-ether $BALANCE) ETH)"

# 5. Update environment variables
echo ""
echo "📍 Step 5: Updating Configuration"
echo "==============================="

# Update .env file with deployed addresses
if [ -f ".env.local" ]; then
    ENV_FILE=".env.local"
elif [ -f ".env" ]; then
    ENV_FILE=".env"
else
    ENV_FILE=".env"
fi

# Create backup
cp $ENV_FILE ${ENV_FILE}.backup

# Update EntryPoint address
sed -i.bak "s/ENTRY_POINT_ADDRESS=.*/ENTRY_POINT_ADDRESS=$ENTRYPOINT_ADDRESS/" $ENV_FILE
sed -i.bak "s/PAYMASTER_ADDRESS=.*/PAYMASTER_ADDRESS=$TEST_PAYMASTER_ADDRESS/" $ENV_FILE

echo "✅ Updated $ENV_FILE with deployed contract addresses"

# 6. Generate summary
echo ""
echo "🎉 Contract Deployment Complete!"
echo "==============================="
echo "📄 All deployed contracts saved to: $CONTRACTS_FILE"
echo "📋 Deployment log saved to: data/deployment.log"
echo ""
echo "📍 Deployed Contracts:"
echo "  • EntryPoint v0.6:      $ENTRYPOINT_ADDRESS"
echo "  • SimpleAccountFactory: ${FACTORY_ADDRESS:-"Not deployed"}"
echo "  • Test Paymaster:       $TEST_PAYMASTER_ADDRESS"
echo ""
echo "⚙️  Configuration Updated:"
echo "  • Environment file: $ENV_FILE"
echo "  • Legacy file: .entrypoint_address"
echo ""
echo "🎯 Next Steps:"
echo "1. Verify contracts in block explorer (if using testnet)"
echo "2. Run tests: ./scripts/test_all.sh"
echo "3. Start Super-Relay: ./scripts/start_super_relay.sh"
echo ""
echo "📖 Contract addresses are also available in:"
echo "   cat $CONTRACTS_FILE | jq ."