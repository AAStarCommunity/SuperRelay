#!/bin/bash
# SuperRelay正确启动脚本
# 使用super-relay包装器而非直接调用rundler

set -e
trap "cleanup" EXIT

echo "🚀 SuperRelay 企业级账户抽象服务启动"
echo "======================================"

# 加载开发环境配置
if [ -f ".env.dev" ]; then
    echo "📁 加载开发环境配置: .env.dev"
    source .env.dev
    echo "✅ 环境配置已加载"
else
    echo "⚠️ 未找到.env.dev文件，使用默认配置"
    # 设置默认值
    export SIGNER_PRIVATE_KEYS="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80,0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2"
    export PAYMASTER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    export RPC_URL="http://localhost:8545"
    export NETWORK="dev"
    export CHAIN_ID="31337"
fi

# 文件路径
ANVIL_PID_FILE="scripts/.anvil.pid"
SUPERRELAY_PID_FILE="scripts/.superrelay.pid"
ENTRYPOINT_ADDRESS_FILE=".entrypoint_address"

# 创建scripts目录
mkdir -p scripts

# 清理函数
cleanup() {
    echo -e "\n🧹 正在清理资源..."
    if [ -f "$ANVIL_PID_FILE" ]; then
        echo "❌ 正在停止 Anvil..."
        kill $(cat $ANVIL_PID_FILE) 2>/dev/null || true
        rm -f $ANVIL_PID_FILE
    fi
    if [ -f "$SUPERRELAY_PID_FILE" ]; then
        echo "❌ 正在停止 SuperRelay..."
        kill $(cat $SUPERRELAY_PID_FILE) 2>/dev/null || true
        rm -f $SUPERRELAY_PID_FILE
    fi
    echo "✅ 清理完成"
}

# 检查工具是否安装
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo -e "❌ 错误: $1 未安装. 请先安装."
        exit 1
    fi
}

# 显示配置信息
show_config() {
    echo ""
    echo "📋 当前配置:"
    echo "  🌐 网络: $NETWORK"
    echo "  📡 RPC: $RPC_URL"
    echo "  🆔 Chain ID: $CHAIN_ID"
    echo "  🔑 Paymaster私钥: ${PAYMASTER_PRIVATE_KEY:0:10}..."
    echo "  🔗 Signer私钥数量: $(echo $SIGNER_PRIVATE_KEYS | tr ',' '\n' | wc -l)"
    echo ""
    
    # 验证关键环境变量
    if [ -z "$SIGNER_PRIVATE_KEYS" ]; then
        echo "❌ 错误: SIGNER_PRIVATE_KEYS 环境变量未设置"
        echo "💡 请确保 .env.dev 文件存在或手动设置环境变量"
        exit 1
    fi
    
    if [ -z "$PAYMASTER_PRIVATE_KEY" ]; then
        echo "❌ 错误: PAYMASTER_PRIVATE_KEY 环境变量未设置"
        echo "💡 请确保 .env.dev 文件存在或手动设置环境变量"
        exit 1
    fi
}

# 主要逻辑
echo "🔍 检查必需工具..."
check_tool "anvil"
check_tool "cargo"

# 验证和显示配置
show_config

# 1. 启动 Anvil (如果需要)
if [ "$NETWORK" = "dev" ] && [ "$RPC_URL" = "http://localhost:8545" ]; then
    if [ -f "$ANVIL_PID_FILE" ]; then
        echo "ℹ️  Anvil 似乎已在运行 (PID: $(cat $ANVIL_PID_FILE)). 跳过启动."
    else
        echo "🔥 启动本地Anvil区块链..."
        anvil --host 0.0.0.0 --port 8545 --chain-id $CHAIN_ID > anvil.log 2>&1 &
        echo $! > $ANVIL_PID_FILE
        sleep 3 # 等待 Anvil 完全启动
        echo "✅ Anvil 已启动 (PID: $(cat $ANVIL_PID_FILE))"
        
        # 验证Anvil是否正常工作
        if curl -s -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
            $RPC_URL > /dev/null; then
            echo "✅ Anvil 连接验证成功"
        else
            echo "❌ Anvil 连接失败"
            exit 1
        fi
    fi
else
    echo "🌐 使用外部网络: $RPC_URL"
fi

# 2. 构建SuperRelay
echo "🔨 构建SuperRelay..."
cargo build --package super-relay --release

# 3. 启动 SuperRelay (使用super-relay包装器)
echo ""
echo "🚀 启动SuperRelay企业级服务..."
echo "------------------------------------"
echo "💡 架构说明:"
echo "  • SuperRelay = 企业级包装器"
echo "  • rundler = 底层ERC-4337引擎"
echo "  • paymaster-relay = Gas赞助服务"
echo "  • 配置文件: config/config.toml"
echo "------------------------------------"
echo ""

# 显示启动命令
echo "🔧 执行命令:"
echo "  ./target/release/super-relay node --config config/config.toml"
echo ""

# 前台启动SuperRelay服务
./target/release/super-relay node --config config/config.toml