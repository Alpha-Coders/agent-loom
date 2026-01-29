# Agent Specification - Entry Point

This specification works with any AI coding agent (Claude Code, Cursor, Copilot, etc.).

## Loading Instructions

Load `agents/core.md` plus your project's language overlay:

| Project Type | Load |
|--------------|------|
| Swift/iOS/macOS | `agents/core.md` + `agents/swift.md` |
| Python | `agents/core.md` + `agents/python.md` |
| TypeScript/JavaScript | `agents/core.md` + `agents/typescript.md` |
| Rust | `agents/core.md` + `agents/rust.md` |
| Go | `agents/core.md` + `agents/go.md` |
| **This Project** | `agents/core.md` + `agents/rust-tauri-svelte.md` |

## Quick Start

1. Read `agents/core.md` for universal principles
2. Read `agents/rust-tauri-svelte.md` for project-specific commands and conventions
3. Check `plans/` for any in-progress implementation plans

## File Structure

```
talent/
├── AGENTS.md               # This file (entry point, source of truth)
├── CLAUDE.md -> AGENTS.md  # Symlink for Claude Code
├── .claude/
│   └── settings.json       # Claude Code settings (plansDirectory)
├── agents/
│   ├── core.md             # Language-agnostic spec (always load)
│   ├── rust-tauri-svelte.md # Project overlay (Rust + Tauri + Svelte)
│   ├── research/           # Index cards pointing to reference/
│   └── reference/          # Full offline content (SEARCH, don't load)
├── plans/                  # Implementation plans (use TEMPLATE.md)
│   └── TEMPLATE.md         # Plan format template
├── Cargo.toml              # Rust workspace config
├── crates/                 # Rust crates
│   ├── talent-core/        # Core library
│   └── talent-cli/         # CLI application
├── src-tauri/              # Tauri backend
├── src/                    # Svelte frontend
└── package.json            # Frontend dependencies
```

## Section Map

| Section | File | Content |
|---------|------|---------|
| 0. Orientation | `agents/core.md` | Project context |
| 1. Goals | `agents/core.md` | Build, test, lint, run |
| 2. Code Style | `agents/core.md` | Universal principles |
| 3. Git Workflow | `agents/core.md` | Commits, branches, PRs |
| 4. Testing | `agents/core.md` | Testing philosophy |
| 5. Engineering | `agents/core.md` | Simplicity, anti-patterns |
| **6. Planning** | `agents/core.md` | **Use `plans/TEMPLATE.md`** |
| 7. Troubleshooting | `agents/core.md` | Debug strategy |
| **8. Claude Code Plugins** | `agents/core.md` | **Plugin usage and priority rules** |
| **99999. Boundaries** | `agents/core.md` | **CRITICAL: Always/Ask/Never** |
| Language Specifics | `agents/rust-tauri-svelte.md` | Rust/Tauri/Svelte conventions |

## Planning (IMPORTANT)

**When creating plans, ALWAYS use `plans/TEMPLATE.md` format and save to `plans/YYYYMMDD-topic.md`.**

This overrides any tool-specific planning instructions (e.g., Claude Code's built-in plan mode). The project's planning format takes precedence.

## Reference Materials (Search Only)

**Do not load reference/ into context.** Search when needed:

```bash
grep -r "Plan-Then-Execute" agents/reference/
grep -r "Lethal Trifecta" agents/reference/
grep -r "Reflection Loop" agents/reference/
```

| Reference File | Content |
|----------------|---------|
| `good-spec-full.md` | Six core areas, three-tier boundaries |
| `agentic-handbook-full.md` | 113 patterns, security framework |
| `agentic-patterns-full.md` | 130+ patterns by category |
| `ralph-wiggum-full.md` | Loop mechanics, steering techniques |

**When to search**: Pattern implementations, security guidance, multi-agent architectures, feedback loops, boundary setup.

## Claude Code Plugins

**Use Claude Code plugins for specialized capabilities.**

### Recommended Plugins for This Project

| Category | Recommended Plugins |
|----------|---------------------|
| Rust Backend | `rust-analyzer-lsp`, `pr-review-toolkit` |
| Svelte Frontend | `frontend-design`, `code-simplifier` |
| General | `claude-md-management`, `explanatory-output-style` |

### Plugin vs Project Script Priority

**Project scripts always take precedence over plugin commands:**
- Custom commit scripts -> Use instead of `/commit`
- Custom PR scripts -> Use instead of `/commit-push-pr`

### Installing Plugins

```bash
/plugin add <plugin-name>
/plugin list
```

---

## Project-Specific Quick Reference

### Commands

| Command | Description |
|---------|-------------|
| `cargo build` | Build all Rust crates |
| `cargo test` | Run Rust tests |
| `cargo test -p talent-core` | Run core library tests only |
| `cargo clippy` | Lint Rust code |
| `cargo fmt` | Format Rust code |
| `npm install` | Install frontend dependencies |
| `npm run tauri dev` | Run app in development mode |
| `npm run tauri build` | Build production app |

### Project Overview

**Talent** is a cross-platform GUI application for managing Agent Skills across multiple AI CLI tools:
- Central skill storage in `~/.talent/skills/`
- Symlink syncing to: Claude Code, Codex, Gemini, Cursor, Amp, Goose
- Validation engine for skill quality
- File watching for auto-sync

### Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust (Cargo workspace) |
| Framework | Tauri v2 |
| Frontend | Svelte 5 + TypeScript + Vite |
| CLI | clap 4 |
| File Watching | notify crate |

### Current Implementation Status

Check `docs/plans/2026-01-29-talent-implementation.md` for the 13-task MVP plan.
