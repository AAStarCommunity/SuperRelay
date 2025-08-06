#!/bin/bash

# SuperRelay 基础Gateway功能测试
# 简化版本用于快速验证基本功能

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🚀 SuperRelay 基础Gateway功能测试"
echo "================================="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试结果数组
declare -a test_results

# 清理现有进程
echo "🧹 清理现有进程..."
pkill -f "anvil|super-relay|rundler" || true
sleep 2

# 检查基础工具
echo "🔍 检查基础工具..."
if ! command -v anvil &> /dev/null; then
    echo -e "${RED}❌ Foundry (anvil) 未安装${NC}"
    exit 1
fi

if ! command -v curl &> /dev/null; then
    echo -e "${RED}❌ curl 未安装${NC}"
    exit 1
fi

if [[ ! -f "$PROJECT_ROOT/target/debug/super-relay" ]]; then
    echo -e "${RED}❌ SuperRelay debug版本不存在${NC}"
    exit 1
fi

echo -e "${GREEN}✅ 基础工具检查通过${NC}"

# 启动Anvil测试链
echo "⛓️  启动Anvil测试链..."
anvil --port 8545 --host 0.0.0.0 --chain-id 31337 &
ANVIL_PID=$!
sleep 3

# 验证Anvil启动
if ! curl -s http://localhost:8545 > /dev/null; then
    echo -e "${RED}❌ Anvil启动失败${NC}"
    kill $ANVIL_PID 2>/dev/null || true
    exit 1
fi
echo -e "${GREEN}✅ Anvil测试链启动成功${NC}"

# 测试用配置文件
cat > "$PROJECT_ROOT/config/test-gateway.toml" << 'EOF'
[network]
name = "dev"
node_http = "http://localhost:8545"
chain_id = 31337

[gateway]
enabled = true
host = "0.0.0.0"
port = 3000

[paymaster]
enabled = true
private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

[rpc]
api = ["eth", "rundler", "paymaster"]
EOF

# 启动SuperRelay Gateway
echo "🚀 启动SuperRelay Gateway..."
export PAYMASTER_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# 使用super-relay gateway模式
echo "🔧 启动SuperRelay Gateway服务..."
"$PROJECT_ROOT/target/debug/super-relay" gateway \
    --config config/config.toml \
    --host 0.0.0.0 \
    --port 3000 \
    --enable-paymaster \
    --paymaster-private-key "$PAYMASTER_PRIVATE_KEY" > /tmp/super-relay-test.log 2>&1 &
SUPER_RELAY_PID=$!

# 等待服务启动
echo "⏳ 等待服务启动..."
max_attempts=15
attempt=1

while [[ $attempt -le $max_attempts ]]; do
    if curl -s "http://localhost:3000" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ 服务启动成功${NC}"
        break
    fi
    
    if [[ $attempt -eq $max_attempts ]]; then
        echo -e "${RED}❌ 服务启动超时${NC}"
        echo "=== 服务日志 ==="
        cat /tmp/super-relay-test.log 2>/dev/null || echo "无日志文件"
        kill $SUPER_RELAY_PID $ANVIL_PID 2>/dev/null || true
        exit 1
    fi
    
    echo -n "."
    sleep 2
    ((attempt++))
done

# 基础功能测试
echo ""
echo "🧪 开始基础功能测试..."

# 1. 健康检查测试
echo "🏥 1. 健康检查测试"
if curl -s "http://localhost:3000" | grep -q "Method not found\|Healthy"; then
    test_results+=("✅ 服务响应测试: 通过")
    echo -e "${GREEN}   ✅ 服务正常响应${NC}"
else
    test_results+=("❌ 服务响应测试: 失败")
    echo -e "${RED}   ❌ 服务无响应${NC}"
fi

# 2. RPC接口测试
echo "📡 2. RPC接口基础测试"
rpc_response=$(curl -s -X POST http://localhost:3000 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc": "2.0", "id": 1, "method": "eth_supportedEntryPoints", "params": []}' || echo "error")

if [[ "$rpc_response" != "error" ]] && echo "$rpc_response" | grep -q "jsonrpc"; then
    test_results+=("✅ RPC接口测试: 通过")
    echo -e "${GREEN}   ✅ RPC接口正常响应${NC}"
else
    test_results+=("❌ RPC接口测试: 失败")
    echo -e "${RED}   ❌ RPC接口无响应${NC}"
fi

# 3. 错误处理测试
echo "🔍 3. 错误处理测试"
error_response=$(curl -s -X POST http://localhost:3000 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc": "2.0", "id": 1, "method": "invalid_method", "params": []}' || echo "error")

if [[ "$error_response" != "error" ]] && echo "$error_response" | grep -q "error"; then
    test_results+=("✅ 错误处理测试: 通过")
    echo -e "${GREEN}   ✅ 错误处理正常${NC}"
else
    test_results+=("❌ 错误处理测试: 失败") 
    echo -e "${RED}   ❌ 错误处理异常${NC}"
fi

# 清理环境
echo ""
echo "🧹 清理测试环境..."
kill $SUPER_RELAY_PID $ANVIL_PID 2>/dev/null || true
pkill -f "anvil|super-relay" || true
rm -f /tmp/super-relay-test.log
rm -f "$PROJECT_ROOT/config/test-gateway.toml"

# 生成测试报告
echo ""
echo "📊 基础功能测试结果"
echo "=================="
for result in "${test_results[@]}"; do
    echo "   $result"
done

# 计算通过率
passed_count=$(printf '%s\n' "${test_results[@]}" | grep -c "✅" || true)
total_count=${#test_results[@]}
pass_rate=$((passed_count * 100 / total_count))

echo ""
echo "🎯 测试通过率: ${passed_count}/${total_count} (${pass_rate}%)"

if [[ $pass_rate -ge 70 ]]; then
    echo -e "${GREEN}🎉 SuperRelay基础功能正常${NC}"
    exit 0
else
    echo -e "${RED}⚠️  SuperRelay基础功能需要优化${NC}"
    exit 1
fi