#!/bin/bash
set -e

echo "Running Ruflo CI Test Suite..."

cd ruflo-command

# Format check
echo "Checking code formatting..."
cargo fmt -- --check

# Clippy lints
echo "Running clippy..."
cargo clippy -- -D warnings

# Unit tests
echo "Running unit tests..."
cargo test --lib --verbose

# Integration tests (if database is available)
if [ -n "$DATABASE_URL" ]; then
    echo "Running integration tests..."
    cargo test --test integration_tests --verbose
fi

echo "All tests passed!"
