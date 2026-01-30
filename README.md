# Talent

A cross-platform GUI application for managing Agent Skills across multiple AI CLI tools.

## Overview

Talent provides a unified interface to manage AI agent skills (like Claude Code skills, Codex skills, etc.) from a single location. Instead of manually copying skills to each tool's directory, Talent maintains a central skill repository and syncs them via symlinks.

## Architecture

```
~/.talent/
├── config.toml          # Application configuration
└── skills/              # Central skill storage
    ├── my-skill/
    │   └── SKILL.md
    └── another-skill/
        └── SKILL.md

~/.claude/skills/        # Symlinks to ~/.talent/skills/*
~/.codex/skills/         # Symlinks to ~/.talent/skills/*
~/.gemini/skills/        # Symlinks to ~/.talent/skills/*
```

## Features

- **Unified Management**: Manage all your AI skills from one place
- **Multi-Target Sync**: Automatically sync skills to Claude, Codex, Gemini, Cursor, Amp, and Goose
- **Validation Engine**: Validate skills against agentskills.io rules before deployment
- **File Watching**: Auto-sync on skill changes
- **Cross-Platform**: Works on macOS, Windows, and Linux

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust |
| Framework | Tauri v2 |
| Frontend | Svelte 5 + TypeScript + Vite |
| Editor | CodeMirror 6 |
| File Watching | notify crate |
| CLI | clap |

## Architecture Diagram

```mermaid
flowchart TB
    subgraph Frontend["Frontend (WebView)"]
        direction TB
        Svelte["Svelte 5<br/>Reactive UI"]
        TS["TypeScript"]
        Vite["Vite<br/>Dev Server & Bundler"]
        CM["CodeMirror 6<br/>Markdown Editor"]

        Svelte --> TS
        Vite -.->|"HMR"| Svelte
        CM -->|"Editor Component"| Svelte
    end

    subgraph Tauri["Tauri v2 Runtime"]
        direction TB
        WebView["WebView<br/>(WKWebView/WebView2)"]
        IPC["IPC Bridge<br/>invoke() / emit()"]
        Menu["Native Menu<br/>& Shortcuts"]
        Tray["System Tray"]
        Window["Window Manager"]
    end

    subgraph Backend["Rust Backend"]
        direction TB
        TauriApp["talent-app<br/>(src-tauri)"]
        Commands["Tauri Commands<br/>get_skills, sync_all, etc."]

        TauriApp --> Commands
    end

    subgraph Core["Core Library"]
        direction TB
        TalentCore["talent-core<br/>(crates/talent-core)"]

        subgraph Modules["Modules"]
            Manager["SkillManager"]
            Validator["Validator<br/>agentskills.io spec"]
            Syncer["Syncer<br/>Symlink Engine"]
            Watcher["Watcher<br/>(notify crate)"]
            Config["Config<br/>(TOML)"]
        end

        TalentCore --> Modules
    end

    subgraph CLI["CLI Application"]
        TalentCLI["talent-cli<br/>(clap)"]
    end

    subgraph Storage["File System"]
        direction TB
        TalentDir["~/.talent/"]
        SkillsDir["skills/<br/>Central Repository"]
        ConfigFile["config.toml"]

        TalentDir --> SkillsDir
        TalentDir --> ConfigFile
    end

    subgraph Targets["AI CLI Tools"]
        direction LR
        Claude["Claude Code<br/>~/.claude/skills/"]
        Codex["Codex<br/>~/.codex/skills/"]
        Gemini["Gemini<br/>~/.gemini/skills/"]
        Cursor["Cursor<br/>~/.cursor/skills/"]
        Amp["Amp<br/>~/.amp/skills/"]
        Goose["Goose<br/>~/.config/goose/skills/"]
    end

    %% Connections
    Frontend -->|"Tauri API"| IPC
    IPC <-->|"Commands & Events"| Commands
    WebView --> Frontend
    Menu -->|"Events"| IPC
    Tray -->|"Events"| IPC
    Window --> WebView

    Commands --> TalentCore
    TalentCLI --> TalentCore

    Manager --> Validator
    Manager --> Syncer
    Manager --> Watcher
    Manager --> Config

    Syncer -->|"Read Skills"| SkillsDir
    Syncer -->|"Create Symlinks"| Targets
    Watcher -->|"Monitor Changes"| SkillsDir
    Config -->|"Read/Write"| ConfigFile

    %% Styling
    classDef frontend fill:#ff6b6b,stroke:#333,color:#fff
    classDef tauri fill:#24c8db,stroke:#333,color:#fff
    classDef rust fill:#dea584,stroke:#333,color:#000
    classDef storage fill:#95d5b2,stroke:#333,color:#000
    classDef targets fill:#a8dadc,stroke:#333,color:#000

    class Svelte,TS,Vite,CM frontend
    class WebView,IPC,Menu,Tray,Window tauri
    class TauriApp,Commands,TalentCore,Manager,Validator,Syncer,Watcher,Config,TalentCLI rust
    class TalentDir,SkillsDir,ConfigFile storage
    class Claude,Codex,Gemini,Cursor,Amp,Goose targets
```

### Data Flow

1. **User Interaction** → Svelte UI captures events
2. **Frontend → Backend** → `invoke()` calls Tauri commands
3. **Commands → Core** → Business logic in `talent-core`
4. **Core → File System** → Read/write skills, create symlinks
5. **File System → Targets** → Symlinks point to central skill storage
6. **Backend → Frontend** → Events emitted via `emit()` for updates

## Project Structure

```
talent/
├── Cargo.toml                    # Workspace configuration
├── crates/
│   ├── talent-core/              # Core library
│   │   └── src/
│   │       ├── config.rs         # Configuration management
│   │       ├── error.rs          # Error types
│   │       ├── skill.rs          # Skill model
│   │       ├── target.rs         # Target (CLI tool) model
│   │       ├── validator.rs      # Skill validation
│   │       ├── syncer.rs         # Symlink synchronization
│   │       ├── watcher.rs        # File system watching
│   │       └── manager.rs        # Integration layer
│   └── talent-cli/               # CLI application
│       └── src/main.rs
├── src-tauri/                    # Tauri backend
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       └── commands.rs           # Tauri commands
├── src/                          # Svelte frontend
│   ├── main.ts
│   ├── App.svelte
│   └── app.css
└── package.json
```

## CLI Usage

```bash
# Sync skills to all targets
talent sync

# Sync to specific target
talent sync --target claude

# List all skills
talent list

# Create new skill
talent new my-skill

# Validate skills
talent validate --all

# Show targets
talent targets

# Show configuration
talent config

# Diagnose issues
talent doctor
```

## Supported Targets

| Target | Skills Path |
|--------|-------------|
| Claude Code | `~/.claude/skills/` |
| OpenAI Codex | `~/.codex/skills/` |
| Gemini CLI | `~/.gemini/skills/` |
| Cursor | `~/.cursor/skills/` |
| Amp | `~/.amp/skills/` |
| Goose | `~/.config/goose/skills/` |

## Development

### Prerequisites

- Rust (via rustup)
- Node.js 18+
- Tauri CLI v2

### Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI
cargo install tauri-cli --version "^2"

# Install frontend dependencies
npm install

# Run development server
npm run tauri dev
```

### Building

```bash
# Build for production
npm run tauri build
```

## License

MIT
