#!/bin/bash
# Coverage Check Script
# Verifies test coverage meets thresholds

set -e

COVERAGE_THRESHOLD=85

# Rust coverage
echo "🦀 Checking Rust Coverage..."
cd truffle-core

if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Generate coverage
cargo tarpaulin --out Html --out Json

# Parse coverage from JSON
COVERAGE=$(cat tarpaulin-report.json | grep -o '"coverage":[0-9.]*' | cut -d':' -f2)

echo "Coverage: ${COVERAGE}%"

if (( $(echo "$COVERAGE >= $COVERAGE_THRESHOLD" | bc -l) )); then
    echo "✓ Coverage meets threshold (${COVERAGE_THRESHOLD}%)"
    exit 0
else
    echo "✗ Coverage below threshold (${COVERAGE_THRESHOLD}%)"
    echo "View report: truffle-core/tarpaulin-report.html"
    exit 1
fi
