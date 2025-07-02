#!/bin/bash

# Restart Super Relay Service Script
# Properly stops all running services and restarts them

set -e

echo "🔄 Restarting Super Relay Service..."

# Function to check if port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0  # Port is in use
    else
        return 1  # Port is free
    fi
}

# Function to wait for port to be free
wait_for_port_free() {
    local port=$1
    local max_wait=30
    local count=0
    
    echo "⏳ Waiting for port $port to be free..."
    
    while check_port $port && [ $count -lt $max_wait ]; do
        sleep 1
        count=$((count + 1))
        echo -n "."
    done
    
    if [ $count -ge $max_wait ]; then
        echo "❌ Timeout waiting for port $port to be free"
        return 1
    fi
    
    echo "✅ Port $port is now free"
    return 0
}

# Step 1: Stop all running services
echo "🛑 Stopping all running services..."

# Kill rundler processes
if pgrep -f "rundler" > /dev/null; then
    echo "  Stopping rundler processes..."
    pkill -f "rundler" || true
    sleep 2
fi

# Kill anvil processes
if pgrep -f "anvil" > /dev/null; then
    echo "  Stopping anvil processes..."
    pkill -f "anvil" || true
    sleep 2
fi

# Kill any processes using our ports
for port in 3000 8080 8545; do
    if check_port $port; then
        echo "  Killing process on port $port..."
        lsof -ti :$port | xargs kill -9 2>/dev/null || true
        sleep 1
    fi
done

# Step 2: Wait for ports to be free
echo "🔍 Verifying ports are free..."
for port in 3000 8080 8545; do
    if ! wait_for_port_free $port; then
        echo "❌ Failed to free port $port"
        exit 1
    fi
done

# Step 3: Start Anvil
echo "🚀 Starting Anvil..."
anvil --host 0.0.0.0 --port 8545 --chain-id 31337 --block-time 1 > /tmp/anvil.log 2>&1 &
ANVIL_PID=$!

# Wait for Anvil to start
echo "⏳ Waiting for Anvil to start..."
for i in {1..30}; do
    if curl -s http://localhost:8545 > /dev/null 2>&1; then
        echo "✅ Anvil is running"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Anvil failed to start"
        exit 1
    fi
    sleep 1
done

# Step 4: Deploy EntryPoint
echo "📜 Deploying EntryPoint contract..."
if ! ./scripts/deploy_entrypoint.sh; then
    echo "❌ Failed to deploy EntryPoint"
    exit 1
fi

# Step 4.5: Generate Chain Spec
echo "📝 Generating custom chain spec..."
if ! ./scripts/generate_chain_spec.sh; then
    echo "❌ Failed to generate chain spec"
    exit 1
fi

# Step 4.6: Fund Paymaster
echo "💰 Setting up paymaster funding..."
if ! ./scripts/fund_paymaster.sh; then
    echo "❌ Failed to fund paymaster"
    exit 1
fi

# Step 5: Start Super Relay with proper logging
echo "🚀 Starting Super Relay service..."

# Create log directory
mkdir -p logs

# Start service with explicit logging and custom chain spec
./target/release/rundler node \
    --node_http http://localhost:8545 \
    --chain_spec bin/rundler/chain_specs/local_dev.toml \
    --disable_entry_point_v0_7 \
    --paymaster.enabled \
    --paymaster.private_key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2 \
    --paymaster.policy_file config/paymaster-policies.toml \
    --max_verification_gas 10000000 \
    --rpc.port 3000 \
    --rpc.host 0.0.0.0 \
    --signer.private_keys 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    --metrics.port 8081 \
    --log.json \
    > logs/super-relay.log 2>&1 &

RELAY_PID=$!

# Wait for service to start
echo "⏳ Waiting for Super Relay to start..."
for i in {1..30}; do
    if curl -s http://localhost:3000 > /dev/null 2>&1; then
        echo "✅ Super Relay is running"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Super Relay failed to start"
        echo "📋 Last few lines of log:"
        tail -10 logs/super-relay.log
        exit 1
    fi
    sleep 1
done

# Step 6: Health check
echo "🏥 Running health checks..."

# Test standard RPC
echo "  Testing standard RPC..."
if curl -s -X POST http://localhost:3000 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"eth_supportedEntryPoints","params":[]}' \
    | grep -q "result"; then
    echo "  ✅ Standard RPC working"
else
    echo "  ❌ Standard RPC failed"
fi

# Test paymaster API
echo "  Testing paymaster API..."
response=$(curl -s -X POST http://localhost:3000 \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "id": 1,
        "method": "pm_sponsorUserOperation",
        "params": [{"sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"}, "0x0000000000000000000000000000000000000007"]
    }')

if echo "$response" | grep -q '"id":1'; then
    echo "  ✅ Paymaster API responding"
    if echo "$response" | grep -q '"error"'; then
        echo "  📝 API returned error (expected for incomplete params): $(echo "$response" | jq -r '.error.message' 2>/dev/null || echo 'parsing error')"
    fi
else
    echo "  ❌ Paymaster API not responding"
fi

echo ""
echo "🎉 Super Relay service restarted successfully!"
echo ""
echo "📊 Service Information:"
echo "  🌐 Anvil RPC:     http://localhost:8545"
echo "  🚀 Super Relay:   http://localhost:3000"
echo "  📈 Metrics:       http://localhost:8081"
echo "  📋 Logs:          logs/super-relay.log"
echo ""
echo "📜 Process IDs:"
echo "  Anvil:      $ANVIL_PID"
echo "  Super Relay: $RELAY_PID"
echo ""
echo "🔍 To monitor logs in real-time:"
echo "  tail -f logs/super-relay.log"
echo ""
echo "🛑 To stop services:"
echo "  kill $ANVIL_PID $RELAY_PID" 