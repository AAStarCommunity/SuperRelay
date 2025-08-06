#!/bin/bash
# SuperRelay 优化构建脚本
# 基于 Jason Cursor Rules Rust 构建优化实践

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
    echo -e "${CYAN}🚀 SuperRelay 优化构建脚本${NC}"
    echo -e "${CYAN}基于 Jason Cursor Rules Rust 构建优化实践${NC}"
    echo ""
    echo "使用方法:"
    echo "  $0 [OPTIONS] [TARGET]"
    echo ""
    echo "OPTIONS:"
    echo "  --profile PROFILE    构建配置 [dev|release] (默认: dev)"
    echo "  --package PACKAGE    指定包名 (默认: super-relay)"
    echo "  --check             仅检查语法，不构建"
    echo "  --clean             清理后构建"
    echo "  --timing            显示构建时间分析"
    echo "  --cache             使用 sccache 缓存"
    echo "  --jobs N            并行作业数量 (默认: 自动)"
    echo "  --help              显示此帮助信息"
    echo ""
    echo "TARGET:"
    echo "  super-relay         构建 SuperRelay (默认)"
    echo "  rundler            构建 Rundler"
    echo "  all                构建所有包"
    echo ""
    echo "示例:"
    echo "  $0                           # 快速开发构建"
    echo "  $0 --profile release         # 生产构建"
    echo "  $0 --profile dev            # 开发构建"
    echo "  $0 --check                   # 快速语法检查"
    echo "  $0 --clean --profile release # 清理后生产构建"
}

# 默认参数
PROFILE="dev"
PACKAGE="super-relay"
CHECK_ONLY=false
CLEAN_BUILD=false
SHOW_TIMING=false
USE_CACHE=false
JOBS=""
TARGET="super-relay"

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
        --timing)
            SHOW_TIMING=true
            shift
            ;;
        --cache)
            USE_CACHE=true
            shift
            ;;
        --jobs)
            JOBS="$2"
            shift 2
            ;;
        --jobs*)
            # 支持 --jobs2, --jobs4 等格式
            JOBS="${1#--jobs}"
            shift
            ;;
        --help)
            show_usage
            exit 0
            ;;
        super-relay|rundler|all)
            TARGET="$1"
            shift
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
    dev|release)
        ;;
    *)
        echo -e "${RED}❌ 无效的构建配置: $PROFILE${NC}"
        echo "支持的配置: dev, release"
        exit 1
        ;;
esac

echo -e "${PURPLE}🚀 SuperRelay 优化构建系统${NC}"
echo -e "${PURPLE}基于 Jason Cursor Rules 构建优化${NC}"
echo "=================================="
echo -e "${BLUE}📋 构建配置:${NC}"
echo "  🎯 目标: $TARGET"
echo "  📊 配置: $PROFILE"
echo "  📦 包名: $PACKAGE" 
echo "  🔍 仅检查: $CHECK_ONLY"
echo "  🧹 清理构建: $CLEAN_BUILD"
echo "  ⏱️  时间分析: $SHOW_TIMING"
echo "  💾 缓存: $USE_CACHE"
echo "  🔧 并行作业: ${JOBS:-自动}"
echo ""

# 检查系统优化建议
echo -e "${YELLOW}🔍 检查构建环境优化...${NC}"

# 检查链接器优化
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  📝 建议 (macOS): 安装 zld 快速链接器: brew install michaeleisel/zld/zld"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "  📝 建议 (Linux): 安装 mold 链接器: sudo apt install mold"
fi

# 检查 sccache
if command -v sccache >/dev/null 2>&1; then
    echo -e "${GREEN}  ✅ sccache 可用${NC}"
    USE_CACHE=true
else
    echo -e "${YELLOW}  💡 建议安装 sccache 构建缓存: cargo install sccache${NC}"
fi

# 检查 cargo-watch
if command -v cargo-watch >/dev/null 2>&1; then
    echo -e "${GREEN}  ✅ cargo-watch 可用 (开发时自动重建)${NC}"
else
    echo -e "${YELLOW}  💡 建议安装 cargo-watch: cargo install cargo-watch${NC}"
fi

echo ""

# 设置构建环境
if [[ $USE_CACHE == true && -n "$(command -v sccache)" ]]; then
    export RUSTC_WRAPPER=sccache
    echo -e "${GREEN}🚀 启用 sccache 构建缓存${NC}"
