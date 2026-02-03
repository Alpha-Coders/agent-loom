//! AgentLoom Core Library
//!
//! This crate provides the core functionality for managing agent skills:
//! - Skill discovery and parsing
//! - Target CLI detection
//! - Symlink synchronization
//! - Validation

pub mod config;
pub mod error;
pub mod importer;
pub mod manager;
pub mod migration;
pub mod skill;
pub mod syncer;
pub mod target;
pub mod validator;

pub use config::Config;
pub use error::{Error, Result};
pub use migration::{migrate_if_needed, has_legacy_skills, legacy_skills_dir, MigrationResult};
pub use importer::{
    check_filemerge_available, open_filemerge, ConflictInfo, ConflictResolution, DiscoveredSkill,
    FolderImportSelection, ImportResult, ImportSelection, Importer, ScannedSkill,
};
pub use manager::{ManagerStats, SkillManager};
pub use skill::{
    discover_skills, normalize_frontmatter, to_kebab_case, NormalizeResult, Skill, SkillMeta,
    ValidationStatus, SKILL_FILE_NAME,
};
pub use syncer::{SyncError, SyncResult, Syncer};
pub use target::{Target, TargetInfo, TargetKind};
pub use validator::Validator;
