#!/bin/bash
# 专门测试方法名更改 (sponsorUserOperation -> pm_sponsorUserOperation) 的系统测试
# 验证新方法名在整个系统中的一致性和可用性

set -e

echo "🔬 SuperRelay 方法名迁移系统测试"
echo "=================================="
echo "测试范围: sponsorUserOperation -> pm_sponsorUserOperation"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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
    if curl -s -f -o /dev/null "$url"; then
        return 0
    else
        return 1
    fi
}

# Configuration
SERVICE_URL="http://localhost:3000"
SWAGGER_URL="http://localhost:9000"
ANVIL_URL="http://localhost:8545"

echo "📋 测试环境检查"
echo "----------------"

# Check if Anvil is running
if check_service "$ANVIL_URL"; then
    print_result 0 "Anvil blockchain 运行正常 (端口 8545)"
else
    print_result 1 "Anvil 未运行，需要启动: ./scripts/start_superrelay.sh"
fi

# Check if SuperRelay service is running
if curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
    "$SERVICE_URL" > /dev/null 2>&1; then
    print_result 0 "SuperRelay 主服务运行正常 (端口 3000)"
else
    print_result 1 "SuperRelay 主服务未运行，需要启动: ./scripts/start_superrelay.sh"
fi

# Check if Web UI is running
if check_service "$SWAGGER_URL"; then
    print_result 0 "Web UI 服务运行正常 (端口 9000)"
else
    print_result 1 "Web UI 服务未运行，需要启动: ./scripts/start_web_ui.sh"
fi

echo ""
echo "🔍 方法名一致性检查"
echo "--------------------"

# Test 1: Check OpenAPI spec contains new method name
echo -e "${BLUE}测试 1: OpenAPI 规范中的方法名${NC}"
if curl -s "$SWAGGER_URL/openapi.json" | grep -q "pm_sponsorUserOperation"; then
    print_result 0 "OpenAPI 规范包含新方法名 pm_sponsorUserOperation"
else
    print_result 1 "OpenAPI 规范不包含新方法名"
fi

# Test 2: Check old method name is not present (should be replaced)
echo -e "${BLUE}测试 2: 确认旧方法名已被替换${NC}"
if curl -s "$SWAGGER_URL/openapi.json" | grep -q '"sponsorUserOperation"' | grep -v pm_; then
    print_result 1 "OpenAPI 规范仍包含旧方法名 sponsorUserOperation (应该已被替换)"
else
    print_result 0 "OpenAPI 规范不包含旧方法名，替换成功"
fi

# Test 3: Test new method name with real service
echo -e "${BLUE}测试 3: 使用新方法名调用真实服务${NC}"
NEW_METHOD_REQUEST='{
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

NEW_METHOD_RESPONSE=$(curl -s -X POST "$SERVICE_URL" \
    -H "Content-Type: application/json" \
    -d "$NEW_METHOD_REQUEST" 2>/dev/null)

if echo "$NEW_METHOD_RESPONSE" | grep -q '"result"'; then
    print_result 0 "新方法名 pm_sponsorUserOperation 调用成功"
    echo "    响应包含 result 字段"
elif echo "$NEW_METHOD_RESPONSE" | grep -q '"error"'; then
    ERROR_MSG=$(echo "$NEW_METHOD_RESPONSE" | jq -r '.error.message' 2>/dev/null || echo "Parse failed")
    if echo "$ERROR_MSG" | grep -q -i "method.*not.*found\|unknown.*method"; then
        print_result 1 "新方法名不被服务识别: $ERROR_MSG"
    else
        print_result 0 "新方法名被服务识别，但返回业务错误: $ERROR_MSG"
    fi
else
    print_result 1 "新方法名调用失败，无响应或格式错误"
fi

