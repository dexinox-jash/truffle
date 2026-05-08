#!/bin/bash
# Project Truffle - Cross-Platform Build Script
# Section 6.2.1: Build System
# Reproducible builds for macOS, Windows, Linux

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
DIST_DIR="${PROJECT_ROOT}/dist"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
TARGET="all"
BUILD_TYPE="release"
CLEAN=false
VERBOSE=false
SKIP_FRONTEND=false

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
Truffle Build Script

Usage: $0 [OPTIONS]

Options:
    -t, --target <target>     Build target (all|macos|windows|linux|ios|android)
                              Default: all
    -b, --build-type <type>   Build type (debug|release)
                              Default: release
    -c, --clean               Clean before building
    -v, --verbose             Verbose output
    --skip-frontend           Skip frontend build (use existing)
    -h, --help                Show this help message

Examples:
    $0                        # Build all targets in release mode
    $0 -t macos               # Build macOS only
    $0 -t windows -b debug    # Build Windows in debug mode
    $0 -c -v                  # Clean build with verbose output

EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -t|--target)
                TARGET="$2"
                shift 2
                ;;
            -b|--build-type)
                BUILD_TYPE="$2"
                shift 2
                ;;
            -c|--clean)
                CLEAN=true
                shift
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            --skip-frontend)
                SKIP_FRONTEND=true
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
}

# ============================================================================
# Build Functions
# ============================================================================

clean_build() {
    log_info "Cleaning build directories..."
    rm -rf "${BUILD_DIR}" "${DIST_DIR}"
    rm -rf "${PROJECT_ROOT}/src-tauri/target"
    rm -rf "${PROJECT_ROOT}/dist"
    log_success "Clean complete"
}

setup_environment() {
    log_info "Setting up build environment..."
    
    # Create directories
    mkdir -p "${BUILD_DIR}" "${DIST_DIR}"
    
    # Check Node.js version
    if ! command -v node &> /dev/null; then
        log_error "Node.js is not installed"
        exit 1
    fi
    
    NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
    if [[ "$NODE_VERSION" -lt 18 ]]; then
        log_error "Node.js 18+ required, found $(node --version)"
        exit 1
    fi
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        log_error "Rust is not installed"
        exit 1
    fi
    
    # Check Tauri CLI
    if ! command -v cargo-tauri &> /dev/null; then
        log_info "Installing Tauri CLI..."
        cargo install tauri-cli
    fi
    
    log_success "Environment setup complete"
}

install_dependencies() {
    log_info "Installing dependencies..."
    
    cd "${PROJECT_ROOT}"
    
    if [[ "$VERBOSE" == true ]]; then
        npm ci
    else
        npm ci --silent
    fi
    
    log_success "Dependencies installed"
}

build_frontend() {
    if [[ "$SKIP_FRONTEND" == true ]]; then
        log_info "Skipping frontend build (using existing)"
        return 0
    fi
    
    log_info "Building frontend..."
    
    cd "${PROJECT_ROOT}"
    
    if [[ "$BUILD_TYPE" == "debug" ]]; then
        npm run build:dev
    else
        npm run build
    fi
    
    log_success "Frontend build complete"
}

build_tauri() {
    local target="$1"
    local target_flag=""
    
    log_info "Building Tauri for ${target}..."
    
    cd "${PROJECT_ROOT}"
    
    # Set target flag
    case "$target" in
        macos-arm64)
            target_flag="--target aarch64-apple-darwin"
            ;;
        macos-x64)
            target_flag="--target x86_64-apple-darwin"
            ;;
        windows)
            target_flag="--target x86_64-pc-windows-msvc"
            ;;
        linux)
            target_flag="--target x86_64-unknown-linux-gnu"
            ;;
    esac
    
    # Build flags
    local build_flags=""
    if [[ "$BUILD_TYPE" == "debug" ]]; then
        build_flags="--debug"
    fi
    
    if [[ "$VERBOSE" == true ]]; then
        build_flags="${build_flags} --verbose"
    fi
    
    # Run Tauri build
    # shellcheck disable=SC2086
    cargo tauri build ${target_flag} ${build_flags}
    
    log_success "Tauri build complete for ${target}"
}

copy_artifacts() {
    local target="$1"
    local source_dir=""
    local target_dir="${DIST_DIR}/${target}"
    
    log_info "Copying artifacts for ${target}..."
    
    mkdir -p "$target_dir"
    
    case "$target" in
        macos-arm64)
            source_dir="${PROJECT_ROOT}/src-tauri/target/aarch64-apple-darwin/release/bundle"
            ;;
        macos-x64)
            source_dir="${PROJECT_ROOT}/src-tauri/target/x86_64-apple-darwin/release/bundle"
            ;;
        windows)
            source_dir="${PROJECT_ROOT}/src-tauri/target/x86_64-pc-windows-msvc/release/bundle"
            ;;
        linux)
            source_dir="${PROJECT_ROOT}/src-tauri/target/x86_64-unknown-linux-gnu/release/bundle"
            ;;
        *)
            source_dir="${PROJECT_ROOT}/src-tauri/target/release/bundle"
            ;;
    esac
    
    # Copy all bundle files
    if [[ -d "$source_dir" ]]; then
        find "$source_dir" -type f -exec cp {} "$target_dir/" \;
        log_success "Artifacts copied to ${target_dir}"
    else
        log_warn "Source directory not found: ${source_dir}"
    fi
}

