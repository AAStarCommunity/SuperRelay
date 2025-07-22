#!/bin/bash
# set -x # Print commands and their arguments as they are executed.

# SuperRelay 开发环境一键启动脚本
# 功能:
# 1. 检查和安装必要的开发工具 (anvil, cargo, jq)
# 2. 启动 Anvil 本地测试链
# 3. 部署 EntryPoint 合约
# 4. 启动 SuperRelay 服务 (包含 paymaster-relay)
# 5. 提供健康检查和清理机制

trap "cleanup" INT TERM

# 环境变量和默认值
export RUST_LOG="info,rundler_rpc=debug"
export RUNDLER_CONFIG=${RUNDLER_CONFIG:-"config/config.toml"}
export ANVIL_RPC_URL=${ANVIL_RPC_URL:-"http://localhost:8545"}
export PAYMASTER_RPC_URL=${PAYMASTER_RPC_URL:-"http://localhost:3000"}
export CHAIN_ID=${CHAIN_ID:-31337}
export PAYMASTER_SIGNER_KEY=${PAYMASTER_SIGNER_KEY:-"0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"} # Anvil default private key 0
export BUNDLER_SIGNER_KEY_1=${BUNDLER_SIGNER_KEY_1:-"0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"} # Anvil default private key 1
export BUNDLER_SIGNER_KEY_2=${BUNDLER_SIGNER_KEY_2:-"0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"} # Anvil default private key 2

# 文件路径
ANVIL_PID_FILE="scripts/.anvil.pid"
RUNDLER_PID_FILE="scripts/.rundler.pid"
ENTRYPOINT_ADDRESS_FILE=".entrypoint_address"
TEMP_POLICY_FILE=".temp_policy.toml"

# Ensure the scripts directory exists for PID files
mkdir -p scripts

