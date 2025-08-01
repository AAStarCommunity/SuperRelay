#!/bin/bash

# Super-Relay Code Formatting Script
# This script intelligently formats code, dependencies, and cleans up trailing whitespace,
# while respecting and ignoring git submodules.
# Usage: ./scripts/format.sh

set -e

# Helper function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# --- Check for dependencies ---
if ! command_exists jq; then
    echo "⚠️ jq is not installed. This script requires jq to parse workspace members."
    echo "   Please install it (e.g., 'brew install jq' or 'sudo apt-get install jq')."
    exit 1
fi

# --- Submodule Exclusion Logic ---
echo "ℹ️  Identifying submodules to exclude..."
# Get a list of submodule paths, formatted for use in grep.
SUBMODULE_PATHS_PATTERN=$(git submodule status | awk '{print $2}' | paste -sd '|' -)
if [ -z "$SUBMODULE_PATHS_PATTERN" ]; then
    echo "✅ No submodules found."
else
    echo "   Excluding paths matching: $SUBMODULE_PATHS_PATTERN"
fi

# --- Rust Formatting (Submodule Aware) ---
echo "🔧 Formatting Rust code for workspace members..."
# Use cargo metadata and jq to get a clean list of manifest paths for all packages.
# This is the most reliable way to identify and iterate over them.
for manifest_path in $(cargo metadata --no-deps --format-version=1 | jq -r '.packages[].manifest_path'); do
    pkg_path=$(dirname "$manifest_path")

    # Check if the package path is inside a submodule path
    if [ ! -z "$SUBMODULE_PATHS_PATTERN" ] && echo "$pkg_path" | grep -qE "^($SUBMODULE_PATHS_PATTERN)"; then
        echo "   Skipping submodule member: $pkg_path"
        continue
    fi

    echo "   Formatting $pkg_path..."
    cargo +nightly fmt --manifest-path "$manifest_path"

    echo "   Checking $pkg_path..."
    cargo clippy --manifest-path "$manifest_path" -- -D warnings
done


# --- Dependency and Protobuf Formatting ---
echo "📋 Running cargo-sort..."
if ! command_exists cargo-sort; then
    echo "ℹ️ cargo-sort not installed. Attempting to install..."
    cargo install cargo-sort
fi
cargo sort --workspace

echo "🔧 Running buf format..."
if command_exists buf; then
    buf format -w
    buf lint
else
    echo "⚠️ buf not installed, skipping protobuf formatting."
    echo "   Please install it manually. See: https://buf.build/docs/installation"
fi

# --- Whitespace Cleanup (Submodule Aware) ---
echo "🧹 Cleaning trailing whitespace and updating git index..."
# List all tracked files matching patterns (null-terminated).
# Filter out files within submodules using grep with -z flag for null-terminated lines.
# Pipe to xargs (reading null-terminated) to run sed.
if [ -z "$SUBMODULE_PATHS_PATTERN" ]; then
    git ls-files -z '*.md' '*.toml' '*.sh' '*.yaml' '*.yml' '*.proto' | xargs -0 sed -i '' 's/[[:space:]]*$//'
else
    git ls-files -z '*.md' '*.toml' '*.sh' '*.yaml' '*.yml' '*.proto' | grep -z -vE "^($SUBMODULE_PATHS_PATTERN)" | xargs -0 sed -i '' 's/[[:space:]]*$//'
fi

echo "ℹ️  Staging all formatting changes..."
git add -u

echo "✅ All formatting completed!"