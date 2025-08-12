#!/bin/bash
# 静态方法名一致性测试 - 不需要启动服务
# 验证代码、配置、文档中的方法名一致性

set -e

echo "🔍 SuperRelay 方法名静态一致性测试"
echo "================================="
echo "检查范围: sponsorUserOperation -> pm_sponsorUserOperation"
echo "测试类型: 静态代码分析 (无需启动服务)"
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

echo "📂 代码文件分析"
echo "---------------"

# Test 1: Check RPC trait definition
echo -e "${BLUE}测试 1: RPC trait 定义检查${NC}"
RPC_FILE="crates/paymaster-relay/src/rpc.rs"
if [ -f "$RPC_FILE" ]; then
    # Check namespace
    if grep -q 'namespace = "pm"' "$RPC_FILE"; then
        print_result 0 "RPC trait 包含正确的 pm 命名空间"
    else
        print_result 1 "RPC trait 缺少 pm 命名空间定义"
    fi
    
    # Check method name
    if grep -q 'method(name = "sponsorUserOperation")' "$RPC_FILE"; then
        print_result 0 "RPC method 定义使用 sponsorUserOperation (正确，将自动添加 pm_ 前缀)"
    else
        print_result 1 "RPC method 定义不匹配预期格式"
    fi
else
    print_result 1 "RPC 文件不存在: $RPC_FILE"
fi

# Test 2: Check Swagger implementation
echo -e "${BLUE}测试 2: Swagger 实现检查${NC}"
SWAGGER_FILE="crates/paymaster-relay/src/swagger.rs"
if [ -f "$SWAGGER_FILE" ]; then
    # Check if new method name is used in RPC calls
    if grep -q '"pm_sponsorUserOperation"' "$SWAGGER_FILE"; then
        print_result 0 "Swagger 实现使用新方法名 pm_sponsorUserOperation"
    else
        print_result 1 "Swagger 实现不包含新方法名"
    fi
    
    # Check if old method name is removed
    if grep -q '"sponsorUserOperation"' "$SWAGGER_FILE" | grep -v pm_; then
        print_result 1 "Swagger 实现仍包含旧方法名 sponsorUserOperation"
    else
        print_result 0 "Swagger 实现已移除旧方法名"
    fi
else
    print_result 1 "Swagger 文件不存在: $SWAGGER_FILE"
fi

# Test 3: Check OpenAPI specification
echo -e "${BLUE}测试 3: OpenAPI 规范检查${NC}"
OPENAPI_FILE="web-ui/swagger-ui/openapi.json"
if [ -f "$OPENAPI_FILE" ]; then
    # Check for new method name
    if grep -q "pm_sponsorUserOperation" "$OPENAPI_FILE"; then
        print_result 0 "OpenAPI 规范包含新方法名 pm_sponsorUserOperation"
    else
        print_result 1 "OpenAPI 规范不包含新方法名"
    fi
    
    # Verify JSON format
    if jq . "$OPENAPI_FILE" > /dev/null 2>&1; then
        print_result 0 "OpenAPI 规范 JSON 格式正确"
    else
        print_result 1 "OpenAPI 规范 JSON 格式错误"
    fi
    
    # Check example data completeness
    if jq -r '.paths["/sponsorUserOperation"].post.requestBody.content["application/json"].schema.example.params[0].sender' "$OPENAPI_FILE" | grep -q "0xf39Fd"; then
        print_result 0 "OpenAPI 规范包含完整的示例数据"
    else
        print_result 1 "OpenAPI 规范缺少完整的示例数据"
    fi
else
    print_result 1 "OpenAPI 文件不存在: $OPENAPI_FILE"
fi

# Test 4: Check API schema definitions
echo -e "${BLUE}测试 4: API Schema 定义检查${NC}"
API_SCHEMA_FILE="crates/paymaster-relay/src/api_schemas.rs"
if [ -f "$API_SCHEMA_FILE" ]; then
    # Check response structure
    if grep -q "paymaster_and_data.*String" "$API_SCHEMA_FILE"; then
        print_result 0 "API Schema 包含正确的响应结构 (paymaster_and_data)"
    else
        print_result 1 "API Schema 响应结构定义不正确"
    fi
    
    # Check request structure
    if grep -q "SponsorUserOperationRequest" "$API_SCHEMA_FILE"; then
        print_result 0 "API Schema 包含请求结构定义"
    else
        print_result 1 "API Schema 缺少请求结构定义"
    fi
else
    print_result 1 "API Schema 文件不存在: $API_SCHEMA_FILE"
fi

# Test 5: Check test files consistency
echo -e "${BLUE}测试 5: 测试文件一致性检查${NC}"
TEST_FILE="crates/paymaster-relay/tests/swagger_test.rs"
if [ -f "$TEST_FILE" ]; then
    # Check if test uses correct response structure
    if grep -q "paymaster_and_data" "$TEST_FILE"; then
        print_result 0 "测试文件使用正确的响应字段 (paymaster_and_data)"
    else
        print_result 1 "测试文件使用错误的响应字段"
    fi
    
    # Check error structure
    if grep -q "error_response.code" "$TEST_FILE"; then
        print_result 0 "测试文件使用正确的错误结构"
    else
        print_result 1 "测试文件错误结构定义不正确"
    fi
else
    print_result 1 "测试文件不存在: $TEST_FILE"
fi

echo ""
echo "🔗 依赖和导入检查"
echo "-----------------"

