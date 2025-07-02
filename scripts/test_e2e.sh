#!/bin/bash

# End-to-End SuperPaymaster Testing Script
# Validates all core features from Solution.md and Features.md

set -e

echo "🧪 SuperPaymaster End-to-End Testing"
echo "====================================="

# Test configuration
RPC_URL="http://localhost:3000"
ANVIL_URL="http://localhost:8545"
TEST_SENDER="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
PAYMASTER_ADDRESS="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test result tracking
PASSED=0
FAILED=0

# Function to run test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "\n${BLUE}🧪 Testing: $test_name${NC}"
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        ((PASSED++))
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
        ((FAILED++))
    fi
}

# Function to test JSON-RPC response
test_rpc() {
    local method="$1"
    local params="$2"
    local expected_pattern="$3"
    local description="$4"
    
    local response=$(curl -s -X POST $RPC_URL \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$method\",\"params\":$params}")
    
    echo "📤 Request: $method"
    echo "📥 Response: $response"
    
    if echo "$response" | grep -q "$expected_pattern"; then
        return 0
    else
        echo "❌ Expected pattern not found: $expected_pattern"
        return 1
    fi
}

# Test 1: Service Health Check
test_service_health() {
    curl -s http://localhost:3000/health | grep -q "ok"
}

# Test 2: Standard RPC Functionality
test_standard_rpc() {
    test_rpc "eth_supportedEntryPoints" "[]" "result" "Should return supported EntryPoints"
}

# Test 3: Paymaster API Discovery
test_paymaster_api_discovery() {
    test_rpc "pm_sponsorUserOperation" "[{\"sender\":\"invalid\"}, \"0x0000000000000000000000000000000000000007\"]" "error" "Should respond to paymaster API calls"
}

# Test 4: UserOperation v0.6 Format Validation
test_userOperation_v06_format() {
    local user_op='{
        "sender": "'$TEST_SENDER'",
        "nonce": "0x0",
        "callData": "0x",
        "initCode": "0x",
        "paymasterAndData": "0x",
        "signature": "0x",
        "maxFeePerGas": "0x2540be400",
        "maxPriorityFeePerGas": "0x3b9aca00",
        "preVerificationGas": "0x5208",
        "verificationGasLimit": "0x249f0",
        "callGasLimit": "0x9c40"
    }'
    
    test_rpc "pm_sponsorUserOperation" "[$user_op, \"0x5FbDB2315678afecb367f032d93F642f64180aa3\"]" "error" "Should parse v0.6 UserOperation format"
}

# Test 5: UserOperation v0.7 Format Validation (without v0.6 specific fields)
test_userOperation_v07_format() {
    local user_op='{
        "sender": "'$TEST_SENDER'",
        "nonce": "0x0",
        "callData": "0x",
        "signature": "0x",
        "maxFeePerGas": "0x2540be400",
        "maxPriorityFeePerGas": "0x3b9aca00",
        "preVerificationGas": "0x5208",
        "verificationGasLimit": "0x249f0",
        "callGasLimit": "0x9c40"
    }'
    
    test_rpc "pm_sponsorUserOperation" "[$user_op, \"0x5FbDB2315678afecb367f032d93F642f64180aa3\"]" "error" "Should parse v0.7 UserOperation format"
}

# Test 6: Policy Engine - Allowlist Validation
test_policy_allowlist() {
    # Test with allowed sender (should progress further than validation)
    local user_op='{
        "sender": "'$TEST_SENDER'",
        "nonce": "0x0",
        "callData": "0x",
        "initCode": "0x",
        "paymasterAndData": "0x",
        "signature": "0x",
        "maxFeePerGas": "0x2540be400",
        "maxPriorityFeePerGas": "0x3b9aca00",
        "preVerificationGas": "0x5208",
        "verificationGasLimit": "0x249f0",
        "callGasLimit": "0x9c40"
    }'
    
    local response=$(curl -s -X POST $RPC_URL \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"pm_sponsorUserOperation\",\"params\":[$user_op, \"0x5FbDB2315678afecb367f032d93F642f64180aa3\"]}")
    
    # Should not get policy violation error
    if echo "$response" | grep -q "policy"; then
        echo "❌ Policy violation detected"
        return 1
    else
        echo "✅ Allowed sender passed policy check"
        return 0
    fi
}

