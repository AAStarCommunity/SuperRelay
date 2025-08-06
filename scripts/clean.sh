#!/bin/bash

# SuperRelay Clean Script
# 独立的Rust target目录缓存清理脚本
# Usage: ./scripts/clean.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🧹 Starting SuperRelay cleanup..."

# 调用已有的清理脚本
"${SCRIPT_DIR}/cleanup_target.sh"

echo "✅ SuperRelay cleanup completed!"