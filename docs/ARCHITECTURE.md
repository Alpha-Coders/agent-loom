# Architecture

This document describes the technical architecture of Agent Skills Manager.

## Overview

Agent Skills Manager (ASM) is built with a Rust backend (Tauri v2) and a Svelte 5 frontend. The core logic lives in a separate `asm-core` crate, making it reusable across the GUI app and CLI.

## System Diagram

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
        Window["Window Manager"]
    end

    subgraph Backend["Rust Backend"]
        direction TB
        TauriApp["asm-app<br/>(src-tauri)"]
        Commands["Tauri Commands<br/>get_skills, sync_all, etc."]

        TauriApp --> Commands
    end

    subgraph Core["Core Library"]
        direction TB
        ASMCore["asm-core<br/>(crates/talent-core)"]

        subgraph Modules["Modules"]
            Manager["SkillManager"]
            Validator["Validator<br/>agentskills.io spec"]
            Syncer["Syncer<br/>Symlink Engine"]
            Config["Config<br/>(TOML)"]
        end

        ASMCore --> Modules
    end

    subgraph CLI["CLI Application"]
        ASMCLI["asm-cli<br/>(clap)"]
    end

    subgraph Storage["File System"]
        direction TB
        ASMDir["~/.agentskills/"]
        SkillsDir["skills/<br/>Central Repository"]
        ConfigFile["config.toml"]

        ASMDir --> SkillsDir
        ASMDir --> ConfigFile
    end

    subgraph Targets["AI CLI Tools"]
        direction LR
        Claude["Claude Code<br/>~/.claude/skills/"]
        Codex["Codex<br/>~/.codex/skills/"]
        Gemini["Gemini<br/>~/.gemini/skills/"]
        Cursor["Cursor<br/>~/.cursor/skills-cursor/"]
        Amp["Amp<br/>~/.amp/skills/"]
        Goose["Goose<br/>~/.goose/skills/"]
    end

    %% Connections
    Frontend -->|"Tauri API"| IPC
    IPC <-->|"Commands & Events"| Commands
    WebView --> Frontend
    Menu -->|"Events"| IPC
    Window --> WebView

    Commands --> ASMCore
    ASMCLI --> ASMCore

    Manager --> Validator
    Manager --> Syncer
    Manager --> Config

    Syncer -->|"Read Skills"| SkillsDir
    Syncer -->|"Create Symlinks"| Targets
    Config -->|"Read/Write"| ConfigFile

    %% Styling
    classDef frontend fill:#ff6b6b,stroke:#333,color:#fff
    classDef tauri fill:#24c8db,stroke:#333,color:#fff
    classDef rust fill:#dea584,stroke:#333,color:#000
    classDef storage fill:#95d5b2,stroke:#333,color:#000
    classDef targets fill:#a8dadc,stroke:#333,color:#000

    class Svelte,TS,Vite,CM frontend
    class WebView,IPC,Menu,Window tauri
    class TauriApp,Commands,ASMCore,Manager,Validator,Syncer,Config,ASMCLI rust
    class ASMDir,SkillsDir,ConfigFile storage
    class Claude,Codex,Gemini,Cursor,Amp,Goose targets
```

## Data Flow

1. **User Interaction** → Svelte UI captures events
2. **Frontend → Backend** → `invoke()` calls Tauri commands
3. **Commands → Core** → Business logic in `asm-core`
4. **Core → File System** → Read/write skills, create symlinks
5. **File System → Targets** → Symlinks point to central skill storage

## Project Structure

```
agent-skills-manager/
├── Cargo.toml                    # Workspace configuration
├── crates/
│   ├── talent-core/              # Core library (asm-core)
│   │   └── src/
│   │       ├── config.rs         # Configuration management
│   │       ├── error.rs          # Error types
│   │       ├── skill.rs          # Skill model
│   │       ├── target.rs         # Target (CLI tool) model
│   │       ├── validator.rs      # Skill validation
│   │       ├── syncer.rs         # Symlink synchronization
│   │       └── manager.rs        # Integration layer
│   └── talent-cli/               # CLI application (asm-cli)
│       └── src/main.rs
├── src-tauri/                    # Tauri backend (asm-app)
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       └── commands.rs           # Tauri commands
├── src/                          # Svelte frontend
│   ├── main.ts
│   ├── App.svelte
│   └── lib/
└── package.json
```

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust |
| Framework | Tauri v2 |
| Frontend | Svelte 5 + TypeScript + Vite |
| Editor | CodeMirror 6 |
| CLI | clap 4 |

## Storage Layout

```
~/.agentskills/
├── config.toml          # Application configuration
└── skills/              # Central skill storage
    ├── my-skill/
    │   └── SKILL.md
    └── another-skill/
        └── SKILL.md

~/.claude/skills/        # Symlinks → ~/.agentskills/skills/*
~/.codex/skills/         # Symlinks → ~/.agentskills/skills/*
~/.gemini/skills/        # Symlinks → ~/.agentskills/skills/*
```

## CLI Reference

```bash
# Sync skills to all targets
asm sync

# Sync to specific target
asm sync --target claude

# List all skills
asm list

# Create new skill
asm create my-skill

# Validate skills
asm validate

# Show targets
asm targets

# Diagnose issues
asm doctor
```
