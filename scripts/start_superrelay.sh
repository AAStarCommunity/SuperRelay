#!/bin/bash
# SuperRelay startup script
# Use super-relay wrapper instead of calling rundler directly

set -e
trap "cleanup" EXIT

echo "🚀 SuperRelay v0.1.5 - Enterprise API Gateway Starting"
echo "🌐 Single Binary Gateway Mode with Internal Routing"
echo "======================================"

# Load development environment configuration (try multiple config files)
if [ -f ".env" ]; then
    echo "📁 Loading environment config: .env"
    source .env
    echo "✅ Environment configuration loaded"
elif [ -f ".env.dev" ]; then
    echo "📁 Loading development environment config: .env.dev"
    source .env.dev
    echo "✅ Environment configuration loaded"
else
    echo "⚠️ No .env or .env.dev file found, using default configuration"
    # Set default values
    export SIGNER_PRIVATE_KEYS="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80,0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2"
    export PAYMASTER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    export RPC_URL="http://localhost:8545"
    export NETWORK="dev"
    export CHAIN_ID="31337"
fi

# File paths
ANVIL_PID_FILE="scripts/.anvil.pid"
SUPERRELAY_PID_FILE="scripts/.superrelay.pid"
ENTRYPOINT_ADDRESS_FILE=".entrypoint_address"

# Create scripts directory
mkdir -p scripts

# Cleanup function
cleanup() {
    echo -e "\n🧹 Cleaning up resources..."
    if [ -f "$ANVIL_PID_FILE" ]; then
        echo "❌ Stopping Anvil..."
        kill $(cat $ANVIL_PID_FILE) 2>/dev/null || true
        rm -f $ANVIL_PID_FILE
    fi
    if [ -f "$SUPERRELAY_PID_FILE" ]; then
        echo "❌ Stopping SuperRelay..."
        kill $(cat $SUPERRELAY_PID_FILE) 2>/dev/null || true
        rm -f $SUPERRELAY_PID_FILE
    fi
    echo "✅ Cleanup complete"
}

# Kill existing processes function
kill_existing_processes() {
    echo "🔄 Checking for existing processes..."

    # Kill processes using port 8545 (Anvil)
    if lsof -ti:8545 >/dev/null 2>&1; then
        echo "🛑 Killing existing processes on port 8545 (Anvil)..."
        lsof -ti:8545 | xargs kill -9 2>/dev/null || true
        sleep 1
    fi

    # Kill processes using port 3000 (SuperRelay RPC)
    if lsof -ti:3000 >/dev/null 2>&1; then
        echo "🛑 Killing existing processes on port 3000 (SuperRelay RPC)..."
        lsof -ti:3000 | xargs kill -9 2>/dev/null || true
        sleep 1
    fi

    # Kill processes using port 9000 (Swagger UI)
    if lsof -ti:9000 >/dev/null 2>&1; then
        echo "🛑 Killing existing processes on port 9000 (Swagger UI)..."
        lsof -ti:9000 | xargs kill -9 2>/dev/null || true
        sleep 1
    fi

    # Kill processes using port 8080 (Metrics)
    if lsof -ti:8080 >/dev/null 2>&1; then
        echo "🛑 Killing existing processes on port 8080 (Metrics)..."
        lsof -ti:8080 | xargs kill -9 2>/dev/null || true
        sleep 1
    fi

    # Kill any rundler or super-relay processes
    pkill -f "rundler" 2>/dev/null || true
    pkill -f "super-relay" 2>/dev/null || true

    echo "✅ Process cleanup complete"
}

# Check if tools are installed
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo -e "❌ Error: $1 not installed. Please install it first."
        exit 1
    fi
}

# Display configuration information
show_config() {
    echo ""
    echo "📋 Current configuration:"
    echo "  🌐 Network: $NETWORK"
    echo "  📡 RPC: $RPC_URL"
    echo "  🆔 Chain ID: $CHAIN_ID"
    echo "  🔑 Paymaster private key: ${PAYMASTER_PRIVATE_KEY:0:10}..."
    echo "  🔗 Signer private keys count: $(echo $SIGNER_PRIVATE_KEYS | tr ',' '\n' | wc -l)"
    echo ""

    # Validate critical environment variables
    if [ -z "$SIGNER_PRIVATE_KEYS" ]; then
        echo "❌ Error: SIGNER_PRIVATE_KEYS environment variable not set"
        echo "💡 Please ensure .env.dev file exists or manually set environment variables"
        exit 1
    fi

    if [ -z "$PAYMASTER_PRIVATE_KEY" ]; then
        echo "❌ Error: PAYMASTER_PRIVATE_KEY environment variable not set"
        echo "💡 Please ensure .env.dev file exists or manually set environment variables"
        exit 1
    fi
}

