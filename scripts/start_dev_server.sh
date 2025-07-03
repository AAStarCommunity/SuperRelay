#!/bin/bash

# SuperRelay Development Server Startup Script
# This script starts all necessary services for development and testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ANVIL_PORT=8545
RUNDLER_PORT=3000
ENTRY_POINT_ADDRESS="0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
PAYMASTER_SIGNER_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2"

echo -e "${BLUE}🚀 Starting SuperRelay Development Environment${NC}"
echo "=================================================="

# Check if required tools are installed
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}❌ $1 is not installed${NC}"
        exit 1
    fi
}

echo -e "${BLUE}🔍 Checking required tools...${NC}"
check_tool "anvil"
check_tool "cargo"
check_tool "jq"

# Kill existing processes
echo -e "${YELLOW}🧹 Cleaning up existing processes...${NC}"
pkill -f "anvil" || true
pkill -f "rundler" || true
sleep 2

# Start Anvil
echo -e "${BLUE}⛏️  Starting Anvil test network...${NC}"
anvil --port $ANVIL_PORT --host 0.0.0.0 &
ANVIL_PID=$!
echo "Anvil PID: $ANVIL_PID"

# Wait for Anvil to be ready
echo -e "${YELLOW}⏳ Waiting for Anvil to be ready...${NC}"
for i in {1..30}; do
    if curl -s "http://localhost:$ANVIL_PORT" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Anvil is ready${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}❌ Anvil failed to start${NC}"
        kill $ANVIL_PID 2>/dev/null || true
        exit 1
    fi
    sleep 1
done

# Deploy EntryPoint contract (if needed)
echo -e "${BLUE}📄 Checking EntryPoint contract...${NC}"
if ! curl -s -X POST "http://localhost:$ANVIL_PORT" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_getCode","params":["'$ENTRY_POINT_ADDRESS'","latest"],"id":1}' | \
    jq -r '.result' | grep -q "0x" 2>/dev/null || [ "$(curl -s -X POST "http://localhost:$ANVIL_PORT" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_getCode","params":["'$ENTRY_POINT_ADDRESS'","latest"],"id":1}' | \
    jq -r '.result')" = "0x" ]; then
    
    echo -e "${YELLOW}📦 Deploying EntryPoint contract...${NC}"
    if [ -f "./scripts/deploy_entrypoint.sh" ]; then
        ./scripts/deploy_entrypoint.sh
    else
        echo -e "${YELLOW}⚠️  EntryPoint deployment script not found, using pre-configured address${NC}"
    fi
else
    echo -e "${GREEN}✅ EntryPoint contract already deployed${NC}"
fi

# Create temporary policy file
echo -e "${BLUE}📋 Creating temporary policy file...${NC}"
TEMP_POLICY=$(mktemp)
cat > $TEMP_POLICY << EOF
[default]
senders = [
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",  # Anvil account #0
    "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",  # Anvil account #1
    "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",  # Anvil account #2
]
EOF

echo "Policy file created at: $TEMP_POLICY"

# Build the project
echo -e "${BLUE}🔨 Building SuperRelay...${NC}"
cargo build --release

# Start SuperRelay
echo -e "${BLUE}🚀 Starting SuperRelay service...${NC}"
RUST_LOG=info cargo run --bin rundler -- node \
    --network dev \
    --node_http http://localhost:$ANVIL_PORT \
    --rpc.host 127.0.0.1 \
    --rpc.port $RUNDLER_PORT \
    --metrics.port 8081 \
    --paymaster.enabled \
    --signer.private_keys $PAYMASTER_SIGNER_KEY,0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    --paymaster.private_key $PAYMASTER_SIGNER_KEY \
    --paymaster.policy_file $TEMP_POLICY \
    --pool.same_sender_mempool_count 4 \
    --max_verification_gas 10000000 \
    --rpc.api eth,rundler,paymaster &

RUNDLER_PID=$!
echo "SuperRelay PID: $RUNDLER_PID"

