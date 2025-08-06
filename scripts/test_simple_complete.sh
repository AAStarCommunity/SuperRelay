#!/bin/bash

# SuperRelay 简单功能完整测试脚本
set -e

echo "🧪 SuperRelay 简单功能测试"
echo "========================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

RPC_URL="http://localhost:3000"
TEST_SENDER="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

# 清理现有进程
pkill -f "super-relay|anvil" || true
sleep 2

# 检查二进制文件
SUPER_RELAY_BIN=""
if [[ -f "$PROJECT_ROOT/target/release/super-relay" ]]; then
    SUPER_RELAY_BIN="$PROJECT_ROOT/target/release/super-relay"
elif [[ -f "$PROJECT_ROOT/target/debug/super-relay" ]]; then
    SUPER_RELAY_BIN="$PROJECT_ROOT/target/debug/super-relay"
else
    echo -e "${RED}❌ super-relay 二进制文件不存在${NC}"
    exit 1
fi

# 设置环境变量
export SIGNER_PRIVATE_KEYS="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export PAYMASTER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

echo "🔧 启动测试环境..."

# 启动 Anvil
echo "🔥 启动 Anvil..."
anvil --host 0.0.0.0 --port 8545 --chain-id 31337 > anvil.log 2>&1 &
ANVIL_PID=$!
sleep 3

# 验证 Anvil
if ! curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' http://localhost:8545 > /dev/null; then
    echo -e "${RED}❌ Anvil 启动失败${NC}"
    kill $ANVIL_PID 2>/dev/null || true
    exit 1
fi
echo -e "${GREEN}✅ Anvil 启动成功${NC}"

# 启动 SuperRelay Gateway
echo "🚀 启动 SuperRelay Gateway..."
$SUPER_RELAY_BIN gateway \
    --host 127.0.0.1 \
    --port 3000 \
    --enable-paymaster \
    --paymaster-private-key "$PAYMASTER_PRIVATE_KEY" \
    > gateway.log 2>&1 &
GATEWAY_PID=$!

# 等待服务启动
echo "⏳ 等待服务启动..."
for i in {1..10}; do
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ SuperRelay 启动成功${NC}"
        break
    fi
    if [[ $i -eq 10 ]]; then
        echo -e "${RED}❌ SuperRelay 启动超时${NC}"
        kill $GATEWAY_PID $ANVIL_PID 2>/dev/null || true
        exit 1
    fi
    sleep 2
done

echo ""
echo "🧪 开始功能测试..."

test_count=0
pass_count=0

# Test 1: Health Check
echo "🧪 Test 1: 健康检查"
test_count=$((test_count + 1))
health_response=$(curl -s http://localhost:3000/health || echo "error")
if [[ "$health_response" != "error" ]] && echo "$health_response" | grep -q '"status"'; then
    echo -e "${GREEN}✅ 健康检查通过${NC}"
    pass_count=$((pass_count + 1))
else
    echo -e "${RED}❌ 健康检查失败${NC}"
    echo "Response: $health_response"
fi

echo ""

# Test 2: Standard RPC
echo "🧪 Test 2: 标准RPC功能"
test_count=$((test_count + 1))
rpc_response=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"eth_supportedEntryPoints","params":[]}' || echo "error")
if [[ "$rpc_response" != "error" ]] && echo "$rpc_response" | grep -q '"result"'; then
    echo -e "${GREEN}✅ 标准RPC功能正常${NC}"
    pass_count=$((pass_count + 1))
else
    echo -e "${RED}❌ 标准RPC功能失败${NC}"
    echo "Response: $rpc_response"
fi

echo ""

# Test 3: Paymaster API Discovery
echo "🧪 Test 3: Paymaster API检测"
test_count=$((test_count + 1))
paymaster_response=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"pm_sponsorUserOperation","params":[{"sender":"invalid"}, "0x0000000000000000000000000000000000000007"]}' || echo "error")
if [[ "$paymaster_response" != "error" ]] && echo "$paymaster_response" | grep -q '"error"' && ! echo "$paymaster_response" | grep -q "Method not found"; then
    echo -e "${GREEN}✅ Paymaster API可用（收到期望的错误响应）${NC}"
    pass_count=$((pass_count + 1))
else
    echo -e "${RED}❌ Paymaster API不可用${NC}"
    echo "Response: $paymaster_response"
fi

echo ""

# Test 4: UserOperation Format Parsing
echo "🧪 Test 4: UserOperation格式解析"
test_count=$((test_count + 1))
user_op='{
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

format_response=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"pm_sponsorUserOperation\",\"params\":[$user_op, \"0x5FbDB2315678afecb367f032d93F642f64180aa3\"]}" || echo "error")
if [[ "$format_response" != "error" ]] && echo "$format_response" | grep -q '"error"' && ! echo "$format_response" | grep -q "Invalid params"; then
    echo -e "${GREEN}✅ UserOperation格式解析正常（到达业务逻辑层）${NC}"
    pass_count=$((pass_count + 1))
else
    echo -e "${YELLOW}⚠️  UserOperation格式解析结果异常${NC}"
    echo "Response: $format_response"
fi

echo ""

# 清理环境
echo "🧹 清理测试环境..."
if [[ -n "${GATEWAY_PID:-}" ]] && kill -0 $GATEWAY_PID 2>/dev/null; then
    kill $GATEWAY_PID
    sleep 2
fi

if [[ -n "${ANVIL_PID:-}" ]] && kill -0 $ANVIL_PID 2>/dev/null; then
    kill $ANVIL_PID
    sleep 1
fi

rm -f anvil.log gateway.log

# Summary
echo ""
echo "📊 测试结果总结"
echo "==============="
echo "🎯 通过率: $pass_count/$test_count ($((pass_count * 100 / test_count))%)"

if [[ $pass_count -ge $((test_count * 3 / 4)) ]]; then
    echo -e "${GREEN}🎉 SuperRelay简单功能测试通过！${NC}"
    exit 0
else
    echo -e "${RED}❌ SuperRelay简单功能测试失败${NC}"
    exit 1
fi