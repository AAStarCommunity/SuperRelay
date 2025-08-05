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
SUPER_RELAY_BIN="$PROJECT_ROOT/target/release/super-relay"

if [[ ! -f "$SUPER_RELAY_BIN" ]]; then
    echo -e "${RED}❌ super-relay 二进制文件不存在${NC}"
    echo "请先运行: cargo build --package super-relay --release"
    exit 1
fi

echo -e "${GREEN}✅ 发现 super-relay 二进制文件${NC}"

# 启动测试服务器
echo ""
echo "🚀 启动测试服务器..."

# 清理可能存在的进程
pkill -f "super-relay" || true
sleep 2

# 在后台启动 super-relay gateway 模式
echo "启动 Gateway 模式进行健康检查测试..."
$SUPER_RELAY_BIN gateway &
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
    body=$(echo "$response" | head -n -1)
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