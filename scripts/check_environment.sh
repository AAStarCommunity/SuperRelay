#!/bin/bash

# SuperRelay Environment Check
# Verifies that all required tools and dependencies are available

echo "🔍 SuperRelay Environment Check"
echo "==============================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

errors=0

# Check required tools
check_tool() {
    local tool=$1
    local install_info=$2
    
    if command -v "$tool" &> /dev/null; then
        echo -e "${GREEN}✅ $tool is installed${NC}"
        return 0
    else
        echo -e "${RED}❌ $tool is missing${NC}"
        echo -e "${YELLOW}   Install: $install_info${NC}"
        errors=$((errors + 1))
        return 1
    fi
}

echo -e "${BLUE}📋 Checking required tools...${NC}"

check_tool "cargo" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
check_tool "anvil" "curl -L https://foundry.paradigm.xyz | bash && foundryup"
check_tool "cast" "Part of Foundry - install with foundryup"
check_tool "node" "https://nodejs.org/ or use nvm/brew"
check_tool "jq" "brew install jq (macOS) or apt-get install jq (Linux)"

# Check if project can build
echo -e "\n${BLUE}🔨 Checking project build...${NC}"
if cargo check --package rundler > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Project builds successfully${NC}"
else
    echo -e "${RED}❌ Project build failed${NC}"
    echo -e "${YELLOW}   Try: cargo build --package rundler${NC}"
    errors=$((errors + 1))
fi

# Check Node.js dependencies
echo -e "\n${BLUE}📦 Checking Node.js dependencies...${NC}"
if [ -f "demo/package.json" ]; then
    if [ -d "demo/node_modules" ]; then
        echo -e "${GREEN}✅ Node.js dependencies installed${NC}"
    else
        echo -e "${YELLOW}⚠️  Node.js dependencies not installed${NC}"
        echo -e "${YELLOW}   Run: cd demo && npm install${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  demo/package.json not found${NC}"
fi

# Check configuration files
echo -e "\n${BLUE}⚙️  Checking configuration files...${NC}"

config_files=(
    "config/config.toml"
    "config/paymaster-policies.toml"
)

for file in "${config_files[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✅ $file exists${NC}"
    else
        echo -e "${RED}❌ $file missing${NC}"
        errors=$((errors + 1))
    fi
done

# Check ports availability
echo -e "\n${BLUE}🔌 Checking port availability...${NC}"

check_port() {
    local port=$1
    local service=$2
    
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Port $port is in use ($service)${NC}"
        echo -e "${YELLOW}   Stop existing service or use different port${NC}"
    else
        echo -e "${GREEN}✅ Port $port is available ($service)${NC}"
    fi
}

check_port 8545 "Anvil"
check_port 3000 "SuperRelay RPC"
check_port 8080 "Metrics"
check_port 9000 "Swagger UI"

# Summary
echo -e "\n${BLUE}📊 Environment Check Summary${NC}"
echo "============================="

if [ $errors -eq 0 ]; then
    echo -e "${GREEN}🎉 Environment is ready!${NC}"
    echo -e "${GREEN}✅ You can run: ./scripts/test_complete_flow.sh${NC}"
    exit 0
else
    echo -e "${RED}❌ Found $errors issue(s)${NC}"
    echo -e "${YELLOW}Please fix the issues above before running tests${NC}"
    exit 1
fi