fi

# 设置并行作业数
if [[ -n "$JOBS" ]]; then
    export CARGO_BUILD_JOBS="$JOBS"
fi

# 清理构建
if [[ $CLEAN_BUILD == true ]]; then
    echo -e "${YELLOW}🧹 清理构建缓存...${NC}"
    cargo clean
    if [[ $USE_CACHE == true ]]; then
        sccache --zero-stats 2>/dev/null || true
    fi
    echo -e "${GREEN}✅ 清理完成${NC}"
fi

# 构建函数
build_target() {
    local target=$1
    local start_time=$(date +%s)
    
    echo -e "${CYAN}🔨 开始构建: $target${NC}"
    
    # 构建命令组装
    cmd="cargo"
    args=""
    
    if [[ $CHECK_ONLY == true ]]; then
        args="check"
    else
        args="build"
    fi
    
    # 添加配置参数
    if [[ "$PROFILE" == "release" ]]; then
        args="$args --release"
    fi
    
    # 添加包参数
    case $target in
        super-relay)
            args="$args --package super-relay"
            ;;
        rundler)
            args="$args --package rundler"
            ;;
        all)
            # 构建所有包
            ;;
    esac
    
    # 添加时间分析
    if [[ $SHOW_TIMING == true ]]; then
        args="$args --timings"
    fi
    
    # 执行构建
    echo -e "${BLUE}📋 执行命令: $cmd $args${NC}"
    
    if $cmd $args; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo -e "${GREEN}✅ 构建成功: $target (${duration}秒)${NC}"
        
        # 显示缓存统计
        if [[ $USE_CACHE == true ]]; then
            echo -e "${CYAN}📊 缓存统计:${NC}"
            sccache --show-stats 2>/dev/null || true
        fi
        
        return 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo -e "${RED}❌ 构建失败: $target (${duration}秒)${NC}"
        return 1
    fi
}

# 显示构建前的优化提示
echo -e "${PURPLE}💡 构建优化提示:${NC}"
case $PROFILE in
    dev)
        echo "  🚀 开发模式: 快速编译，完整调试信息"
        ;;
    release)
        echo "  🏆 发布模式: 最大性能优化"
        ;;
esac

if [[ $CHECK_ONLY == true ]]; then
    echo "  🔍 语法检查模式: 仅验证代码正确性，不生成二进制"
fi

echo ""

# 执行构建
total_start=$(date +%s)

if build_target "$TARGET"; then
    total_end=$(date +%s)
    total_duration=$((total_end - total_start))
    
    echo ""
    echo -e "${GREEN}🎉 构建完成！${NC}"
    echo -e "${GREEN}📊 总时间: ${total_duration}秒${NC}"
    
    # 显示构建产物信息
    if [[ $CHECK_ONLY == false ]]; then
        case $PROFILE in
            dev)
                BINARY_PATH="target/debug"
                ;;
            release)
                BINARY_PATH="target/release"
                ;;
        esac
        
        if [[ "$TARGET" != "all" ]]; then
            binary_name="$TARGET"
            if [[ -f "$BINARY_PATH/$binary_name" ]]; then
                size=$(du -h "$BINARY_PATH/$binary_name" | cut -f1)
                echo -e "${CYAN}📦 二进制文件: $BINARY_PATH/$binary_name ($size)${NC}"
                
                # 性能建议
                if [[ "$PROFILE" == "dev" ]] && [[ "$size" > "50M" ]]; then
                    echo -e "${YELLOW}💡 提示: 二进制较大，考虑使用 --profile release 优化体积${NC}"
                fi
            fi
        fi
    fi
    
    # 开发模式建议
    if [[ "$PROFILE" == "dev" ]]; then
        echo ""
        echo -e "${YELLOW}🔥 开发提示:${NC}"
        echo "  • 使用 cargo-watch 自动重建: cargo watch -x 'run --package super-relay'"
        echo "  • 快速检查: $0 --check"
        echo "  • 生产构建: $0 --profile release"
    fi
    
else
    echo ""
    echo -e "${RED}💥 构建失败${NC}"
    echo -e "${YELLOW}🔧 构建优化建议:${NC}"
    echo "  • 清理重试: $0 --clean"
    echo "  • 仅检查语法: $0 --check"
    echo "  • 降低并行度: $0 --jobs 2"
    exit 1
fi