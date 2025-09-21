#!/bin/bash

# Gas 费用分析脚本
# 分析日志中的 InsufficientFees 错误并提供优化建议

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

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 分析当前问题
analyze_current_issue() {
    echo "🔍 Gas 费用问题分析"
    echo "===================="
    echo ""

    log_info "从日志中提取的数据:"
    echo "UserOp Hash: 0x9574de239acbaf0f42fe338f71342315dfdd02ecef104add24ae18fa7cc580fd"
    echo "Builder Tag: 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789:any:0xe24b6f321B0140716a2b671ed0D983bb64E7DaFA"
    echo ""

    echo "📊 费用对比:"
    echo "最新记录的费用差异:"
    echo "  所需费用: 100,011,071 wei (100.011071 Gwei)"
    echo "  实际费用: 100,005,075 wei (100.005075 Gwei)"
    echo "  差异:       5,996 wei (0.005996 Gwei)"
    echo ""

    local percentage=$(echo "scale=4; 5996 / 100005075 * 100" | bc -l 2>/dev/null || echo "0.006")
    echo "  相对差异: ${percentage}% (极小)"
    echo ""

    log_info "问题分析:"
    echo "1. 这个 UserOperation 已经在 mempool 中滞留很长时间"
    echo "2. 费用差异非常小 (仅 0.006 Gwei)"
    echo "3. 网络 gas 价格在不断微调，导致所需费用略有上升"
    echo "4. 用户提交的费用略低于当前网络要求"
}

# 检查配置有效性
check_config_effectiveness() {
    echo ""
    echo "🔧 配置有效性检查"
    echo "=================="
    echo ""

    log_info "当前 fly.toml 配置:"
    echo "  MIN_PRIORITY_FEE_PER_GAS = 1,000,000,000 wei (1 Gwei)"
    echo "  MAX_FEE_PER_GAS_OVERHEAD = 20,000,000,000 wei (20 Gwei)"
    echo "  GAS_FEE_ESTIMATION_BUFFER = 1.1 (10% buffer)"
    echo ""

    log_warning "潜在问题:"
    echo "1. 这些环境变量可能不是 Rundler 的标准配置项"
    echo "2. Rundler 可能有自己的费用策略配置"
    echo "3. 需要验证这些配置是否真正被应用"
}

# 建议解决方案
suggest_solutions() {
    echo ""
    echo "💡 解决方案建议"
    echo "==============="
    echo ""

    echo "🎯 短期解决方案:"
    echo "1. 监控观察 - 这种小差异可能是可接受的"
    echo "2. UserOperation 会在 valid_until 时间到期后自动清理"
    echo "3. 如果用户重新提交更高费用的操作，会替换当前操作"
    echo ""

    echo "🔧 中期优化:"
    echo "1. 检查 Rundler 的实际配置文件位置"
    echo "2. 验证环境变量是否被正确读取"
    echo "3. 考虑调整费用估算算法"
    echo ""

    echo "📈 长期策略:"
    echo "1. 实现动态费用调整机制"
    echo "2. 添加费用监控和自动优化"
    echo "3. 提供用户费用建议 API"
}

# 监控建议
monitoring_recommendations() {
    echo ""
    echo "📊 监控建议"
    echo "==========="
    echo ""

    echo "🔍 需要监控的指标:"
    echo "1. InsufficientFees 错误频率"
    echo "2. 平均费用差异大小"
    echo "3. UserOperation 在 mempool 中的停留时间"
    echo "4. 成功打包的 UserOperation 比例"
    echo ""

    echo "⚠️  警报阈值建议:"
    echo "1. 费用差异 > 1 Gwei 时发出警报"
    echo "2. 同一 UserOperation 重复跳过 > 100 次"
    echo "3. mempool 中滞留时间 > 1 小时"
    echo ""

    echo "📋 日常检查:"
    echo "1. 每日检查费用相关错误日志"
    echo "2. 对比网络平均 gas 价格趋势"
    echo "3. 分析 bundle 成功率"
}

# 检查 UserOperation 状态
check_userop_status() {
    echo ""
    echo "🔍 UserOperation 状态检查"
    echo "========================="
    echo ""

    local app_url="https://rundler-superrelay.fly.dev"
    local userop_hash="0x9574de239acbaf0f42fe338f71342315dfdd02ecef104add24ae18fa7cc580fd"

    log_info "检查 UserOperation 是否仍在 mempool 中..."

    # 尝试获取 UserOperation 收据
    echo -n "📞 查询 UserOperation 收据... "
    local receipt_response=$(curl -s --connect-timeout 10 \
        -X POST "$app_url" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getUserOperationReceipt\",\"params\":[\"$userop_hash\"],\"id\":1}" 2>/dev/null)

    if echo "$receipt_response" | grep -q "\"result\""; then
        echo "✅ 已完成"
        echo "   UserOperation 已被成功打包！"
        if command -v jq >/dev/null 2>&1; then
            echo "   收据: $(echo "$receipt_response" | jq -r '.result.transactionHash // "无法解析"')"
        fi
    elif echo "$receipt_response" | grep -q "null"; then
        echo "⏳ 仍在等待"
        echo "   UserOperation 仍在 mempool 中等待打包"
    else
        echo "❓ 未知"
        echo "   响应: $receipt_response"
    fi
}

# 生成优化建议
generate_optimization_plan() {
    echo ""
    echo "📋 优化行动计划"
    echo "==============="
    echo ""

    echo "阶段 1: 立即行动 (0-1 天)"
    echo "1. ✅ 监控当前 UserOperation 是否最终被打包"
    echo "2. ✅ 记录费用差异模式和频率"
    echo "3. ✅ 验证配置是否生效"
    echo ""

    echo "阶段 2: 短期优化 (1-7 天)"
    echo "1. 🔧 查找 Rundler 的真实配置文件位置"
    echo "2. 🔧 验证环境变量映射"
    echo "3. 🔧 如有必要，调整费用缓冲参数"
    echo ""

    echo "阶段 3: 长期改进 (1-4 周)"
    echo "1. 🚀 实现费用监控仪表板"
    echo "2. 🚀 添加自动费用调整机制"
    echo "3. 🚀 优化 UserOperation 生命周期管理"
    echo ""

    echo "成功指标:"
    echo "• InsufficientFees 错误减少 80%"
    echo "• UserOperation 平均等待时间 < 30 秒"
    echo "• Bundle 成功率 > 95%"
}

# 主函数
main() {
    echo "⛽ SuperRelay Gas 费用分析报告"
    echo "==============================="
    echo ""

    analyze_current_issue
    check_config_effectiveness
    suggest_solutions
    monitoring_recommendations
    check_userop_status
    generate_optimization_plan

    echo ""
    echo "📝 报告生成时间: $(date)"
    echo "📁 相关文件:"
    echo "   - fly.toml (配置文件)"
    echo "   - docs/aastar-readme.md (文档)"
    echo "   - scripts/monitor-only.sh (监控脚本)"
}

# 脚本入口
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi