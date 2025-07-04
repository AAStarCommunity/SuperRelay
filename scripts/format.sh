#!/bin/bash

# Super-Relay Code Formatting Script
# This script formats all code, checks dependencies, and cleans up trailing whitespace.
# Usage: ./scripts/format.sh

set -e

# Helper function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

echo "🔧 Formatting Rust code..."
cargo +nightly fmt --all

echo "🔍 Running clippy..."
# Check all packages, all features, and treat warnings as errors.
cargo clippy --all-targets --all-features -- -D warnings

echo "📋 Running cargo-sort..."
if ! command_exists cargo-sort; then
    echo "ℹ️ cargo-sort not installed. Attempting to install..."
    cargo install cargo-sort
fi
# Sort the workspace's dependencies.
cargo sort --workspace


echo "🔧 Running buf format..."
if command_exists buf; then
    buf format -w
    buf lint
else
    echo "⚠️ buf not installed, skipping protobuf formatting."
    echo "   Please install it manually. See: https://buf.build/docs/installation"
fi

echo "🧹 Cleaning trailing whitespace and updating git index..."
# Use 'git ls-files' to find all tracked files matching our patterns, then use xargs to run sed on them.
git ls-files -z '*.md' '*.toml' '*.sh' '*.yaml' '*.yml' '*.proto' | xargs -0 sed -i '' 's/[[:space:]]*$//'

# Now, use 'git add -u' to stage all tracked files that have been modified by the sed command.
# This is safer than adding file by file and correctly handles .gitignore.
echo "ℹ️  Staging all formatting changes..."
git add -u

echo "✅ All formatting completed!"