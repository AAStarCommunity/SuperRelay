#!/bin/bash

# SuperRelay 测试套件管理器
# 启动一次服务，运行所有测试，最后清理
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🚀 SuperRelay 测试套件管理器"
echo "==========================="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 全局服务状态
ANVIL_PID=""
GATEWAY_PID=""
SERVICE_RUNNING=false

# 清理函数
cleanup() {
    echo ""
    echo "🧹 清理测试环境..."
    
    if [[ -n "${GATEWAY_PID:-}" ]] && kill -0 $GATEWAY_PID 2>/dev/null; then
        echo "  🛑 停止 SuperRelay Gateway (PID: $GATEWAY_PID)"
        kill $GATEWAY_PID
        sleep 2
    fi
    
    if [[ -n "${ANVIL_PID:-}" ]] && kill -0 $ANVIL_PID 2>/dev/null; then
        echo "  🛑 停止 Anvil (PID: $ANVIL_PID)"  
        kill $ANVIL_PID
        sleep 1
    fi
    
    # 强制清理
    pkill -f "super-relay|anvil" 2>/dev/null || true
    
    # 清理日志文件
    rm -f anvil.log gateway.log
    
    echo -e "${GREEN}✅ 清理完成${NC}"
}

# 设置清理陷阱
trap cleanup EXIT INT TERM

# 检查并启动服务
start_services() {
    echo "🔧 检查测试服务状态..."
    
    # 清理可能的残留进程
    pkill -f "super-relay|anvil" 2>/dev/null || true
    sleep 2
    
    # 检查二进制文件
    SUPER_RELAY_BIN=""
    if [[ -f "$PROJECT_ROOT/target/release/super-relay" ]]; then
        SUPER_RELAY_BIN="$PROJECT_ROOT/target/release/super-relay"
        echo -e "${GREEN}✅ 使用 release 版本${NC}"
    elif [[ -f "$PROJECT_ROOT/target/debug/super-relay" ]]; then
        SUPER_RELAY_BIN="$PROJECT_ROOT/target/debug/super-relay"
        echo -e "${GREEN}✅ 使用 debug 版本${NC}"
    else
        echo -e "${RED}❌ super-relay 二进制文件不存在${NC}"
        echo "请运行: ./scripts/build.sh"
        exit 1
    fi
    
    # 设置环境变量
    export SIGNER_PRIVATE_KEYS="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    export PAYMASTER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    export RPC_URL="http://localhost:8545"
    export NETWORK="dev"
    export CHAIN_ID="31337"
    
    # 启动 Anvil
    echo "🔥 启动 Anvil 测试链..."
    anvil --host 0.0.0.0 --port 8545 --chain-id 31337 > anvil.log 2>&1 &
    ANVIL_PID=$!
    sleep 3
    
    # 验证 Anvil
    if ! curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' http://localhost:8545 > /dev/null; then
        echo -e "${RED}❌ Anvil 启动失败${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Anvil 启动成功 (PID: $ANVIL_PID)${NC}"
    
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
    echo "⏳ 等待服务启动 (最大30秒)..."
    for i in {1..15}; do
        if curl -s http://localhost:3000/health > /dev/null 2>&1; then
            echo -e "${GREEN}✅ SuperRelay Gateway 启动成功 (PID: $GATEWAY_PID)${NC}"
            SERVICE_RUNNING=true
            return 0
        fi
        
        # 检查进程是否还在运行
        if ! kill -0 $GATEWAY_PID 2>/dev/null; then
            echo -e "${RED}❌ SuperRelay Gateway 进程已退出${NC}"
            echo "=== Gateway 日志 ==="
            cat gateway.log 2>/dev/null || echo "无日志"
            exit 1
        fi
        
        echo -n "."
        sleep 2
    done
    
    echo -e "${RED}❌ SuperRelay Gateway 启动超时${NC}"
    exit 1
}

# 运行单个测试
run_test() {
    local test_name="$1"
    local test_description="$2"
    local test_function="$3"
    
    echo ""
    echo -e "${BLUE}🧪 [$test_name] $test_description${NC}"
    echo "   测试时间: $(date '+%H:%M:%S')"
    
    if $test_function; then
        echo -e "${GREEN}   ✅ 测试通过${NC}"
        return 0
    else
        echo -e "${RED}   ❌ 测试失败${NC}"
        return 1
    fi
}

