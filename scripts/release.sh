#!/bin/bash
# Project Truffle - Release Automation Script
# Section 6.2.3: Release Strategy
# Canary (5% opt-in) -> Stable (auto-update)

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
RELEASE_TYPE="canary"
VERSION=""
SKIP_BUILD=false
SKIP_TESTS=false
DRY_RUN=false
FORCE=false

# ============================================================================
# Helper Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_usage() {
    cat << EOF
Truffle Release Script

Usage: $0 [OPTIONS]

Options:
    -t, --type <type>       Release type (canary|stable|hotfix)
                            Default: canary
    -v, --version <ver>     Version number (e.g., 1.2.3)
                            Auto-generated if not specified
    --skip-build            Skip building (use existing artifacts)
    --skip-tests            Skip running tests
    --dry-run               Show what would be done without executing
    -f, --force             Force release without confirmation
    -h, --help              Show this help message

Release Types:
    canary      Pre-release for 5% opt-in users
    stable      Production release with auto-update
    hotfix      Emergency patch release

Examples:
    $0 -t canary                          # Create canary release
    $0 -t stable -v 1.2.0                 # Create stable release v1.2.0
    $0 -t hotfix -v 1.2.1 --skip-tests    # Emergency hotfix
    $0 -t stable --dry-run                # Preview stable release

EOF
}

# ============================================================================
# Version Functions
# ============================================================================

get_current_version() {
    jq -r '.version' "${PROJECT_ROOT}/package.json"
}

bump_version() {
    local current="$1"
    local type="$2"
    
    case "$type" in
        canary)
            # Append -canary.N
            local base
            base=$(echo "$current" | sed 's/-canary.*//')
            
            # Find last canary number
            local last_canary
            last_canary=$(git tag -l "canary-${base}-canary.*" 2>/dev/null | sort -V | tail -1 | sed 's/.*-canary.//')
            
            if [[ -z "$last_canary" ]]; then
                last_canary=0
            fi
            
            echo "${base}-canary.$((last_canary + 1))"
            ;;
        
        patch|hotfix)
            # Increment patch version
            echo "$current" | awk -F. '{$NF++;print}' OFS=.
            ;;
        
        minor|stable)
            # Increment minor version, reset patch
            echo "$current" | awk -F. '{$(NF-1)++;$NF=0;print}' OFS=.
            ;;
        
        major)
            # Increment major version, reset minor and patch
            echo "$current" | awk -F. '{$1++;$2=0;$3=0;print}' OFS=.
            ;;
        
        *)
            echo "$current"
            ;;
    esac
}

validate_version() {
    local version="$1"
    
    # Check semver format
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
        log_error "Invalid version format: $version"
        log_error "Expected format: X.Y.Z or X.Y.Z-prerelease.N"
        exit 1
    fi
    
    # Check if tag already exists
    if git tag -l "v$version" | grep -q "v$version"; then
        log_error "Tag v$version already exists"
        exit 1
    fi
}

# ============================================================================
# Build Functions
# ============================================================================

run_tests() {
    if [[ "$SKIP_TESTS" == true ]]; then
        log_warn "Skipping tests"
        return 0
    fi
    
    log_info "Running tests..."
    
    cd "$PROJECT_ROOT"
    
    # TypeScript tests
    npm run test:unit
    
    # Rust tests
    cd src-tauri
    cargo test
    cd ..
    
    log_success "Tests passed"
}

build_release() {
    if [[ "$SKIP_BUILD" == true ]]; then
        log_warn "Skipping build (using existing artifacts)"
        return 0
    fi
    
    log_info "Building release artifacts..."
    
    cd "$PROJECT_ROOT"
    
    # Build frontend
    npm run build
    
    # Build Tauri for all platforms (via GitHub Actions)
    # Local builds are platform-specific
    case "$(uname)" in
        Darwin)
            bash "${SCRIPT_DIR}/build.sh" -t macos -b release
            ;;
        Linux)
            bash "${SCRIPT_DIR}/build.sh" -t linux -b release
            ;;
        MINGW*|MSYS*|CYGWIN*)
            bash "${SCRIPT_DIR}/build.sh" -t windows -b release
            ;;
    esac
    
    log_success "Build complete"
}

# ============================================================================
# Git Functions
# ============================================================================

