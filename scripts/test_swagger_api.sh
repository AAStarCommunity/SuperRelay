#!/bin/bash
# Test script for Swagger UI API interaction
# This script tests that Swagger UI can correctly interact with the real SuperRelay service

set -e

echo "🧪 SuperRelay Swagger API Test Suite"
echo "========================================"
echo ""

# Configuration
SWAGGER_URL="http://localhost:9000"
SERVICE_URL="http://localhost:3000"
ANVIL_URL="http://localhost:8545"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test result counters
PASSED=0
FAILED=0

# Function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ PASSED${NC}: $2"
        ((PASSED++))
    else
        echo -e "${RED}❌ FAILED${NC}: $2"
        ((FAILED++))
    fi
}

# Function to check if service is running
check_service() {
    local url=$1
    local service_name=$2
    
    if curl -s -f -o /dev/null "$url"; then
        return 0
    else
        return 1
    fi
}

echo "📋 Pre-flight checks..."
echo "------------------------"

# Check if Anvil is running
if check_service "$ANVIL_URL" "Anvil"; then
    print_result 0 "Anvil is running on port 8545"
else
    print_result 1 "Anvil is not running. Start with: ./scripts/start_superrelay.sh"
fi

# Check if SuperRelay service is running
if curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
    "$SERVICE_URL" > /dev/null 2>&1; then
    print_result 0 "SuperRelay service is running on port 3000"
else
    print_result 1 "SuperRelay service is not running. Start with: ./scripts/start_superrelay.sh"
fi

# Check if Web UI service is running
if curl -s -f -o /dev/null "$SWAGGER_URL" > /dev/null 2>&1; then
    print_result 0 "Web UI service is running on port 9000"
else
    print_result 1 "Web UI service is not running. Start with: ./scripts/start_web_ui.sh"
fi

# Check if OpenAPI spec has real data
if curl -s "$SWAGGER_URL/openapi.json" | grep -q "pm_sponsorUserOperation"; then
    print_result 0 "OpenAPI spec contains updated sponsor operation method"
else
    print_result 1 "OpenAPI spec needs updating or Web UI service is not running"
fi

echo ""
echo "🧪 API Endpoint Tests..."
echo "------------------------"

# Test 1: Health Check Endpoint
echo -e "${BLUE}Test 1: Health Check${NC}"
RESPONSE=$(curl -s -X GET "$SWAGGER_URL/health" 2>/dev/null)
if echo "$RESPONSE" | grep -q "healthy"; then
    print_result 0 "Health check endpoint returns healthy status"
    echo "  Response: $(echo $RESPONSE | jq -r '.status' 2>/dev/null || echo 'N/A')"
else
    print_result 1 "Health check endpoint failed"
fi

# Test 2: Metrics Endpoint
echo -e "${BLUE}Test 2: Metrics Endpoint${NC}"
RESPONSE=$(curl -s -X GET "$SWAGGER_URL/metrics" 2>/dev/null)
if echo "$RESPONSE" | grep -q "service"; then
    print_result 0 "Metrics endpoint returns data"
else
    print_result 1 "Metrics endpoint failed"
fi

# Test 3: OpenAPI Spec
echo -e "${BLUE}Test 3: OpenAPI Specification${NC}"
RESPONSE=$(curl -s -X GET "$SWAGGER_URL/api-docs/openapi.json" 2>/dev/null)
if echo "$RESPONSE" | grep -q "SuperPaymaster"; then
    print_result 0 "OpenAPI spec is accessible"
else
    print_result 1 "OpenAPI spec not found"
fi

