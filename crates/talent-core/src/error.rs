//! Error types for Talent Core
//!
//! Uses thiserror for ergonomic error handling with automatic Display/Error impls.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Talent error types covering all failure modes
#[derive(Debug, Error)]
pub enum Error {
    // === Configuration Errors ===
    /// Failed to load or parse configuration file
    #[error("Failed to load config from {path}: {source}")]
    ConfigLoad {
        path: PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Failed to save configuration file
    #[error("Failed to save config to {path}: {source}")]
    ConfigSave {
        path: PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Configuration directory not found
    #[error("Could not determine config directory")]
    ConfigDirNotFound,

    // === Skill Errors ===
    /// Skill not found at specified path
    #[error("Skill not found: {0}")]
    SkillNotFound(PathBuf),

    /// Skill already exists with that name
    #[error("Skill already exists: {0}")]
    SkillAlreadyExists(String),

    /// Invalid skill name format
    #[error("Invalid skill name '{0}': must be kebab-case (lowercase letters, numbers, hyphens)")]
    InvalidSkillName(String),

    /// SKILL.md file missing from skill directory
    #[error("Missing SKILL.md in skill directory: {0}")]
    MissingSkillFile(PathBuf),

    /// Failed to parse skill frontmatter
    #[error("Invalid frontmatter in {path}: {message}")]
    InvalidFrontmatter { path: PathBuf, message: String },

    /// Skill validation failed
    #[error("Skill validation failed for '{name}': {message}")]
    ValidationFailed { name: String, message: String },

    // === Target Errors ===
    /// Target CLI not found on system
    #[error("Target CLI '{name}' not found at expected path: {path}")]
    TargetNotFound { name: String, path: PathBuf },

    /// Target skills directory doesn't exist and couldn't be created
    #[error("Could not access skills directory for '{name}': {path}")]
    TargetDirInaccessible { name: String, path: PathBuf },

    /// Target-related operation error
    #[error("Target error: {0}")]
    TargetError(String),

    /// Target already exists
    #[error("Target already exists: {0}")]
    TargetAlreadyExists(String),

    /// Unknown target type
    #[error("Unknown target type: {0}")]
    UnknownTargetType(String),

    // === Sync Errors ===
    /// Failed to create symlink
    #[error("Failed to create symlink from {link_source} to {link_target}: {message}")]
    SymlinkCreate {
        link_source: PathBuf,
        link_target: PathBuf,
        message: String,
    },

    /// Failed to remove symlink
    #[error("Failed to remove symlink at {path}: {message}")]
    SymlinkRemove { path: PathBuf, message: String },

    /// Symlink target already exists and is not a symlink
    #[error("Path exists and is not a symlink: {0}")]
    NotASymlink(PathBuf),

    // === File System Errors ===
    /// Generic I/O error with context
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to read directory
    #[error("Failed to read directory {path}: {source}")]
    ReadDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to create directory
    #[error("Failed to create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    // === Watcher Errors ===
    /// File watcher initialization failed
    #[error("Failed to initialize file watcher: {0}")]
    WatcherInit(String),

    /// File watcher error during operation
    #[error("File watcher error: {0}")]
    WatcherError(String),

    // === Serialization Errors ===
    /// YAML parsing/serialization error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// JSON parsing/serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parsing/serialization error
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// TOML serialization error
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

impl Error {
    /// Create an I/O error with path context
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    /// Create a read directory error
    pub fn read_dir(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::ReadDir {
            path: path.into(),
            source,
        }
    }

    /// Create a create directory error
    pub fn create_dir(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::CreateDir {
            path: path.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_formats_correctly() {
        let err = Error::SkillNotFound(PathBuf::from("/path/to/skill"));
        assert_eq!(err.to_string(), "Skill not found: /path/to/skill");
    }

    #[test]
    fn validation_error_includes_details() {
        let err = Error::ValidationFailed {
            name: "my-skill".to_string(),
            message: "missing required field 'name'".to_string(),
        };
        assert!(err.to_string().contains("my-skill"));
        assert!(err.to_string().contains("missing required field"));
    }

    #[test]
    fn io_helper_creates_error_with_path() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::io("/test/path", io_err);
        assert!(err.to_string().contains("/test/path"));
    }
}