# 清理函数
cleanup() {
    echo -e "\n🧹 正在清理资源..."
    if [ -f "$ANVIL_PID_FILE" ]; then
        echo "❌ 正在停止 Anvil..."
        pkill -P $(cat $ANVIL_PID_FILE) || kill $(cat $ANVIL_PID_FILE) || true
        rm -f $ANVIL_PID_FILE
    fi
    if [ -f "$RUNDLER_PID_FILE" ]; then
        echo "❌ 正在停止 SuperRelay (rundler)..."
        kill $(cat $RUNDLER_PID_FILE) || true
        rm -f $RUNDLER_PID_FILE
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

# 2. 强制停止并清理旧的 Anvil 和 Rundler 实例
echo "🧼 正在清理旧的 Anvil 和 Rundler 实例 (如有)..."
# Terminate existing anvil process
if [ -f scripts/.anvil.pid ]; then
    kill "$(cat scripts/.anvil.pid)" || true
    rm scripts/.anvil.pid
fi
# Terminate existing rundler process
if [ -f scripts/.rundler.pid ]; then
    kill "$(cat scripts/.rundler.pid)" || true
    rm scripts/.rundler.pid
fi
# Force kill anything on the RPC and metrics ports
lsof -t -i:3000 | xargs kill -9 2>/dev/null || true
lsof -t -i:8080 | xargs kill -9 2>/dev/null || true

# 3. 启动 Anvil
echo "🔥 正在启动一个新的 Anvil 实例..."
anvil --silent > scripts/.anvil.log 2>&1 &
ANVIL_PID=$!
echo $ANVIL_PID > scripts/.anvil.pid

echo "⏳ 正在等待 Anvil 启动..."
max_attempts=30
count=0
while ! cast chain-id --rpc-url "$ANVIL_RPC_URL" > /dev/null 2>&1; do
    if [ $count -ge $max_attempts ]; then
        echo "❌ Anvil 未能在 30 秒内启动。请检查日志。"
        exit 1
    fi
    sleep 1
    count=$((count+1))
done

echo "✅ Anvil 已在后台运行 (PID: $(cat $ANVIL_PID_FILE))"

# 3. 部署 EntryPoint
if [ -f "$ENTRYPOINT_ADDRESS_FILE" ]; then
    ENTRY_POINT_ADDRESS=$(cat $ENTRYPOINT_ADDRESS_FILE)
    echo "✅ EntryPoint 已部署在地址: $ENTRY_POINT_ADDRESS"
else
    echo "📦 正在部署 EntryPoint 合约..."
    ./scripts/deploy_entrypoint.sh > deploy_entrypoint.log 2>&1
    if [ $? -ne 0 ]; then
        echo "❌ EntryPoint 部署失败。请检查 deploy_entrypoint.log"
        cat deploy_entrypoint.log
        exit 1
    fi
    ENTRY_POINT_ADDRESS=$(cat $ENTRYPOINT_ADDRESS_FILE)
    echo "✅ EntryPoint 已部署在地址: $ENTRY_POINT_ADDRESS"
fi
export CHAIN_ENTRY_POINT_ADDRESS_V0_6=$ENTRY_POINT_ADDRESS
export CHAIN_ENTRY_POINT_ADDRESS_V0_7=$ENTRY_POINT_ADDRESS

# 3.5. 为 Paymaster 充值
PAYMASTER_ADDRESS=$(cast wallet address "$PAYMASTER_SIGNER_KEY")
echo "ℹ️  Paymaster 地址: $PAYMASTER_ADDRESS"

# 查询Paymaster余额，替换为cast balance
PAYMASTER_BALANCE=$(cast balance $PAYMASTER_ADDRESS --rpc-url http://127.0.0.1:8545)
echo "ℹ️  Paymaster 当前存款: $PAYMASTER_BALANCE Wei"

# 5. 为 Paymaster 存款
DEPOSIT_HEX=$(cast call "$ENTRY_POINT_ADDRESS" "balanceOf(address)" "$PAYMASTER_ADDRESS" --rpc-url "$ANVIL_RPC_URL" | tail -n 1)
# Add a fallback for empty output or "0x" from cast call
if [ -z "$DEPOSIT_HEX" ] || [ "$DEPOSIT_HEX" == "0x" ]; then
    DEPOSIT_HEX="0x0"
fi
DEPOSIT_WEI=$(cast --to-dec "$DEPOSIT_HEX")
MIN_DEPOSIT_WEI="100000000000000000" # 0.1 ETH

echo "ℹ️  Paymaster 当前存款: $DEPOSIT_WEI Wei"

if [ "$DEPOSIT_WEI" -lt "$MIN_DEPOSIT_WEI" ]; then
    echo "💰 Paymaster 存款不足，正在存入 1 ETH..."
    # The output of cast send is now a JSON object, but might have other text.
    # We grep for the line with the hash and then parse it with jq.
    TX_OUTPUT=$(cast send "$ENTRY_POINT_ADDRESS" \
        "depositTo(address)" "$PAYMASTER_ADDRESS" \
        --value 1ether \
        --private-key "$PAYMASTER_SIGNER_KEY" \
        --rpc-url "$ANVIL_RPC_URL" --json)

    TX_HASH=$(echo "$TX_OUTPUT" | grep '"transactionHash"' | jq -r .transactionHash)

    echo "⏳ 等待存款交易确认 (Hash: $TX_HASH)..."
    if [ -z "$TX_HASH" ]; then
        echo "❌ 未能获取交易 HASH. 输出如下:"
        echo "$TX_OUTPUT"
        exit 1
    fi
    cast receipt --confirmations 1 --rpc-url "$ANVIL_RPC_URL" "$TX_HASH" > /dev/null

    echo "✅ 资金存入成功."
else
    echo "✅ Paymaster 存款充足，跳过充值."
fi

echo "🔍 正在验证存款..."
DEPOSIT_HEX_AFTER=$(cast call "$ENTRY_POINT_ADDRESS" "balanceOf(address)" "$PAYMASTER_ADDRESS" --rpc-url "$ANVIL_RPC_URL" | tail -n 1)
if [ -z "$DEPOSIT_HEX_AFTER" ] || [ "$DEPOSIT_HEX_AFTER" == "0x" ]; then
    DEPOSIT_HEX_AFTER="0x0"
fi
DEPOSIT_WEI_AFTER=$(cast --to-dec "$DEPOSIT_HEX_AFTER")
echo "ℹ️  Paymaster 最新存款: $DEPOSIT_WEI_AFTER Wei"

# 4. 创建临时策略文件
cat > $TEMP_POLICY_FILE <<- EOM
[default]
senders = ["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"] # Anvil default account
EOM
export PAYMASTER_POLICY_PATH=$TEMP_POLICY_FILE

# 5. 编译并启动 SuperRelay (rundler)
echo "🛠️  正在编译 SuperRelay... (首次运行可能需要一些时间)"
cargo build --package rundler

echo "🚀 正在启动 SuperRelay 服务..."
# Run with required parameters and default logging to stdout
nohup target/debug/rundler node \
    --node_http="$ANVIL_RPC_URL" \
    --signer.private_keys="$BUNDLER_SIGNER_KEY_1,$BUNDLER_SIGNER_KEY_2" \
    --paymaster.enabled \
    --paymaster.private_key="$PAYMASTER_SIGNER_KEY" \
    --paymaster.policy_file="$TEMP_POLICY_FILE" \
    --rpc.api="eth,rundler,paymaster" \
    --unsafe \
    "$@" &
RUNDLER_PID=$!
echo $RUNDLER_PID > "$RUNDLER_PID_FILE"

echo "✅ SuperRelay (rundler) 已在后台运行 (PID: $RUNDLER_PID)"
echo "按 Ctrl+C 停止所有服务."

# Wait for rundler to exit
wait $RUNDLER_PID