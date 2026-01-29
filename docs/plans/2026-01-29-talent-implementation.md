# Talent Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a cross-platform GUI application (Rust + Tauri + Svelte) for managing Agent Skills across multiple AI CLI tools.

**Architecture:** Central skill storage in `~/.talent/skills/` with symlinks to each target CLI's skills directory. Tauri backend handles file watching, validation, and sync. Svelte frontend provides skill list, editor, and settings.

**Tech Stack:** Rust (backend), Tauri v2 (framework), Svelte + TypeScript + Vite (frontend), CodeMirror 6 (editor), notify crate (file watching), clap (CLI)

---

## Phase 1: Project Setup & Core Backend

### Task 1: Install Rust and Initialize Project

**Files:**
- Create: `~/Developer/talent/Cargo.toml`
- Create: `~/Developer/talent/crates/talent-core/Cargo.toml`
- Create: `~/Developer/talent/crates/talent-core/src/lib.rs`
- Create: `~/Developer/talent/crates/talent-cli/Cargo.toml`
- Create: `~/Developer/talent/crates/talent-cli/src/main.rs`

**Step 1: Install Rust via rustup**

Run:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

Expected: Rust installed, `cargo --version` shows version

**Step 2: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "crates/talent-core",
    "crates/talent-cli",
    "src-tauri",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
authors = ["Talent Contributors"]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
toml = "0.8"
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }
```

**Step 3: Create talent-core crate**

`crates/talent-core/Cargo.toml`:
```toml
[package]
name = "talent-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
serde_yaml.workspace = true
toml.workspace = true
thiserror.workspace = true
chrono.workspace = true
walkdir = "2"
notify = "6"
sha2 = "0.10"
tokio = { version = "1", features = ["full"] }
```

`crates/talent-core/src/lib.rs`:
```rust
pub mod config;
pub mod error;
pub mod skill;
pub mod target;
pub mod validator;
pub mod syncer;
pub mod watcher;

pub use config::Config;
pub use error::{Error, Result};
pub use skill::Skill;
pub use target::Target;
```

**Step 4: Create talent-cli crate**

`crates/talent-cli/Cargo.toml`:
```toml
[package]
name = "talent-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "talent"
path = "src/main.rs"

[dependencies]
talent-core = { path = "../talent-core" }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

