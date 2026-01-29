#!/usr/bin/env bash
#
# build.sh - Build Talent for the current platform
#
# Usage:
#   ./scripts/build.sh           # Build release
#   ./scripts/build.sh --debug   # Build debug
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

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

# Check if a command exists
require_cmd() {
    if ! command -v "$1" &> /dev/null; then
        log_error "Required command not found: $1"
        exit 1
    fi
}

# Validate dependencies
check_dependencies() {
    log_info "Checking dependencies..."

    require_cmd "node"
    require_cmd "npm"
    require_cmd "cargo"
    require_cmd "rustc"

    # Check Node version (need 18+)
    NODE_VERSION=$(node -v | sed 's/v//' | cut -d. -f1)
    if [ "$NODE_VERSION" -lt 18 ]; then
        log_error "Node.js 18+ required, found v$NODE_VERSION"
        exit 1
    fi

    log_info "All dependencies satisfied"
}

# Install npm dependencies if needed
install_deps() {
    cd "$PROJECT_ROOT"

    if [ ! -d "node_modules" ]; then
        log_info "Installing npm dependencies..."
        npm install
    else
        log_info "npm dependencies already installed"
    fi
}

# Build the application
build_app() {
    local build_type="${1:-release}"

    cd "$PROJECT_ROOT"

    log_info "Building Talent ($build_type)..."

    if [ "$build_type" = "debug" ]; then
        npm run tauri build -- --debug
    else
        npm run tauri build
    fi

    log_info "Build complete!"
}

# Main
main() {
    local build_type="release"

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --debug)
                build_type="debug"
                shift
                ;;
            -h|--help)
                echo "Usage: $0 [--debug]"
                echo ""
                echo "Options:"
                echo "  --debug    Build debug version"
                echo "  -h, --help Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    PLATFORM=$(detect_platform)
    ARCH=$(detect_arch)

    log_info "Platform: $PLATFORM ($ARCH)"

    if [ "$PLATFORM" = "unknown" ]; then
        log_error "Unsupported platform"
        exit 1
    fi

    check_dependencies
    install_deps
    build_app "$build_type"

    # Get version from package.json
    VERSION=$(node -p "require('./package.json').version")

    # Run package script
    log_info "Packaging artifacts..."
    "$SCRIPT_DIR/package.sh" "$VERSION"

    log_info "Done! Artifacts in dist/releases/$VERSION/"
}

main "$@"
