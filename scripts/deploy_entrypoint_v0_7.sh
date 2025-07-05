#!/bin/bash
# Deploy EntryPoint contract to local anvil network

set -e

# Check if anvil is running
if ! nc -z localhost 8545; then
    echo "Error: Anvil is not running on port 8545"
    echo "Please start anvil with: anvil --port 8545 --host 0.0.0.0 --chain-id 31337"
    exit 1
fi

echo "Deploying EntryPoint v0.7 contract to anvil..."

DEPLOYER_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"  # First anvil key

# TODO: Add the bytecode for the v0.7 EntryPoint contract
BYTECODE=""

if [ -z "$BYTECODE" ]; then
    echo "❌ Error: BYTECODE for EntryPoint v0.7 is not set in deploy_entrypoint_v0_7.sh"
    exit 1
fi

# Deploy using cast and capture the deployed address
echo "Deploying EntryPoint contract..."
DEPLOY_RESULT=$(cast send --rpc-url http://localhost:8545 --private-key $DEPLOYER_KEY --value 0 --create $BYTECODE)

# Extract the actual deployed address
DEPLOYED_ADDRESS=$(echo "$DEPLOY_RESULT" | grep "contractAddress" | awk '{print $2}')

echo "Contract deployed at: $DEPLOYED_ADDRESS"

# Verify deployment
echo "Verifying deployment..."
CODE=$(cast code --rpc-url http://localhost:8545 $DEPLOYED_ADDRESS)
if [ ${#CODE} -gt 2 ]; then
    echo "✅ EntryPoint deployed successfully at $DEPLOYED_ADDRESS"
    echo "Contract code length: ${#CODE} characters"

    # Save the deployed address for later use
    echo $DEPLOYED_ADDRESS > .entrypoint_v0_7_address
    echo "📝 Contract address saved to .entrypoint_v0_7_address"
else
    echo "❌ Deployment failed - no code at address"
    exit 1
fi