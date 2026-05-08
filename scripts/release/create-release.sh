#!/bin/bash
# Create a new release with artifacts

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/../.."

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get version from Cargo.toml
VERSION=$(grep '^version' "$ROOT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)
TAG="v${VERSION}"

echo "Creating Release ${TAG}"
echo ""

# Check if tag exists
if ! git rev-parse "$TAG" >/dev/null 2>&1; then
    echo -e "${RED}Error: Tag ${TAG} does not exist${NC}"
    echo "Run: ./scripts/release/bump-version.sh first"
    exit 1
fi

# Build artifacts
echo "Building release artifacts..."
cd "$ROOT_DIR"

# Run builds
if [ "${BUILD_LINUX:-true}" = "true" ]; then
    echo "  - Building Linux..."
    "$ROOT_DIR/docker/build-scripts/build-linux.sh"
fi

# Display release info
echo ""
echo "========================================"
echo "Release: ${TAG}"
echo "========================================"
echo ""
echo "Artifacts:"
find "$ROOT_DIR/artifacts" -type f 2>/dev/null | while read -r file; do
    size=$(du -h "$file" | cut -f1)
    echo "  - $(basename "$file") ($size)"
done

echo ""
echo -e "${GREEN}Release process complete!${NC}"
echo "Artifacts available in: $ROOT_DIR/artifacts/"
