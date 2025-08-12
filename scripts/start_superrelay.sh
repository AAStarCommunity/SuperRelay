#!/bin/bash
# SuperRelay startup script
# Use super-relay wrapper instead of calling rundler directly
# 支持profile参数：release/debug，默认debug模式速度更快

set -e

# 默认参数
BUILD_PROFILE="debug"
SKIP_BUILD=false

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        release)
            BUILD_PROFILE="release"
            shift
            ;;
        debug)
            BUILD_PROFILE="debug"
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --help|-h)
            echo "🚀 SuperRelay 启动脚本"
            echo ""
            echo "使用方法: $0 [OPTIONS] [MODES]"
            echo ""
            echo "构建选项:"
            echo "  release      使用生产优化的release版本（更小更快）"
            echo "  debug        使用开发版本（默认，编译快）"
            echo "  --skip-build 跳过构建检测，直接使用现有二进制"
            echo ""
            echo "运行模式:"
            echo "  legacy       使用兼容模式"
            echo "  node         使用node模式"
            echo ""
            echo "优化建议:"
            echo "  • 日常开发: $0 debug (默认，编译快）"
            echo "  • 性能测试: $0 release (最优性能）"
            echo "  • 快速启动: $0 --skip-build (跳过构建检测）"
            exit 0
            ;;
        legacy|node)
            # 这些参数后面处理
            break
            ;;
        *)
            echo "⚠️  未知参数: $1"
            echo "使用 --help 查看使用说明"
            break
            ;;
    esac
done

echo "🚀 SuperRelay v0.1.5 - Enterprise API Gateway Starting"
echo "🌐 Single Binary Gateway Mode with Internal Routing"
echo "📊 构建模式: $BUILD_PROFILE (基于 Jason Cursor Rules 优化)"
echo "======================================"

# Load development environment configuration (try multiple config files)
if [ -f ".env" ]; then
    echo "📁 Loading environment config: .env"
    source .env
    echo "✅ Environment configuration loaded"
elif [ -f ".env.dev" ]; then
    echo "📁 Loading development environment config: .env.dev"
    source .env.dev
    echo "✅ Environment configuration loaded"
else
    echo "⚠️ No .env or .env.dev file found, using default configuration"
    # Set default values
    export SIGNER_PRIVATE_KEYS="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80,0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2"
    export PAYMASTER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    export RPC_URL="http://localhost:8545"
    export NETWORK="dev"
    export CHAIN_ID="31337"
fi

# File paths
ANVIL_PID_FILE="scripts/.anvil.pid"
SUPERRELAY_PID_FILE="scripts/.superrelay.pid"
ENTRYPOINT_ADDRESS_FILE=".entrypoint_address"

# Create scripts directory
mkdir -p scripts

# 设置cleanup trap
trap "cleanup" EXIT

# Cleanup function
cleanup() {
    echo -e "\n🧹 Cleaning up resources..."
    if [ -f "$ANVIL_PID_FILE" ]; then
        echo "❌ Stopping Anvil..."
        kill $(cat $ANVIL_PID_FILE) 2>/dev/null || true
        rm -f $ANVIL_PID_FILE
    fi
    if [ -f "$SUPERRELAY_PID_FILE" ]; then
        echo "❌ Stopping SuperRelay..."
        kill $(cat $SUPERRELAY_PID_FILE) 2>/dev/null || true
        rm -f $SUPERRELAY_PID_FILE
    fi
    echo "✅ Cleanup complete"
}

# Kill existing processes function
kill_existing_processes() {
    echo "🔄 Checking for existing processes..."

    # Kill processes using port 8545 (Anvil)
    if lsof -ti:8545 >/dev/null 2>&1; then
        echo "🛑 Killing existing processes on port 8545 (Anvil)..."
        lsof -ti:8545 | xargs kill -9 2>/dev/null || true
        sleep 1
    fi

    # Kill processes using port 3000 (SuperRelay RPC)
    if lsof -ti:3000 >/dev/null 2>&1; then
        echo "🛑 Killing existing processes on port 3000 (SuperRelay RPC)..."
        lsof -ti:3000 | xargs kill -9 2>/dev/null || true
        sleep 1
    fi

    # Kill processes using port 9000 (Swagger UI)
    if lsof -ti:9000 >/dev/null 2>&1; then
        echo "🛑 Killing existing processes on port 9000 (Swagger UI)..."
        lsof -ti:9000 | xargs kill -9 2>/dev/null || true
        sleep 1
    fi

    # Kill processes using port 8080 (Metrics)
    if lsof -ti:8080 >/dev/null 2>&1; then
        echo "🛑 Killing existing processes on port 8080 (Metrics)..."
        lsof -ti:8080 | xargs kill -9 2>/dev/null || true
        sleep 1
    fi

    # Kill any rundler or super-relay processes
    pkill -f "rundler" 2>/dev/null || true
    pkill -f "super-relay" 2>/dev/null || true

    echo "✅ Process cleanup complete"
}

# Check if tools are installed
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo -e "❌ Error: $1 not installed. Please install it first."
        exit 1
    fi
}

# Check if rebuild is needed based on source changes
check_rebuild_needed() {
    local binary_path="$1"
    local build_profile="$2"

    # If binary doesn't exist, rebuild is needed
    if [ ! -f "$binary_path" ]; then
        echo "⚡ 二进制文件不存在，需要构建"
        return 0  # rebuild needed
    fi

    # Check if binary supports gateway command (version compatibility)
    if ! $binary_path --help | grep -q "gateway"; then
        echo "⚡ 二进制版本过旧，需要重新构建"
        return 0  # rebuild needed
    fi

    # Get binary modification time
    local binary_time=$(stat -f %m "$binary_path" 2>/dev/null || stat -c %Y "$binary_path" 2>/dev/null)

    if [ -z "$binary_time" ]; then
        echo "⚡ 无法获取二进制文件时间，重新构建"
        return 0  # rebuild needed
    fi

    # Check if any source files are newer than binary
    local newer_files=$(find . -name "*.rs" -o -name "Cargo.toml" -o -name "Cargo.lock" | while read -r file; do
        if [ -f "$file" ]; then
            local file_time=$(stat -f %m "$file" 2>/dev/null || stat -c %Y "$file" 2>/dev/null)
            if [ "$file_time" -gt "$binary_time" ]; then
                echo "$file"
                break
            fi
        fi
    done)

    if [ -n "$newer_files" ]; then
        echo "⚡ 检测到源码变更，需要重新构建"
        echo "  变更文件: $(echo $newer_files | head -1)"
        return 0  # rebuild needed
    fi

    echo "✅ 二进制文件是最新的，跳过构建"
    return 1  # rebuild not needed
}

# Display configuration information
show_config() {
    echo ""
    echo "📋 Current configuration:"
    echo "  🌐 Network: $NETWORK"
    echo "  📡 RPC: $RPC_URL"
    echo "  🆔 Chain ID: $CHAIN_ID"
    echo "  🔑 Paymaster private key: ${PAYMASTER_PRIVATE_KEY:0:10}..."
    echo "  🔗 Signer private keys count: $(echo $SIGNER_PRIVATE_KEYS | tr ',' '\n' | wc -l)"
    echo ""

    # Validate critical environment variables
    if [ -z "$SIGNER_PRIVATE_KEYS" ]; then
        echo "❌ Error: SIGNER_PRIVATE_KEYS environment variable not set"
        echo "💡 Please ensure .env.dev file exists or manually set environment variables"
        exit 1
    fi

    if [ -z "$PAYMASTER_PRIVATE_KEY" ]; then
        echo "❌ Error: PAYMASTER_PRIVATE_KEY environment variable not set"
        echo "💡 Please ensure .env.dev file exists or manually set environment variables"
        exit 1
    fi
}

# Main logic
echo "🔍 Checking required tools..."
check_tool "anvil"
check_tool "cargo"

# Kill existing processes to prevent port conflicts
kill_existing_processes

# Validate and display configuration
show_config

# 1. Start Anvil (if needed)
if [ "$NETWORK" = "dev" ] && [ "$RPC_URL" = "http://localhost:8545" ]; then
    if [ -f "$ANVIL_PID_FILE" ]; then
        echo "ℹ️  Anvil seems to be already running (PID: $(cat $ANVIL_PID_FILE)). Skipping startup."
    else
        echo "🔥 Starting local Anvil blockchain..."
        anvil --host 0.0.0.0 --port 8545 --chain-id $CHAIN_ID > anvil.log 2>&1 &
        echo $! > $ANVIL_PID_FILE
        sleep 3 # Wait for Anvil to fully start
        echo "✅ Anvil started (PID: $(cat $ANVIL_PID_FILE))"

        # Verify Anvil is working properly
        if curl -s -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
            $RPC_URL > /dev/null; then
            echo "✅ Anvil connection verification successful"
        else
            echo "❌ Anvil connection failed"
            exit 1
        fi
    fi
else
    echo "🌐 Using external network: $RPC_URL"
fi

# 2. 智能选择或构建SuperRelay binary
if [ "$BUILD_PROFILE" = "release" ]; then
    BINARY_PATH="./target/release/super-relay"
    BUILD_COMMAND="./scripts/build.sh --profile release"
