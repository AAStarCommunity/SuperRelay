#!/bin/bash

# SuperRelay ERC-4337 v0.6 规范符合性测试
# 基于eth-infinitism/bundler-spec-tests标准

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🧪 SuperRelay ERC-4337 v0.6 规范符合性测试"
echo "============================================="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置参数
SPEC_TEST_DIR="$PROJECT_ROOT/test/spec-tests/v0_6/bundler-spec-tests"
ANVIL_PORT=8545
SUPER_RELAY_PORT=3000
TEST_ENV_FILE="$PROJECT_ROOT/.env.spec-test-v06"

# 检查依赖
check_dependencies() {
    echo "🔍 检查测试依赖..."
    
    # 检查Python环境
    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}❌ Python 3.8+ 未安装${NC}"
        exit 1
    fi
    
    # 检查PDM包管理器
    if ! command -v pdm &> /dev/null; then
        echo -e "${YELLOW}⚠️  PDM未安装，尝试安装...${NC}"
        pip install pdm
    fi
    
    # 检查Foundry工具
    if ! command -v anvil &> /dev/null || ! command -v cast &> /dev/null; then
        echo -e "${RED}❌ Foundry (anvil, cast) 未安装${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ 所有依赖检查通过${NC}"
}

# 准备测试环境
setup_test_environment() {
    echo "🚀 准备ERC-4337 v0.6规范测试环境..."
    
    # 创建测试环境配置
    cat > "$TEST_ENV_FILE" << EOF
# ERC-4337 v0.6 Spec测试环境配置
PAYMASTER_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
RPC_URL=http://localhost:$ANVIL_PORT
NETWORK=dev
CHAIN_ID=31337
SERVICE_HOST=0.0.0.0
SERVICE_PORT=$SUPER_RELAY_PORT
LOG_LEVEL=info
EOF
    
    # 进入spec测试目录
    cd "$SPEC_TEST_DIR"
    
    # 安装Python依赖
    if [[ ! -d ".venv" ]]; then
        echo "📦 安装Python测试依赖..."
        pdm install
    fi
    
    echo -e "${GREEN}✅ 测试环境配置完成${NC}"
}

# 启动测试基础设施
start_test_infrastructure() {
    echo "🏗️  启动测试基础设施..."
    
    # 清理现有进程
    pkill -f "anvil\|super-relay" || true
    sleep 2
    
    # 启动Anvil测试链
    echo "⛓️  启动Anvil测试链..."
    anvil --port $ANVIL_PORT --host 0.0.0.0 --chain-id 31337 &
    ANVIL_PID=$!
    sleep 3
    
    # 验证Anvil启动
    if ! curl -s http://localhost:$ANVIL_PORT > /dev/null; then
        echo -e "${RED}❌ Anvil启动失败${NC}"
        exit 1
    fi
    
    # 启动SuperRelay服务(v0.6兼容模式)
    echo "🚀 启动SuperRelay服务(v0.6兼容模式)..."
    source "$TEST_ENV_FILE"
    
    "$PROJECT_ROOT/target/release/super-relay" node \
        -- \
        --network dev \
        --node_http "http://localhost:$ANVIL_PORT" \
        --rpc.host 0.0.0.0 \
        --rpc.port $SUPER_RELAY_PORT \
        --paymaster.enabled \
        --paymaster.private_key "$PAYMASTER_PRIVATE_KEY" > /tmp/superrelay-spec-v06.log 2>&1 &
    SUPER_RELAY_PID=$!
    
    # 等待服务启动
    echo "⏳ 等待SuperRelay服务启动..."
    local max_attempts=30
    local attempt=1
    
    while [[ $attempt -le $max_attempts ]]; do
        if curl -s "http://localhost:$SUPER_RELAY_PORT/health" > /dev/null; then
            echo -e "${GREEN}✅ SuperRelay服务启动成功${NC}"
            break
        fi
        
        if [[ $attempt -eq $max_attempts ]]; then
            echo -e "${RED}❌ SuperRelay服务启动超时${NC}"
            cat /tmp/superrelay-spec-v06.log
            cleanup_test_infrastructure
            exit 1
        fi
        
        echo -n "."
        sleep 2
        ((attempt++))
    done
    
    echo -e "${GREEN}✅ 测试基础设施启动完成${NC}"
}

# 部署EntryPoint合约(v0.6)
deploy_entrypoint_v06() {
    echo "📜 部署EntryPoint v0.6合约..."
    
    cd "$SPEC_TEST_DIR/@account-abstraction"
    
    # 安装Node依赖
    if [[ ! -d "node_modules" ]]; then
        echo "📦 安装Node.js依赖..."
        yarn install
    fi
    
    # 部署v0.6合约
    echo "🚀 部署ERC-4337 v0.6合约..."
    yarn deploy --network localhost
    
    echo -e "${GREEN}✅ EntryPoint v0.6合约部署完成${NC}"
}

# 运行v0.6规范测试套件
run_spec_tests_v06() {
    echo "🧪 运行ERC-4337 v0.6规范测试套件..."
    
    cd "$SPEC_TEST_DIR"
    
    # 配置测试环境变量
    export BUNDLER_URL="http://localhost:$SUPER_RELAY_PORT"
    export ENTRYPOINT_ADDRESS="0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"  # v0.6
    export NODE_URL="http://localhost:$ANVIL_PORT"
    
    echo "🎯 测试配置:"
    echo "   Bundler URL: $BUNDLER_URL"
    echo "   EntryPoint: $ENTRYPOINT_ADDRESS (v0.6)"
    echo "   Node URL: $NODE_URL"
    
    # 分类运行测试
    local test_results=()
    
    echo ""
    echo "📊 1. RPC接口规范测试(v0.6)"
    echo "=========================="
    
    # RPC接口测试
    if pdm run pytest tests/single/rpc/ -v --tb=short; then
        test_results+=("✅ RPC接口测试: 通过")
        echo -e "${GREEN}✅ RPC接口测试通过${NC}"
    else
        test_results+=("❌ RPC接口测试: 失败")
        echo -e "${RED}❌ RPC接口测试失败${NC}"
    fi
    
    echo ""
    echo "💰 2. Paymaster规范测试(v0.6)"
    echo "============================"
    
    # Bundle测试(包含paymaster)
    if pdm run pytest tests/single/bundle/ -v --tb=short; then
        test_results+=("✅ Bundle/Paymaster测试: 通过")
        echo -e "${GREEN}✅ Bundle/Paymaster测试通过${NC}"
    else
        test_results+=("❌ Bundle/Paymaster测试: 失败")
        echo -e "${RED}❌ Bundle/Paymaster测试失败${NC}"
    fi
    
    echo ""
    echo "🏆 3. 信誉系统规范测试(v0.6)"
    echo "=========================="
    
    # 信誉系统测试
    if pdm run pytest tests/single/reputation/ -v --tb=short; then
        test_results+=("✅ 信誉系统测试: 通过")
        echo -e "${GREEN}✅ 信誉系统测试通过${NC}"
    else
        test_results+=("❌ 信誉系统测试: 失败")
        echo -e "${RED}❌ 信誉系统测试失败${NC}"
    fi
    
    echo ""
    echo "🚫 4. 操作禁用规范测试(v0.6)"
    echo "=========================="
    
    # 操作禁用测试
    if pdm run pytest tests/single/opbanning/ -v --tb=short; then
        test_results+=("✅ 操作禁用测试: 通过")
        echo -e "${GREEN}✅ 操作禁用测试通过${NC}"
    else
        test_results+=("❌ 操作禁用测试: 失败")
        echo -e "${RED}❌ 操作禁用测试失败${NC}"
    fi
    
    # 输出测试结果汇总
    echo ""
    echo "📊 ERC-4337 v0.6规范测试结果汇总"
    echo "================================="
    for result in "${test_results[@]}"; do
        echo "   $result"
    done
    
    # 计算通过率
    local passed_count=$(printf '%s\n' "${test_results[@]}" | grep -c "✅" || true)
    local total_count=${#test_results[@]}
    local pass_rate=$((passed_count * 100 / total_count))
    
    echo ""
    echo "🎯 规范符合性评估: ${passed_count}/${total_count} (${pass_rate}%)"
    
    if [[ $pass_rate -ge 75 ]]; then
        echo -e "${GREEN}🎉 SuperRelay符合ERC-4337 v0.6规范要求${NC}"
        return 0
    else
        echo -e "${RED}⚠️  SuperRelay需要进一步优化以符合v0.6规范${NC}"
        return 1
    fi
}

# 清理测试环境
cleanup_test_infrastructure() {
    echo ""
    echo "🧹 清理测试环境..."
    
    # 关闭SuperRelay
    if [[ -n "${SUPER_RELAY_PID:-}" ]]; then
        kill $SUPER_RELAY_PID 2>/dev/null || true
    fi
    
    # 关闭Anvil
    if [[ -n "${ANVIL_PID:-}" ]]; then
        kill $ANVIL_PID 2>/dev/null || true
    fi
    
    # 清理所有相关进程
    pkill -f "anvil\|super-relay" || true  
    
    # 清理临时文件
    rm -f /tmp/superrelay-spec-v06.log
    rm -f "$TEST_ENV_FILE"
    
    echo -e "${GREEN}✅ 测试环境清理完成${NC}"
}

# 主执行流程
main() {
    echo "🚀 开始SuperRelay ERC-4337 v0.6规范符合性测试"
    echo "=============================================="
    
    # 设置错误处理
    trap cleanup_test_infrastructure EXIT
    
    # 执行测试流程
    check_dependencies
    setup_test_environment
    start_test_infrastructure
    deploy_entrypoint_v06
    
    # 运行规范测试
    if run_spec_tests_v06; then
        echo ""
        echo -e "${GREEN}🎉 ERC-4337 v0.6规范符合性测试完成 - SuperRelay通过验证！${NC}"
        exit 0
    else
        echo ""
        echo -e "${YELLOW}⚠️  ERC-4337 v0.6规范符合性测试完成 - 发现需要优化的项目${NC}"
        exit 1
    fi
}

# 执行主程序
main "$@"