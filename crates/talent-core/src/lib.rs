//! Talent Core - Agent Skills Manager Library
//!
//! This crate provides the core functionality for managing agent skills:
//! - Skill discovery and parsing
//! - Target CLI detection
//! - Symlink synchronization
//! - File watching for auto-sync

pub mod config;
pub mod error;
pub mod skill;
pub mod target;
pub mod validator;

pub use config::Config;
pub use error::{Error, Result};
pub use skill::{discover_skills, Skill, SkillMeta, ValidationStatus, SKILL_FILE_NAME};
pub use target::{Target, TargetInfo, TargetKind};
pub use validator::Validator;
