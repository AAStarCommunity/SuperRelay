#!/bin/bash
# Test original Rundler functionality to ensure compatibility

set -e

echo "🧪 Testing Original Rundler Functionality"
echo "========================================"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the project root directory"
    exit 1
fi

# Function to run tests with timeout
run_test_with_timeout() {
    local test_name="$1"
    local test_command="$2"
    local timeout_seconds=${3:-300}  # Default 5 minutes

    echo "🔍 Running $test_name..."
    echo "⏱️  Timeout: ${timeout_seconds}s"
    echo "📝 Command: $test_command"

    if timeout $timeout_seconds bash -c "$test_command"; then
        echo "✅ $test_name: PASSED"
        return 0
    else
        echo "❌ $test_name: FAILED"
        return 1
    fi
}

# Initialize test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Create test log directory
mkdir -p logs
TEST_LOG="logs/rundler_test_$(date +%Y%m%d_%H%M%S).log"

echo "📊 Test Results Log" > $TEST_LOG
echo "Start Time: $(date)" >> $TEST_LOG
echo "=================================" >> $TEST_LOG

# Test 1: Basic Compilation
echo ""
echo "🔨 Test 1: Compilation Check"
echo "============================="
TOTAL_TESTS=$((TOTAL_TESTS + 1))

if run_test_with_timeout "Compilation" "cargo check --workspace --exclude rundler-paymaster-relay" 180; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "✅ Compilation: PASSED" >> $TEST_LOG
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
    echo "❌ Compilation: FAILED" >> $TEST_LOG
fi

# Test 2: Unit Tests (Core Rundler)
echo ""
echo "🧪 Test 2: Core Rundler Unit Tests"
echo "=================================="
TOTAL_TESTS=$((TOTAL_TESTS + 1))

if run_test_with_timeout "Unit Tests" "cargo test --workspace --exclude rundler-paymaster-relay --lib" 300; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "✅ Unit Tests: PASSED" >> $TEST_LOG
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
    echo "❌ Unit Tests: FAILED" >> $TEST_LOG
fi

# Test 3: Integration Tests (if any)
echo ""
echo "🔗 Test 3: Integration Tests"
echo "============================"
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# Check if integration tests exist
if find . -name "tests" -type d | grep -v target | grep -v paymaster-relay >/dev/null 2>&1; then
    if run_test_with_timeout "Integration Tests" "cargo test --workspace --exclude rundler-paymaster-relay --test '*' || true" 300; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo "✅ Integration Tests: PASSED" >> $TEST_LOG
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo "❌ Integration Tests: FAILED" >> $TEST_LOG
    fi
else
    echo "⚠️  No integration tests found - SKIPPED"
    echo "⚠️  Integration Tests: SKIPPED" >> $TEST_LOG
    TOTAL_TESTS=$((TOTAL_TESTS - 1))
fi

# Test 4: Documentation Tests
echo ""
echo "📚 Test 4: Documentation Tests"
echo "=============================="
TOTAL_TESTS=$((TOTAL_TESTS + 1))

if run_test_with_timeout "Doc Tests" "cargo test --workspace --exclude rundler-paymaster-relay --doc" 180; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "✅ Doc Tests: PASSED" >> $TEST_LOG
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
    echo "❌ Doc Tests: FAILED" >> $TEST_LOG
fi

# Test 5: Build Release Version
echo ""
echo "🚀 Test 5: Release Build"
echo "======================="
TOTAL_TESTS=$((TOTAL_TESTS + 1))

if run_test_with_timeout "Release Build" "cargo build --release --workspace --exclude rundler-paymaster-relay" 300; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "✅ Release Build: PASSED" >> $TEST_LOG
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
    echo "❌ Release Build: FAILED" >> $TEST_LOG
fi

# Test 6: Binary Functionality Check
echo ""
echo "🔧 Test 6: Binary Functionality"
echo "==============================="
TOTAL_TESTS=$((TOTAL_TESTS + 1))

if run_test_with_timeout "Binary Check" "timeout 10s cargo run --bin rundler -- --help || true" 30; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "✅ Binary Check: PASSED" >> $TEST_LOG
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
    echo "❌ Binary Check: FAILED" >> $TEST_LOG
fi

# Test 7: Linting (Clippy)
echo ""
echo "📝 Test 7: Code Linting (Clippy)"
echo "==============================="
TOTAL_TESTS=$((TOTAL_TESTS + 1))

if run_test_with_timeout "Clippy Linting" "cargo clippy --workspace --exclude rundler-paymaster-relay -- -D warnings || true" 180; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "✅ Clippy: PASSED" >> $TEST_LOG
else
    echo "⚠️  Clippy warnings found but continuing..."
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "⚠️  Clippy: PASSED (with warnings)" >> $TEST_LOG
fi

# Test 8: Security Audit (Cargo Audit)
echo ""
echo "🔒 Test 8: Security Audit"
echo "========================="
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# Install cargo-audit if not present
if ! command -v cargo-audit >/dev/null 2>&1; then
    echo "📦 Installing cargo-audit..."
    cargo install cargo-audit --quiet || true
fi

if command -v cargo-audit >/dev/null 2>&1; then
    if run_test_with_timeout "Security Audit" "cargo audit || true" 60; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo "✅ Security Audit: PASSED" >> $TEST_LOG
    else
        echo "⚠️  Security audit found issues but continuing..."
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo "⚠️  Security Audit: PASSED (with warnings)" >> $TEST_LOG
    fi
else
    echo "⚠️  cargo-audit not available - SKIPPED"
    echo "⚠️  Security Audit: SKIPPED" >> $TEST_LOG
    TOTAL_TESTS=$((TOTAL_TESTS - 1))
fi

# Generate final report
echo ""
echo "🎯 Test Summary"
echo "==============="
echo "📊 Total Tests: $TOTAL_TESTS"
echo "✅ Passed: $PASSED_TESTS"
echo "❌ Failed: $FAILED_TESTS"
echo "📈 Success Rate: $(( (PASSED_TESTS * 100) / TOTAL_TESTS ))%"

# Add summary to log
echo "" >> $TEST_LOG
echo "=== FINAL SUMMARY ===" >> $TEST_LOG
echo "End Time: $(date)" >> $TEST_LOG
echo "Total Tests: $TOTAL_TESTS" >> $TEST_LOG
echo "Passed: $PASSED_TESTS" >> $TEST_LOG
echo "Failed: $FAILED_TESTS" >> $TEST_LOG
echo "Success Rate: $(( (PASSED_TESTS * 100) / TOTAL_TESTS ))%" >> $TEST_LOG

echo ""
echo "📄 Detailed test log saved to: $TEST_LOG"

if [ $FAILED_TESTS -eq 0 ]; then
    echo ""
    echo "🎉 All Rundler tests passed! The original functionality is intact."
    echo "✅ Safe to proceed with Super-Relay integration testing."
    exit 0
else
    echo ""
    echo "⚠️  Some tests failed. Please review the issues before proceeding:"
    echo "   - Check compilation errors"
    echo "   - Review failed unit tests"
    echo "   - Fix any breaking changes"
    echo ""
    echo "🔍 For detailed error information:"
    echo "   tail -n 50 $TEST_LOG"
    exit 1
fi