# Test 4: Test old method name (should fail or be deprecated)
echo -e "${BLUE}测试 4: 使用旧方法名调用服务 (应该失败)${NC}"
OLD_METHOD_REQUEST='{
    "jsonrpc": "2.0",
    "method": "sponsorUserOperation",
    "params": [
        {
            "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "nonce": "0x0",
            "initCode": "0x",
            "callData": "0x",
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

OLD_METHOD_RESPONSE=$(curl -s -X POST "$SERVICE_URL" \
    -H "Content-Type: application/json" \
    -d "$OLD_METHOD_REQUEST" 2>/dev/null)

if echo "$OLD_METHOD_RESPONSE" | grep -q '"error"'; then
    ERROR_MSG=$(echo "$OLD_METHOD_RESPONSE" | jq -r '.error.message' 2>/dev/null || echo "Parse failed")
    if echo "$ERROR_MSG" | grep -q -i "method.*not.*found\|unknown.*method"; then
        print_result 0 "旧方法名正确返回 'method not found' 错误"
    else
        print_result 1 "旧方法名返回意外错误: $ERROR_MSG"
    fi
elif echo "$OLD_METHOD_RESPONSE" | grep -q '"result"'; then
    print_result 1 "旧方法名仍然有效，迁移可能不完整"
else
    print_result 1 "旧方法名测试异常，无法确定状态"
fi

# Test 5: Check RPC trait definition consistency
echo -e "${BLUE}测试 5: RPC trait 定义一致性${NC}"
if grep -r "namespace.*pm" crates/paymaster-relay/src/rpc.rs > /dev/null; then
    print_result 0 "RPC trait 定义包含 pm 命名空间"
else
    print_result 1 "RPC trait 定义缺少 pm 命名空间"
fi

if grep -r 'method.*name.*=.*"sponsorUserOperation"' crates/paymaster-relay/src/rpc.rs > /dev/null; then
    print_result 0 "RPC method 定义使用 sponsorUserOperation (正确，会自动加前缀)"
else
    print_result 1 "RPC method 定义不匹配预期格式"
fi

# Test 6: Check Swagger UI functionality with new method name
echo -e "${BLUE}测试 6: Swagger UI 与新方法名的集成${NC}"
if curl -s "$SWAGGER_URL/" | grep -q "pm_sponsorUserOperation"; then
    print_result 0 "Swagger UI 包含新方法名"
else
    print_result 1 "Swagger UI 不包含新方法名"
fi

echo ""
echo "🧪 端到端测试"
echo "--------------"

# Test 7: Full end-to-end test with Swagger UI proxy
echo -e "${BLUE}测试 7: Swagger UI 代理转发功能${NC}"
# This test the proxy functionality implemented in swagger.rs
PROXY_RESPONSE=$(curl -s -X POST "$SWAGGER_URL/api/v1/sponsor" \
    -H "Content-Type: application/json" \
    -d '{
        "user_operation": {
            "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "nonce": "0x0",
            "initCode": "0x",
            "callData": "0x",
            "callGasLimit": "0x30D40",
            "verificationGasLimit": "0x186A0",
            "preVerificationGas": "0xC350",
            "maxFeePerGas": "0x59682F00",
            "maxPriorityFeePerGas": "0x59682F00",
            "paymasterAndData": "0x",
            "signature": "0xfffffffffffffffffffffffffffffff0000000000000000000000000000000007aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1c"
        },
        "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
    }' 2>/dev/null)

if echo "$PROXY_RESPONSE" | grep -q "paymaster_and_data\|paymasterAndData"; then
    print_result 0 "Swagger UI 代理成功转发到真实服务"
elif echo "$PROXY_RESPONSE" | grep -q "not running"; then
    print_result 1 "Swagger UI 代理检测到服务未运行"
else
    print_result 1 "Swagger UI 代理功能异常: $(echo $PROXY_RESPONSE | head -c 100)"
fi

echo ""
echo "📊 测试结果总结"
echo "================"
echo -e "${GREEN}通过: $PASSED${NC}"
echo -e "${RED}失败: $FAILED${NC}"
echo ""

# Final assessment
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 方法名迁移测试全部通过！${NC}"
    echo ""
    echo "✅ 迁移确认："
    echo "  • 旧方法名: sponsorUserOperation (已废弃)"
    echo "  • 新方法名: pm_sponsorUserOperation (正常工作)"
    echo "  • OpenAPI 规范: 已更新"
    echo "  • Swagger UI: 工作正常"
    echo "  • 服务集成: 完全兼容"
    echo ""
    echo "🚀 系统可以正常使用新的 API 方法名！"
    exit 0
elif [ $FAILED -le 2 ]; then
    echo -e "${YELLOW}⚠️  方法名迁移基本成功，有少量问题需要关注${NC}"
    echo ""
    echo "💡 建议："
    echo "  • 检查失败的测试项目"
    echo "  • 确认所有服务都已重启"
    echo "  • 验证配置文件是否正确"
    exit 1
else
    echo -e "${RED}❌ 方法名迁移存在重大问题${NC}"
    echo ""
    echo "🔧 需要修复："
    echo "  • 检查 RPC 方法定义"
    echo "  • 验证服务间通信"
    echo "  • 确认 OpenAPI 规范一致性"
    exit 2
fi