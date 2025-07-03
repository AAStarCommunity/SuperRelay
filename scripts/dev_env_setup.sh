#!/bin/bash

# SuperRelay Development Environment Setup & Check
# Automated development environment preparation and validation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 SuperRelay Development Environment Setup${NC}"
echo "=========================================="

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
echo -e "\n${BLUE}🔍 Checking development dependencies...${NC}"

# Check Rust
if command_exists rustc; then
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    echo -e "${GREEN}✅ Rust: $RUST_VERSION${NC}"
else
    echo -e "${RED}❌ Rust not found. Installing...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    rustup component add rustfmt clippy
    rustup toolchain install nightly --component rustfmt
fi

# Check Foundry tools
echo -e "\n${BLUE}🔨 Checking Foundry tools...${NC}"
for tool in forge cast anvil; do
    if command_exists $tool; then
        echo -e "${GREEN}✅ $tool: available${NC}"
    else
        echo -e "${RED}❌ $tool not found. Installing Foundry...${NC}"
        curl -L https://foundry.paradigm.xyz | bash
        export PATH="$HOME/.foundry/bin:$PATH"
        foundryup
        break
    fi
done

# Check Node.js
if command_exists node; then
    NODE_VERSION=$(node --version)
    echo -e "${GREEN}✅ Node.js: $NODE_VERSION${NC}"
else
    echo -e "${RED}❌ Node.js not found${NC}"
fi

# Check protobuf compiler
if command_exists protoc; then
    PROTOC_VERSION=$(protoc --version | awk '{print $2}')
    echo -e "${GREEN}✅ protoc: $PROTOC_VERSION${NC}"
else
    echo -e "${RED}❌ protobuf compiler not found. Installing...${NC}"
    if command_exists brew; then
        brew install protobuf
    else
        echo -e "${YELLOW}Please install protobuf manually${NC}"
    fi
fi

# Check Git submodules
echo -e "\n${BLUE}�� Checking Git submodules...${NC}"
if [ -f .gitmodules ]; then
    git submodule update --init --recursive
    echo -e "${GREEN}✅ Git submodules updated${NC}"
fi

# Create .env file
echo -e "\n${BLUE}⚙️  Setting up environment configuration...${NC}"
if [ ! -f .env ]; then
    cat > .env << 'ENVEOF'
# SuperRelay Development Environment Configuration
NETWORK=dev
RPC_URL=http://localhost:8545
CHAIN_ID=1337
SIGNER_PRIVATE_KEYS=0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2,0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
PAYMASTER_PRIVATE_KEY=0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2
PAYMASTER_POLICY_FILE=config/paymaster-policies.toml
ANVIL_PORT=8545
RUNDLER_RPC_PORT=3000
METRICS_PORT=8081
SWAGGER_PORT=9000
RUST_LOG=info
ENVEOF
    echo -e "${GREEN}✅ Created .env configuration file${NC}"
fi

# Create quick start script
echo -e "\n${BLUE}🚀 Creating quick start script...${NC}"
cat > scripts/quick_start.sh << 'QUICKEOF'
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
QUICKEOF

chmod +x scripts/quick_start.sh
echo -e "${GREEN}✅ Created scripts/quick_start.sh${NC}"

# Build optimization tips
echo -e "\n${BLUE}⚡ Build optimization recommendations:${NC}"
echo -e "${CYAN}• First build: ~60 seconds (full compilation)${NC}"
echo -e "${CYAN}• Incremental builds: ~20-30 seconds${NC}"
echo -e "${CYAN}• Use 'cargo check' for faster syntax checking${NC}"

echo -e "\n${GREEN}🎉 Development environment setup complete!${NC}"
echo -e "${CYAN}Next steps:${NC}"
echo -e "  1. Run './scripts/quick_start.sh' to start the full environment"
echo -e "  2. Test with: './scripts/fund_paymaster.sh status'"
