#!/bin/bash

# Rundler 重启脚本
# 重启 Fly.io 上的 Rundler 应用以清除 mempool

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# 应用配置
APP_NAME="rundler-superrelay"
APP_URL="https://${APP_NAME}.fly.dev"

# 显示重启前状态
show_current_status() {
    log_info "检查重启前状态..."
    echo ""

    echo "📊 当前应用状态:"
    echo "  应用: $APP_NAME"
    echo "  URL: $APP_URL"
    echo ""

    # 测试连接
    echo -n "🔗 测试连接... "
    if curl -s --connect-timeout 5 "$APP_URL" > /dev/null 2>&1; then
        echo "✅ 连接正常"
    else
        echo "❌ 连接失败"
    fi

    # 测试 RPC
    echo -n "🔌 测试 RPC... "
    local rpc_response=$(curl -s --connect-timeout 5 \
        -X POST "$APP_URL" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"web3_clientVersion","params":[],"id":1}' 2>/dev/null)

    if echo "$rpc_response" | grep -q "jsonrpc"; then
        echo "✅ RPC 正常"
    else
        echo "❌ RPC 异常"
    fi
}

# 执行重启 (方法 1: 重新部署 - 慢但确保最新配置)
restart_via_deploy() {
    log_info "方法 1: 重新部署 (完整构建)"
    echo ""

    log_warning "⚠️  这将触发完整的构建和部署过程"
    echo "   - 重新构建 Docker 镜像 (~3-5 分钟)"
    echo "   - 应用最新代码和配置更改"
    echo "   - 确保所有优化都生效"
    echo ""

    read -p "确认执行重新部署? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "重新部署已取消"
        return 1
    fi

    log_info "开始重新部署..."

    # 检查当前分支
    local current_branch=$(git branch --show-current)
    log_info "当前分支: $current_branch"

    if [ "$current_branch" != "pure-rundler-deploy" ]; then
        log_warning "当前不在 pure-rundler-deploy 分支"
        read -p "是否切换到 pure-rundler-deploy 分支? (y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git checkout pure-rundler-deploy
        fi
    fi

    # 执行部署 (如果有 fly CLI)
    if command -v fly >/dev/null 2>&1; then
        fly deploy
    else
        log_error "Fly CLI 未安装，无法执行部署"
        log_info "请安装 Fly CLI: curl -L https://fly.io/install.sh | sh"
        return 1
    fi
}

# 执行重启 (方法 2: 机器重启 - 快速但需要 Fly CLI)
restart_via_machine() {
    log_info "方法 2: 机器重启 (使用现有镜像)"
    echo ""

    log_info "✅ 这是更快的重启方式"
    echo "   - 停止当前机器 (~10 秒)"
    echo "   - 用相同镜像启动新机器 (~30-60 秒)"
    echo "   - 保持当前配置不变"
    echo ""

    read -p "确认执行机器重启? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "机器重启已取消"
        return 1
    fi

    # 检查 Fly CLI
    if ! command -v fly >/dev/null 2>&1; then
        log_error "Fly CLI 未安装，无法执行机器重启"
        log_info "请安装 Fly CLI: curl -L https://fly.io/install.sh | sh"
        return 1
    fi

    log_info "获取机器列表..."
    local machines=$(fly machine list --json 2>/dev/null | jq -r '.[].id' 2>/dev/null || echo "")

    if [ -z "$machines" ]; then
        log_error "无法获取机器列表，使用重新部署方式"
        restart_via_deploy
        return
    fi

    log_info "重启所有机器..."
    echo "$machines" | while read -r machine_id; do
        if [ -n "$machine_id" ]; then
            log_info "重启机器: $machine_id"
            fly machine restart "$machine_id" || log_warning "重启机器 $machine_id 失败"
        fi
    done
}

