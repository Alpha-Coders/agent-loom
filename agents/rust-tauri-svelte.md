# Agent Specification - Rust + Tauri + Svelte Overlay

> This overlay covers the full-stack: Rust backend (core library + Tauri), Svelte 5 + TypeScript frontend.

---

## Project: Talent

A cross-platform GUI application for managing Agent Skills across multiple AI CLI tools (Claude Code, Codex, Gemini, Cursor, Amp, Goose).

## Quick Reference

| Command | Description |
|---------|-------------|
| `cargo build` | Build all Rust crates |
| `cargo test` | Run all Rust tests |
| `cargo test -p talent-core` | Run core library tests |
| `cargo clippy` | Lint Rust code |
| `cargo fmt` | Format Rust code |
| `npm install` | Install frontend dependencies |
| `npm run dev` | Start Vite dev server |
| `npm run tauri dev` | Run full Tauri app (dev mode) |
| `npm run tauri build` | Build production app |

## Architecture

```
talent/
├── Cargo.toml                    # Workspace configuration
├── crates/
│   ├── talent-core/              # Core library (pure Rust)
│   │   └── src/
│   │       ├── config.rs         # Configuration management
│   │       ├── error.rs          # Error types (thiserror)
│   │       ├── skill.rs          # Skill model + YAML parsing
│   │       ├── target.rs         # Target (CLI tool) model
│   │       ├── validator.rs      # Skill validation rules
│   │       ├── syncer.rs         # Symlink synchronization
│   │       ├── watcher.rs        # File system watching (notify)
│   │       └── manager.rs        # Integration layer
│   └── talent-cli/               # CLI application (clap)
│       └── src/main.rs
├── src-tauri/                    # Tauri backend
│   └── src/
│       ├── main.rs               # Entry point
│       ├── lib.rs                # Tauri setup
│       └── commands.rs           # Tauri IPC commands
├── src/                          # Svelte frontend
│   ├── main.ts                   # Entry point
│   ├── App.svelte                # Root component
│   └── app.css                   # Global styles
├── package.json                  # Frontend dependencies
└── vite.config.ts                # Vite configuration
```

## Tech Stack

- **Backend**: Rust 2021 edition
- **Framework**: Tauri v2
- **Frontend**: Svelte 5 + TypeScript + Vite
- **CLI**: clap 4
- **File Watching**: notify crate
- **Serialization**: serde, serde_json, serde_yaml, toml
- **Error Handling**: thiserror

## Code Style

### Rust Conventions

```rust
// Use thiserror for error types
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Result alias pattern
pub type Result<T> = std::result::Result<T, Error>;

// Builder pattern for complex structs
impl Target {
    pub fn new(id: impl Into<String>, name: impl Into<String>, path: PathBuf) -> Self { ... }
    pub fn auto_detected(mut self) -> Self { self.auto_detected = true; self }
}

// Derive serde traits for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config { ... }
```

### TypeScript/Svelte Conventions

```typescript
// Use $state() for reactive state (Svelte 5 runes)
let skills: Skill[] = $state([]);
let loading = $state(true);

// Use $derived() for computed values
const filteredSkills = $derived(
  skills.filter(s => s.name.includes(searchQuery))
);

// Invoke Tauri commands with type safety
import { invoke } from "@tauri-apps/api/core";
const skills = await invoke<Skill[]>("get_skills");
```

### Naming Conventions

- **Rust Files**: `snake_case.rs`
- **Rust Types**: `PascalCase` (structs, enums, traits)
- **Rust Functions**: `snake_case`
- **TypeScript Files**: `PascalCase.svelte`, `camelCase.ts`
- **TypeScript Variables**: `camelCase`
- **TypeScript Types**: `PascalCase`
- **CSS Variables**: `--kebab-case`

## Testing

### Running Tests

```bash
# All Rust tests
cargo test

# Specific crate
cargo test -p talent-core

# Single test
cargo test test_name

# With output
cargo test -- --nocapture
```

### Test Conventions

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let dir = tempdir().unwrap();

        // Act
        let result = some_function(&dir.path());

        // Assert
        assert!(result.is_ok());
    }
}
```

## Rust-Specific Boundaries

### Always Safe (in addition to core.md)

- Run `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`
- Add `#[derive(...)]` attributes
- Use workspace dependencies
- Create new modules within existing crates

### Ask First (in addition to core.md)

- Add new crates to workspace
- Add new dependencies to `Cargo.toml`
- Change `[workspace.dependencies]` versions
- Modify Tauri permissions/capabilities

### Never Do (in addition to core.md)

- Use `unsafe` without explicit approval
- Add `panic!()` in library code (use `Result` instead)
- Disable clippy lints with `#[allow(...)]` without justification

## Common Tasks

### Add a New Tauri Command

1. Add function in `src-tauri/src/commands.rs`:
```rust
#[tauri::command]
pub fn my_command(arg: String) -> Result<String, String> {
    // Implementation
}
```

2. Register in `src-tauri/src/lib.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    commands::my_command,  // Add here
])
```

3. Call from frontend:
```typescript
const result = await invoke<string>("my_command", { arg: "value" });
```

### Add a New Core Module

1. Create `crates/talent-core/src/newmodule.rs`
2. Add `pub mod newmodule;` to `lib.rs`
3. Add `pub use newmodule::PublicType;` if needed

### Add a Svelte Component

1. Create `src/components/MyComponent.svelte`
2. Import in parent: `import MyComponent from "./components/MyComponent.svelte"`

## Troubleshooting

### Tauri Build Fails

```bash
# Clean and rebuild
cargo clean
npm run tauri build
```

### Frontend Not Updating

```bash
# Clear Vite cache
rm -rf node_modules/.vite
npm run dev
```

### Rust Analyzer Issues

```bash
# Regenerate rust-analyzer config
cargo clean
cargo build
```

### Permission Errors on macOS

Ensure Tauri capabilities are configured in `src-tauri/capabilities/default.json`.
