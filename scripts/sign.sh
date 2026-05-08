#!/bin/bash
# Project Truffle - Code Signing Script
# Section 6.2.1: Code Signing (Apple Developer ID, EV Cert)
# Signs binaries for macOS, Windows, and Linux

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

# Signing configuration
APPLE_DEVELOPER_ID="${APPLE_DEVELOPER_ID:-}"
APPLE_TEAM_ID="${APPLE_TEAM_ID:-}"
APPLE_KEYCHAIN="${APPLE_KEYCHAIN:-build.keychain}"
WINDOWS_CERT_THUMBPRINT="${WINDOWS_CERT_THUMBPRINT:-}"

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
Truffle Code Signing Script

Usage: $0 [OPTIONS] <path-to-binary>

Options:
    -p, --platform <platform>   Target platform (macos|windows|linux)
    -t, --type <type>           Binary type (app|dmg|msi|exe|deb|rpm)
    -v, --verbose               Verbose output
    --notarize                  Notarize macOS app (requires Apple ID)
    -h, --help                  Show this help message

Environment Variables:
    APPLE_DEVELOPER_ID          Apple Developer ID for signing
    APPLE_TEAM_ID               Apple Team ID
    APPLE_CERTIFICATE           Base64-encoded Apple certificate
    APPLE_CERTIFICATE_PASSWORD  Apple certificate password
    APPLE_ID                    Apple ID for notarization
    APPLE_PASSWORD              App-specific password for notarization
    WINDOWS_CERTIFICATE         Base64-encoded Windows certificate
    WINDOWS_CERTIFICATE_PASSWORD Windows certificate password
    GPG_PRIVATE_KEY             GPG private key for Linux signing
    GPG_PASSPHRASE              GPG passphrase

Examples:
    $0 -p macos -t app ./Truffle.app
    $0 -p macos -t dmg --notarize ./Truffle.dmg
    $0 -p windows -t msi ./Truffle.msi
    $0 -p linux -t deb ./truffle.deb

EOF
}

# ============================================================================
# macOS Signing Functions
# ============================================================================

setup_macos_keychain() {
    log_info "Setting up macOS keychain..."
    
    local keychain_password="${KEYCHAIN_PASSWORD:-$(openssl rand -base64 32)}"
    
    # Create keychain
    security create-keychain -p "$keychain_password" "$APPLE_KEYCHAIN" 2>/dev/null || true
    security default-keychain -s "$APPLE_KEYCHAIN"
    security unlock-keychain -p "$keychain_password" "$APPLE_KEYCHAIN"
    
    # Import certificate if provided
    if [[ -n "${APPLE_CERTIFICATE:-}" ]]; then
        log_info "Importing Apple certificate..."
        
        echo "$APPLE_CERTIFICATE" | base64 --decode > /tmp/apple_certificate.p12
        
        security import /tmp/apple_certificate.p12 \
            -k "$APPLE_KEYCHAIN" \
            -P "${APPLE_CERTIFICATE_PASSWORD:-}" \
            -T /usr/bin/codesign \
            -T /usr/bin/productsign
        
        security set-key-partition-list \
            -S apple-tool:,apple:,codesign:,productsign: \
            -s -k "$keychain_password" "$APPLE_KEYCHAIN"
        
        rm -f /tmp/apple_certificate.p12
    fi
    
    log_success "Keychain setup complete"
}

sign_macos_app() {
    local app_path="$1"
    
    log_info "Signing macOS app: $app_path"
    
    # Verify app exists
    if [[ ! -d "$app_path" ]]; then
        log_error "App not found: $app_path"
        exit 1
    fi
    
    # Find signing identity
    local signing_identity
    signing_identity=$(security find-identity -v -p codesigning "$APPLE_KEYCHAIN" | grep -o '"Developer ID Application:.*"' | head -1 | tr -d '"')
    
    if [[ -z "$signing_identity" ]]; then
        signing_identity="$APPLE_DEVELOPER_ID"
    fi
    
    if [[ -z "$signing_identity" ]]; then
        log_error "No signing identity found"
        exit 1
    fi
    
    log_info "Using signing identity: $signing_identity"
    
    # Sign the main binary
    codesign --force --options runtime \
        --sign "$signing_identity" \
        --timestamp \
        --entitlements "${PROJECT_ROOT}/src-tauri/entitlements.plist" \
        "$app_path/Contents/MacOS/"* 2>/dev/null || true
    
    # Sign frameworks and libraries
    find "$app_path/Contents/Frameworks" -type f -name "*.dylib" -o -name "*.framework" 2>/dev/null | while read -r file; do
        codesign --force --sign "$signing_identity" --timestamp "$file" 2>/dev/null || true
    done
    
    # Sign the app bundle
    codesign --force --deep --options runtime \
        --sign "$signing_identity" \
        --timestamp \
        --entitlements "${PROJECT_ROOT}/src-tauri/entitlements.plist" \
        "$app_path"
    
    # Verify signature
    codesign --verify --verbose "$app_path"
    
    log_success "macOS app signed successfully"
}

