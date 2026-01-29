//! Target CLI model with auto-detection
//!
//! Targets are AI CLI tools that support skills/commands.
//! This module handles detecting installed CLIs and resolving their skills directories.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Known target CLI tools
/// See https://agentskills.io for the full list of compatible agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetKind {
    /// Claude Code CLI (Anthropic)
    ClaudeCode,
    /// Codex CLI (OpenAI)
    Codex,
    /// Gemini CLI (Google)
    Gemini,
    /// Cursor IDE
    Cursor,
    /// Amp CLI (Sourcegraph)
    Amp,
    /// Goose CLI (Block)
    Goose,
    /// Roo Code
    RooCode,
    /// OpenCode
    OpenCode,
    /// Vibe (Mistral AI)
    Vibe,
    /// Firebender
    Firebender,
    /// Mux (Coder)
    Mux,
    /// Autohand Code CLI
    Autohand,
}

impl TargetKind {
    /// Get all known target kinds
    pub fn all() -> &'static [TargetKind] {
        &[
            TargetKind::ClaudeCode,
            TargetKind::Codex,
            TargetKind::Gemini,
            TargetKind::Cursor,
            TargetKind::Amp,
            TargetKind::Goose,
            TargetKind::RooCode,
            TargetKind::OpenCode,
            TargetKind::Vibe,
            TargetKind::Firebender,
            TargetKind::Mux,
            TargetKind::Autohand,
        ]
    }

    /// Get the display name for this target
    pub fn display_name(&self) -> &'static str {
        match self {
            TargetKind::ClaudeCode => "Claude Code",
            TargetKind::Codex => "Codex",
            TargetKind::Gemini => "Gemini",
            TargetKind::Cursor => "Cursor",
            TargetKind::Amp => "Amp",
            TargetKind::Goose => "Goose",
            TargetKind::RooCode => "Roo Code",
            TargetKind::OpenCode => "OpenCode",
            TargetKind::Vibe => "Vibe",
            TargetKind::Firebender => "Firebender",
            TargetKind::Mux => "Mux",
            TargetKind::Autohand => "Autohand",
        }
    }

    /// Get the identifier string for this target
    pub fn id(&self) -> &'static str {
        match self {
            TargetKind::ClaudeCode => "claude-code",
            TargetKind::Codex => "codex",
            TargetKind::Gemini => "gemini",
            TargetKind::Cursor => "cursor",
            TargetKind::Amp => "amp",
            TargetKind::Goose => "goose",
            TargetKind::RooCode => "roo-code",
            TargetKind::OpenCode => "opencode",
            TargetKind::Vibe => "vibe",
            TargetKind::Firebender => "firebender",
            TargetKind::Mux => "mux",
            TargetKind::Autohand => "autohand",
        }
    }

    /// Get the config directory name (relative to home)
    fn config_dir_name(&self) -> &'static str {
        match self {
            TargetKind::ClaudeCode => ".claude",
            TargetKind::Codex => ".codex",
            TargetKind::Gemini => ".gemini",
            TargetKind::Cursor => ".cursor",
            TargetKind::Amp => ".amp",
            TargetKind::Goose => ".goose",
            TargetKind::RooCode => ".roo-code",
            TargetKind::OpenCode => ".opencode",
            TargetKind::Vibe => ".vibe",
            TargetKind::Firebender => ".firebender",
            TargetKind::Mux => ".mux",
            TargetKind::Autohand => ".autohand",
        }
    }

    /// Get the skills subdirectory name within the config dir
    fn skills_subdir(&self) -> &'static str {
        // All targets use "skills" directory
        "skills"
    }
}

/// A detected or configured target CLI
#[derive(Debug, Clone)]
pub struct Target {
    /// The kind of CLI tool
    pub kind: TargetKind,

    /// Path to the target's skills directory
    pub skills_path: PathBuf,

    /// Whether the target was auto-detected (vs manually configured)
    pub auto_detected: bool,

    /// Whether this target is enabled for syncing
    pub enabled: bool,
}

impl Target {
    /// Create a new target with a specific skills path
    pub fn new(kind: TargetKind, skills_path: PathBuf) -> Self {
        Self {
            kind,
            skills_path,
            auto_detected: false,
            enabled: true,
        }
    }

    /// Try to auto-detect a target by checking for its config directory
    pub fn detect(kind: TargetKind) -> Option<Self> {
        let home = dirs::home_dir()?;
        let config_dir = home.join(kind.config_dir_name());

        if config_dir.exists() {
            let skills_path = config_dir.join(kind.skills_subdir());
            Some(Self {
                kind,
                skills_path,
                auto_detected: true,
                enabled: true,
            })
        } else {
            None
        }
    }

    /// Detect all installed target CLIs
    pub fn detect_all() -> Vec<Self> {
        TargetKind::all()
            .iter()
            .filter_map(|kind| Target::detect(*kind))
            .collect()
    }