# Set environment variables for the subprocess
export RUNDLER__PAYMASTER__SIGNER__TYPE="local_hot_wallet"
export RUNDLER__PAYMASTER__SIGNER__PRIVATE_KEY="$PAYMASTER_SIGNER_KEY"

# Wait for SuperRelay to be ready
echo -e "${YELLOW}⏳ Waiting for SuperRelay to be ready...${NC}"
for i in {1..60}; do
    if curl -s "http://localhost:$RUNDLER_PORT/health" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ SuperRelay is ready${NC}"
        break
    fi
    if [ $i -eq 60 ]; then
        echo -e "${RED}❌ SuperRelay failed to start${NC}"
        kill $RUNDLER_PID 2>/dev/null || true
        kill $ANVIL_PID 2>/dev/null || true
        rm -f $TEMP_POLICY
        exit 1
    fi
    sleep 1
done

# Display service information
echo ""
echo -e "${GREEN}🎉 Development environment is ready!${NC}"
echo "=================================================="
echo -e "📡 Anvil RPC:      ${BLUE}http://localhost:$ANVIL_PORT${NC}"
echo -e "🚀 SuperRelay API: ${BLUE}http://localhost:$RUNDLER_PORT${NC}"
echo -e "📚 Swagger UI:     ${BLUE}http://localhost:$RUNDLER_PORT/swagger-ui/${NC}"
echo -e "🏥 Health Check:   ${BLUE}http://localhost:$RUNDLER_PORT/health${NC}"
echo -e "📊 Metrics:        ${BLUE}http://localhost:$RUNDLER_PORT/metrics${NC}"
echo ""
echo -e "🔑 Test Accounts:"
echo -e "   User:      ${YELLOW}0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266${NC}"
echo -e "   Paymaster: ${YELLOW}0x70997970C51812dc3A010C7d01b50e0d17dc79C8${NC}"
echo ""
echo -e "📄 EntryPoint:    ${YELLOW}$ENTRY_POINT_ADDRESS${NC}"
echo -e "📋 Policy File:   ${YELLOW}$TEMP_POLICY${NC}"
echo ""

# Test basic functionality
echo -e "${BLUE}🧪 Running basic tests...${NC}"
echo -e "${YELLOW}Testing health endpoint...${NC}"
if curl -s "http://localhost:$RUNDLER_PORT/health" | jq -r '.status' | grep -q "healthy"; then
    echo -e "${GREEN}✅ Health check passed${NC}"
else
    echo -e "${RED}❌ Health check failed${NC}"
fi

echo ""
echo -e "${GREEN}🎮 Ready to test! Try these commands:${NC}"
echo ""
echo -e "${BLUE}1. Run demo tests:${NC}"
echo "   cd demo && ./curl-test.sh"
echo ""
echo -e "${BLUE}2. Run Node.js demo:${NC}"
echo "   cd demo && npm install && node superPaymasterDemo.js"
echo ""
echo -e "${BLUE}3. Open Web UI:${NC}"
echo "   open demo/interactive-demo.html"
echo ""
echo -e "${BLUE}4. Test with curl:${NC}"
echo '   curl -X POST http://localhost:3000 -H "Content-Type: application/json" \'
echo '   -d '"'"'{"jsonrpc":"2.0","id":1,"method":"pm_sponsorUserOperation","params":[{"sender":"0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266","nonce":"0x0","initCode":"0x","callData":"0x","callGasLimit":"0x186A0","verificationGasLimit":"0x186A0","preVerificationGas":"0x5208","maxFeePerGas":"0x3B9ACA00","maxPriorityFeePerGas":"0x3B9ACA00","paymasterAndData":"0x","signature":"0x"},"'$ENTRY_POINT_ADDRESS'"]}'"'"' | jq'
echo ""

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}🧹 Shutting down services...${NC}"
    kill $RUNDLER_PID 2>/dev/null || true
    kill $ANVIL_PID 2>/dev/null || true
    rm -f $TEMP_POLICY
    echo -e "${GREEN}✅ Cleanup completed${NC}"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Keep the script running
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
wait 