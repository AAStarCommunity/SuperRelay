#!/bin/bash

# SuperRelay 监控脚本 (只监控，不部署)
# 监控 Fly.io 应用状态和日志

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 配置
APP_URL="https://rundler-superrelay.fly.dev"
HEALTH_CHECK_URL="$APP_URL/health"
CHECK_INTERVAL=10
MAX_CHECKS=60

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 测试基本连接
test_basic_connection() {
    echo -n "🔗 测试基本连接到 $APP_URL... "

    local response=$(curl -s -w "%{http_code}" --connect-timeout 10 "$APP_URL" -o /dev/null 2>/dev/null || echo "000")

    if [ "$response" = "200" ] || [ "$response" = "404" ] || [ "$response" = "405" ]; then
        echo "✅ (HTTP $response)"
        return 0
    else
        echo "❌ (HTTP $response)"
        return 1
    fi
}

# 测试健康检查端点
test_health_check() {
    echo -n "🏥 测试健康检查端点... "

    local response=$(curl -s -w "%{http_code}" --connect-timeout 10 "$HEALTH_CHECK_URL" -o /dev/null 2>/dev/null || echo "000")

    if [ "$response" = "200" ]; then
        echo "✅ (HTTP $response)"
        return 0
    else
        echo "❌ (HTTP $response)"
        return 1
    fi
}

# 测试 RPC 端点
test_rpc_endpoint() {
    echo -n "🔌 测试 RPC 端点... "

    local rpc_response=$(curl -s --connect-timeout 10 \
        -X POST "$APP_URL" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_supportedEntryPoints","params":[],"id":1}' 2>/dev/null)

    if echo "$rpc_response" | grep -q "\"result\""; then
        echo "✅"
        local entry_points=$(echo "$rpc_response" | jq -r '.result[]' 2>/dev/null || echo "解析失败")
        echo "   📞 支持的 EntryPoints: $entry_points"
        return 0
    elif echo "$rpc_response" | grep -q "jsonrpc"; then
        echo "⚠️"
        local error_msg=$(echo "$rpc_response" | jq -r '.error.message // "未知错误"' 2>/dev/null || echo "解析失败")
        echo "   📞 错误: $error_msg"
        return 1
    else
        echo "❌"
        if [ -n "$rpc_response" ]; then
            echo "   📞 响应: $rpc_response"
        fi
        return 1
    fi
}

# 监控循环
monitor_loop() {
    log_info "开始持续监控 (每 ${CHECK_INTERVAL} 秒检查一次)"
    log_info "按 Ctrl+C 停止监控"
    echo ""

    local check_count=0
    local success_count=0
    local failed_count=0

    while [ $check_count -lt $MAX_CHECKS ]; do
        local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
        echo "📊 检查 #$((check_count + 1)) [$timestamp]"
        echo "----------------------------------------"

        local all_passed=true

        # 基本连接测试
        if ! test_basic_connection; then
            all_passed=false
        fi

        # 健康检查测试
        if ! test_health_check; then
            all_passed=false
        fi

        # RPC 端点测试
        if ! test_rpc_endpoint; then
            all_passed=false
        fi

        if [ "$all_passed" = true ]; then
            success_count=$((success_count + 1))
            echo "✅ 所有检查通过"
        else
            failed_count=$((failed_count + 1))
            echo "❌ 某些检查失败"
        fi

        check_count=$((check_count + 1))

        echo "📈 统计: $success_count 成功, $failed_count 失败, $check_count 总计"
        echo ""

        if [ $check_count -lt $MAX_CHECKS ]; then
            sleep $CHECK_INTERVAL
        fi
    done

    log_info "监控完成"
    echo "📊 最终统计: $success_count/$check_count 次检查成功"
}

# 单次检查
single_check() {
    log_info "执行单次应用状态检查"
    echo ""

    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "📊 应用状态检查 [$timestamp]"
    echo "========================================"

    echo "🌐 应用 URL: $APP_URL"
    echo ""

    # 执行所有测试
    test_basic_connection
    test_health_check
    test_rpc_endpoint

    echo ""
    log_info "单次检查完成"
}

# 显示使用帮助
show_help() {
    echo "SuperRelay 应用监控脚本"
    echo ""
    echo "用法:"
    echo "  $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -h, --help     显示此帮助信息"
    echo "  -c, --check    执行单次检查"
    echo "  -m, --monitor  持续监控 (默认)"
    echo ""
    echo "示例:"
    echo "  $0              # 持续监控"
    echo "  $0 --check      # 单次检查"
    echo "  $0 --monitor    # 持续监控"
}

# 主函数
main() {
    local mode="monitor"

    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -c|--check)
                mode="check"
                shift
                ;;
            -m|--monitor)
                mode="monitor"
                shift
                ;;
            *)
                log_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done

    echo "🚀 SuperRelay 应用监控"
    echo "======================="
    echo ""

    if [ "$mode" = "check" ]; then
        single_check
    else
        monitor_loop
    fi
}

# 信号处理
trap 'echo ""; log_info "监控已停止"; exit 0' INT TERM

# 脚本入口点
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi