#!/bin/bash
# OWASP Dependency Check for Rust

set -e

REPORT_DIR="security-reports"
mkdir -p $REPORT_DIR

echo "Running OWASP Dependency Check..."

# Check each service
for dir in ruflo-*; do
    if [ -f "$dir/Cargo.toml" ]; then
        echo "Checking $dir..."
        
        # Run cargo audit
        cd $dir
        cargo audit --json > ../$REPORT_DIR/${dir}-audit.json 2>/dev/null || true
        cd ..
    fi
done

# Run cargo-deny for license and security checks
if command -v cargo-deny &> /dev/null; then
    cargo deny check --format json > $REPORT_DIR/cargo-deny-report.json 2>/dev/null || true
fi

echo "Dependency check complete. Reports in $REPORT_DIR/"
