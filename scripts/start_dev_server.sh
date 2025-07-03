#!/bin/bash

# SuperRelay 开发环境一键启动脚本
# 功能:
# 1. 检查和安装必要的开发工具 (anvil, cargo, jq)
# 2. 启动 Anvil 本地测试链
# 3. 部署 EntryPoint 合约
# 4. 启动 SuperRelay 服务 (包含 paymaster-relay)
# 5. 提供健康检查和清理机制

set -e
trap "cleanup" EXIT

# 环境变量和默认值
export RUNDLER_CONFIG=${RUNDLER_CONFIG:-"config/config.toml"}
export ANVIL_RPC_URL=${ANVIL_RPC_URL:-"http://localhost:8545"}
export PAYMASTER_RPC_URL=${PAYMASTER_RPC_URL:-"http://localhost:3000"}
export CHAIN_ID=${CHAIN_ID:-31337}
export PAYMASTER_SIGNER_KEY=${PAYMASTER_SIGNER_KEY:-"0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"} # Anvil default private key

# 文件路径
ANVIL_PID_FILE="scripts/.anvil.pid"
RUNDLER_PID_FILE="scripts/.rundler.pid"
ENTRYPOINT_ADDRESS_FILE=".entrypoint_address"
TEMP_POLICY_FILE=".temp_policy.toml"

# 清理函数
cleanup() {
    echo -e "\n🧹 正在清理资源..."
    if [ -f "$ANVIL_PID_FILE" ]; then
        echo "❌ 正在停止 Anvil..."
        kill $(cat $ANVIL_PID_FILE) || true
        rm $ANVIL_PID_FILE
    fi
    if [ -f "$RUNDLER_PID_FILE" ]; then
        echo "❌ 正在停止 SuperRelay (rundler)..."
        kill $(cat $RUNDLER_PID_FILE) || true
        rm $RUNDLER_PID_FILE
    fi
    if [ -f "$TEMP_POLICY_FILE" ]; then
        rm $TEMP_POLICY_FILE
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

# 主要逻辑
echo "🚀 正在启动 SuperRelay 开发环境..."

# 1. 检查工具
check_tool "anvil"
check_tool "cargo"
check_tool "jq"

# 2. 启动 Anvil
if [ -f "$ANVIL_PID_FILE" ]; then
    echo "ℹ️  Anvil 似乎已在运行 (PID: $(cat $ANVIL_PID_FILE)). 跳过启动."
else
    echo "🔥 正在启动 Anvil..."
    anvil > /dev/null 2>&1 &
    echo $! > $ANVIL_PID_FILE
    sleep 2 # 等待 Anvil 完全启动
    echo "✅ Anvil 已在后台运行 (PID: $(cat $ANVIL_PID_FILE))"
fi

# 3. 部署 EntryPoint
if [ -f "$ENTRYPOINT_ADDRESS_FILE" ]; then
    ENTRY_POINT_ADDRESS=$(cat $ENTRYPOINT_ADDRESS_FILE)
    echo "ℹ️  EntryPoint 已部署在地址: $ENTRY_POINT_ADDRESS"
else
    echo "📦 正在部署 EntryPoint 合约..."
    ./scripts/deploy_entrypoint.sh
    ENTRY_POINT_ADDRESS=$(cat $ENTRYPOINT_ADDRESS_FILE)
    echo "✅ EntryPoint 已部署在地址: $ENTRY_POINT_ADDRESS"
fi
export ENTRY_POINT_ADDRESS

# 4. 创建临时策略文件
cat > $TEMP_POLICY_FILE <<- EOM
[default]
type = "allowlist"
addresses = ["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"] # Anvil default account
EOM
export PAYMASTER_POLICY_PATH=$TEMP_POLICY_FILE

# 5. 启动 SuperRelay (rundler)
echo "🚀 正在启动 SuperRelay 服务..."
cargo run --bin rundler -- node \
    --rpc.port 3000 \
    --rpc.listen 0.0.0.0 \
    --web.port 3001 \
    --web.listen 0.0.0.0 \
    --paymaster.enabled true &
echo $! > $RUNDLER_PID_FILE
sleep 5 # 等待服务启动

# 6. 健康检查
echo -e "\n🩺 正在进行健康检查..."
HEALTH_CHECK_URL="$PAYMASTER_RPC_URL"
RESPONSE=$(curl -s -X POST "$HEALTH_CHECK_URL" -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"pm_healthCheck","params":[]}')
STATUS=$(echo $RESPONSE | jq -r '.result.status')

if [ "$STATUS" == "UP" ]; then
    echo -e "✅ SuperRelay 健康检查通过, 服务状态: $STATUS"
    echo -e "\n🎉 SuperRelay 开发环境已准备就绪!"
    echo "=========================================="
    echo "🔗 Anvil RPC: $ANVIL_RPC_URL"
    echo "🔗 SuperRelay RPC: $PAYMASTER_RPC_URL"
    echo "📄 EntryPoint 地址: $ENTRY_POINT_ADDRESS"
    echo "🔐 Paymaster 签名者: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    echo "=========================================="
    echo -e "\nℹ️  按 Ctrl+C 停止所有服务并清理资源."
    wait
else
    echo -e "❌ SuperRelay 健康检查失败. 响应: $RESPONSE"
    exit 1
fi 