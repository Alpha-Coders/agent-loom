//! Talent Core - Agent Skills Manager Library
//!
//! This crate provides the core functionality for managing agent skills:
//! - Skill discovery and parsing
//! - Target CLI detection
//! - Symlink synchronization
//! - File watching for auto-sync

pub mod config;
pub mod error;

pub use config::Config;
pub use error::{Error, Result};