# Test 4: Sponsor UserOperation (direct RPC call)
echo -e "${BLUE}Test 4: Sponsor UserOperation (JSON-RPC to SuperRelay)${NC}"
TEST_RPC_REQUEST='{
    "jsonrpc": "2.0",
    "method": "pm_sponsorUserOperation",
    "params": [
        {
            "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "nonce": "0x0",
            "initCode": "0x",
            "callData": "0xb61d27f6000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000",
            "callGasLimit": "0x30D40",
            "verificationGasLimit": "0x186A0",
            "preVerificationGas": "0xC350",
            "maxFeePerGas": "0x59682F00",
            "maxPriorityFeePerGas": "0x59682F00",
            "paymasterAndData": "0x",
            "signature": "0xfffffffffffffffffffffffffffffff0000000000000000000000000000000007aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1c"
        },
        "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
    ],
    "id": 1
}'

RESPONSE=$(curl -s -X POST "$SERVICE_URL" \
    -H "Content-Type: application/json" \
    -d "$TEST_RPC_REQUEST" 2>/dev/null)

if echo "$RESPONSE" | grep -q '"result"'; then
    print_result 0 "Sponsor UserOperation returns success result"
    echo "  Response contains result field"
elif echo "$RESPONSE" | grep -q '"error"'; then
    print_result 1 "Sponsor UserOperation returned error"
    echo "  Error: $(echo $RESPONSE | jq -r '.error.message' 2>/dev/null || echo 'Parse failed')"
else
    print_result 1 "Sponsor UserOperation failed with unexpected response"
    echo "  Response: $(echo $RESPONSE | head -c 200)"
fi

# Test 5: Example Data Endpoints
echo -e "${BLUE}Test 5: Example Data Endpoints${NC}"
RESPONSE=$(curl -s -X GET "$SWAGGER_URL/examples/v06" 2>/dev/null)
if echo "$RESPONSE" | grep -q "sender"; then
    print_result 0 "Example v0.6 endpoint returns valid data"
else
    print_result 1 "Example v0.6 endpoint failed"
fi

# Test 6: Code Generation Endpoints
echo -e "${BLUE}Test 6: Code Generation${NC}"
RESPONSE=$(curl -s -X GET "$SWAGGER_URL/codegen/curl/sponsor" 2>/dev/null)
if echo "$RESPONSE" | grep -q "curl"; then
    print_result 0 "Code generation endpoint works"
else
    print_result 1 "Code generation endpoint failed"
fi

# Test 7: Dashboard Access
echo -e "${BLUE}Test 7: Dashboard UI${NC}"
RESPONSE=$(curl -s -X GET "$SWAGGER_URL/dashboard" 2>/dev/null)
if echo "$RESPONSE" | grep -q "SuperPaymaster"; then
    print_result 0 "Dashboard UI is accessible"
else
    print_result 1 "Dashboard UI not accessible"
fi

echo ""
echo "========================================"
echo "📊 Test Summary"
echo "========================================"
echo -e "${GREEN}Passed:${NC} $PASSED"
echo -e "${RED}Failed:${NC} $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed! Swagger UI is properly configured for real API interaction.${NC}"
    echo ""
    echo "📝 Next Steps:"
    echo "1. Open Swagger UI: http://localhost:9000/"
    echo "2. Look for the 'pm_sponsorUserOperation' endpoint under 'Paymaster API'"
    echo "3. Click on the endpoint to expand it"
    echo "4. Click 'Try it out' to enable interactive testing"
    echo "5. Use the pre-filled example data (complete UserOperation + EntryPoint)"
    echo "6. Click 'Execute' to send the request to the real SuperRelay service"
    echo "7. View the actual response with paymasterAndData"
    echo ""
    echo "🎯 The request will be sent directly to: http://localhost:3000"
    exit 0
else
    echo -e "${YELLOW}⚠️  Some tests failed. Please check the services and try again.${NC}"
    echo ""
    echo "🔧 Troubleshooting:"
    echo "1. Start SuperRelay service: ./scripts/start_superrelay.sh"
    echo "2. Start Web UI service: ./scripts/start_web_ui.sh"  
    echo "3. Wait for services to fully initialize (~10-15 seconds)"
    echo "4. Check service logs for errors"
    echo "5. Verify ports 3000, 8545, 9000 are not blocked"
    exit 1
fi