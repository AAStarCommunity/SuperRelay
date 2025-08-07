#!/bin/bash
# SuperRelay 智能OpenAPI更新脚本
# 真正从代码中提取API信息，动态生成OpenAPI规范

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "${SCRIPT_DIR}")"

echo "🔍 SuperRelay 智能API文档生成器"
echo "================================================"
echo "📂 项目目录: $PROJECT_ROOT"

# 检查Python环境
if ! command -v python3 &> /dev/null; then
    echo "❌ Python3未安装，请先安装Python3"
    exit 1
fi

# 检查依赖目录
WEB_UI_DIR="${PROJECT_ROOT}/web-ui/swagger-ui"
if [[ ! -d "$WEB_UI_DIR" ]]; then
    echo "📁 创建Swagger UI目录..."
    mkdir -p "$WEB_UI_DIR"
fi

echo "🤖 正在分析Rust源代码..."

# 运行代码分析器
if python3 "${SCRIPT_DIR}/extract_api_info.py" "$PROJECT_ROOT"; then
    echo "✅ 代码分析完成"
else
    echo "❌ 代码分析失败"
    exit 1
fi

# 验证生成的JSON
OPENAPI_FILE="${WEB_UI_DIR}/openapi.json"
if [[ -f "$OPENAPI_FILE" ]]; then
    if python3 -m json.tool "$OPENAPI_FILE" > /dev/null 2>&1; then
        echo "✅ 生成的OpenAPI规范格式正确"
        
        # 显示统计信息
        echo "📊 生成统计:"
        METHODS_COUNT=$(python3 -c "import json; data=json.load(open('$OPENAPI_FILE')); print(len(data['paths']))")
        SCHEMAS_COUNT=$(python3 -c "import json; data=json.load(open('$OPENAPI_FILE')); print(len(data['components']['schemas']))")
        VERSION=$(python3 -c "import json; data=json.load(open('$OPENAPI_FILE')); print(data['info']['version'])")
        
        echo "   • 项目版本: $VERSION"
        echo "   • API端点: $METHODS_COUNT 个"
        echo "   • 数据模型: $SCHEMAS_COUNT 个"
        
    else
        echo "❌ 生成的JSON格式无效"
        exit 1
    fi
else
    echo "❌ OpenAPI文件未生成"
    exit 1
fi

# 检查Swagger UI状态
if curl -s http://localhost:9000/ > /dev/null 2>&1; then
    echo "🎉 Swagger UI正在运行，文档将自动更新"
    echo "🌐 访问地址: http://localhost:9000/"
else
    echo "💡 启动Swagger UI查看更新后的文档:"
    echo "   ./scripts/start_web_ui.sh"
fi

# 显示API变更对比（如果有备份文件）
BACKUP_FILE="${WEB_UI_DIR}/openapi-backup.json"
if [[ -f "$BACKUP_FILE" ]]; then
    echo ""
    echo "🔄 API变更检测:"
    
    # 比较API数量
    OLD_COUNT=$(python3 -c "import json; data=json.load(open('$BACKUP_FILE')); print(len(data.get('paths', {})))" 2>/dev/null || echo "0")
    NEW_COUNT=$METHODS_COUNT
    
    if [[ "$NEW_COUNT" -gt "$OLD_COUNT" ]]; then
        echo "   📈 新增API: $((NEW_COUNT - OLD_COUNT)) 个"
    elif [[ "$NEW_COUNT" -lt "$OLD_COUNT" ]]; then
        echo "   📉 移除API: $((OLD_COUNT - NEW_COUNT)) 个"
    else
        echo "   📊 API数量无变化: $NEW_COUNT 个"
    fi
fi

echo "================================================"
echo "🚀 智能API文档生成完成！"
echo ""
echo "💫 与传统方式的区别:"
echo "   • ❌ 旧方式: 手动维护静态JSON"
echo "   • ✅ 新方式: 自动从代码提取API信息"
echo "   • 🎯 真正的'代码即文档'"
echo ""
echo "📋 生成内容:"
echo "   • API端点: 从#[method(name = \"...\")] 注解自动提取"
echo "   • 参数类型: 从函数签名自动解析"
echo "   • 数据模型: 从struct定义自动生成"
echo "   • 文件位置: 包含源码文件路径和行号"