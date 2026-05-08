#!/bin/bash
# Version bumping script
# Supports: major, minor, patch, prerelease

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/../.."

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Parse arguments
BUMP_TYPE=${1:-patch}
NEW_VERSION=${2:-}

# Get current version
CURRENT_VERSION=$(grep '^version' "$ROOT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)

echo "Current version: $CURRENT_VERSION"

# Calculate new version
if [ -z "$NEW_VERSION" ]; then
    # Parse current version
    MAJOR=$(echo "$CURRENT_VERSION" | cut -d. -f1)
    MINOR=$(echo "$CURRENT_VERSION" | cut -d. -f2)
    PATCH=$(echo "$CURRENT_VERSION" | cut -d. -f3 | cut -d- -f1)
    PRERELEASE=$(echo "$CURRENT_VERSION" | grep -oP '(?<=-).+' || echo "")

    case "$BUMP_TYPE" in
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            PRERELEASE=""
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            PRERELEASE=""
            ;;
        patch)
            PATCH=$((PATCH + 1))
            PRERELEASE=""
            ;;
        prerelease)
            if [ -z "$PRERELEASE" ]; then
                PATCH=$((PATCH + 1))
                PRERELEASE="beta.1"
            else
                PRERELEASE_NUM=$(echo "$PRERELEASE" | grep -oP '\d+$')
                PRERELEASE_BASE=$(echo "$PRERELEASE" | grep -oP '^\D+')
                PRERELEASE_NUM=$((PRERELEASE_NUM + 1))
                PRERELEASE="${PRERELEASE_BASE}${PRERELEASE_NUM}"
            fi
            ;;
        *)
            echo -e "${RED}Error: Invalid bump type. Use: major, minor, patch, prerelease${NC}"
            exit 1
            ;;
    esac

    NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
    [ -n "$PRERELEASE" ] && NEW_VERSION="${NEW_VERSION}-${PRERELEASE}"
fi

echo "New version: $NEW_VERSION"

# Confirm
read -p "Proceed with version bump? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled"
    exit 0
fi

# Update Cargo.toml files
echo "Updating Cargo.toml files..."
sed -i "s/^version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" "$ROOT_DIR/Cargo.toml"

# Update package.json
echo "Updating package.json..."
if [ -f "$ROOT_DIR/truffle-desktop/package.json" ]; then
    sed -i "s/\"version\": \"${CURRENT_VERSION}\"/\"version\": \"${NEW_VERSION}\"/" "$ROOT_DIR/truffle-desktop/package.json"
fi

# Update tauri.conf.json
echo "Updating tauri.conf.json..."
if [ -f "$ROOT_DIR/truffle-desktop/src-tauri/tauri.conf.json" ]; then
    sed -i "s/\"version\": \"${CURRENT_VERSION}\"/\"version\": \"${NEW_VERSION}\"/" "$ROOT_DIR/truffle-desktop/src-tauri/tauri.conf.json"
fi

# Create git commit and tag
echo "Creating git commit..."
git add -A
git commit -m "chore(release): bump version to ${NEW_VERSION}"

# Create tag
echo "Creating git tag..."
git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"

echo ""
echo -e "${GREEN}Version bumped to ${NEW_VERSION}${NC}"
echo ""
echo "Next steps:"
echo "  1. Push commit: git push origin main"
echo "  2. Push tags: git push origin v${NEW_VERSION}"
