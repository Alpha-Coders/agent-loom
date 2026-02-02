---
layout: default
---

<p align="center">
  <img src="https://raw.githubusercontent.com/Alpha-Coders/agent-loom/main/src-tauri/icons/readme-icon.png" alt="AgentLoom Logo" width="120" height="120" style="margin-bottom: 20px;">
</p>

<p align="center">
  <a href="https://github.com/Alpha-Coders/agent-loom/releases"><img src="https://img.shields.io/github/v/release/Alpha-Coders/agent-loom?style=for-the-badge&color=FFD700&labelColor=1a1a2e" alt="Release"></a>
  &nbsp;
  <a href="https://github.com/Alpha-Coders/agent-loom/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-FFD700?style=for-the-badge&labelColor=1a1a2e" alt="License"></a>
  &nbsp;
  <a href="https://github.com/Alpha-Coders/agent-loom/actions"><img src="https://img.shields.io/github/actions/workflow/status/Alpha-Coders/agent-loom/ci.yml?style=for-the-badge&color=FFD700&labelColor=1a1a2e&label=build" alt="Build"></a>
</p>

---

## What is AgentLoom?

AgentLoom is a **cross-platform desktop app** that lets you manage AI agent skills from a single location. It works with any tool that supports the [agentskills.io](https://agentskills.io) open format.

Instead of manually copying skills to each tool's directory, AgentLoom maintains a central repository and syncs them via symlinks:

```
~/.claude/skills/my-skill        →  ~/.agentloom/skills/my-skill
~/.cursor/skills-cursor/my-skill →  ~/.agentloom/skills/my-skill
~/.codex/skills/my-skill         →  ~/.agentloom/skills/my-skill
```

<p align="center" style="margin: 40px 0;">
  <img src="screenshot-themes.png" alt="AgentLoom Screenshot" width="800">
</p>

---

## Features

- **Unified Management** — Create, edit, and organize all your skills in one place
- **Multi-Target Sync** — One click to sync skills across all your AI tools
- **Built-in Editor** — Full-featured markdown editor with syntax highlighting
- **Validation** — Validates skills against the agentskills.io specification
- **Import** — Import existing skills from any target, folder, or drag-and-drop
- **Theme Support** — System, light, and dark themes with native integration
- **Cross-Platform** — Works on macOS, Windows, and Linux

---

## Supported Tools

AgentLoom works with any tool that supports the [agentskills.io](https://agentskills.io) specification:

| Tool | Vendor |
|------|--------|
| Claude Code | Anthropic |
| Codex | OpenAI |
| Gemini CLI | Google |
| Cursor | Anysphere |
| Amp | Sourcegraph |
| Goose | Block |
| Roo Code | Roo |
| OpenCode | Anthropic |
| Vibe | Vibe |
| Firebender | Firebender |

*Plus custom folder targets for any other tools.*

---

## Installation

### Download

<p align="center">
  <a href="https://github.com/Alpha-Coders/agent-loom/releases/latest"><strong>Download Latest Release</strong></a>
</p>

Available for **macOS** (Apple Silicon & Intel), **Windows**, and **Linux**.

### macOS

1. Unzip and drag `AgentLoom.app` to Applications
2. On first run: right-click → Open (to bypass Gatekeeper)

### Windows

Run the installer and follow the prompts.

### Linux

```bash
chmod +x AgentLoom-*.AppImage
./AgentLoom-*.AppImage
```

---

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (via rustup)
- [Node.js](https://nodejs.org/) 18+
- [Tauri CLI v2](https://v2.tauri.app/)

### Quick Start

```bash
cargo install tauri-cli --version "^2"
npm install
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

---

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust |
| Framework | Tauri v2 |
| Frontend | Svelte 5 + TypeScript |
| Editor | CodeMirror 6 |

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

See [Architecture Documentation](https://github.com/Alpha-Coders/agent-loom/blob/main/docs/ARCHITECTURE.md) for technical details.

---

<p align="center" style="margin-top: 40px; color: #a1a1a6;">
  Built with <a href="https://tauri.app/">Tauri</a>, <a href="https://svelte.dev/">Svelte</a>, and <a href="https://lucide.dev/">Lucide</a> icons.
  <br><br>
  <a href="https://github.com/Alpha-Coders/agent-loom/blob/main/LICENSE">MIT License</a>
</p>
