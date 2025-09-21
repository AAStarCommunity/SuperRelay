#!/bin/bash

# SuperRelay Fly.io 部署和监控脚本
# 自动部署并监控直到部署成功

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
APP_NAME="rundler-superrelay"
MAX_WAIT_TIME=600  # 最大等待时间 (秒)
CHECK_INTERVAL=10  # 检查间隔 (秒)

# 日志函数
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

# 检查 fly 命令是否可用
check_fly_cli() {
    log_info "检查 Fly CLI..."
    if ! command -v fly &> /dev/null; then
        log_error "Fly CLI 未安装。请安装后再试："
        echo "curl -L https://fly.io/install.sh | sh"
        exit 1
    fi
    log_success "Fly CLI 已安装"
}

# 检查登录状态
check_fly_auth() {
    log_info "检查 Fly.io 登录状态..."
    if ! fly auth whoami &> /dev/null; then
        log_error "未登录到 Fly.io。请先登录："
        echo "fly auth login"
        exit 1
    fi
    local user=$(fly auth whoami)
    log_success "已登录为: $user"
}

# 检查是否在正确的分支
check_branch() {
    log_info "检查当前分支..."
    local current_branch=$(git branch --show-current)
    if [ "$current_branch" != "pure-rundler-deploy" ]; then
        log_warning "当前分支是 '$current_branch'，不是 'pure-rundler-deploy'"
        read -p "是否切换到 pure-rundler-deploy 分支? (y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git checkout pure-rundler-deploy
            log_success "已切换到 pure-rundler-deploy 分支"
        else
            log_warning "继续在当前分支 '$current_branch' 部署"
        fi
    else
        log_success "当前在正确的分支: $current_branch"
    fi
}

# 检查 fly.toml 文件
check_fly_config() {
    log_info "检查 fly.toml 配置..."
    if [ ! -f "fly.toml" ]; then
        log_error "fly.toml 文件不存在"
        exit 1
    fi

    local app_name=$(grep "^app" fly.toml | cut -d'"' -f2)
    if [ "$app_name" != "$APP_NAME" ]; then
        log_warning "fly.toml 中的应用名称是 '$app_name'，预期是 '$APP_NAME'"
    fi

    log_success "fly.toml 配置正确"
}

# 显示应用当前状态
show_app_status() {
    log_info "获取应用当前状态..."
    echo "----------------------------------------"
    if fly apps list | grep -q "$APP_NAME"; then
        echo "📱 应用信息:"
        fly apps list | head -1  # 标题
        fly apps list | grep "$APP_NAME"
        echo ""

        echo "🖥️  机器状态:"
        fly machine list 2>/dev/null || echo "无法获取机器状态"
        echo ""

        echo "💾 应用详细状态:"
        fly status 2>/dev/null || echo "无法获取应用状态"
    else
        log_warning "应用 '$APP_NAME' 不存在或无法访问"
    fi
    echo "----------------------------------------"
}

# 开始部署
start_deployment() {
    log_info "开始部署到 Fly.io..."

    # 显示部署前信息
    echo "🚀 部署信息:"
    echo "  应用名称: $APP_NAME"
    echo "  当前分支: $(git branch --show-current)"
    echo "  最后提交: $(git log -1 --oneline)"
    echo ""

    # 确认部署
    read -p "确认开始部署? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "部署已取消"
        exit 0
    fi

    # 执行部署
    log_info "执行 fly deploy..."
    if fly deploy; then
        log_success "部署命令执行完成"
        return 0
    else
        log_error "部署命令执行失败"
        return 1
    fi
}

