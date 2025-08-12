#!/bin/bash
# 测试 utoipa 自动生成的 OpenAPI 文档功能

set -e

echo "🔧 utoipa OpenAPI 文档生成测试"
echo "==============================="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test result counters
PASSED=0
FAILED=0

# Function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ PASSED${NC}: $2"
        ((PASSED++))
    else
        echo -e "${RED}❌ FAILED${NC}: $2"
        ((FAILED++))
    fi
}

echo "📦 编译和语法检查"
echo "----------------"

# Test 1: Compile paymaster-relay with utoipa features
echo -e "${BLUE}测试 1: 编译 paymaster-relay 包${NC}"
if cargo check --package rundler-paymaster-relay --quiet > /dev/null 2>&1; then
    print_result 0 "paymaster-relay 包编译成功"
else
    print_result 1 "paymaster-relay 包编译失败"
fi

# Test 2: Run unit tests to verify utoipa integration
echo -e "${BLUE}测试 2: utoipa 单元测试${NC}"
if cargo test --package rundler-paymaster-relay test_openapi_generation --quiet > /dev/null 2>&1; then
    print_result 0 "OpenAPI 文档生成测试通过"
else
    print_result 1 "OpenAPI 文档生成测试失败"
fi

# Test 3: Check that required utoipa dependencies are present
echo -e "${BLUE}测试 3: utoipa 依赖检查${NC}"
if grep -q "utoipa.*=" crates/paymaster-relay/Cargo.toml && grep -q "utoipa-swagger-ui.*=" crates/paymaster-relay/Cargo.toml; then
    print_result 0 "utoipa 依赖项正确配置"
else
    print_result 1 "utoipa 依赖项缺失或配置错误"
fi

echo ""
echo "🔍 OpenAPI 文档结构验证"
echo "----------------------"

# Test 4: Create a test program to generate OpenAPI JSON
echo -e "${BLUE}测试 4: 生成 OpenAPI JSON 文档${NC}"
cat > /tmp/test_openapi_gen.rs << 'EOF'
use rundler_paymaster_relay::api_schemas::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let openapi = ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&openapi).expect("Failed to serialize OpenAPI");
    println!("{}", json);
}
EOF

# Create a temporary Cargo.toml for the test
cat > /tmp/Cargo.toml << 'EOF'
[package]
name = "test-openapi-gen"
version = "0.1.0"
edition = "2021"

[dependencies]
rundler-paymaster-relay = { path = "crates/paymaster-relay" }
utoipa = { version = "4.2", features = ["axum_extras"] }
serde_json = "1.0"
EOF

# Try to run the test program
if cd /tmp && cargo run --quiet > openapi_output.json 2>/dev/null && cd - > /dev/null; then
    if [ -f /tmp/openapi_output.json ] && [ -s /tmp/openapi_output.json ]; then
        print_result 0 "OpenAPI JSON 文档生成成功"

        # Validate JSON structure
        if jq . /tmp/openapi_output.json > /dev/null 2>&1; then
            print_result 0 "生成的 OpenAPI JSON 格式有效"
        else
            print_result 1 "生成的 OpenAPI JSON 格式无效"
        fi

        # Check for required fields
        if jq -r '.info.title' /tmp/openapi_output.json | grep -q "SuperPaymaster"; then
            print_result 0 "OpenAPI 文档包含正确的标题"
        else
            print_result 1 "OpenAPI 文档标题不正确"
        fi

        # Check for API paths
        if jq -r '.paths | keys[]' /tmp/openapi_output.json | grep -q "/api/v1/sponsor"; then
            print_result 0 "OpenAPI 文档包含 API 端点路径"
        else
            print_result 1 "OpenAPI 文档缺少 API 端点路径"
        fi

        # Check for components/schemas
        if jq -r '.components.schemas | keys[]' /tmp/openapi_output.json | grep -q "SponsorUserOperationRequest"; then
            print_result 0 "OpenAPI 文档包含请求结构定义"
        else
            print_result 1 "OpenAPI 文档缺少请求结构定义"
        fi

    else
        print_result 1 "OpenAPI JSON 文档生成失败 (文件为空)"
    fi
else
    print_result 1 "OpenAPI JSON 文档生成失败 (编译或运行错误)"
fi

# Clean up
rm -f /tmp/test_openapi_gen.rs /tmp/Cargo.toml /tmp/openapi_output.json
rm -rf /tmp/target

echo ""
echo "📋 代码质量检查"
echo "---------------"

# Test 5: Check for utoipa annotations in handlers
echo -e "${BLUE}测试 5: API 处理程序注解检查${NC}"
if grep -q "#\[utoipa::path" crates/paymaster-relay/src/api_handlers.rs; then
    print_result 0 "API 处理程序包含 utoipa path 注解"
else
    print_result 1 "API 处理程序缺少 utoipa path 注解"
fi

# Test 6: Check for ToSchema derives in data structures
echo -e "${BLUE}测试 6: 数据结构 Schema 注解检查${NC}"
if grep -q "#\[derive.*ToSchema" crates/paymaster-relay/src/api_handlers.rs; then
    print_result 0 "数据结构包含 ToSchema 派生"
else
    print_result 1 "数据结构缺少 ToSchema 派生"
fi

# Test 7: Verify OpenApi derive in main doc structure
echo -e "${BLUE}测试 7: 主文档结构检查${NC}"
if grep -q "#\[derive(OpenApi)\]" crates/paymaster-relay/src/api_schemas.rs; then
    print_result 0 "主文档结构包含 OpenApi 派生"
else
    print_result 1 "主文档结构缺少 OpenApi 派生"
fi

echo ""
echo "📊 测试结果总结"
echo "================"
echo -e "${GREEN}通过: $PASSED${NC}"
echo -e "${RED}失败: $FAILED${NC}"
echo ""

# Final assessment
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 utoipa 集成测试全部通过！${NC}"
    echo ""
    echo "✅ 完成的功能："
    echo "  • RPC 方法 utoipa 注解 ✓"
    echo "  • API 处理程序端点定义 ✓"
    echo "  • OpenAPI 自动文档生成 ✓"
    echo "  • Schema 结构定义 ✓"
    echo "  • 编译和测试通过 ✓"
    echo ""
    echo "🚀 utoipa 自动生成的 OpenAPI 系统已就绪！"
    exit 0
elif [ $FAILED -le 2 ]; then
    echo -e "${YELLOW}⚠️  utoipa 集成基本成功，有少量问题需要关注${NC}"
    exit 1
else
    echo -e "${RED}❌ utoipa 集成存在重要问题需要修复${NC}"
    exit 2
fi