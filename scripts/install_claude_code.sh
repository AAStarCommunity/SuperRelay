#!/bin/bash

# Claude Code 安装脚本
# 修复在 Rust 项目中安装 Claude Code 的问题

set -e

echo "🔧 Installing Claude Code CLI..."

# 方式1: 全局安装 (推荐)
echo "📦 尝试全局安装..."
if npm install -g @anthropic-ai/claude-code; then
    echo "✅ Claude Code 全局安装成功"
    exit 0
fi

# 方式2: 使用 npx (临时使用)
echo "⚡ 全局安装失败，使用 npx 作为备选方案"
echo "💡 您可以使用 'npx @anthropic-ai/claude-code' 来运行 Claude Code"

# 方式3: 在 demo 目录中安装
echo "📁 尝试在 demo 目录中安装..."
if [ -d "demo" ] && [ -f "demo/package.json" ]; then
    cd demo
    npm install @anthropic-ai/claude-code
    echo "✅ Claude Code 已安装到 demo 目录"
    echo "💡 在 demo 目录中使用: 'npx claude-code'"
else
    echo "⚠️  demo 目录不存在或没有 package.json"
fi

echo ""
echo "🎯 安装完成提示:"
echo "  • 全局使用: claude-code"
echo "  • 临时使用: npx @anthropic-ai/claude-code"
echo "  • demo 目录: cd demo && npx claude-code"

claude --version
