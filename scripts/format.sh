#!/bin/bash

# Super-Relay Code Formatting Script
# Usage: ./scripts/format.sh

set -e

echo "🔧 Formatting Rust code..."
cargo +nightly fmt --all

echo "🔍 Running clippy..."
cargo clippy -p rundler-paymaster-relay --fix --allow-dirty

echo "📋 Running cargo-sort..."
if command -v cargo-sort &> /dev/null; then
    cargo sort -w
else
    echo "ℹ️  cargo-sort not installed, skipping..."
fi

echo "🔧 Running buf format..."
if command -v buf &> /dev/null; then
    buf format -w
else
    echo "ℹ️  buf not installed, skipping protobuf formatting..."
fi

echo "✅ All formatting completed!" 