#!/bin/bash
# set -ex # Exit on error, print commands

echo "🚀 Running Super-Relay Integration Tests..."

# 1. Check if server is running
if [ ! -f "scripts/.rundler.pid" ]; then
    echo "❌ Super-Relay server not running."
    echo "Please start the server in another terminal with: ./scripts/start_dev_server.sh"
    exit 1
fi

RUNDLER_PID=$(cat scripts/.rundler.pid)
echo "✅ Super-Relay server is running with PID: $RUNDLER_PID"

# 2. Get EntryPoint address
if [ ! -f ".entrypoint_address" ]; then
    echo "❌ EntryPoint address file not found."
    echo "Please ensure start_dev_server.sh has run successfully."
    exit 1
fi
ENTRYPOINT_ADDRESS=$(cat .entrypoint_address)
echo "📍 Using EntryPoint at: $ENTRYPOINT_ADDRESS"

# 3. Test pm_sponsorUserOperation
echo "📤 Calling pm_sponsorUserOperation with a minimal valid UserOp..."

# This is a sample UserOperation. In a real scenario, many of these values
# would be calculated by a client library. For this test, we use minimal
# valid values to ensure the RPC endpoint is responsive.
USER_OP_SENDER="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
USER_OP='{
    "sender": "'$USER_OP_SENDER'",
    "nonce": "0x0",
    "initCode": "0x",
    "callData": "0xdeadbeef",
    "callGasLimit": "0x55555",
    "verificationGasLimit": "0x186A0",
    "preVerificationGas": "0xC350",
    "maxFeePerGas": "0x1",
    "maxPriorityFeePerGas": "0x1",
    "paymasterAndData": "0x",
    "signature": "0xdeadbeef"
}'

# Use a HEREDOC to build the JSON payload, ensuring the UserOp is a compact, single-line string.
USER_OP_COMPACT=$(echo "$USER_OP" | tr -d '\\n\\r\\t ')
JSON_PAYLOAD=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "pm_sponsorUserOperation",
    "params": [
        ${USER_OP_COMPACT},
        "${ENTRYPOINT_ADDRESS}"
    ],
    "id": 1
}
EOF
)

RESPONSE=$(curl -v --max-time 15 -s -X POST http://localhost:3000 \
    -H "Content-Type: application/json" \
    -d "${JSON_PAYLOAD}")

echo "Full server response:"
echo "$RESPONSE"
echo "-------------------"

# Check for a result field first
if ! echo "$RESPONSE" | jq -e '.result' > /dev/null; then
    echo "❌ Test FAILED: pm_sponsorUserOperation did not return a 'result' field."
    exit 1
fi

# Check that paymasterAndData is not null or empty
PAYMASTER_AND_DATA=$(echo "$RESPONSE" | jq -r '.result.paymasterAndData')
if [ -z "$PAYMASTER_AND_DATA" ] || [ "$PAYMASTER_AND_DATA" == "null" ] || [ "$PAYMASTER_AND_DATA" == "0x" ]; then
    echo "❌ Test FAILED: pm_sponsorUserOperation returned empty 'paymasterAndData'."
    echo "This might indicate the paymaster logic rejected the UserOp."
    exit 1
fi

echo "✅ Test PASSED: pm_sponsorUserOperation returned a valid paymasterAndData."


# 4. Test eth_supportedEntryPoints
echo "📤 Calling eth_supportedEntryPoints..."
ENTRY_POINTS_RESPONSE=$(curl -s -X POST http://localhost:3000 \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "eth_supportedEntryPoints",
        "params": [],
        "id": 2
    }')

echo "📥 Response: $ENTRY_POINTS_RESPONSE"

if ! echo "$ENTRY_POINTS_RESPONSE" | jq -e --arg addr "$ENTRYPOINT_ADDRESS" '.result | index($addr)' > /dev/null; then
    echo "❌ Test FAILED: eth_supportedEntryPoints did not contain the deployed EntryPoint address."
    exit 1
fi
echo "✅ Test PASSED: eth_supportedEntryPoints contains the correct EntryPoint."

echo -e "\n🎉 All tests passed successfully!"