# 等待重启完成
wait_for_restart() {
    log_info "等待应用重启完成..."
    echo ""

    local max_wait=120  # 最大等待 2 分钟
    local wait_count=0
    local check_interval=5

    while [ $wait_count -lt $max_wait ]; do
        echo -n "⏳ 检查应用状态 (${wait_count}s/${max_wait}s)... "

        # 测试基本连接
        if curl -s --connect-timeout 10 "$APP_URL" > /dev/null 2>&1; then
            echo "✅"
            log_success "应用已重启并响应!"
            return 0
        else
            echo "⏸️"
        fi

        sleep $check_interval
        wait_count=$((wait_count + check_interval))
    done

    log_warning "等待超时，但应用可能仍在启动中"
}

# 验证重启效果
verify_restart() {
    log_info "验证重启效果..."
    echo ""

    # 测试之前卡住的 UserOperation
    local stuck_userop="0x9574de239acbaf0f42fe338f71342315dfdd02ecef104add24ae18fa7cc580fd"

    echo "🔍 检查之前卡住的 UserOperation:"
    echo "   Hash: $stuck_userop"
    echo ""

    echo -n "📞 查询状态... "
    local receipt_response=$(curl -s --connect-timeout 10 \
        -X POST "$APP_URL" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getUserOperationReceipt\",\"params\":[\"$stuck_userop\"],\"id\":1}" 2>/dev/null)

    if echo "$receipt_response" | grep -q "null"; then
        echo "✅ 已清除"
        log_success "UserOperation 已从 mempool 中清除!"
    elif echo "$receipt_response" | grep -q "\"result\""; then
        echo "✅ 已完成"
        log_success "UserOperation 已被打包完成!"
    else
        echo "❓ 未知状态"
        echo "   响应: $receipt_response"
    fi
}

# 显示重启后的建议
show_next_steps() {
    echo ""
    log_info "重启完成! 接下来可以:"
    echo ""

    echo "1. 🧪 测试新的转账:"
    echo "   cd aa-flow && npm run test:pnt 1"
    echo ""

    echo "2. 📊 监控应用状态:"
    echo "   ./scripts/monitor-only.sh --monitor"
    echo ""

    echo "3. 📋 查看应用日志:"
    echo "   fly logs (如果有 Fly CLI)"
    echo ""

    echo "4. 🔍 验证 gas 费用优化:"
    echo "   观察新的 UserOperation 是否能正常处理"
}

# 主函数
main() {
    echo "🔄 Rundler 重启工具"
    echo "==================="
    echo ""

    log_info "目的: 重启 Rundler 以清除 mempool 中滞留的 UserOperation"
    echo ""

    # 显示当前状态
    show_current_status

    echo ""
    log_info "🎯 重启的效果:"
    echo "  ✅ 清除内存中的所有 UserOperation"
    echo "  ✅ 重置 mempool 状态"
    echo "  ✅ 应用最新的配置更改"
    echo "  ⚠️  需要等待重新启动时间"
    echo ""

    # 选择重启方式
    echo "🔄 选择重启方式:"
    echo "1. 重新部署 (慢，但确保最新配置) - 推荐首次或有配置更改时"
    echo "2. 机器重启 (快，使用现有镜像) - 推荐仅清除 mempool 时"
    echo "3. 取消"
    echo ""

    read -p "请选择 (1/2/3): " -n 1 -r
    echo ""

    local restart_success=false

    case $REPLY in
        1)
            if restart_via_deploy; then
                restart_success=true
            fi
            ;;
        2)
            if restart_via_machine; then
                restart_success=true
            fi
            ;;
        3)
            log_info "重启已取消"
            return 0
            ;;
        *)
            log_error "无效选择"
            return 1
            ;;
    esac

    if [ "$restart_success" = true ]; then
        # 等待重启完成
        wait_for_restart

        # 验证效果
        verify_restart

        # 显示后续步骤
        show_next_steps

        log_success "🎉 重启完成!"
    else
        log_error "重启失败或被取消"
    fi
}

# 脚本入口
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi