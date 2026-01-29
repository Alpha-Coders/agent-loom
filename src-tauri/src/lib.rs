//! Talent App - Tauri backend library
//!
//! This module provides the Tauri commands that expose talent-core functionality
//! to the Svelte frontend.

mod commands;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use talent_core::{SkillManager, ValidationStatus};

/// Skill information for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub path: String,
    pub validation_status: String,
    pub validation_errors: Vec<String>,
}

impl From<&talent_core::Skill> for SkillInfo {
    fn from(skill: &talent_core::Skill) -> Self {
        Self {
            name: skill.meta.name.clone(),
            description: skill.meta.description.clone(),
            tags: skill.meta.tags.clone(),
            version: skill.meta.version.clone(),
            author: skill.meta.author.clone(),
            path: skill.path.display().to_string(),
            validation_status: match skill.validation_status {
                ValidationStatus::Unknown => "unknown".to_string(),
                ValidationStatus::Valid => "valid".to_string(),
                ValidationStatus::Invalid => "invalid".to_string(),
            },
            validation_errors: skill.validation_errors.clone(),
        }
    }
}

/// Statistics for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsInfo {
    pub total_skills: usize,
    pub valid_skills: usize,
    pub invalid_skills: usize,
    pub total_targets: usize,
    pub enabled_targets: usize,
    pub is_watching: bool,
}

/// Application state shared across commands
pub struct AppState {
    pub manager: Mutex<SkillManager>,
}

/// Initialize and run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the skill manager
    let manager = SkillManager::new().expect("Failed to initialize SkillManager");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            manager: Mutex::new(manager),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_skills,
            commands::get_targets,
            commands::sync_all,
            commands::create_skill,
            commands::validate_skill,
            commands::validate_all,
            commands::refresh_skills,
            commands::delete_skill,
            commands::get_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
