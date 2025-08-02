#!/bin/bash
# SuperRelay生产环境启动脚本
# 用于生产环境部署和systemd服务集成

set -e

echo "🏭 SuperRelay Production Startup"
echo "================================"

# 生产环境配置文件路径
PRODUCTION_CONFIG="${PRODUCTION_CONFIG:-config/production.toml}"
SERVICE_NAME="${SERVICE_NAME:-super-relay}"
LOG_DIR="${LOG_DIR:-/var/log/super-relay}"
PID_FILE="${PID_FILE:-/var/run/super-relay.pid}"

# 创建日志目录
mkdir -p "$LOG_DIR"

# 检查必需的环境变量
check_required_env() {
    local required_vars=(
        "SIGNER_PRIVATE_KEYS"
        "PAYMASTER_PRIVATE_KEY" 
        "RPC_URL"
        "NETWORK"
    )
    
    echo "🔍 检查必需的环境变量..."
    
    for var in "${required_vars[@]}"; do
        if [ -z "${!var}" ]; then
            echo "❌ 错误: 环境变量 $var 未设置"
            echo ""
            echo "💡 生产环境需要设置以下环境变量:"
            echo "   export SIGNER_PRIVATE_KEYS=0x..."
            echo "   export PAYMASTER_PRIVATE_KEY=0x..."
            echo "   export RPC_URL=https://..."
            echo "   export NETWORK=mainnet|sepolia|polygon"
            echo ""
            echo "🔒 安全提醒:"
            echo "   • 使用环境变量而非配置文件存储私钥"
            echo "   • 考虑使用硬件钱包或KMS服务"
            echo "   • 定期轮换密钥"
            exit 1
        else
            echo "✅ $var 已设置"
        fi
    done
}

# 检查配置文件
check_config() {
    echo "📁 检查配置文件: $PRODUCTION_CONFIG"
    
    if [ ! -f "$PRODUCTION_CONFIG" ]; then
        echo "❌ 生产配置文件不存在: $PRODUCTION_CONFIG"
        echo ""
        echo "💡 请创建生产配置文件，可以从模板复制:"
        echo "   cp config/config.toml $PRODUCTION_CONFIG"
        echo "   编辑生产环境特定设置"
        exit 1
    fi
    
    echo "✅ 配置文件存在"
}

# 验证网络连接
check_network() {
    echo "🌐 验证网络连接: $RPC_URL"
    
    if curl -s -f -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        "$RPC_URL" > /dev/null; then
        echo "✅ 网络连接正常"
    else
        echo "❌ 无法连接到RPC端点: $RPC_URL"
        exit 1
    fi
}

# 启动服务
start_service() {
    echo "🚀 启动SuperRelay生产服务..."
    echo "📝 日志位置: $LOG_DIR"
    echo "🔧 配置文件: $PRODUCTION_CONFIG"
    echo "🌐 网络: $NETWORK"
    echo "📡 RPC: $RPC_URL"
    echo ""
    
    # 后台启动服务并记录PID
    nohup ./target/release/super-relay node \
        --config "$PRODUCTION_CONFIG" \
        > "$LOG_DIR/super-relay.log" 2>&1 &
    
    local pid=$!
    echo $pid > "$PID_FILE"
    
    echo "✅ SuperRelay已启动 (PID: $pid)"
    echo "📋 日志跟踪: tail -f $LOG_DIR/super-relay.log"
    echo "🛑 停止服务: kill $pid 或 ./scripts/stop_production.sh"
}

# 生成systemd服务文件
generate_systemd_service() {
    local service_file="/etc/systemd/system/$SERVICE_NAME.service"
    
    echo "📄 生成systemd服务文件: $service_file"
    
    cat > "/tmp/$SERVICE_NAME.service" << EOF
[Unit]
Description=SuperRelay - Enterprise Account Abstraction Service
Documentation=https://github.com/AAStarCommunity/SuperRelay
After=network.target
Wants=network.target

[Service]
Type=simple
User=super-relay
Group=super-relay
WorkingDirectory=/opt/super-relay
ExecStart=/opt/super-relay/target/release/super-relay node --config $PRODUCTION_CONFIG
ExecReload=/bin/kill -s HUP \$MAINPID
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=super-relay

# 环境变量文件
EnvironmentFile=/opt/super-relay/.env.production

# 安全设置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/super-relay /var/lib/super-relay

# 资源限制
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
EOF

    echo "💡 将服务文件复制到系统目录:"
    echo "   sudo cp /tmp/$SERVICE_NAME.service $service_file"
    echo "   sudo systemctl daemon-reload"
    echo "   sudo systemctl enable $SERVICE_NAME"
    echo "   sudo systemctl start $SERVICE_NAME"
}

# 生成环境变量模板
generate_env_template() {
    local env_file=".env.production.template"
    
    echo "📝 生成生产环境变量模板: $env_file"
    
    cat > "$env_file" << 'EOF'
# SuperRelay生产环境配置
# 复制到.env.production并填写实际值

# 🔐 私钥配置 (必需)
SIGNER_PRIVATE_KEYS=0x...
PAYMASTER_PRIVATE_KEY=0x...

# 🌐 网络配置 (必需)
RPC_URL=https://eth-mainnet.alchemyapi.io/v2/YOUR_KEY
NETWORK=mainnet
CHAIN_ID=1

# 🏭 服务配置
SERVICE_HOST=0.0.0.0
SERVICE_PORT=3000
LOG_LEVEL=info

# 📊 监控配置
METRICS_ENABLED=true
METRICS_PORT=8080
HEALTH_CHECK_PORT=9000

# 🔒 安全配置
CORS_ENABLED=true
ALLOWED_ORIGINS=https://your-frontend.com
RATE_LIMITING_ENABLED=true

# 💾 数据库配置 (可选)
DATABASE_URL=postgresql://user:pass@localhost/super_relay

# 🚨 告警配置 (可选)
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/...
EMAIL_ALERTS_ENABLED=true
ALERT_EMAIL=admin@yourcompany.com
EOF

    echo "✅ 环境变量模板已生成"
}

# 主执行逻辑
main() {
    case "${1:-start}" in
        "start")
            check_required_env
            check_config
            check_network
            start_service
            ;;
        "systemd")
            generate_systemd_service
            ;;
        "env-template")
            generate_env_template
            ;;
        "check")
            check_required_env
            check_config
            check_network
            echo "✅ 所有检查通过，准备就绪"
            ;;
        *)
            echo "用法: $0 [start|systemd|env-template|check]"
            echo ""
            echo "命令说明:"
            echo "  start        启动生产服务 (默认)"
            echo "  systemd      生成systemd服务文件"
            echo "  env-template 生成环境变量模板"
            echo "  check        检查环境和配置"
            echo ""
            echo "生产部署步骤:"
            echo "1. $0 env-template  # 生成环境变量模板"
            echo "2. 编辑 .env.production 文件"
            echo "3. $0 check         # 验证配置"
            echo "4. $0 systemd       # 生成systemd服务"
            echo "5. $0 start         # 启动服务"
            exit 1
            ;;
    esac
}

main "$@"