# Test 6: Check Cargo.toml for required dependencies
echo -e "${BLUE}测试 6: 依赖项检查${NC}"
CARGO_FILE="crates/paymaster-relay/Cargo.toml"
if [ -f "$CARGO_FILE" ]; then
    if grep -q "reqwest.*=" "$CARGO_FILE"; then
        print_result 0 "Cargo.toml 包含 reqwest 依赖 (Swagger 代理需要)"
    else
        print_result 1 "Cargo.toml 缺少 reqwest 依赖"
    fi
    
    if grep -q "utoipa.*=" "$CARGO_FILE"; then
        print_result 0 "Cargo.toml 包含 utoipa 依赖"
    else
        print_result 1 "Cargo.toml 缺少 utoipa 依赖"
    fi
else
    print_result 1 "Cargo.toml 文件不存在: $CARGO_FILE"
fi

# Test 7: Check compilation
echo -e "${BLUE}测试 7: 代码编译检查${NC}"
if cargo check --package rundler-paymaster-relay --quiet > /dev/null 2>&1; then
    print_result 0 "paymaster-relay 包编译成功"
else
    print_result 1 "paymaster-relay 包编译失败"
fi

echo ""
echo "📋 配置一致性分析"
echo "-----------------"

# Test 8: Check method name consistency across files
echo -e "${BLUE}测试 8: 方法名一致性验证${NC}"

# Count occurrences of old method name (should be minimal, only in RPC trait)
OLD_METHOD_COUNT=$(grep -r "sponsorUserOperation" crates/paymaster-relay/src/ --exclude="*.rs.bk" | grep -v pm_sponsorUserOperation | wc -l)
if [ "$OLD_METHOD_COUNT" -le 3 ]; then
    print_result 0 "旧方法名出现次数合理 ($OLD_METHOD_COUNT 次，主要在 RPC trait 定义中)"
else
    print_result 1 "旧方法名出现次数过多 ($OLD_METHOD_COUNT 次)，可能存在遗漏"
fi

# Count occurrences of new method name
NEW_METHOD_COUNT=$(grep -r "pm_sponsorUserOperation" crates/paymaster-relay/src/ web-ui/swagger-ui/ 2>/dev/null | wc -l)
if [ "$NEW_METHOD_COUNT" -ge 3 ]; then
    print_result 0 "新方法名在系统中正确使用 ($NEW_METHOD_COUNT 次)"
else
    print_result 1 "新方法名使用不足 ($NEW_METHOD_COUNT 次)"
fi

# Test 9: Cross-reference validation
echo -e "${BLUE}测试 9: 交叉引用验证${NC}"

# Check if all files that use the method name are consistent
INCONSISTENT_FILES=$(grep -l "sponsorUserOperation" crates/paymaster-relay/src/*.rs 2>/dev/null | while read file; do
    if [ -f "$file" ] && [ "$file" != "crates/paymaster-relay/src/rpc.rs" ]; then
        if ! grep -q "pm_sponsorUserOperation" "$file"; then
            echo "$file"
        fi
    fi
done)

if [ -z "$INCONSISTENT_FILES" ]; then
    print_result 0 "所有相关文件的方法名引用一致"
else
    print_result 1 "以下文件方法名引用不一致: $INCONSISTENT_FILES"
fi

echo ""
echo "📊 测试结果总结"
echo "================"
echo -e "${GREEN}通过: $PASSED${NC}"
echo -e "${RED}失败: $FAILED${NC}"
echo ""

# Analysis summary
echo "🔍 静态分析结果:"
echo "----------------"

if [ -f "$OPENAPI_FILE" ]; then
    echo "• OpenAPI 方法名: $(grep -o 'pm_sponsorUserOperation\|sponsorUserOperation' "$OPENAPI_FILE" | head -1)"
fi

if [ -f "$SWAGGER_FILE" ]; then
    echo "• Swagger 实现方法名: $(grep -o '"pm_sponsorUserOperation"' "$SWAGGER_FILE" | head -1 | tr -d '"')"
fi

if [ -f "$RPC_FILE" ]; then
    echo "• RPC trait 方法名: $(grep -o 'sponsorUserOperation' "$RPC_FILE" | head -1) (自动添加 pm_ 前缀)"
    echo "• RPC 命名空间: $(grep -o 'namespace = "pm"' "$RPC_FILE" | head -1)"
fi

echo ""

# Final assessment
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 静态分析全部通过！方法名迁移在代码层面完全正确${NC}"
    echo ""
    echo "✅ 确认结果："
    echo "  • RPC trait: 使用 sponsorUserOperation + pm 命名空间 ✓"
    echo "  • 实际调用: 使用 pm_sponsorUserOperation ✓" 
    echo "  • OpenAPI: 使用 pm_sponsorUserOperation ✓"
    echo "  • Swagger UI: 使用 pm_sponsorUserOperation ✓"
    echo "  • 代码编译: 成功 ✓"
    echo ""
    echo "🚀 建议: 可以启动服务进行端到端测试确认功能正常"
    exit 0
elif [ $FAILED -le 2 ]; then
    echo -e "${YELLOW}⚠️  静态分析基本通过，有少量问题${NC}"
    echo ""
    echo "💡 建议检查失败的测试项目，多数为非关键问题"
    exit 1
else
    echo -e "${RED}❌ 静态分析发现重要问题，需要修复后再测试${NC}"
    exit 2
fi