    /// Get the display name
    pub fn name(&self) -> &'static str {
        self.kind.display_name()
    }

    /// Get the identifier
    pub fn id(&self) -> &'static str {
        self.kind.id()
    }

    /// Check if the skills directory exists
    pub fn skills_dir_exists(&self) -> bool {
        self.skills_path.exists()
    }

    /// Ensure the skills directory exists, creating it if needed
    pub fn ensure_skills_dir(&self) -> Result<()> {
        if !self.skills_path.exists() {
            std::fs::create_dir_all(&self.skills_path)
                .map_err(|e| Error::create_dir(&self.skills_path, e))?;
        }
        Ok(())
    }

    /// Get the path where a skill symlink should be created
    pub fn skill_link_path(&self, skill_name: &str) -> PathBuf {
        self.skills_path.join(skill_name)
    }

    /// Validate that the target is accessible
    pub fn validate(&self) -> Result<()> {
        // Check if the parent config directory exists
        if let Some(parent) = self.skills_path.parent() {
            if !parent.exists() {
                return Err(Error::TargetNotFound {
                    name: self.name().to_string(),
                    path: parent.to_path_buf(),
                });
            }
        }
        Ok(())
    }
}

/// Information about a target for serialization/display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub id: String,
    pub name: String,
    pub skills_path: PathBuf,
    pub auto_detected: bool,
    pub enabled: bool,
    pub exists: bool,
}

impl From<&Target> for TargetInfo {
    fn from(target: &Target) -> Self {
        Self {
            id: target.id().to_string(),
            name: target.name().to_string(),
            skills_path: target.skills_path.clone(),
            auto_detected: target.auto_detected,
            enabled: target.enabled,
            exists: target.skills_dir_exists(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn target_kind_display_names() {
        assert_eq!(TargetKind::ClaudeCode.display_name(), "Claude Code");
        assert_eq!(TargetKind::Codex.display_name(), "Codex");
        assert_eq!(TargetKind::Gemini.display_name(), "Gemini");
    }

    #[test]
    fn target_kind_ids() {
        assert_eq!(TargetKind::ClaudeCode.id(), "claude-code");
        assert_eq!(TargetKind::Codex.id(), "codex");
        assert_eq!(TargetKind::Gemini.id(), "gemini");
    }

    #[test]
    fn target_kind_all_returns_all_variants() {
        let all = TargetKind::all();
        assert!(all.contains(&TargetKind::ClaudeCode));
        assert!(all.contains(&TargetKind::Codex));
        assert!(all.contains(&TargetKind::Gemini));
        assert!(all.contains(&TargetKind::Cursor));
        assert!(all.contains(&TargetKind::Amp));
        assert!(all.contains(&TargetKind::Goose));
        assert!(all.contains(&TargetKind::RooCode));
        assert!(all.contains(&TargetKind::OpenCode));
        assert!(all.contains(&TargetKind::Vibe));
        assert!(all.contains(&TargetKind::Firebender));
        assert!(all.contains(&TargetKind::Mux));
        assert!(all.contains(&TargetKind::Autohand));
        assert_eq!(all.len(), 12);
    }

    #[test]
    fn target_new_creates_with_custom_path() {
        let path = PathBuf::from("/custom/path");
        let target = Target::new(TargetKind::ClaudeCode, path.clone());

        assert_eq!(target.kind, TargetKind::ClaudeCode);
        assert_eq!(target.skills_path, path);
        assert!(!target.auto_detected);
        assert!(target.enabled);
    }

    #[test]
    fn target_skill_link_path() {
        let target = Target::new(TargetKind::ClaudeCode, PathBuf::from("/skills"));
        let link_path = target.skill_link_path("my-skill");
        assert_eq!(link_path, PathBuf::from("/skills/my-skill"));
    }

    #[test]
    fn target_ensure_skills_dir_creates_directory() {
        let temp = TempDir::new().unwrap();
        let skills_path = temp.path().join("skills");

        let target = Target::new(TargetKind::ClaudeCode, skills_path.clone());
        assert!(!skills_path.exists());

        target.ensure_skills_dir().unwrap();
        assert!(skills_path.exists());
    }

    #[test]
    fn target_info_from_target() {
        let target = Target::new(TargetKind::ClaudeCode, PathBuf::from("/test/skills"));
        let info: TargetInfo = (&target).into();

        assert_eq!(info.id, "claude-code");
        assert_eq!(info.name, "Claude Code");
        assert!(!info.auto_detected);
        assert!(info.enabled);
        assert!(!info.exists); // Path doesn't exist
    }

    #[test]
    fn target_validate_fails_for_missing_parent() {
        let target = Target::new(
            TargetKind::ClaudeCode,
            PathBuf::from("/nonexistent/parent/skills"),
        );

        let result = target.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::TargetNotFound { .. }
        ));
    }

    #[test]
    fn target_validate_succeeds_for_existing_parent() {
        let temp = TempDir::new().unwrap();
        let skills_path = temp.path().join("skills");

        let target = Target::new(TargetKind::ClaudeCode, skills_path);
        let result = target.validate();
        assert!(result.is_ok());
    }
}
