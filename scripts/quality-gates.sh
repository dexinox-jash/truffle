#!/bin/bash
# Quality Gates Verification Script
# Runs all quality checks for the project

set -e

echo "🔍 Running Truffle Quality Gates..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

FAILED=0

# Function to run check
run_check() {
    local name=$1
    local command=$2
    
    echo ""
    echo "📋 Running: $name"
    if eval $command; then
        echo -e "${GREEN}✓ $name passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $name failed${NC}"
        FAILED=1
        return 1
    fi
}

# ==================== RUST CHECKS ====================

echo ""
echo "🦀 Rust Quality Gates"
echo "====================="

# Format check
run_check "Rust Formatting" "cd truffle-core && cargo fmt --check"
run_check "Crypto Formatting" "cd truffle-crypto && cargo fmt --check"

# Clippy linting (zero warnings)
run_check "Rust Linting (truffle-core)" "cd truffle-core && cargo clippy -- -D warnings"
run_check "Rust Linting (truffle-crypto)" "cd truffle-crypto && cargo clippy -- -D warnings"

# Unit tests
run_check "Rust Unit Tests" "cargo test --workspace"

# ==================== TYPESCRIPT CHECKS ====================

echo ""
echo "📘 TypeScript Quality Gates"
echo "=========================="

# Format check
run_check "TypeScript Formatting" "cd truffle-desktop && prettier --check src/"

# Linting
run_check "TypeScript Linting" "cd truffle-desktop && eslint src/ --ext .ts,.tsx"

# Type checking
run_check "TypeScript Type Check" "cd truffle-desktop && tsc --noEmit"

# Unit tests
run_check "TypeScript Unit Tests" "cd truffle-desktop && vitest run"

# ==================== COVERAGE CHECKS ====================

echo ""
echo "📊 Coverage Quality Gates"
echo "========================="

# Rust coverage (using cargo-tarpaulin)
if command -v cargo-tarpaulin &> /dev/null; then
    run_check "Rust Coverage (truffle-core)" "cd truffle-core && cargo tarpaulin --fail-under 85"
else
    echo -e "${YELLOW}⚠ cargo-tarpaulin not installed, skipping coverage check${NC}"
fi

# ==================== SECURITY CHECKS ====================

echo ""
echo "🔒 Security Quality Gates"
echo "========================="

# Cargo audit
if command -v cargo-audit &> /dev/null; then
    run_check "Rust Security Audit" "cargo audit"
else
    echo -e "${YELLOW}⚠ cargo-audit not installed, skipping${NC}"
fi

# NPM audit
run_check "NPM Security Audit" "cd truffle-desktop && npm audit --audit-level=high"

# Secret scanning
if command -v trufflehog &> /dev/null; then
    run_check "Secret Scanning" "trufflehog filesystem ."
else
    echo -e "${YELLOW}⚠ trufflehog not installed, skipping${NC}"
fi

# ==================== RED LINES CHECK ====================

echo ""
echo "🚩 Red Lines Compliance"
echo "======================="

run_check "Red Lines Check" "./scripts/red-lines-check.sh"

# ==================== BUILD CHECKS ====================

echo ""
echo "🔨 Build Quality Gates"
echo "======================"

# Rust build
run_check "Rust Build" "cargo build --workspace --release"

# Desktop build
run_check "Desktop Build" "cd truffle-desktop && pnpm build"

# ==================== SUMMARY ====================

echo ""
echo "===================="
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All quality gates passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some quality gates failed${NC}"
    exit 1
fi