update_version_files() {
    local version="$1"
    
    log_info "Updating version files to $version..."
    
    # Update package.json
    jq ".version = \"$version\"" "${PROJECT_ROOT}/package.json" > "${PROJECT_ROOT}/package.json.tmp"
    mv "${PROJECT_ROOT}/package.json.tmp" "${PROJECT_ROOT}/package.json"
    
    # Update Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$version\"/" "${PROJECT_ROOT}/src-tauri/Cargo.toml"
    rm -f "${PROJECT_ROOT}/src-tauri/Cargo.toml.bak"
    
    # Update tauri.conf.json
    jq ".version = \"$version\"" "${PROJECT_ROOT}/src-tauri/tauri.conf.json" > "${PROJECT_ROOT}/src-tauri/tauri.conf.json.tmp"
    mv "${PROJECT_ROOT}/src-tauri/tauri.conf.json.tmp" "${PROJECT_ROOT}/src-tauri/tauri.conf.json"
    
    log_success "Version files updated"
}

create_git_tag() {
    local version="$1"
    local release_type="$2"
    
    log_info "Creating git tag..."
    
    # Stage version changes
    git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
    
    # Commit version bump
    git commit -m "chore(release): bump version to $version"
    
    # Create tag
    local tag_name
    if [[ "$release_type" == "canary" ]]; then
        tag_name="canary-$version"
    else
        tag_name="v$version"
    fi
    
    git tag -a "$tag_name" -m "Release $version"
    
    log_success "Git tag created: $tag_name"
    
    echo "$tag_name"
}

push_changes() {
    local tag="$1"
    
    if [[ "$DRY_RUN" == true ]]; then
        log_info "[DRY RUN] Would push changes and tag"
        return 0
    fi
    
    log_info "Pushing changes to remote..."
    
    git push origin main
    git push origin "$tag"
    
    log_success "Changes pushed"
}

# ============================================================================
# GitHub Release Functions
# ============================================================================

generate_changelog() {
    local version="$1"
    local release_type="$2"
    
    log_info "Generating changelog..."
    
    if [[ "$release_type" == "canary" ]]; then
        echo "Canary release for testing. Opt-in only."
        return 0
    fi
    
    # Get commits since last tag
    local last_tag
    last_tag=$(git describe --tags --abbrev=0 HEAD~1 2>/dev/null || echo "")
    
    if [[ -n "$last_tag" ]]; then
        git log "${last_tag}..HEAD" --pretty=format:"- %s (%h)" --no-merges | \
            grep -E "^(feat|fix|docs|style|refactor|test|chore)" || \
            echo "- Various improvements and bug fixes"
    else
        echo "- Initial release"
    fi
}

