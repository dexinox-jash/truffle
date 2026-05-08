#!/bin/bash
# Performance Benchmarks
# Measures key performance metrics

set -e

echo "⚡ Running Performance Benchmarks..."

# Rust benchmarks
echo ""
echo "🦀 Rust Benchmarks"
echo "=================="

cd truffle-core
cargo bench --bench graph_benchmarks

# Frontend benchmarks
echo ""
echo "📘 Frontend Benchmarks"
echo "====================="

cd ../truffle-desktop
# Lighthouse CI or similar could go here

# E2E timing
echo ""
echo "⏱️  E2E Test Timing"
echo "=================="

# Run E2E tests with timing
if command -v playwright &> /dev/null; then
    npx playwright test --reporter=line --workers=1 2>&1 | tee e2e-timing.log
fi

echo ""
echo "Benchmarks complete. Results saved."
