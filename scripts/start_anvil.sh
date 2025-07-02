#!/bin/bash
# Start Anvil local blockchain for SuperPaymaster testing

set -e

# Load environment variables if available
if [ -f ".env.local" ]; then
    source .env.local
elif [ -f ".env" ]; then
    source .env
fi

# Set default values
CHAIN_ID=${CHAIN_ID:-31337}
HOST=${HOST:-0.0.0.0}
PORT=${ANVIL_PORT:-8545}

echo "🔗 Starting Anvil Local Blockchain"
echo "================================="
echo "Host: $HOST"
echo "Port: $PORT"
echo "Chain ID: $CHAIN_ID"

# Check if anvil is installed
if ! command -v anvil >/dev/null 2>&1; then
    echo "❌ Anvil not found. Please install Foundry first:"
    echo "   curl -L https://foundry.paradigm.xyz | bash"
    echo "   foundryup"
    exit 1
fi

# Check if port is already in use
if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "⚠️  Port $PORT is already in use. Attempting to kill existing process..."
    lsof -Pi :$PORT -sTCP:LISTEN -t | xargs kill -9 2>/dev/null || true
    sleep 2
fi

# Create logs directory
mkdir -p logs

# Start anvil with predefined accounts and settings
echo "🚀 Starting Anvil..."
anvil \
    --host $HOST \
    --port $PORT \
    --chain-id $CHAIN_ID \
    --accounts 10 \
    --balance 10000 \
    --gas-limit 30000000 \
    --gas-price 1000000000 \
    --base-fee 1000000000 \
    --block-time 1 \
    --fork-block-number 0 \
    --silent \
    > logs/anvil.log 2>&1 &

ANVIL_PID=$!
echo "📊 Anvil started with PID: $ANVIL_PID"
echo "💾 Logs are being written to: logs/anvil.log"

# Save PID for cleanup
echo $ANVIL_PID > .anvil.pid

# Wait a moment for anvil to start
sleep 3

# Verify anvil is running
if curl -s -X POST -H "Content-Type: application/json" \
   --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
   http://$HOST:$PORT >/dev/null 2>&1; then
    echo "✅ Anvil is running successfully!"
else
    echo "❌ Anvil failed to start. Check logs/anvil.log for details."
    exit 1
fi

echo ""
echo "🎯 Blockchain Information:"
echo "🔗 RPC URL: http://$HOST:$PORT"
echo "⛓️  Chain ID: $CHAIN_ID"
echo "⛽ Gas Limit: 30,000,000"
echo "💰 Default Balance: 10,000 ETH per account"
echo ""
echo "👥 Test Accounts (all have 10,000 ETH):"
echo "Account #0: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
echo "Private Key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
echo ""
echo "Account #1: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
echo "Private Key: 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2"
echo ""
echo "Account #2: 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
echo "Private Key: 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"
echo ""
echo "⚠️  To stop Anvil, run: ./scripts/stop_anvil.sh"
echo "📊 To monitor logs: tail -f logs/anvil.log" 