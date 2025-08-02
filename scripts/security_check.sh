#!/bin/bash
# Security Check Script for SuperRelay
# Checks for common security issues and provides recommendations

set -e

echo "🔒 SuperRelay Security Check"
echo "=========================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
CRITICAL_COUNT=0
HIGH_COUNT=0
MEDIUM_COUNT=0

# Function to report issues
report_issue() {
    local severity=$1
    local message=$2
    local recommendation=$3

    case $severity in
        "CRITICAL")
            echo -e "${RED}🚨 CRITICAL: $message${NC}"
            echo -e "${YELLOW}   Recommendation: $recommendation${NC}"
            ((CRITICAL_COUNT++))
            ;;
        "HIGH")
            echo -e "${RED}⚠️  HIGH: $message${NC}"
            echo -e "${YELLOW}   Recommendation: $recommendation${NC}"
            ((HIGH_COUNT++))
            ;;
        "MEDIUM")
            echo -e "${YELLOW}⚠️  MEDIUM: $message${NC}"
            echo -e "   Recommendation: $recommendation"
            ((MEDIUM_COUNT++))
            ;;
        "INFO")
            echo -e "${BLUE}ℹ️  INFO: $message${NC}"
            ;;
    esac
    echo
}

report_ok() {
    echo -e "${GREEN}✅ $1${NC}"
}

echo "🔍 Checking for hardcoded private keys..."

# Check for hardcoded private keys in production source files (exclude tests, docs, contracts, node_modules and target)
HARDCODED_KEYS=$(find . -name "*.rs" -o -name "*.js" -o -name "*.ts" | grep -v node_modules | grep -v target | grep -v test | grep -v contracts | grep -v demo | grep -v docs | grep -v api_docs | grep -v schemas | xargs grep -l "0x[a-fA-F0-9]\{64\}" 2>/dev/null | xargs grep -L "bytecode\|validation_data\|test_parse\|contract\|sol!\|PrivateKeySigner::from_str\|// Test private key\|test.*private_key" 2>/dev/null | head -5)

if [ ! -z "$HARDCODED_KEYS" ]; then
    report_issue "CRITICAL" "Hardcoded private keys found in source files" "Remove all hardcoded private keys and use environment variables or secure key management"
    echo "   Files containing potential private keys:"
    echo "$HARDCODED_KEYS" | sed 's/^/     /'
else
    report_ok "No hardcoded private keys found in Rust source files"
fi

echo "🔍 Checking for sensitive information in configuration..."

# Check for sensitive info in config files (exclude token bucket and similar technical terms)
if grep -r "password\|secret\|private_key.*=" config/ 2>/dev/null | grep -v "example\|template\|YOUR_\|\${.*}\|token.*bucket\|token.*capacity" | grep -q .; then
    report_issue "HIGH" "Sensitive information found in configuration files" "Use environment variables or encrypted configuration"
else
    report_ok "No hardcoded sensitive information in configuration files"
fi

echo "🔍 Checking environment variable usage..."

# Check if .env exists and has proper security warnings
if [ -f ".env" ]; then
    if ! grep -q "SECURITY" .env; then
        report_issue "MEDIUM" ".env file exists without security warnings" "Add security warnings to .env file"
    else
        report_ok ".env file has security warnings"
    fi
else
    report_ok "No .env file found in repository root"
fi

# Check for .env.example
if [ ! -f ".env.example" ]; then
    report_issue "MEDIUM" "No .env.example file found" "Create .env.example with secure defaults"
else
    report_ok ".env.example file exists"
fi

echo "🔍 Checking .gitignore for sensitive files..."

# Check if sensitive files are ignored
SENSITIVE_PATTERNS=(".env" "*.key" "*.pem" "*.p12" "*.keystore")
MISSING_PATTERNS=()

for pattern in "${SENSITIVE_PATTERNS[@]}"; do
    if ! grep -q "^$pattern$" .gitignore 2>/dev/null; then
        MISSING_PATTERNS+=("$pattern")
    fi
done