# 监控部署状态
monitor_deployment() {
    log_info "开始监控部署状态..."

    local start_time=$(date +%s)
    local elapsed=0

    while [ $elapsed -lt $MAX_WAIT_TIME ]; do
        echo -n "⏳ 检查状态 (${elapsed}s/${MAX_WAIT_TIME}s)... "

        # 检查应用状态
        if fly status &> /dev/null; then
            local status_output=$(fly status 2>/dev/null)

            # 检查是否有健康的机器
            if echo "$status_output" | grep -q "started"; then
                echo "✅"
                log_success "发现正在运行的机器"

                # 检查健康检查
                if echo "$status_output" | grep -q "passing"; then
                    log_success "健康检查通过！"
                    return 0
                elif echo "$status_output" | grep -q "critical"; then
                    echo "❌"
                    log_warning "健康检查失败，继续监控..."
                else
                    echo "⚠️"
                    log_info "健康检查状态未知，继续监控..."
                fi
            else
                echo "⏸️"
                log_info "机器还未完全启动..."
            fi
        else
            echo "❌"
            log_warning "无法获取应用状态"
        fi

        sleep $CHECK_INTERVAL
        elapsed=$(($(date +%s) - start_time))
    done

    log_error "监控超时 (${MAX_WAIT_TIME}s)"
    return 1
}

# 显示部署后信息
show_deployment_result() {
    log_info "显示部署结果..."
    echo ""
    echo "🎯 部署完成信息:"
    echo "----------------------------------------"

    # 应用 URL
    local app_url="https://${APP_NAME}.fly.dev"
    echo "🌐 应用 URL: $app_url"

    # 最终状态
    echo ""
    echo "📊 最终状态:"
    fly status 2>/dev/null || echo "无法获取状态"

    echo ""
    echo "🏥 健康检查:"
    fly checks list 2>/dev/null || echo "无法获取健康检查状态"

    echo ""
    echo "📱 机器列表:"
    fly machine list 2>/dev/null || echo "无法获取机器列表"

    echo "----------------------------------------"
}

# 显示日志
show_logs() {
    log_info "显示最近的应用日志..."
    echo ""
    echo "📋 最近日志 (最后 50 行):"
    echo "----------------------------------------"
    fly logs --lines 50 2>/dev/null || log_warning "无法获取日志"
    echo "----------------------------------------"
}

# 测试应用连接
test_application() {
    log_info "测试应用连接..."

    local app_url="https://${APP_NAME}.fly.dev"

    # 测试基本连接
    echo -n "🔗 测试基本连接... "
    if curl -s --connect-timeout 10 "$app_url" > /dev/null; then
        echo "✅"
    else
        echo "❌"
        log_warning "基本连接失败"
    fi

    # 测试 RPC 端点
    echo -n "🔌 测试 RPC 端点... "
    local rpc_response=$(curl -s --connect-timeout 10 \
        -X POST "$app_url" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"web3_clientVersion","params":[],"id":1}' 2>/dev/null)

    if echo "$rpc_response" | grep -q "jsonrpc"; then
        echo "✅"
        log_success "RPC 端点正常工作"
    else
        echo "❌"
        log_warning "RPC 端点可能有问题"
    fi

    # 显示 RPC 响应
    if [ -n "$rpc_response" ]; then
        echo ""
        echo "📞 RPC 响应:"
        echo "$rpc_response" | jq . 2>/dev/null || echo "$rpc_response"
    fi
}

# 主函数
main() {
    echo "🚀 SuperRelay Fly.io 部署和监控脚本"
    echo "===================================="
    echo ""

    # 预检查
    check_fly_cli
    check_fly_auth
    check_branch
    check_fly_config

    # 显示当前状态
    show_app_status

    echo ""
    log_info "准备开始部署流程..."

    # 部署
    if start_deployment; then
        log_success "部署启动成功"

        # 监控
        if monitor_deployment; then
            log_success "部署成功完成！"

            # 显示结果
            show_deployment_result

            # 测试应用
            echo ""
            test_application

            # 显示日志
            echo ""
            read -p "是否查看应用日志? (y/n): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                show_logs
            fi

            echo ""
            log_success "🎉 部署完成！应用现在应该正在运行。"
            echo ""
            echo "📖 有用的命令:"
            echo "  查看状态: fly status"
            echo "  查看日志: fly logs"
            echo "  查看机器: fly machine list"
            echo "  连接SSH: fly ssh console"

        else
            log_error "部署监控失败"
            echo ""
            log_info "您可以手动检查状态："
            echo "  fly status"
            echo "  fly logs"
            exit 1
        fi
    else
        log_error "部署启动失败"
        exit 1
    fi
}

# 脚本入口点
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi