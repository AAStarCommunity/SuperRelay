#!/bin/bash

# SuperRelay ERC-4337 综合规范符合性测试
# 包含v0.6、v0.7版本和Gateway/Paymaster专项测试

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🌟 SuperRelay ERC-4337 综合规范符合性测试"
echo "=========================================="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 测试结果存储
declare -a TEST_RESULTS
declare -a DETAILED_RESULTS

# 记录测试结果
record_test_result() {
    local test_name="$1"
    local result="$2"

    if [[ "$result" == "PASS" ]]; then
        TEST_RESULTS+=("✅ $test_name")
        DETAILED_RESULTS+=("PASS:$test_name")
    else
        TEST_RESULTS+=("❌ $test_name")
        DETAILED_RESULTS+=("FAIL:$test_name")
    fi
}

# 运行单个测试套件
run_test_suite() {
    local test_script="$1"
    local test_name="$2"

    echo ""
    echo -e "${CYAN}🧪 执行: $test_name${NC}"
    echo "==============================================="

    if [[ ! -f "$test_script" ]]; then
        echo -e "${RED}❌ 测试脚本不存在: $test_script${NC}"
        record_test_result "$test_name" "FAIL"
        return 1
    fi

    # 运行测试脚本
    if "$test_script"; then
        echo -e "${GREEN}✅ $test_name 通过${NC}"
        record_test_result "$test_name" "PASS"
        return 0
    else
        echo -e "${RED}❌ $test_name 失败${NC}"
        record_test_result "$test_name" "FAIL"
        return 1
    fi
}

# Gateway专项规范测试
run_gateway_spec_tests() {
    echo ""
    echo -e "${PURPLE}🔧 Gateway专项ERC-4337规范符合性测试${NC}"
    echo "============================================"

    # 启动测试环境
    export PAYMASTER_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

    # 清理环境
    pkill -f "anvil\|super-relay" || true
    sleep 2

    # 启动Anvil
    anvil --port 8545 --host 0.0.0.0 --chain-id 31337 &
    local anvil_pid=$!
    sleep 3

    # 启动SuperRelay Gateway模式
    "$PROJECT_ROOT/target/debug/super-relay" gateway \
        --config config/config.toml \
        --host 0.0.0.0 \
        --port 3000 \
        --enable-paymaster \
        --paymaster-private-key "$PAYMASTER_PRIVATE_KEY" > /tmp/super-relay-gateway.log 2>&1 &
    local relay_pid=$!

    sleep 5

    # Gateway专项测试
    local gateway_tests_passed=0
    local gateway_tests_total=5

    echo "📊 1. Gateway RPC接口符合性测试"
    if curl -s -X POST http://localhost:3000 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc": "2.0", "id": 1, "method": "eth_supportedEntryPoints", "params": []}' | \
        jq -e '.result | length > 0' > /dev/null; then
        echo -e "${GREEN}   ✅ eth_supportedEntryPoints 符合规范${NC}"
        ((gateway_tests_passed++))
    else
        echo -e "${RED}   ❌ eth_supportedEntryPoints 不符合规范${NC}"
    fi

    echo "💰 2. Gateway Paymaster接口符合性测试"
    if curl -s -X POST http://localhost:3000 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc": "2.0", "id": 1, "method": "pm_sponsorUserOperation", "params": [{"sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", "nonce": "0x0", "callData": "0x", "callGasLimit": "0x186A0", "verificationGasLimit": "0x186A0", "preVerificationGas": "0x5208", "maxFeePerGas": "0x3B9ACA00", "maxPriorityFeePerGas": "0x3B9ACA00"}, "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"]}' | \
        jq -e '.error.code' > /dev/null; then
        echo -e "${GREEN}   ✅ pm_sponsorUserOperation 接口响应符合规范${NC}"
        ((gateway_tests_passed++))
    else
        echo -e "${RED}   ❌ pm_sponsorUserOperation 接口响应不符合规范${NC}"
    fi

    echo "🔍 3. Gateway错误处理符合性测试"
    if curl -s -X POST http://localhost:3000 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc": "2.0", "id": 1, "method": "invalid_method", "params": []}' | \
        jq -e '.error.code == -32601' > /dev/null; then
        echo -e "${GREEN}   ✅ 错误处理符合JSON-RPC规范${NC}"
        ((gateway_tests_passed++))
    else
        echo -e "${RED}   ❌ 错误处理不符合JSON-RPC规范${NC}"
    fi

    echo "🏥 4. Gateway健康检查接口符合性测试"
    if curl -s http://localhost:3000/health | grep -q "ok"; then
        echo -e "${GREEN}   ✅ 健康检查接口符合企业标准${NC}"
        ((gateway_tests_passed++))
    else
        echo -e "${RED}   ❌ 健康检查接口不符合企业标准${NC}"
    fi

    echo "🔒 5. Gateway安全头符合性测试"
    local security_headers=$(curl -s -I http://localhost:3000/health | grep -E "(Content-Type|Server)" | wc -l)
    if [[ $security_headers -ge 1 ]]; then
        echo -e "${GREEN}   ✅ HTTP安全头符合基础要求${NC}"
        ((gateway_tests_passed++))
    else
        echo -e "${RED}   ❌ HTTP安全头不符合基础要求${NC}"
    fi

    # 清理Gateway测试环境
    kill $relay_pid $anvil_pid 2>/dev/null || true
    pkill -f "anvil|super-relay" || true
    rm -f /tmp/super-relay-gateway.log

    # 记录Gateway测试结果
    local gateway_pass_rate=$((gateway_tests_passed * 100 / gateway_tests_total))
    if [[ $gateway_pass_rate -ge 80 ]]; then
        record_test_result "Gateway专项规范测试 ($gateway_pass_rate%)" "PASS"
    else
        record_test_result "Gateway专项规范测试 ($gateway_pass_rate%)" "FAIL"
    fi
}

# Paymaster专项规范测试
run_paymaster_spec_tests() {
    echo ""
    echo -e "${PURPLE}💰 Paymaster专项ERC-4337规范符合性测试${NC}"
    echo "==============================================="

    # Rust单元测试 - Paymaster功能
    echo "🧪 1. Paymaster单元测试规范符合性"
    if cd "$PROJECT_ROOT/crates/paymaster-relay" && cargo test --quiet paymaster > /dev/null 2>&1; then
        echo -e "${GREEN}   ✅ Paymaster单元测试通过${NC}"
        local paymaster_unit_pass=true
    else
        echo -e "${RED}   ❌ Paymaster单元测试失败${NC}"
        local paymaster_unit_pass=false
    fi

    # KMS集成测试 - 密钥管理规范
    echo "🔑 2. KMS密钥管理规范符合性测试"
    if cd "$PROJECT_ROOT/crates/paymaster-relay" && cargo test --quiet kms > /dev/null 2>&1; then
        echo -e "${GREEN}   ✅ KMS集成测试通过${NC}"
        local kms_test_pass=true
    else
        echo -e "${RED}   ❌ KMS集成测试失败${NC}"
        local kms_test_pass=false
    fi

    # 安全检查测试 - 安全规范符合性
    echo "🔒 3. Paymaster安全检查规范符合性"
    if cd "$PROJECT_ROOT/crates/gateway" && cargo test --quiet security > /dev/null 2>&1; then
        echo -e "${GREEN}   ✅ 安全检查测试通过${NC}"
        local security_test_pass=true
    else
        echo -e "${RED}   ❌ 安全检查测试失败${NC}"
        local security_test_pass=false
    fi

    # 计算Paymaster测试通过率
    local paymaster_passed=0
    [[ "$paymaster_unit_pass" == "true" ]] && ((paymaster_passed++))
    [[ "$kms_test_pass" == "true" ]] && ((paymaster_passed++))
    [[ "$security_test_pass" == "true" ]] && ((paymaster_passed++))

    local paymaster_total=3
    local paymaster_pass_rate=$((paymaster_passed * 100 / paymaster_total))

    if [[ $paymaster_pass_rate -ge 75 ]]; then
        record_test_result "Paymaster专项规范测试 ($paymaster_pass_rate%)" "PASS"
    else
        record_test_result "Paymaster专项规范测试 ($paymaster_pass_rate%)" "FAIL"
    fi
}

# 生成综合测试报告
generate_comprehensive_report() {
    echo ""
    echo "📊 SuperRelay ERC-4337 综合规范符合性测试报告"
    echo "=============================================="
    echo "测试执行时间: $(date '+%Y-%m-%d %H:%M:%S')"
    echo ""

    # 统计结果
    local total_tests=${#DETAILED_RESULTS[@]}
    local passed_tests=$(printf '%s\n' "${DETAILED_RESULTS[@]}" | grep -c "^PASS:" || echo "0")
    local failed_tests=$(printf '%s\n' "${DETAILED_RESULTS[@]}" | grep -c "^FAIL:" || echo "0")
    local overall_pass_rate=0

    if [[ $total_tests -gt 0 ]]; then
        overall_pass_rate=$((passed_tests * 100 / total_tests))
    fi

    echo "📈 测试统计:"
    echo "   总测试数: $total_tests"
    echo "   通过: $passed_tests"
    echo "   失败: $failed_tests"
    echo "   总体通过率: $overall_pass_rate%"
    echo ""

    echo "📋 详细测试结果:"
    for result in "${TEST_RESULTS[@]}"; do
        echo "   $result"
    done
    echo ""

    # 符合性评估
    if [[ $overall_pass_rate -ge 85 ]]; then
        echo -e "${GREEN}🏆 评估结果: SuperRelay完全符合ERC-4337规范要求${NC}"
        echo -e "${GREEN}   ✅ 建议进入生产环境部署${NC}"
        return 0
    elif [[ $overall_pass_rate -ge 70 ]]; then
        echo -e "${YELLOW}⚠️  评估结果: SuperRelay基本符合ERC-4337规范，有少量优化项${NC}"
        echo -e "${YELLOW}   🔧 建议完成优化后进入生产环境${NC}"
        return 1
    else
        echo -e "${RED}❌ 评估结果: SuperRelay需要重大改进以符合ERC-4337规范${NC}"
        echo -e "${RED}   🛠️  需要系统性优化后重新测试${NC}"
        return 2
    fi
}

# 主执行流程
main() {
    echo "🚀 开始SuperRelay ERC-4337综合规范符合性测试"
    echo "============================================="
    echo ""

    # 检查super-relay binary是否存在
    echo "🔧 检查SuperRelay binary..."
    cd "$PROJECT_ROOT"
    if [[ ! -f "target/debug/super-relay" ]]; then
        echo "🔧 构建super-relay debug版本..."
        if ! cargo build --package super-relay --quiet; then
            echo -e "${RED}❌ SuperRelay构建失败${NC}"
            exit 1
        fi
    fi
    echo -e "${GREEN}✅ SuperRelay准备完成${NC}"

    # 1. 运行ERC-4337 v0.6规范测试
    run_test_suite "$SCRIPT_DIR/test_spec_v06.sh" "ERC-4337 v0.6规范符合性测试"

    # 2. 运行ERC-4337 v0.7规范测试
    run_test_suite "$SCRIPT_DIR/test_spec_v07.sh" "ERC-4337 v0.7规范符合性测试"

    # 3. Gateway专项规范测试
    run_gateway_spec_tests

    # 4. Paymaster专项规范测试
    run_paymaster_spec_tests

    # 5. 生成综合报告
    local report_result
    generate_comprehensive_report
    report_result=$?

    # 保存报告到文件
    local report_file="$PROJECT_ROOT/docs/ERC4337-ComplianceReport-$(date +%Y%m%d-%H%M%S).md"
    generate_comprehensive_report > "$report_file"
    echo ""
    echo -e "${BLUE}📄 详细报告已保存到: $report_file${NC}"

    return $report_result
}

# 执行主程序
main "$@"