else
    BINARY_PATH="./target/debug/super-relay"
    BUILD_COMMAND="./scripts/build.sh --profile debug"
fi

# 智能构建检查 - 仅在需要时构建
if [[ "$SKIP_BUILD" == "true" ]]; then
    echo "⏭️  跳过构建检测，直接使用现有二进制"
    if [[ ! -f "$BINARY_PATH" ]]; then
        echo "❌ 二进制文件不存在: $BINARY_PATH"
        echo "请先运行: ./scripts/build.sh --profile $BUILD_PROFILE"
        exit 1
    fi
else
    echo "🔍 检查是否需要重新构建..."
fi

if [[ "$SKIP_BUILD" != "true" ]] && check_rebuild_needed "$BINARY_PATH" "$BUILD_PROFILE"; then
    echo "🔨 开始构建 $BUILD_PROFILE 版本..."
    if [ -f "./scripts/build.sh" ]; then
        chmod +x ./scripts/build.sh
        $BUILD_COMMAND
        if [ $? -ne 0 ]; then
            echo "❌ 独立构建失败，尝试标准构建..."
            # 后备构建方案
            if [ "$BUILD_PROFILE" = "release" ]; then
                cargo build --package super-relay --release
            else
                cargo build --package super-relay
            fi
        fi
    else
        # 后备构建方案
        if [ "$BUILD_PROFILE" = "release" ]; then
            cargo build --package super-relay --release
        else
            cargo build --package super-relay
        fi
    fi
else
    echo "🚀 使用现有的 $BUILD_PROFILE 版本，节省构建时间"
fi

SUPER_RELAY_BIN="$BINARY_PATH"
echo "✅ 使用 $BUILD_PROFILE 版本: $SUPER_RELAY_BIN"

# 显示优化信息
if [ "$BUILD_PROFILE" = "release" ]; then
    echo "🏆 生产优化模式: 更小的体积、更快的性能"
else
    echo "⚡ 开发模式: 最快编译速度、完整调试信息"
fi

# 3. Start SuperRelay Gateway
echo ""
echo "🌐 Starting SuperRelay API Gateway..."
echo "------------------------------------"
echo "💡 New Architecture (v0.1.5):"
echo "  • SuperRelay Gateway = Single binary with internal routing"
echo "  • Paymaster Service = Enterprise gas sponsorship (internal)"
echo "  • Rundler Components = ERC-4337 engine (internal method calls)"
echo "  • Web UI = Independent deployment (port 9000)"
echo "  • Configuration file: config/config.toml"
echo "------------------------------------"
echo ""

# Choose startup mode (处理剩余的参数)
STARTUP_MODE="gateway"  # 默认gateway模式
# 检查剩余参数中是否有legacy或node
while [[ $# -gt 0 ]]; do
    case $1 in
        legacy|node)
            STARTUP_MODE="$1"
            shift
            ;;
        *)
            # 跳过其他参数
            shift
            ;;
    esac
done

if [ "$STARTUP_MODE" = "legacy" ] || [ "$STARTUP_MODE" = "node" ]; then
    echo "🔧 Starting in Legacy Mode (for compatibility):"
    echo "  $SUPER_RELAY_BIN node --config config/config.toml"
    echo ""

    # Start legacy mode
    env PAYMASTER_PRIVATE_KEY="$PAYMASTER_PRIVATE_KEY" \
        SIGNER_PRIVATE_KEYS="$SIGNER_PRIVATE_KEYS" \
        RPC_URL="$RPC_URL" \
        NETWORK="$NETWORK" \
        CHAIN_ID="$CHAIN_ID" \
        $SUPER_RELAY_BIN node --config config/config.toml
else
    echo "🌐 Starting in Gateway Mode (recommended):"
    echo "  $SUPER_RELAY_BIN gateway --config config/config.toml --host 127.0.0.1 --port 3000 --enable-paymaster"
    echo ""

    # Start gateway mode
    env PAYMASTER_PRIVATE_KEY="$PAYMASTER_PRIVATE_KEY" \
        SIGNER_PRIVATE_KEYS="$SIGNER_PRIVATE_KEYS" \
        RPC_URL="$RPC_URL" \
        NETWORK="$NETWORK" \
        CHAIN_ID="$CHAIN_ID" \
        $SUPER_RELAY_BIN gateway \
            --config config/config.toml \
            --host 127.0.0.1 \
            --port 3000 \
            --enable-paymaster \
            --paymaster-private-key "$PAYMASTER_PRIVATE_KEY"
fi