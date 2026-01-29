#!/usr/bin/env bash
#
# bump-version.sh - Update version across all project files
#
# Usage:
#   ./scripts/bump-version.sh 0.2.0           # Update version
#   ./scripts/bump-version.sh 0.2.0 --tag     # Update and create git tag
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

# Validate semantic version format
validate_version() {
    local version="$1"
    # Matches: 1.0.0, 1.0.0-beta.1, 1.0.0-rc.1, etc.
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
        log_error "Invalid version format: $version"
        log_error "Expected format: X.Y.Z or X.Y.Z-prerelease"
        exit 1
    fi
}

# Update version in package.json
update_package_json() {
    local version="$1"
    local file="$PROJECT_ROOT/package.json"

    if [ ! -f "$file" ]; then
        log_error "package.json not found"
        exit 1
    fi

    # Use node for reliable JSON manipulation
    node -e "
        const fs = require('fs');
        const pkg = JSON.parse(fs.readFileSync('$file', 'utf8'));
        pkg.version = '$version';
        fs.writeFileSync('$file', JSON.stringify(pkg, null, 2) + '\n');
    "

    log_info "Updated package.json"
}

# Update version in Cargo.toml
update_cargo_toml() {
    local version="$1"
    local file="$PROJECT_ROOT/src-tauri/Cargo.toml"

    if [ ! -f "$file" ]; then
        log_error "src-tauri/Cargo.toml not found"
        exit 1
    fi

    # Use sed to update version line
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$version\"/" "$file"
    else
        sed -i "s/^version = \".*\"/version = \"$version\"/" "$file"
    fi

    log_info "Updated src-tauri/Cargo.toml"
}

# Update version in tauri.conf.json
update_tauri_conf() {
    local version="$1"
    local file="$PROJECT_ROOT/src-tauri/tauri.conf.json"

    if [ ! -f "$file" ]; then
        log_error "src-tauri/tauri.conf.json not found"
        exit 1
    fi

    # Use node for reliable JSON manipulation
    node -e "
        const fs = require('fs');
        const conf = JSON.parse(fs.readFileSync('$file', 'utf8'));
        conf.version = '$version';
        fs.writeFileSync('$file', JSON.stringify(conf, null, 2) + '\n');
    "

    log_info "Updated src-tauri/tauri.conf.json"
}

# Create git tag
create_tag() {
    local version="$1"
    local tag="v$version"

    # Check for uncommitted changes
    if ! git diff --quiet || ! git diff --staged --quiet; then
        log_warn "You have uncommitted changes. Commit them first."
        log_info "Staging version changes..."
        git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
        git commit -m "Bump version to $version"
    fi

    # Check if tag already exists
    if git rev-parse "$tag" >/dev/null 2>&1; then
        log_error "Tag $tag already exists"
        exit 1
    fi

    git tag -a "$tag" -m "Release $version"
    log_info "Created tag: $tag"
    log_info "Push with: git push origin main --tags"
}

# Show usage
usage() {
    echo "Usage: $0 <version> [--tag]"
    echo ""
    echo "Arguments:"
    echo "  version    New version (e.g., 0.2.0 or 0.2.0-beta.1)"
    echo ""
    echo "Options:"
    echo "  --tag      Create git tag after updating version"
    echo "  -h, --help Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 0.2.0"
    echo "  $0 0.2.0-beta.1 --tag"
}

# Main
main() {
    local version=""
    local create_git_tag=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --tag)
                create_git_tag=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            -*)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
            *)
                if [ -z "$version" ]; then
                    version="$1"
                else
                    log_error "Multiple versions specified"
                    exit 1
                fi
                shift
                ;;
        esac
    done

    if [ -z "$version" ]; then
        log_error "Version is required"
        usage
        exit 1
    fi

    validate_version "$version"

    # Get current version
    CURRENT_VERSION=$(node -p "require('$PROJECT_ROOT/package.json').version")
    log_info "Current version: $CURRENT_VERSION"
    log_info "New version: $version"

    # Update all files
    update_package_json "$version"
    update_cargo_toml "$version"
    update_tauri_conf "$version"

    log_info "Version updated to $version"

    # Create tag if requested
    if [ "$create_git_tag" = true ]; then
        create_tag "$version"
    fi
}

main "$@"