sign_macos_dmg() {
    local dmg_path="$1"
    
    log_info "Signing macOS DMG: $dmg_path"
    
    local signing_identity
    signing_identity=$(security find-identity -v -p codesigning "$APPLE_KEYCHAIN" | grep -o '"Developer ID Application:.*"' | head -1 | tr -d '"')
    
    if [[ -z "$signing_identity" ]]; then
        signing_identity="$APPLE_DEVELOPER_ID"
    fi
    
    codesign --force --sign "$signing_identity" --timestamp "$dmg_path"
    codesign --verify --verbose "$dmg_path"
    
    log_success "macOS DMG signed successfully"
}

notarize_macos() {
    local path="$1"
    local type="$2"
    
    log_info "Notarizing macOS $type: $path"
    
    if [[ -z "${APPLE_ID:-}" || -z "${APPLE_PASSWORD:-}" ]]; then
        log_warn "Apple ID or password not set, skipping notarization"
        return 0
    fi
    
    local team_id="${APPLE_TEAM_ID:-}"
    
    if [[ "$type" == "app" ]]; then
        # Create ZIP for notarization
        local zip_path="${path}.zip"
        ditto -c -k --keepParent "$path" "$zip_path"
        
        # Submit for notarization
        local request_uuid
        request_uuid=$(xcrun notarytool submit "$zip_path" \
            --apple-id "$APPLE_ID" \
            --password "$APPLE_PASSWORD" \
            --team-id "$team_id" \
            --wait \
            --output-format json | jq -r '.id')
        
        log_info "Notarization request: $request_uuid"
        
        # Clean up
        rm -f "$zip_path"
        
        # Staple ticket
        xcrun stapler staple "$path"
        
    else
        # Notarize DMG directly
        xcrun notarytool submit "$path" \
            --apple-id "$APPLE_ID" \
            --password "$APPLE_PASSWORD" \
            --team-id "$team_id" \
            --wait
        
        # Staple ticket
        xcrun stapler staple "$path"
    fi
    
    # Verify notarization
    spctl -a -vv -t install "$path" 2>/dev/null || true
    
    log_success "macOS notarization complete"
}

# ============================================================================
# Windows Signing Functions
# ============================================================================

sign_windows() {
    local file_path="$1"
    local file_type="$2"
    
    log_info "Signing Windows $file_type: $file_path"
    
    # Import certificate if provided
    if [[ -n "${WINDOWS_CERTIFICATE:-}" ]]; then
        log_info "Importing Windows certificate..."
        
        echo "$WINDOWS_CERTIFICATE" | base64 --decode > /tmp/windows_certificate.pfx
        
        # Import to certificate store
        certutil -f -p "${WINDOWS_CERTIFICATE_PASSWORD:-}" -importpfx /tmp/windows_certificate.pfx
        
        rm -f /tmp/windows_certificate.pfx
    fi
    
    # Find certificate thumbprint
    local thumbprint="$WINDOWS_CERTIFICATE_THUMBPRINT"
    
    if [[ -z "$thumbprint" ]]; then
        thumbprint=$(powershell -Command "Get-ChildItem Cert:\\CurrentUser\\My | Where-Object { \$_.Subject -like '*Truffle*' } | Select-Object -First 1 -ExpandProperty Thumbprint")
    fi
    
    if [[ -z "$thumbprint" ]]; then
        log_error "No Windows signing certificate found"
        exit 1
    fi
    
    log_info "Using certificate thumbprint: $thumbprint"
    
    # Sign with signtool
    local signtool_path="C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.22000.0\\x64\\signtool.exe"
    
    if [[ ! -f "$signtool_path" ]]; then
        # Try to find signtool
        signtool_path=$(find "C:\\Program Files (x86)\\Windows Kits" -name "signtool.exe" 2>/dev/null | head -1)
    fi
    
    if [[ -z "$signtool_path" ]]; then
        log_error "signtool.exe not found"
        exit 1
    fi
    
    "$signtool_path" sign \
        /sha1 "$thumbprint" \
        /tr http://timestamp.digicert.com \
        /td sha256 \
        /fd sha256 \
        /a \
        "$file_path"
    
    # Verify signature
    "$signtool_path" verify /pa "$file_path"
    
    log_success "Windows file signed successfully"
}

# ============================================================================
# Linux Signing Functions
# ============================================================================

setup_gpg() {
    log_info "Setting up GPG..."
    
    if [[ -n "${GPG_PRIVATE_KEY:-}" ]]; then
        echo "$GPG_PRIVATE_KEY" | base64 --decode | gpg --batch --import
    fi
    
    log_success "GPG setup complete"
}

