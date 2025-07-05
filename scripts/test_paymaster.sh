#!/bin/bash

# A simple, focused script to test paymaster functionality.
# This version uses the correct `cast` syntax.

set -e

# --- Configuration ---
ANVIL_RPC_URL="http://localhost:8545"
PAYMASTER_RPC_URL="http://localhost:3000"
DEPLOYER_ADDRESS="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266" # Anvil default account 0
PAYMASTER_SIGNER_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d" # Anvil default key 1
BUNDLER_SIGNER_KEY="0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a" # Anvil default key 2
BYTECODE=$(cat crates/contracts/contracts/bytecode/entrypoint/0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789.txt)

# --- Cleanup function ---
cleanup() {
    echo -e "\n🧹 Cleaning up..."
    if [ ! -z "$RUNDLER_PID" ]; then
        kill $RUNDLER_PID 2>/dev/null || true
    fi
    if [ ! -z "$ANVIL_PID" ]; then
        kill $ANVIL_PID 2>/dev/null || true
    fi
    echo "✅ Done."
}
trap cleanup EXIT

# 1. Start Anvil
echo "🔥 Starting Anvil..."
anvil --port 8545 > /dev/null 2>&1 &
ANVIL_PID=$!
sleep 2

# 2. Deploy EntryPoint
echo "📦 Deploying EntryPoint v0.6..."
# `cast send --create` uses an unlocked account. We specify the sender with ETH_FROM.
export ETH_FROM="$DEPLOYER_ADDRESS"
DEPLOY_RESULT=$(cast send --create "$BYTECODE")
unset ETH_FROM
ENTRYPOINT_ADDRESS=$(echo "$DEPLOY_RESULT" | grep "contractAddress" | awk '{print $2}')
echo "✅ EntryPoint deployed to: $ENTRYPOINT_ADDRESS"

# 3. Deposit funds for Paymaster
echo "💰 Depositing 1 ETH for Paymaster..."
cast send "$ENTRYPOINT_ADDRESS" "deposit()" --value 1ether --private-key "$PAYMASTER_SIGNER_KEY" --rpc-url "$ANVIL_RPC_URL" > /dev/null
echo "✅ Deposit successful."

# 4. Start SuperRelay
echo "🚀 Starting SuperRelay server..."
RUST_LOG=debug cargo run --bin rundler -- node \
    --node_http "$ANVIL_RPC_URL" \
    --signer.private_keys "$BUNDLER_SIGNER_KEY" \
    --paymaster.enabled \
    --paymaster.private_key "$PAYMASTER_SIGNER_KEY" \
    --rpc.port 3000 > rundler.log 2>&1 &
RUNDLER_PID=$!
sleep 10

# 5. Run Test
echo "📤 Calling pm_sponsorUserOperation..."
USER_OP='{"sender":"0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266","nonce":"0x0","initCode":"0x","callData":"0x","callGasLimit":"0x0","verificationGasLimit":"0x0","preVerificationGas":"0x0","maxFeePerGas":"0x0","maxPriorityFeePerGas":"0x0","paymasterAndData":"0x","signature":"0x"}'
RESPONSE=$(curl -s -X POST "$PAYMASTER_RPC_URL" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc": "2.0", "method": "pm_sponsorUserOperation", "params": ['$USER_OP', "'"$ENTRYPOINT_ADDRESS"'"], "id": 1}')

echo "📥 Response: $RESPONSE"

if [[ -z "$RESPONSE" ]] || ! echo "$RESPONSE" | jq -e '.result' > /dev/null; then
    echo "❌ Test FAILED: pm_sponsorUserOperation did not return a valid result."
    echo "--- Server logs ---"
    cat rundler.log
    echo "-------------------"
    exit 1
fi

echo "✅ Test PASSED: pm_sponsorUserOperation returned a valid result."
echo "🎉 Paymaster test successful!"