create_github_release() {
    local version="$1"
    local tag="$2"
    local release_type="$3"
    
    log_info "Creating GitHub release..."
    
    if [[ "$DRY_RUN" == true ]]; then
        log_info "[DRY RUN] Would create GitHub release"
        return 0
    fi
    
    local changelog
    changelog=$(generate_changelog "$version" "$release_type")
    
    local prerelease_flag="false"
    if [[ "$release_type" == "canary" ]]; then
        prerelease_flag="true"
    fi
    
    # Create release body
    local release_body
    release_body=$(cat << EOF
## Truffle ${release_type == "canary" && "Canary" || ""} $version

$changelog

### Installation

**macOS:**
- Download the \`.dmg\` file for your architecture
- Universal binary also available

**Windows:**
- Download the \`.msi\` installer

**Linux:**
- Debian/Ubuntu: Download \`.deb\`
- Fedora/RHEL: Download \`.rpm\`
- Universal: Download \`.AppImage\`

### Verification

Verify checksums:
\`\`\`bash
sha256sum -c checksums.txt
\`\`\`

### Updates

${release_type == "canary" && "This is a canary release. Auto-updates are disabled." || "Auto-updates are enabled."}
EOF
)
    
    # Create release using gh CLI
    if command -v gh &> /dev/null; then
        gh release create "$tag" \
            --title "Truffle ${release_type == "canary" && "Canary" || ""} $version" \
            --notes "$release_body" \
            ${prerelease_flag == "true" && "--prerelease" || ""} \
            ${release_type == "stable" && "--draft" || ""}
    else
        log_warn "GitHub CLI not found, release created via tag push"
    fi
    
    log_success "GitHub release created"
}

# ============================================================================
# Update Server Functions
# ============================================================================

update_update_server() {
    local version="$1"
    local release_type="$2"
    
    log_info "Updating update server..."
    
    if [[ "$DRY_RUN" == true ]]; then
        log_info "[DRY RUN] Would update update server"
        return 0
    fi
    
    local rollout_percentage
    if [[ "$release_type" == "canary" ]]; then
        rollout_percentage=5
    else
        rollout_percentage=100
    fi
    
    # Update via API
    curl -X POST "https://api.truffle.io/v1/releases" \
        -H "Authorization: Bearer ${TRUFFLE_API_TOKEN:-}" \
        -H "Content-Type: application/json" \
        --data "{
            \"version\": \"$version\",
            \"channel\": \"$release_type\",
            \"rollout_percentage\": $rollout_percentage,
            \"mandatory\": false,
            \"platforms\": [\"darwin\", \"windows\", \"linux\"]
        }" 2>/dev/null || log_warn "Update server notification failed"
    
    log_success "Update server configured"
}

# ============================================================================
# Notification Functions
# ============================================================================

notify_slack() {
    local version="$1"
    local release_type="$2"
    local status="$3"
    
    if [[ -z "${SLACK_WEBHOOK_URL:-}" ]]; then
        return 0
    fi
    
    local emoji
    if [[ "$status" == "success" ]]; then
        emoji=":white_check_mark:"
    else
        emoji=":x:"
    fi
    
    local message
    message=$(cat << EOF
{
    "text": "${emoji} Truffle ${release_type} release ${version} ${status}",
    "blocks": [
        {
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": "${emoji} *Truffle ${release_type} release ${version} ${status}*"
            }
        },
        {
            "type": "section",
            "fields": [
                {
                    "type": "mrkdwn",
                    "text": "*Version:*\n${version}"
                },
                {
                    "type": "mrkdwn",
                    "text": "*Type:*\n${release_type}"
                }
            ]
        }
    ]
}
EOF
)
    
    curl -X POST "$SLACK_WEBHOOK_URL" \
        -H "Content-Type: application/json" \
        --data "$message" 2>/dev/null || true
}

# ============================================================================
# Main
# ============================================================================

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -t|--type)
                RELEASE_TYPE="$2"
                shift 2
                ;;
            -v|--version)
                VERSION="$2"
                shift 2
                ;;
            --skip-build)
                SKIP_BUILD=true
                shift
                ;;
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            -f|--force)
                FORCE=true
                shift
                ;;
            -h|--help)
                print_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done
    
    # Validate release type
    if [[ "$RELEASE_TYPE" != "canary" && "$RELEASE_TYPE" != "stable" && "$RELEASE_TYPE" != "hotfix" ]]; then
        log_error "Invalid release type: $RELEASE_TYPE"
        exit 1
    fi
    
    # Determine version
    if [[ -z "$VERSION" ]]; then
        local current
        current=$(get_current_version)
        VERSION=$(bump_version "$current" "$RELEASE_TYPE")
    fi
    
    validate_version "$VERSION"
    
    log_info "Preparing $RELEASE_TYPE release: $VERSION"
    
    if [[ "$DRY_RUN" == true ]]; then
        log_info "[DRY RUN] Would release version $VERSION"
    fi
    
    # Confirm production release
    if [[ "$RELEASE_TYPE" == "stable" && "$FORCE" != true && "$DRY_RUN" != true ]]; then
        echo ""
        log_warn "You are about to create a STABLE release!"
        log_warn "Version: $VERSION"
        read -p "Are you sure? Type 'yes' to continue: " confirm
        
        if [[ "$confirm" != "yes" ]]; then
            log_info "Release cancelled"
            exit 0
        fi
    fi
    
    # Run tests
    run_tests
    
    # Update version files
    update_version_files "$VERSION"
    
    # Build release
    build_release
    
    # Create git tag
    local tag
    tag=$(create_git_tag "$VERSION" "$RELEASE_TYPE")
    
    # Push changes
    push_changes "$tag"
    
    # Create GitHub release
    create_github_release "$VERSION" "$tag" "$RELEASE_TYPE"
    
    # Update update server
    update_update_server "$VERSION" "$RELEASE_TYPE"
    
    # Notify
    notify_slack "$VERSION" "$RELEASE_TYPE" "success"
    
    log_success "Release $VERSION complete!"
    
    if [[ "$RELEASE_TYPE" == "canary" ]]; then
        log_info "This is a canary release. Monitor for issues before promoting to stable."
    fi
}

# Run main function
main "$@"
