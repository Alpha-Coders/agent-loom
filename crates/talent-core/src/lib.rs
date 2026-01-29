//! Talent Core - Agent Skills Manager Library
//!
//! This crate provides the core functionality for managing agent skills:
//! - Skill discovery and parsing
//! - Target CLI detection
//! - Symlink synchronization
//! - File watching for auto-sync

pub mod config;
pub mod error;
pub mod importer;
pub mod manager;
pub mod skill;
pub mod syncer;
pub mod target;
pub mod validator;
pub mod watcher;

pub use config::Config;
pub use error::{Error, Result};
pub use importer::{
    check_filemerge_available, open_filemerge, ConflictInfo, ConflictResolution, DiscoveredSkill,
    ImportResult, ImportSelection, Importer,
};
pub use manager::{ManagerStats, SkillManager};
pub use skill::{discover_skills, Skill, SkillMeta, ValidationStatus, SKILL_FILE_NAME};
pub use syncer::{SyncError, SyncResult, Syncer};
pub use target::{Target, TargetInfo, TargetKind};
pub use validator::Validator;
pub use watcher::{SkillEvent, SkillWatcher};