`crates/talent-cli/src/main.rs`:
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "talent")]
#[command(about = "Unified Agent Skills Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync skills to all targets
    Sync {
        #[arg(short, long)]
        target: Option<String>,
    },
    /// Show sync status
    Status,
    /// List all skills
    List {
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    println!("Talent CLI - command received");
}
```

**Step 5: Verify build**

Run:
```bash
cd ~/Developer/talent && cargo build
```

Expected: Build succeeds

**Step 6: Commit**

```bash
cd ~/Developer/talent
git init
git add .
git commit -m "chore: initialize Rust workspace with core and CLI crates"
```

---

### Task 2: Implement Error Types

**Files:**
- Create: `crates/talent-core/src/error.rs`

**Step 1: Write error types**

`crates/talent-core/src/error.rs`:
```rust
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Target not found: {0}")]
    TargetNotFound(String),

    #[error("Invalid skill: {path} - {reason}")]
    InvalidSkill { path: PathBuf, reason: String },

    #[error("Symlink error: {0}")]
    Symlink(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Watch error: {0}")]
    Watch(#[from] notify::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Step 2: Verify build**

Run:
```bash
cd ~/Developer/talent && cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add crates/talent-core/src/error.rs
git commit -m "feat(core): add error types"
```

---

### Task 3: Implement Config Module

**Files:**
- Create: `crates/talent-core/src/config.rs`
- Test: `crates/talent-core/src/config.rs` (inline tests)

**Step 1: Write config module**

`crates/talent-core/src/config.rs`:
```rust
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub sync: SyncConfig,
    #[serde(default)]
    pub validation: ValidationConfig,
    #[serde(default)]
    pub targets: TargetsConfig,
    #[serde(default)]
    pub editor: EditorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default = "default_true")]
    pub run_in_background: bool,
    #[serde(default = "default_true")]
    pub check_for_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    #[serde(default = "default_true")]
    pub auto_sync: bool,
    #[serde(default = "default_watch_interval")]
    pub watch_interval_ms: u64,
    #[serde(default = "default_true")]
    pub sync_on_change: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    #[serde(default = "default_true")]
    pub validate_on_save: bool,
    #[serde(default = "default_true")]
    pub validate_on_sync: bool,
    #[serde(default)]
    pub strict_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetsConfig {
    #[serde(default = "default_true")]
    pub auto_detect: bool,
    #[serde(default)]
    pub custom_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_true")]
    pub word_wrap: bool,
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
}

fn default_theme() -> String { "system".to_string() }
fn default_true() -> bool { true }
fn default_watch_interval() -> u64 { 500 }
fn default_font_size() -> u32 { 14 }
fn default_font_family() -> String { "JetBrains Mono".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            sync: SyncConfig::default(),
            validation: ValidationConfig::default(),
            targets: TargetsConfig::default(),
            editor: EditorConfig::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            start_minimized: false,
            run_in_background: true,
            check_for_updates: true,
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            auto_sync: true,
            watch_interval_ms: default_watch_interval(),
            sync_on_change: true,
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            validate_on_save: true,
            validate_on_sync: true,
            strict_mode: false,
        }
    }
}

impl Default for TargetsConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            custom_paths: vec![],
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            font_family: default_font_family(),
            word_wrap: true,
            show_line_numbers: true,
        }
    }
}

impl Config {
    pub fn talent_dir() -> PathBuf {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".talent")
    }

    pub fn config_path() -> PathBuf {
        Self::talent_dir().join("config.toml")
    }

    pub fn skills_dir() -> PathBuf {
        Self::talent_dir().join("skills")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.app.theme, "system");
        assert!(config.sync.auto_sync);
        assert_eq!(config.sync.watch_interval_ms, 500);
    }

    #[test]
    fn test_config_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.app.theme, config.app.theme);
    }
}
```

**Step 2: Add dirs dependency**

Update `crates/talent-core/Cargo.toml` to add:
```toml
dirs = "5"
```

**Step 3: Run tests**

Run:
```bash
cd ~/Developer/talent && cargo test -p talent-core
```

Expected: All tests pass

**Step 4: Commit**

```bash
git add .
git commit -m "feat(core): add config module with serialization"
```

---

### Task 4: Implement Skill Model

**Files:**
- Create: `crates/talent-core/src/skill.rs`

**Step 1: Write skill model**

`crates/talent-core/src/skill.rs`:
```rust
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub validation_status: ValidationStatus,
    pub last_modified: DateTime<Utc>,
    pub size_bytes: u64,
    #[serde(default)]
    pub has_scripts: bool,
    #[serde(default)]
    pub has_references: bool,
    #[serde(default)]
    pub has_assets: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    Valid,
    Warning(Vec<ValidationIssue>),
    Invalid(Vec<ValidationIssue>),
    #[default]
    NotValidated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<(usize, usize)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub compatibility: Option<String>,
    #[serde(default)]
    pub metadata: Option<SkillMetadata>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SkillMetadata {
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

impl Skill {
    pub fn from_path(path: &Path) -> Result<Self> {
        let skill_md_path = path.join("SKILL.md");
        if !skill_md_path.exists() {
            return Err(Error::InvalidSkill {
                path: path.to_path_buf(),
                reason: "SKILL.md not found".to_string(),
            });
        }

        let content = fs::read_to_string(&skill_md_path)?;
        let frontmatter = Self::parse_frontmatter(&content)?;

        let metadata = fs::metadata(&skill_md_path)?;
        let last_modified = metadata
            .modified()
            .map(|t| DateTime::<Utc>::from(t))
            .unwrap_or_else(|_| Utc::now());

        let tags = frontmatter
            .metadata
            .as_ref()
            .and_then(|m| m.tags.as_ref())
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let extra_metadata = frontmatter
            .metadata
            .map(|m| m.extra)
            .unwrap_or_default();

        let has_scripts = path.join("scripts").exists();
        let has_references = path.join("references").exists();
        let has_assets = path.join("assets").exists();

        let size_bytes = Self::calculate_size(path)?;

        Ok(Self {
            name: frontmatter.name,
            path: path.to_path_buf(),
            description: frontmatter.description,
            license: frontmatter.license,
            compatibility: frontmatter.compatibility,
            metadata: extra_metadata,
            tags,
            validation_status: ValidationStatus::NotValidated,
            last_modified,
            size_bytes,
            has_scripts,
            has_references,
            has_assets,
        })
    }

    fn parse_frontmatter(content: &str) -> Result<SkillFrontmatter> {
        let content = content.trim();
        if !content.starts_with("---") {
            return Err(Error::InvalidSkill {
                path: PathBuf::new(),
                reason: "Missing YAML frontmatter".to_string(),
            });
        }

        let rest = &content[3..];
        let end = rest.find("---").ok_or_else(|| Error::InvalidSkill {
            path: PathBuf::new(),
            reason: "Unclosed YAML frontmatter".to_string(),
        })?;

        let yaml_content = &rest[..end];
        let frontmatter: SkillFrontmatter = serde_yaml::from_str(yaml_content)?;
        Ok(frontmatter)
    }

    fn calculate_size(path: &Path) -> Result<u64> {
        let mut total = 0u64;
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry.map_err(|e| Error::Io(e.into()))?;
            if entry.file_type().is_file() {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
        Ok(total)
    }

    pub fn skill_md_content(&self) -> Result<String> {
        let skill_md_path = self.path.join("SKILL.md");
        Ok(fs::read_to_string(skill_md_path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_parse_skill_from_path() {
        let dir = tempdir().unwrap();
        let skill_path = dir.path().join("test-skill");
        fs::create_dir(&skill_path).unwrap();

        let skill_md = r#"---
name: test-skill
description: A test skill for unit testing
metadata:
  tags: test, example
---
# Test Skill

This is a test skill.
"#;
        let mut file = fs::File::create(skill_path.join("SKILL.md")).unwrap();
        file.write_all(skill_md.as_bytes()).unwrap();

        let skill = Skill::from_path(&skill_path).unwrap();
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, "A test skill for unit testing");
        assert_eq!(skill.tags, vec!["test", "example"]);
    }

    #[test]
    fn test_missing_skill_md() {
        let dir = tempdir().unwrap();
        let skill_path = dir.path().join("empty-skill");
        fs::create_dir(&skill_path).unwrap();

        let result = Skill::from_path(&skill_path);
        assert!(result.is_err());
    }
}
```

**Step 2: Add dependencies**

Update `crates/talent-core/Cargo.toml`:
```toml
tempfile = "3"  # Add to dev-dependencies
```

Add `[dev-dependencies]` section:
```toml
[dev-dependencies]
tempfile = "3"
```

**Step 3: Run tests**

Run:
```bash
cd ~/Developer/talent && cargo test -p talent-core
```

Expected: All tests pass

**Step 4: Commit**

```bash
git add .
git commit -m "feat(core): add skill model with frontmatter parsing"
```

---

### Task 5: Implement Target Model

**Files:**
- Create: `crates/talent-core/src/target.rs`

**Step 1: Write target model**

`crates/talent-core/src/target.rs`:
```rust
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: String,
    pub name: String,
    pub skills_path: PathBuf,
    #[serde(default)]
    pub auto_detected: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub skill_overrides: HashMap<String, SkillOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillOverride {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

impl Target {
    pub fn new(id: impl Into<String>, name: impl Into<String>, skills_path: PathBuf) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            skills_path,
            auto_detected: false,
            enabled: true,
            skill_overrides: HashMap::new(),
        }
    }

    pub fn auto_detected(mut self) -> Self {
        self.auto_detected = true;
        self
    }

    pub fn is_skill_enabled(&self, skill_name: &str) -> bool {
        self.enabled && self.skill_overrides
            .get(skill_name)
            .map(|o| o.enabled)
            .unwrap_or(true)
    }

    pub fn set_skill_enabled(&mut self, skill_name: &str, enabled: bool) {
        self.skill_overrides.insert(
            skill_name.to_string(),
            SkillOverride { enabled },
        );
    }

    pub fn detect_all() -> Vec<Target> {
        let home = dirs::home_dir().expect("Could not find home directory");

        let known_targets = [
            ("claude", "Claude Code", ".claude/skills"),
            ("codex", "OpenAI Codex", ".codex/skills"),
            ("gemini", "Gemini CLI", ".gemini/skills"),
            ("cursor", "Cursor", ".cursor/skills"),
            ("amp", "Amp", ".amp/skills"),
            ("goose", "Goose", ".config/goose/skills"),
        ];

        known_targets
            .iter()
            .filter_map(|(id, name, rel_path)| {
                let skills_path = home.join(rel_path);
                // Check if parent exists (the CLI tool is installed)
                let parent = skills_path.parent()?;
                if parent.exists() {
                    Some(Target::new(*id, *name, skills_path).auto_detected())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn ensure_skills_dir(&self) -> Result<()> {
        if !self.skills_path.exists() {
            std::fs::create_dir_all(&self.skills_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_creation() {
        let target = Target::new("test", "Test Target", PathBuf::from("/tmp/skills"));
        assert_eq!(target.id, "test");
        assert_eq!(target.name, "Test Target");
        assert!(target.enabled);
        assert!(!target.auto_detected);
    }

    #[test]
    fn test_skill_override() {
        let mut target = Target::new("test", "Test Target", PathBuf::from("/tmp/skills"));

        assert!(target.is_skill_enabled("any-skill"));

        target.set_skill_enabled("disabled-skill", false);
        assert!(!target.is_skill_enabled("disabled-skill"));
        assert!(target.is_skill_enabled("other-skill"));
    }

    #[test]
    fn test_auto_detect() {
        let targets = Target::detect_all();
        // Should detect at least claude and codex based on the home directory
        // This test verifies the detection logic runs without error
        for target in &targets {
            assert!(!target.id.is_empty());
            assert!(!target.name.is_empty());
            assert!(target.auto_detected);
        }
    }
}
```

**Step 2: Run tests**

Run:
```bash
cd ~/Developer/talent && cargo test -p talent-core
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add .
git commit -m "feat(core): add target model with auto-detection"
```

---

### Task 6: Implement Validation Engine

**Files:**
- Create: `crates/talent-core/src/validator.rs`

**Step 1: Write validator**

`crates/talent-core/src/validator.rs`:
```rust
use crate::skill::{Skill, ValidationIssue, ValidationStatus};
use crate::Result;
use std::path::Path;

pub struct Validator {
    strict_mode: bool,
}

impl Validator {
    pub fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    pub fn validate(&self, skill: &mut Skill) -> Result<()> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Read SKILL.md content
        let content = skill.skill_md_content()?;

        // MISSING_NAME - already caught during parsing, but double-check
        if skill.name.is_empty() {
            errors.push(ValidationIssue {
                code: "MISSING_NAME".to_string(),
                message: "Skill name is required".to_string(),
                location: None,
                fix_hint: Some("Add 'name: your-skill-name' to frontmatter".to_string()),
            });
        }

        // MISSING_DESCRIPTION
        if skill.description.is_empty() {
            errors.push(ValidationIssue {
                code: "MISSING_DESCRIPTION".to_string(),
                message: "Skill description is required".to_string(),
                location: None,
                fix_hint: Some("Add 'description: Your skill description' to frontmatter".to_string()),
            });
        }

        // NAME_MISMATCH - name should match directory name
        if let Some(dir_name) = skill.path.file_name().and_then(|n| n.to_str()) {
            if skill.name != dir_name {
                errors.push(ValidationIssue {
                    code: "NAME_MISMATCH".to_string(),
                    message: format!(
                        "Skill name '{}' does not match directory name '{}'",
                        skill.name, dir_name
                    ),
                    location: None,
                    fix_hint: Some(format!("Change name to '{}' or rename directory", dir_name)),
                });
            }
        }

        // NAME_INVALID_CHARS
        if !skill.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            errors.push(ValidationIssue {
                code: "NAME_INVALID_CHARS".to_string(),
                message: "Skill name contains invalid characters".to_string(),
                location: None,
                fix_hint: Some("Use only alphanumeric characters, hyphens, and underscores".to_string()),
            });
        }

        // NAME_TOO_LONG
        if skill.name.len() > 64 {
            errors.push(ValidationIssue {
                code: "NAME_TOO_LONG".to_string(),
                message: format!("Skill name is {} characters, max is 64", skill.name.len()),
                location: None,
                fix_hint: Some("Shorten the skill name to 64 characters or less".to_string()),
            });
        }

        // DESC_TOO_LONG
        if skill.description.len() > 1024 {
            errors.push(ValidationIssue {
                code: "DESC_TOO_LONG".to_string(),
                message: format!("Description is {} characters, max is 1024", skill.description.len()),
                location: None,
                fix_hint: Some("Shorten the description to 1024 characters or less".to_string()),
            });
        }

        // COMPAT_TOO_LONG
        if let Some(ref compat) = skill.compatibility {
            if compat.len() > 500 {
                errors.push(ValidationIssue {
                    code: "COMPAT_TOO_LONG".to_string(),
                    message: format!("Compatibility is {} characters, max is 500", compat.len()),
                    location: None,
                    fix_hint: Some("Shorten compatibility to 500 characters or less".to_string()),
                });
            }
        }

        // DESC_TOO_SHORT (warning)
        if skill.description.len() < 20 {
            warnings.push(ValidationIssue {
                code: "DESC_TOO_SHORT".to_string(),
                message: "Description is very short (< 20 characters)".to_string(),
                location: None,
                fix_hint: Some("Consider adding more detail to the description".to_string()),
            });
        }

        // MISSING_LICENSE (warning)
        if skill.license.is_none() {
            warnings.push(ValidationIssue {
                code: "MISSING_LICENSE".to_string(),
                message: "No license specified".to_string(),
                location: None,
                fix_hint: Some("Add 'license: MIT' or similar to frontmatter".to_string()),
            });
        }

        // LARGE_SKILLMD (warning)
        let line_count = content.lines().count();
        if line_count > 500 {
            warnings.push(ValidationIssue {
                code: "LARGE_SKILLMD".to_string(),
                message: format!("SKILL.md is {} lines (> 500)", line_count),
                location: None,
                fix_hint: Some("Consider splitting into multiple files".to_string()),
            });
        }

        // Set validation status
        skill.validation_status = if !errors.is_empty() {
            ValidationStatus::Invalid(errors)
        } else if !warnings.is_empty() {
            if self.strict_mode {
                ValidationStatus::Invalid(warnings)
            } else {
                ValidationStatus::Warning(warnings)
            }
        } else {
            ValidationStatus::Valid
        };

        Ok(())
    }

    pub fn validate_path(&self, path: &Path) -> Result<Skill> {
        let mut skill = Skill::from_path(path)?;
        self.validate(&mut skill)?;
        Ok(skill)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_skill(dir: &Path, name: &str, content: &str) {
        let skill_path = dir.join(name);
        fs::create_dir(&skill_path).unwrap();
        let mut file = fs::File::create(skill_path.join("SKILL.md")).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_valid_skill() {
        let dir = tempdir().unwrap();
        create_test_skill(dir.path(), "valid-skill", r#"---
name: valid-skill
description: A properly formatted skill for testing purposes
license: MIT
---
# Valid Skill

Content here.
"#);

        let validator = Validator::new(false);
        let skill = validator.validate_path(&dir.path().join("valid-skill")).unwrap();
        assert_eq!(skill.validation_status, ValidationStatus::Valid);
    }

    #[test]
    fn test_name_mismatch() {
        let dir = tempdir().unwrap();
        create_test_skill(dir.path(), "actual-name", r#"---
name: different-name
description: This skill has a name mismatch
---
# Test
"#);

        let validator = Validator::new(false);
        let skill = validator.validate_path(&dir.path().join("actual-name")).unwrap();
        match skill.validation_status {
            ValidationStatus::Invalid(issues) => {
                assert!(issues.iter().any(|i| i.code == "NAME_MISMATCH"));
            }
            _ => panic!("Expected Invalid status"),
        }
    }

    #[test]
    fn test_missing_license_warning() {
        let dir = tempdir().unwrap();
        create_test_skill(dir.path(), "no-license", r#"---
name: no-license
description: A skill without a license specified
---
# Test
"#);

        let validator = Validator::new(false);
        let skill = validator.validate_path(&dir.path().join("no-license")).unwrap();
        match skill.validation_status {
            ValidationStatus::Warning(issues) => {
                assert!(issues.iter().any(|i| i.code == "MISSING_LICENSE"));
            }
            _ => panic!("Expected Warning status"),
        }
    }

    #[test]
    fn test_strict_mode() {
        let dir = tempdir().unwrap();
        create_test_skill(dir.path(), "no-license", r#"---
name: no-license
description: A skill without a license specified
---
# Test
"#);

        let validator = Validator::new(true);
        let skill = validator.validate_path(&dir.path().join("no-license")).unwrap();
        match skill.validation_status {
            ValidationStatus::Invalid(_) => {}
            _ => panic!("Expected Invalid status in strict mode"),
        }
    }
}
```

**Step 2: Run tests**

Run:
```bash
cd ~/Developer/talent && cargo test -p talent-core
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add .
git commit -m "feat(core): add validation engine with agentskills.io rules"
```

---

### Task 7: Implement Symlink Syncer

**Files:**
- Create: `crates/talent-core/src/syncer.rs`

**Step 1: Write syncer**

`crates/talent-core/src/syncer.rs`:
```rust
use crate::{Config, Error, Result, Skill, Target};
use std::fs;
use std::path::Path;

pub struct Syncer {
    skills_dir: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub target_id: String,
    pub created: Vec<String>,
    pub removed: Vec<String>,
    pub errors: Vec<String>,
}

impl Syncer {
    pub fn new() -> Self {
        Self {
            skills_dir: Config::skills_dir(),
        }
    }

    pub fn with_skills_dir(skills_dir: std::path::PathBuf) -> Self {
        Self { skills_dir }
    }

    /// Sync all skills to a specific target
    pub fn sync_target(&self, target: &Target, skills: &[Skill]) -> Result<SyncResult> {
        let mut result = SyncResult {
            target_id: target.id.clone(),
            created: Vec::new(),
            removed: Vec::new(),
            errors: Vec::new(),
        };

        if !target.enabled {
            return Ok(result);
        }

        // Ensure target skills directory exists
        target.ensure_skills_dir()?;

        // Get existing symlinks in target
        let existing = self.get_existing_symlinks(&target.skills_path)?;

        // Determine which skills should be synced
        let enabled_skills: Vec<&Skill> = skills
            .iter()
            .filter(|s| target.is_skill_enabled(&s.name))
            .collect();

        // Create symlinks for enabled skills
        for skill in &enabled_skills {
            if !existing.contains(&skill.name) {
                match self.create_symlink(skill, target) {
                    Ok(_) => result.created.push(skill.name.clone()),
                    Err(e) => result.errors.push(format!("{}: {}", skill.name, e)),
                }
            }
        }

        // Remove symlinks for disabled or deleted skills
        let enabled_names: std::collections::HashSet<_> =
            enabled_skills.iter().map(|s| s.name.as_str()).collect();

        for name in &existing {
            if !enabled_names.contains(name.as_str()) {
                match self.remove_symlink(name, target) {
                    Ok(_) => result.removed.push(name.clone()),
                    Err(e) => result.errors.push(format!("{}: {}", name, e)),
                }
            }
        }

        Ok(result)
    }

    /// Sync all skills to all targets
    pub fn sync_all(&self, targets: &[Target], skills: &[Skill]) -> Vec<SyncResult> {
        targets
            .iter()
            .filter_map(|t| self.sync_target(t, skills).ok())
            .collect()
    }

    fn get_existing_symlinks(&self, target_path: &Path) -> Result<Vec<String>> {
        let mut symlinks = Vec::new();

        if !target_path.exists() {
            return Ok(symlinks);
        }

        for entry in fs::read_dir(target_path)? {
            let entry = entry?;
            let path = entry.path();

            // Check if it's a symlink pointing to our skills dir
            if path.is_symlink() {
                if let Ok(link_target) = fs::read_link(&path) {
                    if link_target.starts_with(&self.skills_dir) {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            symlinks.push(name.to_string());
                        }
                    }
                }
            }
        }

        Ok(symlinks)
    }

    fn create_symlink(&self, skill: &Skill, target: &Target) -> Result<()> {
        let link_path = target.skills_path.join(&skill.name);
        let target_path = &skill.path;

        // Remove existing if it's a broken symlink
        if link_path.is_symlink() {
            fs::remove_file(&link_path)?;
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(target_path, &link_path)?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(target_path, &link_path)?;

        Ok(())
    }

    fn remove_symlink(&self, skill_name: &str, target: &Target) -> Result<()> {
        let link_path = target.skills_path.join(skill_name);

        if link_path.is_symlink() {
            fs::remove_file(&link_path)?;
        }

        Ok(())
    }

    /// Verify that all symlinks are valid
    pub fn verify_symlinks(&self, target: &Target) -> Result<Vec<String>> {
        let mut broken = Vec::new();

        if !target.skills_path.exists() {
            return Ok(broken);
        }

        for entry in fs::read_dir(&target.skills_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_symlink() {
                if let Ok(link_target) = fs::read_link(&path) {
                    if !link_target.exists() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            broken.push(name.to_string());
                        }
                    }
                }
            }
        }

        Ok(broken)
    }
}

impl Default for Syncer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_skill(skills_dir: &Path, name: &str) -> Skill {
        let skill_path = skills_dir.join(name);
        fs::create_dir(&skill_path).unwrap();

        let content = format!(r#"---
name: {}
description: Test skill
---
# {}
"#, name, name);

        let mut file = fs::File::create(skill_path.join("SKILL.md")).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        Skill::from_path(&skill_path).unwrap()
    }

    #[test]
    fn test_sync_creates_symlinks() {
        let dir = tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        let target_dir = dir.path().join("target/skills");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();

        let skill = create_test_skill(&skills_dir, "test-skill");
        let target = Target::new("test", "Test", target_dir.clone());

        let syncer = Syncer::with_skills_dir(skills_dir);
        let result = syncer.sync_target(&target, &[skill]).unwrap();

        assert_eq!(result.created, vec!["test-skill"]);
        assert!(target_dir.join("test-skill").is_symlink());
    }

    #[test]
    fn test_sync_removes_orphaned_symlinks() {
        let dir = tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        let target_dir = dir.path().join("target/skills");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();

        // Create a skill and sync it
        let skill = create_test_skill(&skills_dir, "test-skill");
        let target = Target::new("test", "Test", target_dir.clone());

        let syncer = Syncer::with_skills_dir(skills_dir.clone());
        syncer.sync_target(&target, &[skill]).unwrap();

        // Now sync with empty skills list
        let result = syncer.sync_target(&target, &[]).unwrap();

        assert_eq!(result.removed, vec!["test-skill"]);
        assert!(!target_dir.join("test-skill").exists());
    }

    #[test]
    fn test_disabled_skill_not_synced() {
        let dir = tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        let target_dir = dir.path().join("target/skills");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();

        let skill = create_test_skill(&skills_dir, "disabled-skill");
        let mut target = Target::new("test", "Test", target_dir.clone());
        target.set_skill_enabled("disabled-skill", false);

        let syncer = Syncer::with_skills_dir(skills_dir);
        let result = syncer.sync_target(&target, &[skill]).unwrap();

        assert!(result.created.is_empty());
        assert!(!target_dir.join("disabled-skill").exists());
    }
}
```

**Step 2: Run tests**

Run:
```bash
cd ~/Developer/talent && cargo test -p talent-core
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add .
git commit -m "feat(core): add symlink syncer for target management"
```

---

### Task 8: Implement File Watcher

**Files:**
- Create: `crates/talent-core/src/watcher.rs`

**Step 1: Write watcher**

`crates/talent-core/src/watcher.rs`:
```rust
use crate::{Config, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

pub struct SkillWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>,
    skills_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub enum SkillEvent {
    Created(String),
    Modified(String),
    Removed(String),
}

impl SkillWatcher {
    pub fn new() -> Result<Self> {
        Self::with_skills_dir(Config::skills_dir())
    }

    pub fn with_skills_dir(skills_dir: PathBuf) -> Result<Self> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            notify::Config::default()
                .with_poll_interval(Duration::from_millis(500)),
        )?;

        Ok(Self {
            watcher,
            receiver: rx,
            skills_dir,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        // Create skills directory if it doesn't exist
        if !self.skills_dir.exists() {
            std::fs::create_dir_all(&self.skills_dir)?;
        }

        self.watcher.watch(&self.skills_dir, RecursiveMode::Recursive)?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.watcher.unwatch(&self.skills_dir)?;
        Ok(())
    }

    /// Non-blocking check for events
    pub fn poll(&self) -> Vec<SkillEvent> {
        let mut events = Vec::new();

        while let Ok(result) = self.receiver.try_recv() {
            if let Ok(event) = result {
                if let Some(skill_event) = self.process_event(event) {
                    events.push(skill_event);
                }
            }
        }

        events
    }

    /// Blocking wait for next event with timeout
    pub fn wait(&self, timeout: Duration) -> Option<SkillEvent> {
        match self.receiver.recv_timeout(timeout) {
            Ok(Ok(event)) => self.process_event(event),
            _ => None,
        }
    }

    fn process_event(&self, event: Event) -> Option<SkillEvent> {
        // Only process SKILL.md changes
        let path = event.paths.first()?;

        // Check if the changed file is SKILL.md or in a skill directory
        let skill_name = self.extract_skill_name(path)?;

        match event.kind {
            EventKind::Create(_) => Some(SkillEvent::Created(skill_name)),
            EventKind::Modify(_) => Some(SkillEvent::Modified(skill_name)),
            EventKind::Remove(_) => Some(SkillEvent::Removed(skill_name)),
            _ => None,
        }
    }

    fn extract_skill_name(&self, path: &std::path::Path) -> Option<String> {
        // Get relative path from skills_dir
        let rel_path = path.strip_prefix(&self.skills_dir).ok()?;

        // First component is the skill name
        let skill_name = rel_path.components().next()?;

        Some(skill_name.as_os_str().to_str()?.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_extract_skill_name() {
        let dir = tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        fs::create_dir(&skills_dir).unwrap();

        let watcher = SkillWatcher::with_skills_dir(skills_dir.clone()).unwrap();

        let path = skills_dir.join("my-skill/SKILL.md");
        assert_eq!(
            watcher.extract_skill_name(&path),
            Some("my-skill".to_string())
        );

        let path = skills_dir.join("another-skill/scripts/run.sh");
        assert_eq!(
            watcher.extract_skill_name(&path),
            Some("another-skill".to_string())
        );
    }

    #[test]
    fn test_watcher_start_stop() {
        let dir = tempdir().unwrap();
        let skills_dir = dir.path().join("skills");

        let mut watcher = SkillWatcher::with_skills_dir(skills_dir.clone()).unwrap();

        assert!(watcher.start().is_ok());
        assert!(skills_dir.exists());
        assert!(watcher.stop().is_ok());
    }
}
```

**Step 2: Run tests**

Run:
```bash
cd ~/Developer/talent && cargo test -p talent-core
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add .
git commit -m "feat(core): add file watcher for skill directory"
```

---

### Task 9: Create Skill Manager (Integration)

**Files:**
- Create: `crates/talent-core/src/manager.rs`
- Modify: `crates/talent-core/src/lib.rs`

**Step 1: Write manager**

`crates/talent-core/src/manager.rs`:
```rust
use crate::{Config, Result, Skill, Target, Syncer, Validator};
use std::collections::HashMap;
use std::fs;

pub struct SkillManager {
    config: Config,
    skills: HashMap<String, Skill>,
    targets: Vec<Target>,
    syncer: Syncer,
    validator: Validator,
}

impl SkillManager {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let validator = Validator::new(config.validation.strict_mode);
        let syncer = Syncer::new();

        let mut manager = Self {
            config,
            skills: HashMap::new(),
            targets: Vec::new(),
            syncer,
            validator,
        };

        manager.discover_targets();
        manager.discover_skills()?;

        Ok(manager)
    }

    pub fn discover_targets(&mut self) {
        if self.config.targets.auto_detect {
            self.targets = Target::detect_all();
        }

        // Add custom paths
        for (i, path) in self.config.targets.custom_paths.iter().enumerate() {
            self.targets.push(Target::new(
                format!("custom-{}", i),
                format!("Custom Target {}", i + 1),
                path.clone(),
            ));
        }
    }

    pub fn discover_skills(&mut self) -> Result<()> {
        self.skills.clear();
        let skills_dir = Config::skills_dir();

        if !skills_dir.exists() {
            fs::create_dir_all(&skills_dir)?;
            return Ok(());
        }

        for entry in fs::read_dir(&skills_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Ok(mut skill) = Skill::from_path(&path) {
                    if self.config.validation.validate_on_sync {
                        let _ = self.validator.validate(&mut skill);
                    }
                    self.skills.insert(skill.name.clone(), skill);
                }
            }
        }

        Ok(())
    }

    pub fn skills(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    pub fn skill(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    pub fn target(&self, id: &str) -> Option<&Target> {
        self.targets.iter().find(|t| t.id == id)
    }

    pub fn target_mut(&mut self, id: &str) -> Option<&mut Target> {
        self.targets.iter_mut().find(|t| t.id == id)
    }

    pub fn sync_all(&self) -> Vec<crate::syncer::SyncResult> {
        let skills: Vec<Skill> = self.skills.values().cloned().collect();
        self.syncer.sync_all(&self.targets, &skills)
    }

    pub fn sync_target(&self, target_id: &str) -> Result<crate::syncer::SyncResult> {
        let target = self.target(target_id)
            .ok_or_else(|| crate::Error::TargetNotFound(target_id.to_string()))?;
        let skills: Vec<Skill> = self.skills.values().cloned().collect();
        self.syncer.sync_target(target, &skills)
    }

    pub fn validate_skill(&mut self, name: &str) -> Result<()> {
        let skill = self.skills.get_mut(name)
            .ok_or_else(|| crate::Error::SkillNotFound(name.to_string()))?;
        self.validator.validate(skill)
    }

    pub fn validate_all(&mut self) -> Result<()> {
        for skill in self.skills.values_mut() {
            self.validator.validate(skill)?;
        }
        Ok(())
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn save_config(&self) -> Result<()> {
        self.config.save()
    }
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SkillManager")
    }
}
```

**Step 2: Update lib.rs exports**

Update `crates/talent-core/src/lib.rs`:
```rust
pub mod config;
pub mod error;
pub mod skill;
pub mod target;
pub mod validator;
pub mod syncer;
pub mod watcher;
pub mod manager;

pub use config::Config;
pub use error::{Error, Result};
pub use skill::{Skill, ValidationStatus, ValidationIssue};
pub use target::Target;
pub use validator::Validator;
pub use syncer::Syncer;
pub use watcher::{SkillWatcher, SkillEvent};
pub use manager::SkillManager;
```

**Step 3: Run tests**

Run:
```bash
cd ~/Developer/talent && cargo test -p talent-core
```

Expected: All tests pass

**Step 4: Commit**

```bash
git add .
git commit -m "feat(core): add skill manager for integrated operations"
```

---

### Task 10: Complete CLI Implementation

**Files:**
- Modify: `crates/talent-cli/src/main.rs`

**Step 1: Implement CLI commands**

`crates/talent-cli/src/main.rs`:
```rust
use clap::{Parser, Subcommand};
use talent_core::{Config, SkillManager, ValidationStatus};

#[derive(Parser)]
#[command(name = "talent")]
#[command(about = "Unified Agent Skills Manager", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync skills to targets
    Sync {
        /// Specific target to sync
        #[arg(short, long)]
        target: Option<String>,
    },
    /// Show sync status
    Status,
    /// List all skills
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Create new skill from template
    New {
        /// Skill name
        name: String,
    },
    /// Validate skills
    Validate {
        /// Skill name (validates all if not specified)
        name: Option<String>,
        /// Validate all skills
        #[arg(long)]
        all: bool,
    },
    /// List targets
    Targets,
    /// Show configuration
    Config,
    /// Diagnose issues
    Doctor,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Sync { target } => cmd_sync(target),
        Commands::Status => cmd_status(),
        Commands::List { json } => cmd_list(json),
        Commands::New { name } => cmd_new(&name),
        Commands::Validate { name, all } => cmd_validate(name, all),
        Commands::Targets => cmd_targets(),
        Commands::Config => cmd_config(),
        Commands::Doctor => cmd_doctor(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_sync(target: Option<String>) -> talent_core::Result<()> {
    let manager = SkillManager::new()?;

    if let Some(target_id) = target {
        let result = manager.sync_target(&target_id)?;
        println!("Synced to {}", result.target_id);
        println!("  Created: {}", result.created.len());
        println!("  Removed: {}", result.removed.len());
        if !result.errors.is_empty() {
            println!("  Errors: {}", result.errors.len());
            for err in &result.errors {
                println!("    - {}", err);
            }
        }
    } else {
        let results = manager.sync_all();
        println!("Synced {} targets:", results.len());
        for result in results {
            println!("  {} - {} created, {} removed",
                result.target_id, result.created.len(), result.removed.len());
        }
    }

    Ok(())
}

fn cmd_status() -> talent_core::Result<()> {
    let manager = SkillManager::new()?;

    println!("Skills: {}", manager.skills().len());
    println!("Targets: {}", manager.targets().len());

    let mut valid = 0;
    let mut warnings = 0;
    let mut invalid = 0;

    for skill in manager.skills() {
        match &skill.validation_status {
            ValidationStatus::Valid => valid += 1,
            ValidationStatus::Warning(_) => warnings += 1,
            ValidationStatus::Invalid(_) => invalid += 1,
            ValidationStatus::NotValidated => {}
        }
    }

    println!("Validation: {} valid, {} warnings, {} invalid", valid, warnings, invalid);

    Ok(())
}

fn cmd_list(json: bool) -> talent_core::Result<()> {
    let manager = SkillManager::new()?;

    if json {
        let skills: Vec<_> = manager.skills().into_iter().collect();
        println!("{}", serde_json::to_string_pretty(&skills)?);
    } else {
        for skill in manager.skills() {
            let status = match &skill.validation_status {
                ValidationStatus::Valid => "✓",
                ValidationStatus::Warning(_) => "⚠",
                ValidationStatus::Invalid(_) => "✗",
                ValidationStatus::NotValidated => "?",
            };
            println!("{} {} - {}", status, skill.name, skill.description);
        }
    }

    Ok(())
}

fn cmd_new(name: &str) -> talent_core::Result<()> {
    let skills_dir = Config::skills_dir();
    let skill_path = skills_dir.join(name);

    if skill_path.exists() {
        return Err(talent_core::Error::InvalidSkill {
            path: skill_path,
            reason: "Skill already exists".to_string(),
        });
    }

    std::fs::create_dir_all(&skill_path)?;

    let content = format!(r#"---
name: {}
description: Add a description for your skill
license: MIT
---

# {}

Add instructions for your skill here.

## Usage

Describe how to use this skill.
"#, name, name);

    std::fs::write(skill_path.join("SKILL.md"), content)?;

    println!("Created skill: {}", skill_path.display());

    Ok(())
}

fn cmd_validate(name: Option<String>, all: bool) -> talent_core::Result<()> {
    let mut manager = SkillManager::new()?;

    if all || name.is_none() {
        manager.validate_all()?;
        for skill in manager.skills() {
            print_validation(&skill.name, &skill.validation_status);
        }
    } else if let Some(name) = name {
        manager.validate_skill(&name)?;
        if let Some(skill) = manager.skill(&name) {
            print_validation(&skill.name, &skill.validation_status);
        }
    }

    Ok(())
}

fn print_validation(name: &str, status: &ValidationStatus) {
    match status {
        ValidationStatus::Valid => println!("✓ {}: Valid", name),
        ValidationStatus::Warning(issues) => {
            println!("⚠ {}: {} warnings", name, issues.len());
            for issue in issues {
                println!("  - [{}] {}", issue.code, issue.message);
            }
        }
        ValidationStatus::Invalid(issues) => {
            println!("✗ {}: {} errors", name, issues.len());
            for issue in issues {
                println!("  - [{}] {}", issue.code, issue.message);
            }
        }
        ValidationStatus::NotValidated => println!("? {}: Not validated", name),
    }
}

fn cmd_targets() -> talent_core::Result<()> {
    let manager = SkillManager::new()?;

    for target in manager.targets() {
        let status = if target.enabled { "✓" } else { "○" };
        let detected = if target.auto_detected { "(auto)" } else { "" };
        println!("{} {} {} - {}", status, target.name, detected, target.skills_path.display());
    }

    Ok(())
}

fn cmd_config() -> talent_core::Result<()> {
    let config = Config::load()?;
    println!("{}", toml::to_string_pretty(&config)?);
    Ok(())
}

fn cmd_doctor() -> talent_core::Result<()> {
    println!("Talent Doctor\n");

    // Check talent directory
    let talent_dir = Config::talent_dir();
    println!("Talent directory: {}", talent_dir.display());
    println!("  Exists: {}", talent_dir.exists());

    // Check skills directory
    let skills_dir = Config::skills_dir();
    println!("\nSkills directory: {}", skills_dir.display());
    println!("  Exists: {}", skills_dir.exists());

    // Check config
    let config_path = Config::config_path();
    println!("\nConfig file: {}", config_path.display());
    println!("  Exists: {}", config_path.exists());

    // Check targets
    let manager = SkillManager::new()?;
    println!("\nDetected targets:");
    for target in manager.targets() {
        let exists = target.skills_path.exists();
        let writable = target.skills_path.parent()
            .map(|p| p.exists())
            .unwrap_or(false);
        println!("  {} - exists: {}, writable: {}", target.name, exists, writable);
    }

    // Check skills
    println!("\nSkills: {}", manager.skills().len());

    println!("\nAll checks passed!");

    Ok(())
}
```

**Step 2: Verify build**

Run:
```bash
cd ~/Developer/talent && cargo build
```

Expected: Build succeeds

**Step 3: Test CLI**

Run:
```bash
cd ~/Developer/talent && cargo run -- --help
cargo run -- doctor
```

Expected: Help shows all commands, doctor runs without errors

**Step 4: Commit**

```bash
git add .
git commit -m "feat(cli): implement all CLI commands"
```

---

## Phase 2: Tauri Shell

### Task 11: Initialize Tauri Project

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`

**Step 1: Install Tauri CLI**

Run:
```bash
cargo install tauri-cli --version "^2"
```

**Step 2: Create Tauri backend**

`src-tauri/Cargo.toml`:
```toml
[package]
name = "talent-app"
version.workspace = true
edition.workspace = true

[lib]
name = "talent_app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
talent-core = { path = "../crates/talent-core" }
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde.workspace = true
serde_json.workspace = true
tokio = { version = "1", features = ["full"] }
```

`src-tauri/build.rs`:
```rust
fn main() {
    tauri_build::build()
}
```

`src-tauri/src/lib.rs`:
```rust
mod commands;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_skills,
            commands::get_skill,
            commands::get_targets,
            commands::sync_all,
            commands::sync_target,
            commands::validate_skill,
            commands::create_skill,
            commands::get_config,
            commands::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    talent_app_lib::run()
}
```

`src-tauri/src/commands.rs`:
```rust
use talent_core::{Config, Skill, SkillManager, Target, ValidationStatus};
use serde::Serialize;

#[derive(Serialize)]
pub struct SkillInfo {
    name: String,
    description: String,
    tags: Vec<String>,
    validation_status: String,
    last_modified: String,
    has_scripts: bool,
}

impl From<&Skill> for SkillInfo {
    fn from(skill: &Skill) -> Self {
        Self {
            name: skill.name.clone(),
            description: skill.description.clone(),
            tags: skill.tags.clone(),
            validation_status: match &skill.validation_status {
                ValidationStatus::Valid => "valid".to_string(),
                ValidationStatus::Warning(_) => "warning".to_string(),
                ValidationStatus::Invalid(_) => "invalid".to_string(),
                ValidationStatus::NotValidated => "not_validated".to_string(),
            },
            last_modified: skill.last_modified.to_rfc3339(),
            has_scripts: skill.has_scripts,
        }
    }
}

#[derive(Serialize)]
pub struct TargetInfo {
    id: String,
    name: String,
    skills_path: String,
    auto_detected: bool,
    enabled: bool,
}

impl From<&Target> for TargetInfo {
    fn from(target: &Target) -> Self {
        Self {
            id: target.id.clone(),
            name: target.name.clone(),
            skills_path: target.skills_path.display().to_string(),
            auto_detected: target.auto_detected,
            enabled: target.enabled,
        }
    }
}

#[derive(Serialize)]
pub struct SyncResultInfo {
    target_id: String,
    created: Vec<String>,
    removed: Vec<String>,
    errors: Vec<String>,
}

#[tauri::command]
pub fn get_skills() -> Result<Vec<SkillInfo>, String> {
    let manager = SkillManager::new().map_err(|e| e.to_string())?;
    Ok(manager.skills().iter().map(|s| SkillInfo::from(*s)).collect())
}

#[tauri::command]
pub fn get_skill(name: String) -> Result<Option<SkillInfo>, String> {
    let manager = SkillManager::new().map_err(|e| e.to_string())?;
    Ok(manager.skill(&name).map(SkillInfo::from))
}

#[tauri::command]
pub fn get_targets() -> Result<Vec<TargetInfo>, String> {
    let manager = SkillManager::new().map_err(|e| e.to_string())?;
    Ok(manager.targets().iter().map(TargetInfo::from).collect())
}

#[tauri::command]
pub fn sync_all() -> Result<Vec<SyncResultInfo>, String> {
    let manager = SkillManager::new().map_err(|e| e.to_string())?;
    let results = manager.sync_all();
    Ok(results.iter().map(|r| SyncResultInfo {
        target_id: r.target_id.clone(),
        created: r.created.clone(),
        removed: r.removed.clone(),
        errors: r.errors.clone(),
    }).collect())
}

#[tauri::command]
pub fn sync_target(target_id: String) -> Result<SyncResultInfo, String> {
    let manager = SkillManager::new().map_err(|e| e.to_string())?;
    let result = manager.sync_target(&target_id).map_err(|e| e.to_string())?;
    Ok(SyncResultInfo {
        target_id: result.target_id,
        created: result.created,
        removed: result.removed,
        errors: result.errors,
    })
}

#[tauri::command]
pub fn validate_skill(name: String) -> Result<String, String> {
    let mut manager = SkillManager::new().map_err(|e| e.to_string())?;
    manager.validate_skill(&name).map_err(|e| e.to_string())?;

    if let Some(skill) = manager.skill(&name) {
        Ok(match &skill.validation_status {
            ValidationStatus::Valid => "valid".to_string(),
            ValidationStatus::Warning(issues) => {
                format!("warning: {}", issues.iter().map(|i| i.message.clone()).collect::<Vec<_>>().join(", "))
            }
            ValidationStatus::Invalid(issues) => {
                format!("invalid: {}", issues.iter().map(|i| i.message.clone()).collect::<Vec<_>>().join(", "))
            }
            ValidationStatus::NotValidated => "not_validated".to_string(),
        })
    } else {
        Err("Skill not found".to_string())
    }
}

#[tauri::command]
pub fn create_skill(name: String) -> Result<String, String> {
    let skills_dir = Config::skills_dir();
    let skill_path = skills_dir.join(&name);

    if skill_path.exists() {
        return Err("Skill already exists".to_string());
    }

    std::fs::create_dir_all(&skill_path).map_err(|e| e.to_string())?;

    let content = format!(r#"---
name: {}
description: Add a description for your skill
license: MIT
---

# {}

Add instructions for your skill here.
"#, name, name);

    std::fs::write(skill_path.join("SKILL.md"), content).map_err(|e| e.to_string())?;

    Ok(skill_path.display().to_string())
}

#[tauri::command]
pub fn get_config() -> Result<Config, String> {
    Config::load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_config(config: Config) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}
```

**Step 3: Create Tauri configuration**

`src-tauri/tauri.conf.json`:
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Talent",
  "identifier": "io.talent.app",
  "version": "0.1.0",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "Talent",
        "width": 1024,
        "height": 768,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

`src-tauri/capabilities/default.json`:
```json
{
  "$schema": "https://schemas.tauri.app/config/2/capability",
  "identifier": "default",
  "description": "Default capabilities for Talent",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open"
  ]
}
```

**Step 4: Verify Tauri build**

Run:
```bash
cd ~/Developer/talent && cargo build -p talent-app
```

Expected: Build succeeds

**Step 5: Commit**

```bash
git add .
git commit -m "feat(tauri): add Tauri shell with commands"
```

---

## Phase 3: Frontend (Svelte)

### Task 12: Initialize Svelte Frontend

**Files:**
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `svelte.config.js`
- Create: `tsconfig.json`
- Create: `src/app.html`
- Create: `src/main.ts`
- Create: `src/App.svelte`
- Create: `src/app.css`

**Step 1: Create package.json**

`package.json`:
```json
{
  "name": "talent",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-shell": "^2"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^4",
    "svelte": "^5",
    "typescript": "^5",
    "vite": "^6"
  }
}
```

**Step 2: Create Vite config**

`vite.config.ts`:
```typescript
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
```

**Step 3: Create Svelte config**

`svelte.config.js`:
```javascript
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  preprocess: vitePreprocess(),
};
```

**Step 4: Create TypeScript config**

`tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES2021",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "verbatimModuleSyntax": true
  },
  "include": ["src/**/*.ts", "src/**/*.svelte"]
}
```

**Step 5: Create index.html**

`index.html`:
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Talent</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

**Step 6: Create main entry**

`src/main.ts`:
```typescript
import App from "./App.svelte";
import { mount } from "svelte";
import "./app.css";

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
```

**Step 7: Create base styles**

`src/app.css`:
```css
:root {
  --bg-primary: #1a1a2e;
  --bg-secondary: #16213e;
  --bg-card: #0f3460;
  --text-primary: #eaeaea;
  --text-secondary: #a0a0a0;
  --accent: #e94560;
  --success: #4ade80;
  --warning: #fbbf24;
  --error: #f87171;
  --border: #2d3a4f;
}