sign_linux_deb() {
    local deb_path="$1"
    
    log_info "Signing Debian package: $deb_path"
    
    # Extract control files
    local temp_dir
    temp_dir=$(mktemp -d)
    
    dpkg-deb -R "$deb_path" "$temp_dir/extracted"
    
    # Generate control file hash
    (cd "$temp_dir/extracted" && find DEBIAN -type f -exec md5sum {} \; > DEBIAN/md5sums)
    
    # Sign with GPG if available
    if gpg --list-secret-keys | grep -q "sec"; then
        local gpg_key
        gpg_key=$(gpg --list-secret-keys --with-colons | grep '^sec' | cut -d: -f5 | head -1)
        
        # Create detached signature
        gpg --armor --detach-sign --default-key "$gpg_key" \
            --passphrase "${GPG_PASSPHRASE:-}" \
            --batch --yes \
            -o "$temp_dir/extracted/DEBIAN/control.sig" \
            "$temp_dir/extracted/DEBIAN/control"
    fi
    
    # Repack
    dpkg-deb -b "$temp_dir/extracted" "$deb_path"
    
    # Clean up
    rm -rf "$temp_dir"
    
    log_success "Debian package signed"
}

sign_linux_rpm() {
    local rpm_path="$1"
    
    log_info "Signing RPM package: $rpm_path"
    
    # Import GPG key
    if [[ -n "${GPG_PRIVATE_KEY:-}" ]]; then
        local gpg_key
        gpg_key=$(gpg --list-secret-keys --with-colons | grep '^sec' | cut -d: -f5 | head -1)
        
        # Create RPM macro file
        cat > ~/.rpmmacros << EOF
%_signature gpg
%_gpg_path ~/.gnupg
%_gpg_name $gpg_key
%_gpgbin /usr/bin/gpg
EOF
        
        # Sign RPM
        rpm --addsign "$rpm_path"
    fi
    
    # Verify signature
    rpm --checksig "$rpm_path"
    
    log_success "RPM package signed"
}

sign_linux_appimage() {
    local appimage_path="$1"
    
    log_info "Signing AppImage: $appimage_path"
    
    # AppImage signing requires appimagetool
    if command -v appimagetool &> /dev/null; then
        # Sign with GPG
        if gpg --list-secret-keys | grep -q "sec"; then
            local gpg_key
            gpg_key=$(gpg --list-secret-keys --with-colons | grep '^sec' | cut -d: -f5 | head -1)
            
            gpg --armor --detach-sign --default-key "$gpg_key" \
                --passphrase "${GPG_PASSPHRASE:-}" \
                --batch --yes \
                -o "${appimage_path}.asc" \
                "$appimage_path"
        fi
    fi
    
    log_success "AppImage signed"
}

# ============================================================================
# Main
# ============================================================================

main() {
    local platform=""
    local file_type=""
    local file_path=""
    local notarize=false
    local verbose=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -p|--platform)
                platform="$2"
                shift 2
                ;;
            -t|--type)
                file_type="$2"
                shift 2
                ;;
            --notarize)
                notarize=true
                shift
                ;;
            -v|--verbose)
                verbose=true
                shift
                ;;
            -h|--help)
                print_usage
                exit 0
                ;;
            -*)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
            *)
                file_path="$1"
                shift
                ;;
        esac
    done
    
    # Validate arguments
    if [[ -z "$platform" || -z "$file_path" ]]; then
        log_error "Platform and file path are required"
        print_usage
        exit 1
    fi
    
    if [[ ! -e "$file_path" ]]; then
        log_error "File not found: $file_path"
        exit 1
    fi
    
    log_info "Signing $platform binary: $file_path"
    
    # Platform-specific signing
    case "$platform" in
        macos)
            setup_macos_keychain
            
            if [[ "$file_type" == "app" ]]; then
                sign_macos_app "$file_path"
                if [[ "$notarize" == true ]]; then
                    notarize_macos "$file_path" "app"
                fi
            elif [[ "$file_type" == "dmg" ]]; then
                sign_macos_dmg "$file_path"
                if [[ "$notarize" == true ]]; then
                    notarize_macos "$file_path" "dmg"
                fi
            else
                log_error "Unknown macOS file type: $file_type"
                exit 1
            fi
            ;;
        
        windows)
            sign_windows "$file_path" "${file_type:-binary}"
            ;;
        
        linux)
            setup_gpg
            
            case "${file_type:-}" in
                deb)
                    sign_linux_deb "$file_path"
                    ;;
                rpm)
                    sign_linux_rpm "$file_path"
                    ;;
                AppImage)
                    sign_linux_appimage "$file_path"
                    ;;
                *)
                    log_warn "Unknown Linux file type, skipping signing"
                    ;;
            esac
            ;;
        
        *)
            log_error "Unknown platform: $platform"
            exit 1
            ;;
    esac
    
    log_success "Signing complete!"
}

# Run main function
main "$@"
