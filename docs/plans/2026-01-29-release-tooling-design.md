# Release Tooling Design

**Date:** 2026-01-29
**Status:** Approved

## Overview

Local scripts and GitHub workflows to package Talent for macOS, Linux, and Windows beta releases.

## Distribution

- **Method:** GitHub Releases
- **Triggers:** Git tags (`v*`) + manual workflow dispatch
- **Signing:** None (beta phase)
- **Formats:** Minimal portable (`.app` zip, `.AppImage`, `.exe`)

## File Structure

```
talent-app/
├── scripts/
│   ├── build.sh           # Main build script
│   ├── bump-version.sh    # Version bumping utility
│   └── package.sh         # Organize artifacts for release
├── .github/
│   └── workflows/
│       └── release.yml    # Build & publish workflow
```

## Artifact Naming

| Platform | Format | Filename |
|----------|--------|----------|
| macOS (Intel) | `.zip` containing `.app` | `Talent-{version}-macos-x64.zip` |
| macOS (Apple Silicon) | `.zip` containing `.app` | `Talent-{version}-macos-arm64.zip` |
| Linux | `.AppImage` | `Talent-{version}-linux-x64.AppImage` |
| Windows | `.exe` (portable) | `Talent-{version}-windows-x64.exe` |

## Local Scripts

### `scripts/build.sh`

Main build entry point:
- Detects current platform (macOS/Linux/Windows)
- Validates dependencies (Rust, Node, Tauri CLI)
- Runs `npm install` if node_modules missing
- Executes `npm run tauri build` with release profile
- Calls `package.sh` to organize output

### `scripts/bump-version.sh`

Version management:
- Updates version in: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
- Usage: `./scripts/bump-version.sh 0.2.0-beta.1`
- With tag: `./scripts/bump-version.sh 0.2.0-beta.1 --tag`

### `scripts/package.sh`

Artifact organization:
- Creates `dist/releases/{version}/` directory
- Copies and renames artifacts to consistent naming
- Zips `.app` bundles on macOS
- Generates `checksums.txt` with SHA256 hashes

## GitHub Workflow

### Triggers

```yaml
on:
  push:
    tags: ['v*']
  workflow_dispatch:
    inputs:
      version:
        description: 'Version (e.g., 0.1.0-beta.1)'
        required: false
```

### Jobs

**build** (matrix: macos-latest, ubuntu-22.04, windows-latest)
1. Checkout code
2. Setup Node.js 20
3. Setup Rust stable
4. Install dependencies
5. Install Linux system deps (libgtk-3-dev, libwebkit2gtk-4.1-dev, etc.)
6. Build with `npm run tauri build`
7. Rename artifacts
8. Upload per-platform artifacts

**release** (needs: build)
1. Download all artifacts
2. Generate checksums
3. Create GitHub Release
   - Draft for manual dispatch
   - Published for tag pushes
4. Upload all artifacts

## Usage

```bash
# Bump version and create tag
./scripts/bump-version.sh 0.1.0-beta.3 --tag

# Push to trigger release
git push origin main --tags

# Or manually trigger from GitHub Actions UI
```

## Not Included (YAGNI)

- Code signing (add when leaving beta)
- Auto-update mechanism
- Changelog generation
- Platform installers (.dmg, .msi, .deb)
