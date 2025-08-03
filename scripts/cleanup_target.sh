#!/bin/bash

# SuperRelay Target Directory Cleanup Script
# 自动清理target目录，保留必要的构建输出

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TARGET_DIR="${PROJECT_ROOT}/target"

echo "🧹 Starting SuperRelay target directory cleanup..."
echo "📁 Target directory: ${TARGET_DIR}"

# 检查target目录是否存在
if [ ! -d "${TARGET_DIR}" ]; then
    echo "✅ Target directory doesn't exist, nothing to clean"
    exit 0
fi

# 显示清理前的大小
BEFORE_SIZE=$(du -sh "${TARGET_DIR}" 2>/dev/null | cut -f1 || echo "unknown")
echo "📊 Target directory size before cleanup: ${BEFORE_SIZE}"

# 1. 清理debug目录，保留最近的构建
echo "🔍 Cleaning debug builds..."
if [ -d "${TARGET_DIR}/debug" ]; then
    # 保留最新的可执行文件，删除旧的中间文件
    find "${TARGET_DIR}/debug" -name "*.d" -type f -delete 2>/dev/null || true
    find "${TARGET_DIR}/debug" -name "*.rlib" -type f -mtime +1 -delete 2>/dev/null || true
    find "${TARGET_DIR}/debug/deps" -type f -mtime +1 -delete 2>/dev/null || true
    find "${TARGET_DIR}/debug/incremental" -type d -mtime +1 -exec rm -rf {} + 2>/dev/null || true
    find "${TARGET_DIR}/debug/.fingerprint" -type d -mtime +1 -exec rm -rf {} + 2>/dev/null || true
fi

# 2. 清理release目录，保留当前版本
echo "🔍 Cleaning release builds..."
if [ -d "${TARGET_DIR}/release" ]; then
    # 保留主要的可执行文件：rundler, super-relay
    # 删除旧的中间文件和依赖
    find "${TARGET_DIR}/release" -name "*.d" -type f -delete 2>/dev/null || true
    find "${TARGET_DIR}/release/deps" -type f -mtime +7 -delete 2>/dev/null || true
    find "${TARGET_DIR}/release/incremental" -type d -mtime +3 -exec rm -rf {} + 2>/dev/null || true
    find "${TARGET_DIR}/release/.fingerprint" -type d -mtime +3 -exec rm -rf {} + 2>/dev/null || true

    # 保留最新的可执行文件，删除旧版本
    RELEASE_BINS="${TARGET_DIR}/release"
    if [ -d "${RELEASE_BINS}" ]; then
        # 保留最新的rundler和super-relay可执行文件
        find "${RELEASE_BINS}" -name "rundler-*" -type f -mtime +7 -delete 2>/dev/null || true
        find "${RELEASE_BINS}" -name "super-relay-*" -type f -mtime +7 -delete 2>/dev/null || true
    fi
fi

# 3. 清理tmp目录 - 编译完成后可以完全删除
echo "🔍 Cleaning tmp directories..."
find "${TARGET_DIR}" -name "tmp" -type d -exec rm -rf {} + 2>/dev/null || true
find "${TARGET_DIR}" -name "*.tmp" -type f -delete 2>/dev/null || true

# 4. 清理其他临时文件
echo "🔍 Cleaning other temporary files..."
find "${TARGET_DIR}" -name "*.o" -type f -delete 2>/dev/null || true
find "${TARGET_DIR}" -name "*.so.*" -type f -mtime +7 -delete 2>/dev/null || true
find "${TARGET_DIR}" -name "*.dylib.*" -type f -mtime +7 -delete 2>/dev/null || true

# 5. 清理测试构建输出
echo "🔍 Cleaning test builds..."
find "${TARGET_DIR}" -path "*/test-*" -type d -mtime +3 -exec rm -rf {} + 2>/dev/null || true

# 6. 清理空目录
echo "🔍 Removing empty directories..."
find "${TARGET_DIR}" -type d -empty -delete 2>/dev/null || true

# 显示清理后的大小
AFTER_SIZE=$(du -sh "${TARGET_DIR}" 2>/dev/null | cut -f1 || echo "unknown")
echo "📊 Target directory size after cleanup: ${AFTER_SIZE}"

# 保留的重要文件提示
echo "✅ Cleanup completed! Preserved:"
echo "   - Latest release binaries (rundler, super-relay)"
echo "   - Recent debug builds (last 1 day)"
echo "   - Essential build artifacts"
echo ""
echo "🗑️  Cleaned:"
echo "   - Old incremental build cache"
echo "   - Temporary files and directories"
echo "   - Old dependency artifacts"
echo "   - Test build outputs"

# 显示当前保留的主要文件
echo ""
echo "📋 Current important binaries:"
if [ -f "${TARGET_DIR}/release/rundler" ]; then
    ls -lh "${TARGET_DIR}/release/rundler"
fi
if [ -f "${TARGET_DIR}/release/super-relay" ]; then
    ls -lh "${TARGET_DIR}/release/super-relay"
fi
if [ -f "${TARGET_DIR}/debug/rundler" ]; then
    ls -lh "${TARGET_DIR}/debug/rundler"
fi
if [ -f "${TARGET_DIR}/debug/super-relay" ]; then
    ls -lh "${TARGET_DIR}/debug/super-relay"
fi

echo "🎉 Target cleanup completed successfully!"