if [ ${#MISSING_PATTERNS[@]} -gt 0 ]; then
    report_issue "MEDIUM" "Some sensitive file patterns not in .gitignore" "Add patterns: ${MISSING_PATTERNS[*]}"
else
    report_ok "Sensitive file patterns are properly ignored"
fi

echo "🔍 Checking for unsafe Rust code..."

# Check for unsafe blocks
UNSAFE_COUNT=$(find crates/ -name "*.rs" | xargs grep -c "unsafe" 2>/dev/null | awk -F: '{sum += $2} END {print sum+0}')

if [ "$UNSAFE_COUNT" -gt 0 ]; then
    report_issue "MEDIUM" "Found $UNSAFE_COUNT unsafe blocks in Rust code" "Review all unsafe blocks for memory safety"
else
    report_ok "No unsafe Rust code found"
fi

echo "🔍 Checking dependency security..."

# Check for known vulnerabilities (if cargo-audit is available)
if command -v cargo-audit >/dev/null 2>&1; then
    echo "Running cargo audit..."
    if ! cargo audit --quiet; then
        report_issue "HIGH" "Vulnerable dependencies found" "Run 'cargo audit' and update vulnerable dependencies"
    else
        report_ok "No known vulnerable dependencies"
    fi
else
    report_issue "INFO" "cargo-audit not installed" "Install with 'cargo install cargo-audit' for dependency vulnerability scanning"
fi

echo "🔍 Checking for debug/test code in production..."

# Check for debug prints and test code
DEBUG_PATTERNS=("println!" "dbg!" "debug!" "todo!" "unimplemented!")
for pattern in "${DEBUG_PATTERNS[@]}"; do
    COUNT=$(find crates/ -name "*.rs" | xargs grep -c "$pattern" 2>/dev/null | awk -F: '{sum += $2} END {print sum+0}')
    if [ "$COUNT" -gt 5 ]; then  # Allow some debug code
        report_issue "MEDIUM" "High usage of $pattern ($COUNT occurrences)" "Review and remove debug code from production builds"
    fi
done

echo "🔍 Checking HTTP security headers..."

# This would need to be implemented in the actual server code
report_issue "INFO" "HTTP security headers check" "Ensure CORS, CSP, and other security headers are properly configured"

echo "🔍 Checking for proper error handling..."

# Check for unwrap() usage which can cause panics
UNWRAP_COUNT=$(find crates/ -name "*.rs" | xargs grep -c "\.unwrap()" 2>/dev/null | awk -F: '{sum += $2} END {print sum+0}')

if [ "$UNWRAP_COUNT" -gt 10 ]; then
    report_issue "MEDIUM" "High usage of .unwrap() ($UNWRAP_COUNT occurrences)" "Replace .unwrap() with proper error handling"
else
    report_ok "Reasonable usage of .unwrap()"
fi

echo "🔍 Checking file permissions..."

# Check for overly permissive files
PERMISSIVE_FILES=$(find . -type f -perm -o+w -not -path "./target/*" 2>/dev/null | head -5)
if [ ! -z "$PERMISSIVE_FILES" ]; then
    report_issue "MEDIUM" "World-writable files found" "Remove write permissions for others"
else
    report_ok "No world-writable files found"
fi

echo "=========================="
echo "🔒 Security Check Summary"
echo "=========================="

if [ $CRITICAL_COUNT -gt 0 ]; then
    echo -e "${RED}🚨 CRITICAL ISSUES: $CRITICAL_COUNT${NC}"
fi

if [ $HIGH_COUNT -gt 0 ]; then
    echo -e "${RED}⚠️  HIGH ISSUES: $HIGH_COUNT${NC}"
fi

if [ $MEDIUM_COUNT -gt 0 ]; then
    echo -e "${YELLOW}⚠️  MEDIUM ISSUES: $MEDIUM_COUNT${NC}"
fi

TOTAL_ISSUES=$((CRITICAL_COUNT + HIGH_COUNT + MEDIUM_COUNT))

if [ $TOTAL_ISSUES -eq 0 ]; then
    echo -e "${GREEN}✅ No security issues found!${NC}"
    exit 0
elif [ $CRITICAL_COUNT -gt 0 ]; then
    echo -e "${RED}❌ CRITICAL security issues found! Please fix immediately.${NC}"
    exit 2
elif [ $HIGH_COUNT -gt 0 ]; then
    echo -e "${YELLOW}⚠️  High priority security issues found. Please fix soon.${NC}"
    exit 1
else
    echo -e "${YELLOW}⚠️  Some security improvements recommended.${NC}"
    exit 0
fi