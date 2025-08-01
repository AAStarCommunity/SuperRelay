#!/bin/bash
set -e

if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "🚀 Starting SuperRelay Development Environment..."

# Start Anvil
echo "📡 Starting Anvil local testnet..."
anvil --port ${ANVIL_PORT:-8545} --host 0.0.0.0 &
ANVIL_PID=$!

sleep 3

# Deploy EntryPoint if needed
if [ ! -f .entrypoint_address ]; then
    echo "📋 Deploying EntryPoint contract..."
    ./scripts/deploy_entrypoint.sh
fi

# Start SuperRelay
echo "�� Starting SuperRelay service..."
RUST_LOG=info cargo run --bin rundler -- node \
  --network dev \
  --node_http http://localhost:$\{ANVIL_PORT:-8545\} \
  --rpc.host 0.0.0.0 \
  --rpc.port ${RUNDLER_RPC_PORT:-3000} \
  --metrics.port ${METRICS_PORT:-8081} \
  --signer.private_keys ${SIGNER_PRIVATE_KEYS} \
  --paymaster.enabled \
  --paymaster.private_key ${PAYMASTER_PRIVATE_KEY} \
  --paymaster.policy_file ${PAYMASTER_POLICY_FILE} \
  --pool.same_sender_mempool_count 4 \
  --max_verification_gas 10000000 \
  --rpc.api eth,rundler,paymaster &
RUNDLER_PID=$!

echo ""
echo "✅ SuperRelay Development Environment Started!"
echo "🌐 Services:"
echo "  • Anvil RPC: http://localhost:${ANVIL_PORT:-8545}"
echo "  • Rundler RPC: http://localhost:${RUNDLER_RPC_PORT:-3000}"
echo "  • Metrics: http://localhost:${METRICS_PORT:-8081}/metrics"
echo ""
echo "Press Ctrl+C to stop all services..."

cleanup() {
    echo ""
    echo "🛑 Stopping services..."
    [ ! -z "$RUNDLER_PID" ] && kill $RUNDLER_PID 2>/dev/null || true
    [ ! -z "$ANVIL_PID" ] && kill $ANVIL_PID 2>/dev/null || true
    exit 0
}

trap cleanup INT TERM
wait
