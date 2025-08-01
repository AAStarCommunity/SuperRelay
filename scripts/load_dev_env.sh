#!/bin/bash
# Load development environment variables securely
# Source this script: source ./scripts/load_dev_env.sh

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔒 Loading SuperRelay Development Environment${NC}"

# Check if .env.local exists (priority)
if [ -f ".env.local" ]; then
    echo -e "${GREEN}✅ Loading from .env.local${NC}"
    export $(grep -v '^#' .env.local | xargs)
elif [ -f ".env" ]; then
    echo -e "${YELLOW}⚠️  Loading from .env (consider creating .env.local)${NC}"
    export $(grep -v '^#' .env | xargs)
else
    echo -e "${RED}❌ No .env or .env.local file found${NC}"
    echo -e "${YELLOW}💡 Run './scripts/setup_dev_env.sh' to create .env file${NC}"
    return 1
fi

# Set development defaults for Anvil testing
export NETWORK=${NETWORK:-"dev"}
export RPC_URL=${RPC_URL:-"http://localhost:8545"}

# Development-only private keys (Anvil default accounts)
if [ "$NETWORK" = "dev" ] && [ -z "$SIGNER_PRIVATE_KEYS" ]; then
    echo -e "${YELLOW}🧪 Using Anvil development keys${NC}"
    export SIGNER_PRIVATE_KEYS="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2,0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    echo -e "${RED}⚠️  WARNING: These are PUBLIC test keys - NEVER use in production!${NC}"
fi

# Validate required environment variables
if [ -z "$SIGNER_PRIVATE_KEYS" ]; then
    echo -e "${RED}❌ SIGNER_PRIVATE_KEYS is required${NC}"
    echo -e "${YELLOW}💡 Set it in .env.local or as environment variable${NC}"
    return 1
fi

echo -e "${GREEN}✅ Environment loaded successfully${NC}"
echo -e "   Network: ${NETWORK}"
echo -e "   RPC URL: ${RPC_URL}"
echo -e "   Keys configured: $(echo $SIGNER_PRIVATE_KEYS | tr ',' '\n' | wc -l | tr -d ' ') accounts"