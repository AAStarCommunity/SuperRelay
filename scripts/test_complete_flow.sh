#!/bin/bash

# Complete SuperRelay Test Runner
# This script will start the dev environment and run comprehensive tests

set -e

echo "🚀 SuperRelay Complete Flow Test Runner"
echo "======================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js is not installed. Please install Node.js first.${NC}"
    exit 1
fi

# Check if required packages are installed
if [ ! -d "demo/node_modules" ]; then
    echo -e "${YELLOW}📦 Installing Node.js dependencies...${NC}"
    cd demo
    npm install
    cd ..
fi

# Function to wait for service to be ready
wait_for_service() {
    local url=$1
    local service_name=$2
    local max_attempts=30
    local attempt=1
    
    echo -e "${BLUE}⏳ Waiting for $service_name to be ready...${NC}"
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ $service_name is ready!${NC}"
            return 0
        fi
        
        echo -e "${YELLOW}   Attempt $attempt/$max_attempts: $service_name not ready yet...${NC}"
        sleep 2
        attempt=$((attempt + 1))
    done
    
    echo -e "${RED}❌ $service_name failed to start within timeout${NC}"
    return 1
}

# Function to cleanup processes
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"
    
    # Kill SuperRelay if running
    if [ -f "scripts/.rundler.pid" ]; then
        kill $(cat scripts/.rundler.pid) 2>/dev/null || true
        rm scripts/.rundler.pid
    fi
    
    # Kill Anvil if running
    if [ -f "scripts/.anvil.pid" ]; then
        kill $(cat scripts/.anvil.pid) 2>/dev/null || true
        rm scripts/.anvil.pid
    fi
    
    # Kill any processes on the ports
    lsof -ti:3000 | xargs kill -9 2>/dev/null || true
    lsof -ti:8545 | xargs kill -9 2>/dev/null || true
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Set trap to cleanup on script exit
trap cleanup EXIT

echo -e "${BLUE}🏁 Step 1: Starting development environment...${NC}"

# Start the dev server in background
./scripts/start_dev_server.sh > /dev/null 2>&1 &
DEV_SERVER_PID=$!

# Wait for services to be ready
wait_for_service "http://localhost:8545" "Anvil"
wait_for_service "http://localhost:3000/health" "SuperRelay"

echo -e "${BLUE}🏁 Step 2: Running comprehensive tests...${NC}"

# Set environment variables for test
export SUPER_RELAY_URL="http://localhost:3000"
export RPC_URL="http://localhost:8545"

# Get EntryPoint address if available
if [ -f ".entrypoint_address" ]; then
    export ENTRY_POINT_ADDRESS=$(cat .entrypoint_address)
    echo -e "${GREEN}📍 Using EntryPoint address: $ENTRY_POINT_ADDRESS${NC}"
fi

# Run the complete test
echo -e "${BLUE}🧪 Starting complete flow test...${NC}"
node test_paymaster_complete.js

# Test result
TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "\n${GREEN}🎉 ALL TESTS PASSED!${NC}"
    echo -e "${GREEN}✅ SuperRelay is working correctly and ready for use!${NC}"
    
    echo -e "\n${BLUE}📚 Next Steps:${NC}"
    echo "1. Customize policies in config/paymaster-policies.toml"
    echo "2. Configure production settings"
    echo "3. Deploy to your target network"
    
else
    echo -e "\n${RED}❌ Some tests failed. Please check the output above.${NC}"
    exit 1
fi