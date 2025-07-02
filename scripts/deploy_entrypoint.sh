#!/bin/bash
# Deploy EntryPoint contract to local anvil network

set -e

# Check if anvil is running
if ! nc -z localhost 8545; then
    echo "Error: Anvil is not running on port 8545"
    echo "Please start anvil with: anvil --port 8545 --host 0.0.0.0 --chain-id 31337"
    exit 1
fi

echo "Deploying EntryPoint v0.6 contract to anvil..."

# EntryPoint v0.6 address: 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789
ENTRYPOINT_ADDRESS="0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
DEPLOYER_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"  # First anvil key

# Read the deployment bytecode
BYTECODE=$(cat crates/contracts/contracts/bytecode/entrypoint/0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789_deployed.txt)

# Deploy using cast
echo "Deploying to address: $ENTRYPOINT_ADDRESS"
cast send --rpc-url http://localhost:8545 --private-key $DEPLOYER_KEY --value 0 --create $BYTECODE

# Verify deployment
echo "Verifying deployment..."
CODE=$(cast code --rpc-url http://localhost:8545 $ENTRYPOINT_ADDRESS)
if [ ${#CODE} -gt 2 ]; then
    echo "✅ EntryPoint deployed successfully at $ENTRYPOINT_ADDRESS"
    echo "Contract code length: ${#CODE} characters"
else
    echo "❌ Deployment failed - no code at address"
    exit 1
fi

echo ""
echo "🎉 Local blockchain setup complete!"
echo "📍 EntryPoint v0.6: $ENTRYPOINT_ADDRESS"
echo "🔗 RPC URL: http://localhost:8545"
echo "⛓️  Chain ID: 31337"
echo ""
echo "Available test accounts:"
echo "Account #0: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (10000 ETH)"
echo "Private Key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
echo ""
echo "Account #1: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 (10000 ETH)"  
echo "Private Key: 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2" 