generate_checksums() {
    log_info "Generating checksums..."
    
    cd "${DIST_DIR}"
    
    for dir in */; do
        if [[ -d "$dir" ]]; then
            (
                cd "$dir"
                sha256sum ./* > checksums.sha256 2>/dev/null || true
            )
        fi
    done
    
    log_success "Checksums generated"
}

generate_build_info() {
    log_info "Generating build info..."
    
    local build_info="${DIST_DIR}/build-info.json"
    
    cat > "$build_info" << EOF
{
  "version": "$(jq -r '.version' "${PROJECT_ROOT}/package.json")",
  "buildType": "${BUILD_TYPE}",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "gitCommit": "$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')",
  "gitBranch": "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')",
  "targets": ["${TARGET}"],
  "nodeVersion": "$(node --version)",
  "rustVersion": "$(rustc --version)"
}
EOF
    
    log_success "Build info generated"
}

# ============================================================================
# Platform-Specific Functions
# ============================================================================

build_macos() {
    log_info "Building for macOS..."
    
    if [[ "$(uname)" != "Darwin" ]]; then
        log_warn "Not on macOS, cross-compilation not supported"
        log_warn "Use GitHub Actions for macOS builds"
        return 0
    fi
    
    # Build for both architectures
    build_tauri "macos-arm64"
    build_tauri "macos-x64"
    
    # Create universal binary
    log_info "Creating universal binary..."
    
    local arm64_app="${PROJECT_ROOT}/src-tauri/target/aarch64-apple-darwin/release/bundle/macos/Truffle.app"
    local x64_app="${PROJECT_ROOT}/src-tauri/target/x86_64-apple-darwin/release/bundle/macos/Truffle.app"
    local universal_dir="${PROJECT_ROOT}/src-tauri/target/universal-apple-darwin/release/bundle"
    
    if [[ -d "$arm64_app" && -d "$x64_app" ]]; then
        mkdir -p "$universal_dir"
        
        # Copy ARM64 app as base
        cp -R "$arm64_app" "${universal_dir}/"
        
        # Lipo the binaries
        lipo -create \
            "${arm64_app}/Contents/MacOS/Truffle" \
            "${x64_app}/Contents/MacOS/Truffle" \
            -output "${universal_dir}/Truffle.app/Contents/MacOS/Truffle"
        
        # Create universal DMG
        if command -v create-dmg &> /dev/null; then
            create-dmg \
                --volname "Truffle" \
                --window-pos 200 120 \
                --window-size 800 400 \
                --icon-size 100 \
                --app-drop-link 600 185 \
                "${universal_dir}/Truffle_universal.dmg" \
                "${universal_dir}/Truffle.app" || true
        fi
    fi
    
    copy_artifacts "macos-arm64"
    copy_artifacts "macos-x64"
    
    if [[ -d "$universal_dir" ]]; then
        mkdir -p "${DIST_DIR}/macos-universal"
        cp -R "${universal_dir}/"* "${DIST_DIR}/macos-universal/" 2>/dev/null || true
    fi
}

build_windows() {
    log_info "Building for Windows..."
    
    if [[ "$(uname)" == "Darwin" || "$(uname)" == "Linux" ]]; then
        log_warn "Not on Windows, cross-compilation not supported"
        log_warn "Use GitHub Actions for Windows builds"
        return 0
    fi
    
    build_tauri "windows"
    copy_artifacts "windows"
}

build_linux() {
    log_info "Building for Linux..."
    
    if [[ "$(uname)" != "Linux" ]]; then
        log_warn "Not on Linux, cross-compilation not supported"
        log_warn "Use GitHub Actions for Linux builds"
        return 0
    fi
    
    # Install dependencies if needed
    if command -v apt-get &> /dev/null; then
        log_info "Installing Linux dependencies..."
        sudo apt-get update
        sudo apt-get install -y \
            libgtk-3-dev \
            libwebkit2gtk-4.0-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf \
            fakeroot \
            rpm 2>/dev/null || true
    fi
    
    build_tauri "linux"
    copy_artifacts "linux"
}

# ============================================================================
# Main
# ============================================================================

main() {
    parse_args "$@"
    
    log_info "Starting Truffle build..."
    log_info "Target: ${TARGET}, Build Type: ${BUILD_TYPE}"
    
    # Clean if requested
    if [[ "$CLEAN" == true ]]; then
        clean_build
    fi
    
    # Setup environment
    setup_environment
    
    # Install dependencies
    install_dependencies
    
    # Build frontend
    build_frontend
    
    # Build for target platform(s)
    case "$TARGET" in
        all)
            build_macos
            build_windows
            build_linux
            ;;
        macos)
            build_macos
            ;;
        windows)
            build_windows
            ;;
        linux)
            build_linux
            ;;
        *)
            log_error "Unknown target: ${TARGET}"
            exit 1
            ;;
    esac
    
    # Generate checksums and build info
    generate_checksums
    generate_build_info
    
    log_success "Build complete! Artifacts in: ${DIST_DIR}"
    
    # List artifacts
    if [[ -d "$DIST_DIR" ]]; then
        echo ""
        log_info "Build artifacts:"
        find "$DIST_DIR" -type f -exec ls -lh {} \;
    fi
}

# Run main function
main "$@"
