---
layout: default
---

<p align="center">
  <img src="src-tauri/icons/readme-icon.png" alt="AgentLoom Logo" width="128" height="128">
</p>

<h1 align="center">AgentLoom</h1>

<p align="center">
  <strong>One place to manage all your AI agent skills</strong>
</p>

<p align="center">
  <a href="https://github.com/Alpha-Coders/agent-loom/releases"><img src="https://img.shields.io/github/v/release/Alpha-Coders/agent-loom?style=flat-square" alt="Release"></a>
  <a href="https://github.com/Alpha-Coders/agent-loom/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://github.com/Alpha-Coders/agent-loom/actions"><img src="https://img.shields.io/github/actions/workflow/status/Alpha-Coders/agent-loom/release.yml?style=flat-square" alt="Build"></a>
</p>

---

## What is AgentLoom?

AgentLoom is a desktop app that lets you manage AI agent skills from a single location. It works with any tool that supports the [agentskills.io](https://agentskills.io) open format. Instead of manually copying skills to each tool's directory, it maintains a central repository and syncs them via symlinks.

<p align="center">
  <img src="docs/screenshot-themes.png" alt="AgentLoom Screenshot" width="800">
</p>

```
~/.claude/skills/my-skill           →  ~/.agentloom/skills/my-skill
~/.cursor/skills-cursor/my-skill    →  ~/.agentloom/skills/my-skill
~/.codex/skills/my-skill            →  ~/.agentloom/skills/my-skill
```

## Features

- **Unified Management** — Create, edit, and organize all your skills in one place
- **Multi-Target Sync** — One click to sync skills across all your AI tools
- **Built-in Editor** — Full-featured markdown editor with syntax highlighting (CodeMirror 6)
- **Validation** — Validates skills against the [agentskills.io](https://agentskills.io) specification with auto-fix
- **Import** — Import existing skills from any target, folder, or via drag-and-drop
- **Theme Support** — System, light, and dark themes with native macOS integration
- **Cross-Platform** — Works on macOS, Windows, and Linux

## Supported Tools

AgentLoom works with any tool that supports the [agentskills.io](https://agentskills.io) specification:

| Tool | Vendor | Skills Directory |
|------|--------|------------------|
| Claude Code | Anthropic | `~/.claude/skills/` |
| Codex | OpenAI | `~/.codex/skills/` |
| Gemini CLI | Google | `~/.gemini/skills/` |
| Cursor | Anysphere | `~/.cursor/skills-cursor/` |
| Amp | Sourcegraph | `~/.amp/skills/` |
| Goose | Block | `~/.goose/skills/` |
| Roo Code | Roo | `~/.roo-code/skills/` |
| OpenCode | Anthropic | `~/.opencode/skills/` |
| Vibe | Vibe | `~/.vibe/skills/` |
| Firebender | Firebender | `~/.firebender/skills/` |
| Mux | Mux | `~/.mux/skills/` |
| Autohand | Autohand | `~/.autohand/skills/` |

You can also add custom folder targets for tools not listed above.

## Installation

### Download

Get the latest release for your platform:

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | [Download](https://github.com/Alpha-Coders/agent-loom/releases/latest) |
| macOS (Intel) | [Download](https://github.com/Alpha-Coders/agent-loom/releases/latest) |
| Windows | [Download](https://github.com/Alpha-Coders/agent-loom/releases/latest) |
| Linux | [Download](https://github.com/Alpha-Coders/agent-loom/releases/latest) |

### macOS

1. Unzip and drag `AgentLoom.app` to Applications
2. On first run, right-click → Open (to bypass Gatekeeper)

### Windows

Run the installer and follow the prompts.

### Linux

```bash
chmod +x AgentLoom-*.AppImage
./AgentLoom-*.AppImage
```

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (via rustup)
- [Node.js](https://nodejs.org/) 18+
- [Tauri CLI v2](https://v2.tauri.app/)

### Quick Start

```bash
# Install Tauri CLI
cargo install tauri-cli --version "^2"

# Install dependencies
npm install

# Run development server
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

### Project Structure

See [Architecture Documentation](https://github.com/Alpha-Coders/agent-loom/blob/main/docs/ARCHITECTURE.md) for detailed technical information.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust |
| Framework | Tauri v2 |
| Frontend | Svelte 5 + TypeScript + Vite |
| Editor | CodeMirror 6 |
| Icons | Lucide |

## Credits

Built with [Tauri](https://tauri.app/), [Svelte](https://svelte.dev/), and [Lucide](https://lucide.dev/) icons.

## License

[MIT](https://github.com/Alpha-Coders/agent-loom/blob/main/LICENSE)