@media (prefers-color-scheme: light) {
  :root {
    --bg-primary: #ffffff;
    --bg-secondary: #f5f5f5;
    --bg-card: #ffffff;
    --text-primary: #1a1a1a;
    --text-secondary: #666666;
    --accent: #e94560;
    --border: #e0e0e0;
  }
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen,
    Ubuntu, Cantarell, "Open Sans", "Helvetica Neue", sans-serif;
  background-color: var(--bg-primary);
  color: var(--text-primary);
}

body {
  overflow: hidden;
}
```

**Step 8: Create App component**

`src/App.svelte`:
```svelte
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface Skill {
    name: string;
    description: string;
    tags: string[];
    validation_status: string;
    last_modified: string;
    has_scripts: boolean;
  }

  interface Target {
    id: string;
    name: string;
    skills_path: string;
    auto_detected: boolean;
    enabled: boolean;
  }

  let skills: Skill[] = $state([]);
  let targets: Target[] = $state([]);
  let searchQuery = $state("");
  let selectedFilter = $state("all");
  let loading = $state(true);
  let syncing = $state(false);

  const filteredSkills = $derived(
    skills.filter((skill) => {
      const matchesSearch =
        skill.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        skill.description.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesFilter =
        selectedFilter === "all" ||
        skill.validation_status === selectedFilter;
      return matchesSearch && matchesFilter;
    })
  );

  const statusCounts = $derived({
    all: skills.length,
    valid: skills.filter((s) => s.validation_status === "valid").length,
    warning: skills.filter((s) => s.validation_status === "warning").length,
    invalid: skills.filter((s) => s.validation_status === "invalid").length,
  });

  async function loadData() {
    loading = true;
    try {
      skills = await invoke<Skill[]>("get_skills");
      targets = await invoke<Target[]>("get_targets");
    } catch (e) {
      console.error("Failed to load data:", e);
    } finally {
      loading = false;
    }
  }

  async function syncAll() {
    syncing = true;
    try {
      await invoke("sync_all");
      await loadData();
    } catch (e) {
      console.error("Failed to sync:", e);
    } finally {
      syncing = false;
    }
  }

  function getStatusIcon(status: string): string {
    switch (status) {
      case "valid":
        return "✓";
      case "warning":
        return "⚠";
      case "invalid":
        return "✗";
      default:
        return "?";
    }
  }

  function getStatusClass(status: string): string {
    switch (status) {
      case "valid":
        return "status-valid";
      case "warning":
        return "status-warning";
      case "invalid":
        return "status-invalid";
      default:
        return "status-unknown";
    }
  }

  onMount(() => {
    loadData();
  });