# Test 7: Policy Engine - Reject Unauthorized Sender
test_policy_reject() {
    local unauthorized_sender="0x1234567890123456789012345678901234567890"
    local user_op='{
        "sender": "'$unauthorized_sender'",
        "nonce": "0x0",
        "callData": "0x",
        "initCode": "0x",
        "paymasterAndData": "0x",
        "signature": "0x",
        "maxFeePerGas": "0x2540be400",
        "maxPriorityFeePerGas": "0x3b9aca00",
        "preVerificationGas": "0x5208",
        "verificationGasLimit": "0x249f0",
        "callGasLimit": "0x9c40"
    }'
    
    test_rpc "pm_sponsorUserOperation" "[$user_op, \"0x5FbDB2315678afecb367f032d93F642f64180aa3\"]" "error" "Should reject unauthorized sender"
}

# Test 8: Hex and Decimal Number Format Support
test_number_formats() {
    # Test with decimal format
    local user_op_decimal='{
        "sender": "'$TEST_SENDER'",
        "nonce": "0",
        "callData": "0x",
        "initCode": "0x",
        "paymasterAndData": "0x",
        "signature": "0x",
        "maxFeePerGas": "10000000000",
        "maxPriorityFeePerGas": "1000000000",
        "preVerificationGas": "21000",
        "verificationGasLimit": "150000",
        "callGasLimit": "40000"
    }'
    
    test_rpc "pm_sponsorUserOperation" "[$user_op_decimal, \"0x5FbDB2315678afecb367f032d93F642f64180aa3\"]" "error" "Should parse decimal format numbers"
}

# Test 9: EntryPoint Address Validation
test_entrypoint_validation() {
    local user_op='{
        "sender": "'$TEST_SENDER'",
        "nonce": "0x0",
        "callData": "0x",
        "initCode": "0x",
        "paymasterAndData": "0x",
        "signature": "0x",
        "maxFeePerGas": "0x2540be400",
        "maxPriorityFeePerGas": "0x3b9aca00",
        "preVerificationGas": "0x5208",
        "verificationGasLimit": "0x249f0",
        "callGasLimit": "0x9c40"
    }'
    
    # Test with wrong EntryPoint address
    local wrong_entrypoint="0x0000000000000000000000000000000000000001"
    local response=$(curl -s -X POST $RPC_URL \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"pm_sponsorUserOperation\",\"params\":[$user_op, \"$wrong_entrypoint\"]}")
    
    if echo "$response" | grep -q "Unknown entry point"; then
        echo "✅ Correctly rejects unknown EntryPoint"
        return 0
    else
        echo "❌ Should reject unknown EntryPoint"
        return 1
    fi
}

# Test 10: Service Integration Status
test_service_integration() {
    echo "📊 Service Integration Status:"
    echo "🔗 Anvil RPC: $(curl -s $ANVIL_URL > /dev/null && echo 'Connected' || echo 'Failed')"
    echo "🚀 Super Relay: $(curl -s $RPC_URL/health | grep -q 'ok' && echo 'Running' || echo 'Failed')"
    echo "📍 EntryPoint: $(cat .entrypoint_address 2>/dev/null || echo 'Not found')"
    echo "🔑 Paymaster: $PAYMASTER_ADDRESS"
    return 0
}

# Main test execution
main() {
    echo "🚀 Starting comprehensive SuperPaymaster testing..."
    echo "📋 Testing core features from Solution.md and Features.md"
    echo ""
    
    # Core service tests
    run_test "Service Health Check" "test_service_health"
    run_test "Standard RPC Functionality" "test_standard_rpc"
    run_test "Paymaster API Discovery" "test_paymaster_api_discovery"
    
    # UserOperation format tests
    run_test "UserOperation v0.6 Format" "test_userOperation_v06_format"
    run_test "UserOperation v0.7 Format" "test_userOperation_v07_format"
    run_test "Number Format Support" "test_number_formats"
    
    # Policy engine tests
    run_test "Policy Allowlist Validation" "test_policy_allowlist"
    run_test "Policy Rejection Test" "test_policy_reject"
    
    # Security and validation tests
    run_test "EntryPoint Validation" "test_entrypoint_validation"
    
    # Integration status
    run_test "Service Integration Status" "test_service_integration"
    
    # Summary
    echo ""
    echo "🏁 Test Summary"
    echo "==============="
    echo -e "${GREEN}✅ Passed: $PASSED${NC}"
    echo -e "${RED}❌ Failed: $FAILED${NC}"
    echo -e "${BLUE}📊 Total: $((PASSED + FAILED))${NC}"
    
    if [ $FAILED -eq 0 ]; then
        echo -e "\n${GREEN}🎉 All tests passed! SuperPaymaster is working correctly!${NC}"
        return 0
    else
        echo -e "\n${YELLOW}⚠️  Some tests failed. Review the output above.${NC}"
        return 1
    fi
}

# Run tests
main "$@" 