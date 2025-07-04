#!/bin/bash
# Integration test script for super-relay

set -e

echo "🧪 Starting Super-Relay Integration Tests"

# Check prerequisites
if ! lsof -i :8545 >/dev/null 2>&1; then
    echo "❌ Anvil not running on port 8545"
    echo "Please start anvil first: anvil --port 8545 --host 0.0.0.0 --chain-id 31337"
    exit 1
fi

if [ ! -f ".entrypoint_address" ]; then
    echo "❌ EntryPoint contract not deployed"
    echo "Please deploy EntryPoint first: ./scripts/deploy_entrypoint.sh"
    exit 1
fi

ENTRYPOINT_ADDRESS=$(cat .entrypoint_address)
echo "📍 Using EntryPoint at: $ENTRYPOINT_ADDRESS"

# Kill any existing rundler process
echo "🧹 Cleaning up any existing processes..."
pkill -f rundler || true
sleep 2

# Start super-relay in background
echo "🚀 Starting Super-Relay server..."
RUST_LOG=debug cargo run --bin rundler -- node \
    --node_http http://localhost:8545 \
    --paymaster.enabled \
    --paymaster.private_key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2 \
    --paymaster.policy_file config/paymaster-policies.toml \
    --max_verification_gas 10000000 \
    --rpc.port 3000 \
    --rpc.host 0.0.0.0 &

RUNDLER_PID=$!
echo "📊 Super-Relay started with PID: $RUNDLER_PID"

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 5

# Function to cleanup
cleanup() {
    echo "🧹 Cleaning up..."
    kill $RUNDLER_PID 2>/dev/null || true
    wait $RUNDLER_PID 2>/dev/null || true
}
trap cleanup EXIT

# Test 1: Check if server is responding
echo "🔍 Test 1: Server health check"
if curl -s http://localhost:3000 >/dev/null; then
    echo "✅ Server is responding"
else
    echo "❌ Server is not responding"
    exit 1
fi

# Test 2: Test pm_sponsorUserOperation endpoint
echo "🔍 Test 2: Testing pm_sponsorUserOperation endpoint"

# Create a test UserOperation
USER_OP='{
    "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    "nonce": "0x0",
    "initCode": "0x",
    "callData": "0x",
    "callGasLimit": "0x186A0",
    "verificationGasLimit": "0x186A0",
    "preVerificationGas": "0x5208",
    "maxFeePerGas": "0x3B9ACA00",
    "maxPriorityFeePerGas": "0x3B9ACA00",
    "paymasterAndData": "0x",
    "signature": "0x"
}'

# Test the RPC call
echo "📤 Calling pm_sponsorUserOperation..."
RESPONSE=$(curl -s -X POST http://localhost:3000 \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "pm_sponsorUserOperation",
        "params": ['"$USER_OP"', "'"$ENTRYPOINT_ADDRESS"'"],
        "id": 1
    }')

echo "📥 Response: $RESPONSE"

# Check if we got a valid response
if echo "$RESPONSE" | grep -q '"result"'; then
    echo "✅ pm_sponsorUserOperation endpoint is working"
else
    echo "❌ pm_sponsorUserOperation endpoint failed"
    echo "Response: $RESPONSE"
fi

# Test 3: Check basic RPC endpoints
echo "🔍 Test 3: Testing standard RPC endpoints"

# Test eth_supportedEntryPoints
echo "📤 Calling eth_supportedEntryPoints..."
ENTRY_POINTS_RESPONSE=$(curl -s -X POST http://localhost:3000 \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "eth_supportedEntryPoints",
        "params": [],
        "id": 2
    }')

echo "📥 Supported Entry Points: $ENTRY_POINTS_RESPONSE"

if echo "$ENTRY_POINTS_RESPONSE" | grep -q "$ENTRYPOINT_ADDRESS"; then
    echo "✅ EntryPoint is properly configured"
else
    echo "❌ EntryPoint configuration issue"
fi

echo ""
echo "🎉 Integration tests completed!"
echo "📊 Test Results Summary:"
echo "  - Server Health: ✅"
echo "  - Paymaster Relay API: $(echo "$RESPONSE" | grep -q '"result"' && echo '✅' || echo '❌')"
echo "  - EntryPoint Config: $(echo "$ENTRY_POINTS_RESPONSE" | grep -q "$ENTRYPOINT_ADDRESS" && echo '✅' || echo '❌')"
echo ""
echo "🔗 API Endpoint: http://localhost:3000"
echo "📖 Test your API with tools like curl or Postman"