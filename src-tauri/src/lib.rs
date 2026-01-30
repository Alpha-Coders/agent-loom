//! Talent App - Tauri backend library
//!
//! This module provides the Tauri commands that expose talent-core functionality
//! to the Svelte frontend.

mod commands;
mod menu;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use talent_core::{ConflictResolution, SkillManager, ValidationStatus};

/// Skill information for the frontend
/// See https://agentskills.io/specification for field definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    // Required fields (per spec)
    pub name: String,
    pub folder_name: String,
    pub description: String,

    // Optional spec fields
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
    pub allowed_tools: Option<String>,

    // Legacy fields (not in spec, kept for backward compatibility)
    pub tags: Vec<String>,
    pub version: Option<String>,
    pub author: Option<String>,

    // Internal fields
    pub path: String,
    pub validation_status: String,
    pub validation_errors: Vec<String>,
}

impl From<&talent_core::Skill> for SkillInfo {
    fn from(skill: &talent_core::Skill) -> Self {
        Self {
            name: skill.meta.name.clone(),
            folder_name: skill.folder_name().to_string(),
            description: skill.meta.description.clone(),
            license: skill.meta.license.clone(),
            compatibility: skill.meta.compatibility.clone(),
            metadata: skill.meta.metadata.clone(),
            allowed_tools: skill.meta.allowed_tools.clone(),
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
}

/// Discovered skill for import UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredSkillInfo {
    pub name: String,
    pub description: String,
    pub source_path: String,
    pub source_target: String,
    pub has_conflict: bool,
    pub existing_description: Option<String>,
}

impl From<&talent_core::DiscoveredSkill> for DiscoveredSkillInfo {
    fn from(skill: &talent_core::DiscoveredSkill) -> Self {
        Self {
            name: skill.name.clone(),
            description: skill.description.clone(),
            source_path: skill.source_path.display().to_string(),
            source_target: skill.source_target.display_name().to_string(),
            has_conflict: skill.conflict.is_some(),
            existing_description: skill.conflict.as_ref().map(|c| c.existing_description.clone()),
        }
    }
}

/// Import selection from frontend
#[derive(Debug, Clone, Deserialize)]
pub struct ImportSelectionInfo {
    pub name: String,
    pub source_path: String,
    pub resolution: String,
}

impl ImportSelectionInfo {
    pub fn to_core(&self) -> talent_core::ImportSelection {
        talent_core::ImportSelection {
            name: self.name.clone(),
            source_path: PathBuf::from(&self.source_path),
            resolution: match self.resolution.as_str() {
                "skip" => ConflictResolution::Skip,
                "overwrite" => ConflictResolution::Overwrite,
                _ => ConflictResolution::Import,
            },
        }
    }
}

/// Scanned skill from external folder for import UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedSkillInfo {
    pub name: String,
    pub description: String,
    pub source_path: String,
    pub needs_fixes: bool,
    pub fixes_preview: Vec<String>,
    pub has_conflict: bool,
    pub existing_description: Option<String>,
}

impl From<&talent_core::ScannedSkill> for ScannedSkillInfo {
    fn from(skill: &talent_core::ScannedSkill) -> Self {
        Self {
            name: skill.name.clone(),
            description: skill.description.clone(),
            source_path: skill.source_path.display().to_string(),
            needs_fixes: skill.needs_fixes,
            fixes_preview: skill.fixes_preview.clone(),
            has_conflict: skill.conflict.is_some(),
            existing_description: skill.conflict.as_ref().map(|c| c.existing_description.clone()),
        }
    }
}

/// Import selection from folder scan UI
#[derive(Debug, Clone, Deserialize)]
pub struct FolderImportSelectionInfo {
    pub name: String,
    pub source_path: String,
    pub apply_fixes: bool,
    pub resolution: String,
}

impl FolderImportSelectionInfo {
    pub fn to_core(&self) -> talent_core::FolderImportSelection {
        talent_core::FolderImportSelection {
            name: self.name.clone(),
            source_path: PathBuf::from(&self.source_path),
            apply_fixes: self.apply_fixes,
            resolution: match self.resolution.as_str() {
                "skip" => ConflictResolution::Skip,
                "overwrite" => ConflictResolution::Overwrite,
                _ => ConflictResolution::Import,
            },
        }
    }
}

/// Import result for frontend
#[derive(Debug, Clone, Serialize)]
pub struct ImportResultInfo {
    pub imported: Vec<String>,
    pub skipped: Vec<String>,
    pub errors: Vec<(String, String)>,
    pub synced_to: usize,
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
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            manager: Mutex::new(manager),
        })
        .setup(|app| {
            // Create and set application menu
            let menu = menu::create_menu(app.handle())?;
            app.set_menu(menu)?;
            Ok(())
        })
        .on_menu_event(|app, event| {
            menu::handle_menu_event(app, &event);
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
            commands::rename_skill,
            commands::get_stats,
            commands::get_skill_content,
            commands::save_skill_content,
            commands::discover_importable_skills,
            commands::import_skills,
            commands::import_all_skills,
            commands::is_filemerge_available,
            commands::launch_filemerge,
            // Skill fixing
            commands::fix_skill,
            commands::fix_all_skills,
            // Target management
            commands::toggle_target,
            commands::set_target_enabled,
            commands::add_custom_target,
            commands::remove_custom_target,
            commands::get_available_target_types,
            // Menu state
            commands::set_save_menu_enabled,
            // Folder import
            commands::scan_folder_for_skills,
            commands::import_from_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