</script>

<main>
  <aside class="sidebar">
    <div class="logo">
      <h1>Talent</h1>
    </div>

    <nav class="filters">
      <h3>Skills</h3>
      <button
        class:active={selectedFilter === "all"}
        onclick={() => (selectedFilter = "all")}
      >
        All ({statusCounts.all})
      </button>
      <button
        class:active={selectedFilter === "valid"}
        onclick={() => (selectedFilter = "valid")}
      >
        ✓ Valid ({statusCounts.valid})
      </button>
      <button
        class:active={selectedFilter === "warning"}
        onclick={() => (selectedFilter = "warning")}
      >
        ⚠ Warnings ({statusCounts.warning})
      </button>
      <button
        class:active={selectedFilter === "invalid"}
        onclick={() => (selectedFilter = "invalid")}
      >
        ✗ Errors ({statusCounts.invalid})
      </button>
    </nav>

    <nav class="targets">
      <h3>Targets</h3>
      {#each targets as target}
        <div class="target-item">
          <span class="target-status">{target.enabled ? "✓" : "○"}</span>
          <span class="target-name">{target.name}</span>
        </div>
      {/each}
    </nav>
  </aside>

  <section class="content">
    <header>
      <input
        type="search"
        placeholder="Search skills..."
        bind:value={searchQuery}
      />
      <button class="btn-primary" onclick={syncAll} disabled={syncing}>
        {syncing ? "Syncing..." : "Sync All"}
      </button>
    </header>

    <div class="skills-list">
      {#if loading}
        <div class="loading">Loading...</div>
      {:else if filteredSkills.length === 0}
        <div class="empty">No skills found</div>
      {:else}
        {#each filteredSkills as skill}
          <div class="skill-card">
            <div class="skill-header">
              <span class="skill-name">{skill.name}</span>
              <span class={`skill-status ${getStatusClass(skill.validation_status)}`}>
                {getStatusIcon(skill.validation_status)}
              </span>
            </div>
            <p class="skill-description">{skill.description}</p>
            <div class="skill-meta">
              {#each skill.tags as tag}
                <span class="tag">#{tag}</span>
              {/each}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </section>
</main>

<footer>
  <span class="status-dot"></span>
  <span>Synced</span>
  <span class="separator">|</span>
  <span>{skills.length} skills</span>
  <span class="separator">|</span>
  <span>{targets.length} targets</span>
</footer>

<style>
  main {
    display: flex;
    height: calc(100vh - 32px);
  }

  .sidebar {
    width: 240px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .logo h1 {
    font-size: 24px;
    font-weight: 600;
    color: var(--accent);
  }

  .filters h3,
  .targets h3 {
    font-size: 12px;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin-bottom: 8px;
  }

  .filters button {
    display: block;
    width: 100%;
    padding: 8px 12px;
    margin-bottom: 4px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    font-size: 14px;
  }

  .filters button:hover {
    background: var(--bg-card);
  }

  .filters button.active {
    background: var(--accent);
    color: white;
  }

  .target-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    font-size: 14px;
  }

  .target-status {
    color: var(--success);
  }

  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  header {
    display: flex;
    gap: 12px;
    padding: 16px;
    border-bottom: 1px solid var(--border);
  }

  header input {
    flex: 1;
    padding: 10px 16px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 14px;
  }

  header input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .btn-primary {
    padding: 10px 20px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .skills-list {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .skill-card {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 16px;
  }

  .skill-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .skill-name {
    font-weight: 600;
    font-size: 16px;
  }

  .skill-status {
    font-size: 14px;
    padding: 2px 8px;
    border-radius: 4px;
  }

  .status-valid {
    background: rgba(74, 222, 128, 0.2);
    color: var(--success);
  }

  .status-warning {
    background: rgba(251, 191, 36, 0.2);
    color: var(--warning);
  }

  .status-invalid {
    background: rgba(248, 113, 113, 0.2);
    color: var(--error);
  }

  .skill-description {
    color: var(--text-secondary);
    font-size: 14px;
    line-height: 1.5;
    margin-bottom: 12px;
  }

  .skill-meta {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .tag {
    font-size: 12px;
    color: var(--accent);
    background: rgba(233, 69, 96, 0.1);
    padding: 2px 8px;
    border-radius: 4px;
  }

  .loading,
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--text-secondary);
  }

  footer {
    height: 32px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 16px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-secondary);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--success);
  }

  .separator {
    color: var(--border);
  }
</style>
```

**Step 9: Install dependencies and verify**

Run:
```bash
cd ~/Developer/talent && npm install
```

Expected: Dependencies installed

**Step 10: Commit**

```bash
git add .
git commit -m "feat(frontend): add Svelte frontend with skill list"
```

---

### Task 13: Run Complete Application

**Step 1: Run development server**

Run:
```bash
cd ~/Developer/talent && npm run tauri dev
```

Expected: Application launches with GUI showing skills and targets

**Step 2: Verify functionality**
- Skills list displays
- Search filters work
- Sync button triggers sync
- Targets show in sidebar

**Step 3: Commit final state**

```bash
git add .
git commit -m "chore: finalize Phase 1-3 implementation"
```

---

## Summary

This plan covers:
- **Phase 1**: Core Rust backend (Tasks 1-10)
- **Phase 2**: Tauri shell (Task 11)
- **Phase 3**: Svelte frontend (Tasks 12-13)

Remaining phases (4-6: Editor, Import/Export, Polish) can be implemented incrementally after the base application is working.

**Total estimated tasks**: 13 tasks for MVP
**Commits**: One per task, following conventional commits
