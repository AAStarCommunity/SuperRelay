#!/bin/bash

# SuperRelay 独立构建脚本
# 专门用于构建 super-relay，支持 debug/release 模式，10分钟超时

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 显示使用说明
show_usage() {
    echo -e "${CYAN}🚀 SuperRelay 独立构建脚本${NC}"
    echo ""
    echo "使用方法:"
    echo "  $0 [OPTIONS]"
    echo ""
    echo "OPTIONS:"
    echo "  --profile PROFILE    构建配置 [debug|release] (默认: debug)"
    echo "  --package PACKAGE    指定包名 (默认: super-relay)" 
    echo "  --check             仅检查语法，不构建"
    echo "  --clean             清理后构建"
    echo "  --timeout SECONDS   构建超时时间 (默认: 600秒)"
    echo "  --help              显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0                           # debug 模式构建"
    echo "  $0 --profile release         # release 模式构建"
    echo "  $0 --check                   # 快速语法检查"
    echo "  $0 --timeout 720             # 12分钟超时"
}

# 默认参数
PROFILE="debug"
PACKAGE="super-relay"
CHECK_ONLY=false
CLEAN_BUILD=false
TIMEOUT=600  # 10分钟默认超时

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --profile)
            PROFILE="$2"
            shift 2
            ;;
        --package)
            PACKAGE="$2"
            shift 2
            ;;
        --check)
            CHECK_ONLY=true
            shift
            ;;
        --clean)
            CLEAN_BUILD=true
            shift
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            echo -e "${RED}❌ 未知参数: $1${NC}"
            echo "使用 --help 查看使用说明"
            exit 1
            ;;
    esac
done

# 验证构建配置
case $PROFILE in
    debug|release)
        ;;
    dev)
        # dev 等同于 debug
        PROFILE="debug"
        ;;
    *)
        echo -e "${RED}❌ 无效的构建配置: $PROFILE${NC}"
        echo "支持的配置: debug, release"
        exit 1
        ;;
esac

echo -e "${PURPLE}🚀 SuperRelay 独立构建系统${NC}"
echo "=================================="
echo -e "${BLUE}📋 构建配置:${NC}"
echo "  🎯 目标: $PACKAGE"
echo "  📊 模式: $PROFILE"
echo "  🔍 仅检查: $CHECK_ONLY"
echo "  🧹 清理构建: $CLEAN_BUILD"
echo "  ⏰ 超时时间: ${TIMEOUT}秒 ($(($TIMEOUT / 60))分钟)"
echo ""

# 检查构建环境优化
echo -e "${YELLOW}🔍 检查构建环境优化...${NC}"

# 检查 sccache
if command -v sccache >/dev/null 2>&1; then
    echo -e "${GREEN}  ✅ sccache 可用${NC}"
    export RUSTC_WRAPPER=sccache
    USE_CACHE=true
else
    echo -e "${YELLOW}  💡 建议安装 sccache 构建缓存: cargo install sccache${NC}"
    USE_CACHE=false
fi

# 检查 cargo-watch  
if command -v cargo-watch >/dev/null 2>&1; then
    echo -e "${GREEN}  ✅ cargo-watch 可用 (开发时自动重建)${NC}"
else
    echo -e "${YELLOW}  💡 建议安装 cargo-watch: cargo install cargo-watch${NC}"
fi

echo ""

# 清理构建
if [[ $CLEAN_BUILD == true ]]; then
    echo -e "${YELLOW}🧹 清理构建缓存...${NC}"
    cargo clean
    if [[ $USE_CACHE == true ]]; then
        sccache --zero-stats 2>/dev/null || true
    fi
    echo -e "${GREEN}✅ 清理完成${NC}"
fi

# 构建函数 - 带超时控制
build_with_timeout() {
    local start_time=$(date +%s)
    
    echo -e "${CYAN}🔨 开始构建: $PACKAGE${NC}"
    
    # 构建命令组装
    local cmd_args=""
    
    if [[ $CHECK_ONLY == true ]]; then
        cmd_args="check"
    else
        cmd_args="build"
    fi
    
    # 添加配置参数
    if [[ "$PROFILE" == "release" ]]; then
        cmd_args="$cmd_args --release"
    fi
    
    # 添加包参数
    cmd_args="$cmd_args --package $PACKAGE"
    
    # 执行构建
    echo -e "${BLUE}📋 执行命令: cargo $cmd_args${NC}"
    echo -e "${YELLOW}⏰ 超时时间: ${TIMEOUT}秒，如需更长时间请使用 --timeout 参数${NC}"
    echo ""
    
    # 使用 timeout 命令执行构建
    if timeout ${TIMEOUT}s cargo $cmd_args; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo ""
        echo -e "${GREEN}✅ 构建成功: $PACKAGE (${duration}秒)${NC}"
        
        # 显示缓存统计
        if [[ $USE_CACHE == true ]]; then
            echo -e "${CYAN}📊 sccache 统计:${NC}"
            sccache --show-stats 2>/dev/null || true
        fi
        
        # 显示构建产物信息
        if [[ $CHECK_ONLY == false ]]; then
            local binary_path=""
            case $PROFILE in
                debug)
                    binary_path="target/debug/$PACKAGE"
                    ;;
                release)
                    binary_path="target/release/$PACKAGE"
                    ;;
            esac
            
            if [[ -f "$binary_path" ]]; then
                local size=$(du -h "$binary_path" | cut -f1)
                echo -e "${CYAN}📦 二进制文件: $binary_path ($size)${NC}"
            fi
        fi
        
        return 0
    else
        local exit_code=$?
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        if [[ $exit_code == 124 ]]; then
            echo ""
            echo -e "${RED}❌ 构建超时: $PACKAGE (${duration}秒 > ${TIMEOUT}秒)${NC}"
            echo -e "${YELLOW}💡 建议: 使用 --timeout $(($TIMEOUT + 300)) 延长超时时间${NC}"
        else
            echo ""
            echo -e "${RED}❌ 构建失败: $PACKAGE (${duration}秒)${NC}"
        fi
        
        return 1
    fi
}

# 显示构建前的优化提示
echo -e "${PURPLE}💡 构建优化提示:${NC}"
case $PROFILE in
    debug)
        echo "  🚀 Debug 模式: 快速编译，完整调试信息"
        ;;
    release)
        echo "  🏆 Release 模式: 最大性能优化"
        ;;
esac

if [[ $CHECK_ONLY == true ]]; then
    echo "  🔍 语法检查模式: 仅验证代码正确性，不生成二进制"
fi

if [[ $USE_CACHE == true ]]; then
    echo "  💾 sccache 缓存: 已启用构建缓存加速"
fi

echo ""

# 执行构建
if build_with_timeout; then
    echo ""
    echo -e "${GREEN}🎉 构建完成！${NC}"
    
    # 开发模式建议
    if [[ "$PROFILE" == "debug" ]]; then
        echo ""
        echo -e "${YELLOW}🔥 开发提示:${NC}"
        echo "  • 快速检查: $0 --check"  
        echo "  • 生产构建: $0 --profile release"
        echo "  • 自动重建: cargo watch -x 'run --package $PACKAGE'"
    fi
    
    exit 0
else
    echo ""
    echo -e "${RED}💥 构建失败${NC}"
    echo -e "${YELLOW}🔧 构建优化建议:${NC}"
    echo "  • 清理重试: $0 --clean"
    echo "  • 仅检查语法: $0 --check"
    echo "  • 延长超时: $0 --timeout $(($TIMEOUT + 300))"
    exit 1
fi