# 测试函数定义
test_health_check() {
    local response=$(curl -s http://localhost:3000/health || echo "error")
    [[ "$response" != "error" ]] && echo "$response" | grep -q '"status"'
}

test_ready_check() {
    local response=$(curl -s http://localhost:3000/ready || echo "error") 
    [[ "$response" != "error" ]]
}

test_live_check() {
    local response=$(curl -s http://localhost:3000/live || echo "error")
    [[ "$response" != "error" ]]
}

test_metrics() {
    local response=$(curl -s http://localhost:3000/metrics || echo "error")
    [[ "$response" != "error" ]]
}

test_rpc_basic() {
    local response=$(curl -s -X POST http://localhost:3000 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","id":1,"method":"eth_supportedEntryPoints","params":[]}' || echo "error")
    [[ "$response" != "error" ]] && echo "$response" | grep -q '"result"'
}

test_rpc_error_handling() {
    local response=$(curl -s -X POST http://localhost:3000 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","id":1,"method":"invalid_method","params":[]}' || echo "error")
    [[ "$response" != "error" ]] && echo "$response" | grep -q '"error"'
}

test_paymaster_api() {
    local response=$(curl -s -X POST http://localhost:3000 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","id":1,"method":"pm_sponsorUserOperation","params":[{"sender":"invalid"}, "0x0000000000000000000000000000000000000007"]}' || echo "error")
    [[ "$response" != "error" ]] && echo "$response" | grep -q '"error"' && ! echo "$response" | grep -q "Method not found"
}

# 主测试流程
main() {
    echo "📋 测试计划:"
    echo "   🏥 健康检查系统测试"
    echo "   🚀 基础网关功能测试"  
    echo "   🧪 简单功能测试"
    echo ""
    
    # 启动服务
    start_services
    
    if [[ "$SERVICE_RUNNING" != "true" ]]; then
        echo -e "${RED}❌ 服务启动失败，无法继续测试${NC}"
        exit 1
    fi
    
    echo ""
    echo -e "${BLUE}🚀 开始测试套件执行...${NC}"
    echo "================================="
    
    # 执行测试
    local total_tests=0
    local passed_tests=0
    
    # 健康检查测试组
    echo -e "${YELLOW}📋 健康检查测试组${NC}"
    tests=(
        "health_check:综合健康检查:test_health_check"
        "ready_check:就绪检查:test_ready_check"
        "live_check:存活检查:test_live_check"
        "metrics:监控指标:test_metrics"
    )
    
    for test in "${tests[@]}"; do
        IFS=':' read -r name desc func <<< "$test"
        total_tests=$((total_tests + 1))
        if run_test "$name" "$desc" "$func"; then
            passed_tests=$((passed_tests + 1))
        fi
    done
    
    # 基础功能测试组
    echo ""
    echo -e "${YELLOW}📋 基础功能测试组${NC}"
    tests=(
        "rpc_basic:标准RPC功能:test_rpc_basic"
        "rpc_error:错误处理机制:test_rpc_error_handling"
        "paymaster_api:Paymaster API:test_paymaster_api"
    )
    
    for test in "${tests[@]}"; do
        IFS=':' read -r name desc func <<< "$test"
        total_tests=$((total_tests + 1))
        if run_test "$name" "$desc" "$func"; then
            passed_tests=$((passed_tests + 1))
        fi
    done
    
    # 测试结果总结
    echo ""
    echo "================================="
    echo -e "${BLUE}📊 测试结果总结${NC}"
    echo "================================="
    echo "🎯 总测试数: $total_tests"
    echo "✅ 通过数: $passed_tests"  
    echo "❌ 失败数: $((total_tests - passed_tests))"
    echo "📈 通过率: $((passed_tests * 100 / total_tests))%"
    
    if [[ $passed_tests -eq $total_tests ]]; then
        echo ""
        echo -e "${GREEN}🎉 所有测试通过！SuperRelay 功能正常！${NC}"
        return 0
    else
        echo ""
        echo -e "${RED}⚠️  部分测试失败，请检查相关功能${NC}"
        return 1
    fi
}

# 显示使用帮助
show_help() {
    echo "SuperRelay 测试套件管理器"
    echo ""
    echo "使用方法:"
    echo "  $0                    # 运行完整测试套件"
    echo "  $0 --help           # 显示帮助信息"
    echo ""
    echo "特性:"
    echo "  • 一次启动服务，运行所有测试"
    echo "  • 自动清理测试环境"
    echo "  • 详细的测试报告"
    echo "  • 支持 debug 和 release 版本"
}

# 解析命令行参数
case "${1:-}" in
    --help|-h)
        show_help
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac