<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Agent Skills Manager Logo" width="128" height="128">
</p>

<h1 align="center">Agent Skills Manager</h1>

<p align="center">
  <strong>One place to manage all your AI agent skills</strong>
</p>

<p align="center">
  <a href="https://github.com/Alpha-Coders/agent-skills-manager/releases"><img src="https://img.shields.io/github/v/release/Alpha-Coders/agent-skills-manager?style=flat-square" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://github.com/Alpha-Coders/agent-skills-manager/actions"><img src="https://img.shields.io/github/actions/workflow/status/Alpha-Coders/agent-skills-manager/release.yml?style=flat-square" alt="Build"></a>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#supported-tools">Supported Tools</a> •
  <a href="#development">Development</a> •
  <a href="docs/ARCHITECTURE.md">Architecture</a>
</p>

---

## What is Agent Skills Manager?

Agent Skills Manager (ASM) is a desktop app that lets you manage AI agent skills from a single location. Instead of manually copying skills to each tool's directory, ASM maintains a central repository and syncs them via symlinks.

```
~/.agentskills/skills/my-skill/    →    ~/.claude/skills/my-skill
                           →    ~/.cursor/skills-cursor/my-skill
                           →    ~/.codex/skills/my-skill
                           →    ... and more
```

## Features

- **Unified Management** — Create, edit, and organize all your skills in one place
- **Multi-Target Sync** — One click to sync skills across all your AI tools
- **Built-in Editor** — Markdown editor with syntax highlighting
- **Validation** — Validates skills against the [agentskills.io](https://agentskills.io) specification
- **Import** — Import existing skills from any target or folder
- **Cross-Platform** — Works on macOS, Windows, and Linux

## Supported Tools

| Tool | Path |
|------|------|
| Claude Code | `~/.claude/skills/` |
| Cursor | `~/.cursor/skills-cursor/` |
| OpenAI Codex | `~/.codex/skills/` |
| Gemini CLI | `~/.gemini/skills/` |
| Amp | `~/.amp/skills/` |
| Goose | `~/.goose/skills/` |
| Roo Code | `~/.roo-code/skills/` |
| OpenCode | `~/.opencode/skills/` |
| Vibe | `~/.vibe/skills/` |
| Firebender | `~/.firebender/skills/` |

## Installation

### Download

Get the latest release for your platform:

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | [Download](https://github.com/Alpha-Coders/agent-skills-manager/releases/latest) |
| macOS (Intel) | [Download](https://github.com/Alpha-Coders/agent-skills-manager/releases/latest) |
| Windows | [Download](https://github.com/Alpha-Coders/agent-skills-manager/releases/latest) |
| Linux | [Download](https://github.com/Alpha-Coders/agent-skills-manager/releases/latest) |

### macOS

1. Unzip and drag `Agent Skills Manager.app` to Applications
2. On first run, right-click → Open (to bypass Gatekeeper)

### Windows

Run the installer and follow the prompts.

### Linux

```bash
chmod +x AgentSkillsManager-*.AppImage
./AgentSkillsManager-*.AppImage
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
npm run dev
```

### Build

```bash
npm run tauri build
```

### Project Structure

See [Architecture Documentation](docs/ARCHITECTURE.md) for detailed technical information.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[MIT](LICENSE)
