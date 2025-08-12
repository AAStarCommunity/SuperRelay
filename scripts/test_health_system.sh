#!/bin/bash

# SuperRelay 内部状态检测系统测试脚本
# 测试新实现的健康检查功能

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🏥 SuperRelay 内部状态检测系统测试"
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
    echo -e "${YELLOW}⚠️  super-relay 二进制文件不存在，尝试构建...${NC}"
    echo "构建 super-relay debug 版本..."
    if cargo build --package super-relay; then
        SUPER_RELAY_BIN="$SUPER_RELAY_BIN_DEBUG"
        echo -e "${GREEN}✅ super-relay 构建成功${NC}"
    else
        echo -e "${RED}❌ super-relay 构建失败${NC}"
        exit 1
    fi
fi

# 启动测试服务器
echo ""
echo "🚀 启动测试服务器..."

# 清理可能存在的进程
pkill -f "super-relay" || true
sleep 2

# 设置测试环境变量
echo "🔧 设置测试环境变量..."
export SIGNER_PRIVATE_KEYS="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80,0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2"
export PAYMASTER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export RPC_URL="http://localhost:8545"
export NETWORK="dev"
export CHAIN_ID="31337"

# 启动 Anvil 用于测试
echo "🔥 启动测试用 Anvil..."
# 清理可能存在的 Anvil 进程
pkill -f "anvil" || true
sleep 1

# 检查 Anvil 是否可用
if ! command -v anvil >/dev/null 2>&1; then
    echo -e "${RED}❌ anvil 命令不存在，请安装 Foundry${NC}"
    exit 1
fi

# 启动 Anvil
anvil --host 0.0.0.0 --port 8545 --chain-id $CHAIN_ID > anvil.log 2>&1 &
ANVIL_PID=$!
echo "Anvil PID: $ANVIL_PID"

# 等待 Anvil 启动
echo "等待 Anvil 启动..."
sleep 3

# 验证 Anvil 是否启动成功
if ! curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
    $RPC_URL > /dev/null; then
    echo -e "${RED}❌ Anvil 启动失败或无法连接${NC}"
    kill $ANVIL_PID 2>/dev/null || true
    exit 1
fi

echo -e "${GREEN}✅ Anvil 启动成功${NC}"

# 在后台启动 super-relay node 模式
echo "启动 Node 模式进行健康检查测试..."
$SUPER_RELAY_BIN node &
GATEWAY_PID=$!

# 等待服务启动
echo "等待服务启动..."
sleep 5

# 检查进程是否还在运行
if ! kill -0 $GATEWAY_PID 2>/dev/null; then
    echo -e "${RED}❌ Gateway 服务启动失败${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Gateway 服务启动成功 (PID: $GATEWAY_PID)${NC}"

# 测试函数
test_endpoint() {
    local endpoint=$1
    local description=$2
    local expected_status=${3:-200}

    echo ""
    echo -e "${BLUE}🔍 测试: $description${NC}"
    echo "   Endpoint: http://localhost:3000$endpoint"

    response=$(curl -s -w "\n%{http_code}" "http://localhost:3000$endpoint" 2>/dev/null || echo -e "\n000")

    # 分离响应体和状态码
    body=$(echo "$response" | sed '$d')
    status_code=$(echo "$response" | tail -n 1)

    if [[ "$status_code" == "$expected_status" ]]; then
        echo -e "${GREEN}   ✅ 状态码: $status_code${NC}"

        # 如果是 JSON 响应，尝试格式化
        if echo "$body" | jq . >/dev/null 2>&1; then
            echo -e "${GREEN}   📊 响应数据:${NC}"
            echo "$body" | jq . | head -20
        else
            echo -e "${GREEN}   📝 响应:${NC} $body"
        fi
    else
        echo -e "${RED}   ❌ 状态码: $status_code (期望: $expected_status)${NC}"
        echo -e "${RED}   📝 响应: $body${NC}"
        return 1
    fi
}

# 健康检查系统测试
echo ""
echo "🏥 健康检查系统测试"
echo "==================="

# 1. 综合健康检查
test_endpoint "/health" "综合健康检查 - 包含所有组件状态"

# 2. 就绪检查
test_endpoint "/ready" "就绪检查 - 负载均衡器使用"

# 3. 存活检查
test_endpoint "/live" "存活检查 - 基础服务状态"

# 4. Prometheus 指标
test_endpoint "/metrics" "监控指标 - Prometheus 格式"

# 清理
echo ""
echo "🧹 清理测试环境..."

# 优雅关闭 Gateway
if kill -0 $GATEWAY_PID 2>/dev/null; then
    kill -TERM $GATEWAY_PID
    sleep 3

    # 如果还没关闭，强制终止
    if kill -0 $GATEWAY_PID 2>/dev/null; then
        kill -KILL $GATEWAY_PID
    fi
fi

# 关闭 Anvil
if [[ -n "${ANVIL_PID:-}" ]] && kill -0 $ANVIL_PID 2>/dev/null; then
    kill -TERM $ANVIL_PID
    sleep 2

    # 如果还没关闭，强制终止
    if kill -0 $ANVIL_PID 2>/dev/null; then
        kill -KILL $ANVIL_PID
    fi
fi

# 清理日志文件
rm -f anvil.log

echo -e "${GREEN}✅ 测试环境清理完成${NC}"

echo ""
echo "📊 测试总结"
echo "==========="
echo "✅ 内部状态检测系统已成功实现"
echo "✅ 提供了三种不同级别的健康检查:"
echo "   • /health - 综合健康检查 (包含详细组件状态)"
echo "   • /ready  - 就绪检查 (适用于负载均衡器)"
echo "   • /live   - 存活检查 (基础服务状态)"
echo "✅ 集成了系统监控指标"
echo "✅ 支持组件级别的状态监控"

echo ""
echo -e "${GREEN}🎉 内部状态检测系统测试完成！${NC}"