# Main logic
echo "🔍 Checking required tools..."
check_tool "anvil"
check_tool "cargo"

# Kill existing processes to prevent port conflicts
kill_existing_processes

# Validate and display configuration
show_config

# 1. Start Anvil (if needed)
if [ "$NETWORK" = "dev" ] && [ "$RPC_URL" = "http://localhost:8545" ]; then
    if [ -f "$ANVIL_PID_FILE" ]; then
        echo "ℹ️  Anvil seems to be already running (PID: $(cat $ANVIL_PID_FILE)). Skipping startup."
    else
        echo "🔥 Starting local Anvil blockchain..."
        anvil --host 0.0.0.0 --port 8545 --chain-id $CHAIN_ID > anvil.log 2>&1 &
        echo $! > $ANVIL_PID_FILE
        sleep 3 # Wait for Anvil to fully start
        echo "✅ Anvil started (PID: $(cat $ANVIL_PID_FILE))"

        # Verify Anvil is working properly
        if curl -s -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
            $RPC_URL > /dev/null; then
            echo "✅ Anvil connection verification successful"
        else
            echo "❌ Anvil connection failed"
            exit 1
        fi
    fi
else
    echo "🌐 Using external network: $RPC_URL"
fi

# 2. Build SuperRelay and rundler (if not already built)
if [ ! -f "./target/release/super-relay" ] || [ ! -f "./target/release/rundler" ]; then
    echo "🔨 Building SuperRelay and rundler..."
    cargo build --package super-relay --package rundler --release
else
    echo "✅ Binaries already built, skipping build step"
fi

# 3. Start SuperRelay Gateway
echo ""
echo "🌐 Starting SuperRelay API Gateway..."
echo "------------------------------------"
echo "💡 New Architecture (v0.1.5):"
echo "  • SuperRelay Gateway = Single binary with internal routing"
echo "  • Paymaster Service = Enterprise gas sponsorship (internal)"
echo "  • Rundler Components = ERC-4337 engine (internal method calls)"
echo "  • Web UI = Independent deployment (port 9000)"
echo "  • Configuration file: config/config.toml"
echo "------------------------------------"
echo ""

# Choose startup mode
STARTUP_MODE=${1:-"gateway"}

if [ "$STARTUP_MODE" = "legacy" ] || [ "$STARTUP_MODE" = "node" ]; then
    echo "🔧 Starting in Legacy Mode (for compatibility):"
    echo "  ./target/release/super-relay node --config config/config.toml"
    echo ""
    
    # Start legacy mode
    env PAYMASTER_PRIVATE_KEY="$PAYMASTER_PRIVATE_KEY" \
        SIGNER_PRIVATE_KEYS="$SIGNER_PRIVATE_KEYS" \
        RPC_URL="$RPC_URL" \
        NETWORK="$NETWORK" \
        CHAIN_ID="$CHAIN_ID" \
        ./target/release/super-relay node --config config/config.toml
else
    echo "🌐 Starting in Gateway Mode (recommended):"
    echo "  ./target/release/super-relay gateway --config config/config.toml --host 127.0.0.1 --port 3000 --enable-paymaster"
    echo ""
    
    # Start gateway mode
    env PAYMASTER_PRIVATE_KEY="$PAYMASTER_PRIVATE_KEY" \
        SIGNER_PRIVATE_KEYS="$SIGNER_PRIVATE_KEYS" \
        RPC_URL="$RPC_URL" \
        NETWORK="$NETWORK" \
        CHAIN_ID="$CHAIN_ID" \
        ./target/release/super-relay gateway \
            --config config/config.toml \
            --host 127.0.0.1 \
            --port 3000 \
            --enable-paymaster \
            --paymaster-private-key "$PAYMASTER_PRIVATE_KEY"
fi