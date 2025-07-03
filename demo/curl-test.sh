#!/bin/bash

# SuperPaymaster API Test Script
# Simple one-line curl commands to test core functionality

set -e

# Configuration
SUPER_RELAY_URL=${SUPER_RELAY_URL:-"http://localhost:3000"}
ENTRY_POINT_ADDRESS=${ENTRY_POINT_ADDRESS:-"0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"}

echo "🚀 Testing SuperPaymaster API at: $SUPER_RELAY_URL"
echo "📍 EntryPoint: $ENTRY_POINT_ADDRESS"
echo ""

# 1. Health Check (Simple one-liner)
echo "1️⃣ Health Check:"
echo "curl -s $SUPER_RELAY_URL/health | jq '.status'"
curl -s "$SUPER_RELAY_URL/health" | jq '.status' || echo "❌ Health check failed"
echo ""

# 2. Core API Test - JSON-RPC pm_sponsorUserOperation (One-liner)
echo "2️⃣ Core API Test (JSON-RPC):"
echo 'curl -X POST '$SUPER_RELAY_URL' -H "Content-Type: application/json" -d '"'"'{"jsonrpc":"2.0","id":1,"method":"pm_sponsorUserOperation","params":[{"sender":"0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266","nonce":"0x0","initCode":"0x","callData":"0x","callGasLimit":"0x186A0","verificationGasLimit":"0x186A0","preVerificationGas":"0x5208","maxFeePerGas":"0x3B9ACA00","maxPriorityFeePerGas":"0x3B9ACA00","paymasterAndData":"0x","signature":"0x"},"'$ENTRY_POINT_ADDRESS'"]}'"'"' | jq '"'"'.result'"'"

curl -X POST "$SUPER_RELAY_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "pm_sponsorUserOperation",
    "params": [
      {
        "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "nonce": "0x0",
        "initCode": "0x",
        "callData": "0x",
        "callGasLimit": "0x186A0",
        "verificationGasLimit": "0x186A0",
        "preVerificationGas": "0x5208",
        "maxFeePerGas": "0x3B9ACA00",
        "maxPriorityFeePerGas": "0x3B9ACA00",
        "paymasterAndData": "0x",
        "signature": "0x"
      },
      "'$ENTRY_POINT_ADDRESS'"
    ]
  }' | jq '.result' || echo "❌ JSON-RPC API test failed"
echo ""

# 3. REST API Test (One-liner)
echo "3️⃣ REST API Test:"
echo 'curl -X POST '$SUPER_RELAY_URL'/api/v1/sponsor -H "Content-Type: application/json" -d '"'"'{"user_op":{"sender":"0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266","nonce":"0x0","initCode":"0x","callData":"0x","callGasLimit":"0x186A0","verificationGasLimit":"0x186A0","preVerificationGas":"0x5208","maxFeePerGas":"0x3B9ACA00","maxPriorityFeePerGas":"0x3B9ACA00","paymasterAndData":"0x","signature":"0x"},"entry_point":"'$ENTRY_POINT_ADDRESS'"}'"'"' | jq '"'"'.user_op_hash'"'"

curl -X POST "$SUPER_RELAY_URL/api/v1/sponsor" \
  -H "Content-Type: application/json" \
  -d '{
    "user_op": {
      "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      "nonce": "0x0",
      "initCode": "0x",
      "callData": "0x",
      "callGasLimit": "0x186A0",
      "verificationGasLimit": "0x186A0",
      "preVerificationGas": "0x5208",
      "maxFeePerGas": "0x3B9ACA00",
      "maxPriorityFeePerGas": "0x3B9ACA00",
      "paymasterAndData": "0x",
      "signature": "0x"
    },
    "entry_point": "'$ENTRY_POINT_ADDRESS'"
  }' | jq '.user_op_hash' || echo "❌ REST API test failed"
echo ""

# 4. Metrics Check
echo "4️⃣ Metrics Check:"
echo "curl -s $SUPER_RELAY_URL/metrics | jq '.service'"
curl -s "$SUPER_RELAY_URL/metrics" | jq '.service' || echo "❌ Metrics check failed"
echo ""

echo "✅ Test completed! Check Swagger UI at: $SUPER_RELAY_URL/swagger-ui/" 