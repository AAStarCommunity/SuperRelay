#!/bin/bash

# 简单的健康检查测试脚本 - 更快速和可靠的版本
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🏥 SuperRelay 简单健康检查测试"
echo "=================================="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查二进制文件
SUPER_RELAY_BIN_RELEASE="$PROJECT_ROOT/target/release/super-relay"
SUPER_RELAY_BIN_DEBUG="$PROJECT_ROOT/target/debug/super-relay"

# 优先使用release版本，如果不存在则使用debug版本
if [[ -f "$SUPER_RELAY_BIN_RELEASE" ]]; then
    SUPER_RELAY_BIN="$SUPER_RELAY_BIN_RELEASE"
    echo -e "${GREEN}✅ 发现 super-relay 二进制文件 (release)${NC}"
elif [[ -f "$SUPER_RELAY_BIN_DEBUG" ]]; then
    SUPER_RELAY_BIN="$SUPER_RELAY_BIN_DEBUG"
    echo -e "${GREEN}✅ 发现 super-relay 二进制文件 (debug)${NC}"
else
    echo -e "${RED}❌ super-relay 二进制文件不存在${NC}"
    echo "请运行: ./scripts/build.sh"
    exit 1
fi

# 清理可能存在的进程
pkill -f "super-relay" || true
pkill -f "anvil" || true
sleep 2

# 设置测试环境变量
export SIGNER_PRIVATE_KEYS="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export PAYMASTER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export RPC_URL="http://localhost:8545"
export NETWORK="dev"
export CHAIN_ID="31337"

echo "🔧 启动测试环境..."

# 启动 Anvil
echo "🔥 启动 Anvil..."
anvil --host 0.0.0.0 --port 8545 --chain-id $CHAIN_ID > anvil.log 2>&1 &
ANVIL_PID=$!
sleep 3

# 验证 Anvil
if ! curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' $RPC_URL > /dev/null; then
    echo -e "${RED}❌ Anvil 启动失败${NC}"
    kill $ANVIL_PID 2>/dev/null || true
    exit 1
fi
echo -e "${GREEN}✅ Anvil 启动成功${NC}"

# 启动 Gateway 模式（更快启动）
echo "🚀 启动 Gateway 模式..."
$SUPER_RELAY_BIN gateway \
    --host 127.0.0.1 \
    --port 3000 \
    --enable-paymaster \
    --paymaster-private-key "$PAYMASTER_PRIVATE_KEY" \
    > gateway.log 2>&1 &
GATEWAY_PID=$!

echo "⏳ 等待 Gateway 启动 (15秒)..."
sleep 15

# 检查进程
if ! kill -0 $GATEWAY_PID 2>/dev/null; then
    echo -e "${RED}❌ Gateway 启动失败${NC}"
    cat gateway.log
    kill $ANVIL_PID 2>/dev/null || true
    exit 1
fi
echo -e "${GREEN}✅ Gateway 启动成功 (PID: $GATEWAY_PID)${NC}"

# 简单的健康检查测试
echo ""
echo "🔍 执行健康检查测试..."

test_passed=0
test_total=0

# 测试函数
test_endpoint() {
    local endpoint=$1
    local description=$2
    local expected_status=${3:-200}

    test_total=$((test_total + 1))
    echo -n "  Testing $description... "

    status_code=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3000$endpoint" 2>/dev/null || echo "000")

    if [[ "$status_code" == "$expected_status" ]]; then
        echo -e "${GREEN}✅ ($status_code)${NC}"
        test_passed=$((test_passed + 1))
    else
        echo -e "${RED}❌ ($status_code, expected $expected_status)${NC}"
    fi
}

# 执行测试
test_endpoint "/health" "综合健康检查"
test_endpoint "/ready" "就绪检查"
test_endpoint "/live" "存活检查"
test_endpoint "/metrics" "监控指标"

# 清理
echo ""
echo "🧹 清理测试环境..."

if kill -0 $GATEWAY_PID 2>/dev/null; then
    kill $GATEWAY_PID
    sleep 2
fi

if kill -0 $ANVIL_PID 2>/dev/null; then
    kill $ANVIL_PID
    sleep 1
fi

rm -f anvil.log gateway.log

# 测试结果
echo ""
echo "📊 测试结果: $test_passed/$test_total 通过"

if [[ $test_passed -eq $test_total ]]; then
    echo -e "${GREEN}🎉 所有健康检查测试通过！${NC}"
    exit 0
else
    echo -e "${RED}❌ 部分测试失败${NC}"
    exit 1
fi