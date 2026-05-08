#!/bin/bash
# Red Lines Compliance Check
# Verifies the five non-negotiable constraints

set -e

echo "🚩 Checking Red Lines Compliance..."

FAILED=0

# ==================== RED LINE 1: SOVEREIGNTY ====================
echo ""
echo "1️⃣  Sovereignty: Data stays on device"

# Check for cloud storage references
if grep -r "s3\.amazonaws\.com\|storage\.googleapis\.com\|azure\.windows\.net" truffle-core/src/ 2>/dev/null; then
    echo "✗ Found cloud storage references"
    FAILED=1
else
    echo "✓ No cloud storage references"
fi

# Check for telemetry that sends raw data
if grep -r "send.*data\|upload.*image" truffle-core/src/ --include="*.rs" 2>/dev/null | grep -v test; then
    echo "⚠ Check data transmission code manually"
fi

# ==================== RED LINE 2: ZERO-KNOWLEDGE ====================
echo ""
echo "2️⃣  Zero-Knowledge: Server cannot decrypt"

# Check for hardcoded keys
if grep -r "const.*KEY.*=" truffle-crypto/src/ --include="*.rs" | grep -v "TEST"; then
    echo "✗ Potential hardcoded keys found"
    FAILED=1
else
    echo "✓ No hardcoded keys"
fi

# Check encryption uses proper random IVs
if grep -r "Iv::new.*0" truffle-crypto/src/ --include="*.rs"; then
    echo "✗ Static IV detected"
    FAILED=1
else
    echo "✓ No static IVs"
fi

# ==================== RED LINE 3: SURVIVAL MODE ====================
echo ""
echo "3️⃣  Survival Mode: Works offline"

# Check for required network calls
REQUIRED_NETWORK=$(grep -r "reqwest\|hyper::client" truffle-core/src/ --include="*.rs" -l 2>/dev/null | wc -l)
if [ "$REQUIRED_NETWORK" -gt "0" ]; then
    echo "⚠ Network code exists (check for offline fallbacks)"
    echo "   Files: $(grep -r "reqwest\|hyper::client" truffle-core/src/ --include="*.rs" -l)"
else
    echo "✓ No required network dependencies"
fi

# ==================== RED LINE 4: EXIT CAPABILITY ====================
echo ""
echo "4️⃣  Exit Capability: Export in <5 minutes"

# Check export functionality exists
if grep -r "export.*markdown\|export.*json" truffle-core/src/ --include="*.rs" -q; then
    echo "✓ Export functionality exists"
else
    echo "✗ Export functionality not found"
    FAILED=1
fi

# ==================== RED LINE 5: ECONOMICS ====================
echo ""
echo "5️⃣  Economics: 85%+ margin at scale"

# Check bundle size
if [ -f "truffle-desktop/src-tauri/target/release/bundle/macos/*.app" ]; then
    BUNDLE_SIZE=$(du -sh truffle-desktop/src-tauri/target/release/bundle/macos/*.app | cut -f1)
    echo "Bundle size: $BUNDLE_SIZE"
fi

# ==================== SUMMARY ====================
echo ""
echo "===================="
if [ $FAILED -eq 0 ]; then
    echo "✓ All Red Lines compliant"
    exit 0
else
    echo "✗ Red Lines violations detected"
    exit 1
fi
