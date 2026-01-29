//! Configuration module for Talent
//!
//! Manages application configuration including:
//! - Skills storage directory location
//! - Target CLI configurations (auto-detected and manual)
//! - User preferences

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Default name for the config file
const CONFIG_FILE_NAME: &str = "config.toml";

/// Default name for the skills directory
const DEFAULT_SKILLS_DIR: &str = "skills";

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Directory where skills are stored (default: ~/.talent/skills/)
    pub skills_dir: PathBuf,

    /// Target CLI configurations (key = target name)
    pub targets: HashMap<String, TargetConfig>,

    /// User preferences
    pub preferences: Preferences,
}

/// Configuration for a target CLI tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Whether this target is enabled for syncing
    pub enabled: bool,

    /// Path to the target's skills directory
    /// If None, will attempt auto-detection
    pub skills_path: Option<PathBuf>,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    /// Whether to watch for file changes and auto-sync
    pub auto_sync: bool,

    /// Whether to validate skills before syncing
    pub validate_on_sync: bool,

    /// File watch debounce duration in milliseconds
    pub watch_debounce_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            skills_dir: Self::default_skills_dir(),
            targets: HashMap::new(),
            preferences: Preferences::default(),
        }
    }
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            skills_path: None,
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            auto_sync: true,
            validate_on_sync: true,
            watch_debounce_ms: 500,
        }
    }
}

impl Config {
    /// Load configuration from the default location (~/.talent/config.toml)
    pub fn load() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        Self::load_from(&config_path)
    }

    /// Load configuration from a specific file path
    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path).map_err(|e| Error::io(path, e))?;

        toml::from_str(&contents).map_err(|e| Error::ConfigLoad {
            path: path.to_path_buf(),
            source: Box::new(e),
        })
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::default_config_path()?;
        self.save_to(&config_path)
    }

    /// Save configuration to a specific file path
    pub fn save_to(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| Error::create_dir(parent, e))?;
        }

        let contents = toml::to_string_pretty(self).map_err(Error::TomlSerialize)?;

        fs::write(path, contents).map_err(|e| Error::ConfigSave {
            path: path.to_path_buf(),
            source: Box::new(e),
        })
    }

    /// Get the default config file path (~/.talent/config.toml)
    pub fn default_config_path() -> Result<PathBuf> {
        Self::talent_dir().map(|d| d.join(CONFIG_FILE_NAME))
    }

    /// Get the talent configuration directory (~/.talent/)
    pub fn talent_dir() -> Result<PathBuf> {
        dirs::home_dir()
            .map(|h| h.join(".talent"))
            .ok_or(Error::ConfigDirNotFound)
    }

    /// Get the default skills directory (~/.talent/skills/)
    pub fn default_skills_dir() -> PathBuf {
        dirs::home_dir()
            .map(|h| h.join(".talent").join(DEFAULT_SKILLS_DIR))
            .unwrap_or_else(|| PathBuf::from(".talent").join(DEFAULT_SKILLS_DIR))
    }

    /// Ensure the skills directory exists
    pub fn ensure_skills_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.skills_dir).map_err(|e| Error::create_dir(&self.skills_dir, e))
    }

    /// Get target config, creating default if not present
    pub fn get_or_create_target(&mut self, name: &str) -> &mut TargetConfig {
        self.targets.entry(name.to_string()).or_default()
    }

    /// Enable a target for syncing
    pub fn enable_target(&mut self, name: &str) {
        self.get_or_create_target(name).enabled = true;
    }

    /// Disable a target for syncing
    pub fn disable_target(&mut self, name: &str) {
        self.get_or_create_target(name).enabled = false;
    }

    /// Get all enabled targets
    pub fn enabled_targets(&self) -> impl Iterator<Item = (&String, &TargetConfig)> {
        self.targets.iter().filter(|(_, cfg)| cfg.enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_has_sensible_values() {
        let config = Config::default();
        assert!(config.skills_dir.to_string_lossy().contains(".talent"));
        assert!(config.preferences.auto_sync);
        assert!(config.preferences.validate_on_sync);
        assert_eq!(config.preferences.watch_debounce_ms, 500);
    }

    #[test]
    fn load_returns_default_when_file_missing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");
        let config = Config::load_from(&config_path).unwrap();
        assert!(config.targets.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        config.preferences.auto_sync = false;
        config.enable_target("claude-code");

        config.save_to(&config_path).unwrap();
        let loaded = Config::load_from(&config_path).unwrap();

        assert!(!loaded.preferences.auto_sync);
        assert!(loaded.targets.contains_key("claude-code"));
        assert!(loaded.targets["claude-code"].enabled);
    }

    #[test]
    fn get_or_create_target_creates_default() {
        let mut config = Config::default();
        let target = config.get_or_create_target("new-target");
        assert!(target.enabled);
        assert!(target.skills_path.is_none());
    }

    #[test]
    fn enabled_targets_filters_correctly() {
        let mut config = Config::default();
        config.enable_target("enabled");
        config.disable_target("disabled");

        let enabled: Vec<_> = config.enabled_targets().collect();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].0, "enabled");
    }

    #[test]
    fn parses_valid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let toml_content = r#"
skills_dir = "/custom/skills"

[preferences]
auto_sync = false
validate_on_sync = true
watch_debounce_ms = 1000

[targets.claude-code]
enabled = true
skills_path = "/path/to/claude/skills"
"#;

        fs::write(&config_path, toml_content).unwrap();
        let config = Config::load_from(&config_path).unwrap();

        assert_eq!(config.skills_dir, PathBuf::from("/custom/skills"));
        assert!(!config.preferences.auto_sync);
        assert_eq!(config.preferences.watch_debounce_ms, 1000);
        assert!(config.targets.contains_key("claude-code"));
        assert_eq!(
            config.targets["claude-code"].skills_path,
            Some(PathBuf::from("/path/to/claude/skills"))
        );
    }
}
