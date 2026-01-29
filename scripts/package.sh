#!/usr/bin/env bash
#
# package.sh - Organize build artifacts for release
#
# Usage:
#   ./scripts/package.sh <version>
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Detect platform
detect_platform() {
    case "$(uname -s)" in
        Darwin*)  echo "macos" ;;
        Linux*)   echo "linux" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *)        echo "unknown" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x64" ;;
        arm64|aarch64) echo "arm64" ;;
        *)             echo "unknown" ;;
    esac
}

# Package macOS artifacts
package_macos() {
    local version="$1"
    local arch="$2"
    local output_dir="$3"

    local app_path="$PROJECT_ROOT/src-tauri/target/release/bundle/macos/Talent.app"

    if [ ! -d "$app_path" ]; then
        log_warn "macOS .app not found at $app_path"
        return 1
    fi

    local zip_name="Talent-${version}-macos-${arch}.zip"

    log_info "Creating $zip_name..."

    # Create zip with the .app bundle
    cd "$(dirname "$app_path")"
    zip -r -q "$output_dir/$zip_name" "Talent.app"

    log_info "Created $zip_name"
}

# Package Linux artifacts
package_linux() {
    local version="$1"
    local output_dir="$2"

    local bundle_dir="$PROJECT_ROOT/src-tauri/target/release/bundle"

    # Look for AppImage
    local appimage=$(find "$bundle_dir" -name "*.AppImage" 2>/dev/null | head -1)

    if [ -n "$appimage" ] && [ -f "$appimage" ]; then
        local new_name="Talent-${version}-linux-x64.AppImage"
        cp "$appimage" "$output_dir/$new_name"
        chmod +x "$output_dir/$new_name"
        log_info "Created $new_name"
    else
        log_warn "AppImage not found in $bundle_dir"
    fi
}

# Package Windows artifacts
package_windows() {
    local version="$1"
    local output_dir="$2"

    local bundle_dir="$PROJECT_ROOT/src-tauri/target/release/bundle"

    # Look for portable exe (nsis)
    local exe=$(find "$bundle_dir" -name "*.exe" 2>/dev/null | head -1)

    if [ -n "$exe" ] && [ -f "$exe" ]; then
        local new_name="Talent-${version}-windows-x64.exe"
        cp "$exe" "$output_dir/$new_name"
        log_info "Created $new_name"
    else
        log_warn ".exe not found in $bundle_dir"
    fi
}

# Generate checksums
generate_checksums() {
    local output_dir="$1"
    local checksum_file="$output_dir/checksums.txt"

    log_info "Generating checksums..."

    cd "$output_dir"

    # Use shasum on macOS, sha256sum on Linux
    if command -v sha256sum &> /dev/null; then
        sha256sum *.zip *.AppImage *.exe 2>/dev/null > checksums.txt || true
    elif command -v shasum &> /dev/null; then
        shasum -a 256 *.zip *.AppImage *.exe 2>/dev/null > checksums.txt || true
    else
        log_warn "No SHA256 tool found, skipping checksums"
        return
    fi

    if [ -s checksums.txt ]; then
        log_info "Created checksums.txt"
        cat checksums.txt
    fi
}

# Main
main() {
    if [ $# -lt 1 ]; then
        echo "Usage: $0 <version>"
        exit 1
    fi

    local version="$1"
    local platform=$(detect_platform)
    local arch=$(detect_arch)

    log_info "Packaging Talent v$version for $platform ($arch)"

    # Create output directory
    local output_dir="$PROJECT_ROOT/dist/releases/$version"
    mkdir -p "$output_dir"

    # Package based on platform
    case "$platform" in
        macos)
            package_macos "$version" "$arch" "$output_dir"
            ;;
        linux)
            package_linux "$version" "$output_dir"
            ;;
        windows)
            package_windows "$version" "$output_dir"
            ;;
        *)
            log_error "Unknown platform: $platform"
            exit 1
            ;;
    esac

    # Generate checksums
    generate_checksums "$output_dir"

    log_info "Artifacts packaged to: $output_dir"
}

main "$@"
