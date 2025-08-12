#!/bin/bash
# SuperRelay HTTP REST API Server startup script
# 启动 utoipa/axum HTTP REST API 服务器和 Swagger UI (端口 9000)

set -e

# 默认参数
BUILD_PROFILE="debug"
HOST="0.0.0.0"
PORT="9000"
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
        --host)
            HOST="$2"
            shift 2
            ;;
        --port)
            PORT="$2"
            shift 2
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --help|-h)
            echo "🚀 SuperRelay HTTP REST API Server 启动脚本"
            echo ""
            echo "使用方法: $0 [OPTIONS]"
            echo ""
            echo "构建选项:"
            echo "  release      使用生产优化的release版本"
            echo "  debug        使用开发版本（默认）"
            echo "  --skip-build 跳过构建检测，直接使用现有二进制"
            echo ""
            echo "服务器选项:"
            echo "  --host HOST  绑定地址 (默认: 0.0.0.0)"
            echo "  --port PORT  端口号 (默认: 9000)"
            echo ""
            echo "功能特性:"
            echo "  • HTTP REST API 接口"
            echo "  • 自动生成的 Swagger UI 文档"
            echo "  • utoipa 自动 OpenAPI 规范"
            echo "  • 与 JSON-RPC 服务双协议支持"
            echo ""
            echo "访问地址:"
            echo "  🌐 Swagger UI: http://localhost:9000/swagger-ui/"
            echo "  🏥 健康检查: http://localhost:9000/health"
            echo "  📋 API 文档: http://localhost:9000/api-doc/openapi.json"
            exit 0
            ;;
        *)
            echo "⚠️  未知参数: $1"
            echo "使用 --help 查看使用说明"
            exit 1
            ;;
    esac
done

echo "🚀 SuperRelay HTTP REST API Server v0.2.0"
echo "🌐 Enterprise-grade API with Swagger UI"
echo "📊 构建模式: $BUILD_PROFILE"
echo "======================================"

# Load development environment configuration
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
    export PAYMASTER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
fi

# Kill existing processes on the target port
if lsof -ti:$PORT >/dev/null 2>&1; then
    echo "🛑 Killing existing processes on port $PORT..."
    lsof -ti:$PORT | xargs kill -9 2>/dev/null || true
    sleep 2
fi

# 检查和构建二进制文件
if [ "$BUILD_PROFILE" = "release" ]; then
    BINARY_PATH="./target/release/super-relay"
else
    BINARY_PATH="./target/debug/super-relay"
fi

# 构建检查
if [[ "$SKIP_BUILD" != "true" ]]; then
    if [ ! -f "$BINARY_PATH" ] || ! $BINARY_PATH --help | grep -q "api-server"; then
        echo "🔨 构建 $BUILD_PROFILE 版本..."
        if [ "$BUILD_PROFILE" = "release" ]; then
            cargo build --package super-relay --release
        else
            cargo build --package super-relay
        fi
    else
        echo "✅ 二进制文件存在且支持 api-server 命令"
    fi
fi

# 显示配置信息
echo ""
echo "📋 API Server Configuration:"
echo "  🌐 Host: $HOST"
echo "  🔌 Port: $PORT"
echo "  🔑 Paymaster private key: ${PAYMASTER_PRIVATE_KEY:0:10}..."
echo "  📊 Build profile: $BUILD_PROFILE"
echo ""

# Validate critical environment variables
if [ -z "$PAYMASTER_PRIVATE_KEY" ]; then
    echo "❌ Error: PAYMASTER_PRIVATE_KEY environment variable not set"
    echo "💡 Please ensure .env.dev file exists or manually set environment variables"
    exit 1
fi

echo "🌟 Starting HTTP REST API Server with Swagger UI..."
echo "--------------------------------------------"
echo "💡 Features:"
echo "  • HTTP REST API endpoints"
echo "  • Interactive Swagger UI documentation"
echo "  • Automatic OpenAPI specification generation"
echo "  • Real-time API testing interface"
echo "  • Dual protocol support (REST + JSON-RPC)"
echo ""
echo "🔗 Access URLs:"
echo "  📱 Swagger UI: http://$HOST:$PORT/swagger-ui/"
echo "  🏥 Health Check: http://$HOST:$PORT/health"
echo "  📋 OpenAPI JSON: http://$HOST:$PORT/api-doc/openapi.json"
echo "  🧪 Test Endpoint: curl -X POST http://$HOST:$PORT/api/v1/sponsor"
echo "--------------------------------------------"
echo ""

# Start the API server (Proxy Mode)
echo "⚠️  注意：新的代理模式需要先启动 SuperRelay 服务"
echo "🔧 请确保 SuperRelay 服务正在运行在 localhost:3000"
echo "   命令: ./target/$BUILD_PROFILE/super-relay dual-service --config config.toml"
echo ""

env RUST_LOG=info \
    $BINARY_PATH api-server \
        --host "$HOST" \
        --port "$PORT" \
        --super-relay-url "http://localhost:3000"