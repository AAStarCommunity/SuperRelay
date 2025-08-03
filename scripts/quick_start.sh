#!/bin/bash
set -e

if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "🚀 Starting SuperRelay v0.1.5 Gateway Development Environment..."

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

# Start SuperRelay Gateway
echo "🌐 Starting SuperRelay Gateway service..."
RUST_LOG=info cargo run --bin super-relay -- gateway \
  --config config/config.toml \
  --host 0.0.0.0 \
  --port ${RUNDLER_RPC_PORT:-3000} \
  --enable-paymaster \
  --paymaster-private-key ${PAYMASTER_PRIVATE_KEY} &
RUNDLER_PID=$!

echo ""
echo "✅ SuperRelay Gateway Development Environment Started!"
echo "🌐 Services:"
echo "  • Anvil RPC: http://localhost:${ANVIL_PORT:-8545}"
echo "  • Gateway API: http://localhost:${RUNDLER_RPC_PORT:-3000}"
echo "  • Health Check: http://localhost:${RUNDLER_RPC_PORT:-3000}/health"
echo "  • Metrics: http://localhost:${RUNDLER_RPC_PORT:-3000}/metrics"
echo ""
echo "🎯 Architecture:"
echo "  • SuperRelay Gateway = Single binary with internal routing"
echo "  • Paymaster Service = Integrated gas sponsorship"
echo "  • Rundler Components = Internal method calls"
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