//! Tauri commands for the frontend

use crate::{AppState, SkillInfo, StatsInfo};
use talent_core::{SyncResult, TargetInfo};

/// Get all skills
#[tauri::command]
pub fn get_skills(state: tauri::State<'_, AppState>) -> Result<Vec<SkillInfo>, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.skills().iter().map(SkillInfo::from).collect())
}

/// Get all targets
#[tauri::command]
pub fn get_targets(state: tauri::State<'_, AppState>) -> Result<Vec<TargetInfo>, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.targets().iter().map(TargetInfo::from).collect())
}

/// Sync all skills to all targets
#[tauri::command]
pub fn sync_all(state: tauri::State<'_, AppState>) -> Result<Vec<SyncResult>, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.sync_all())
}

/// Create a new skill
#[tauri::command]
pub fn create_skill(
    state: tauri::State<'_, AppState>,
    name: String,
    description: String,
) -> Result<SkillInfo, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let skill = manager
        .create_skill(&name, &description)
        .map_err(|e| e.to_string())?;
    Ok(SkillInfo::from(skill))
}

/// Validate a specific skill
#[tauri::command]
pub fn validate_skill(state: tauri::State<'_, AppState>, name: String) -> Result<SkillInfo, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager.validate_skill(&name).map_err(|e| e.to_string())?;

    let skill = manager
        .get_skill(&name)
        .ok_or_else(|| format!("Skill not found: {name}"))?;
    Ok(SkillInfo::from(skill))
}

/// Validate all skills
#[tauri::command]
pub fn validate_all(state: tauri::State<'_, AppState>) -> Result<Vec<SkillInfo>, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager.validate_all();
    Ok(manager.skills().iter().map(SkillInfo::from).collect())
}

/// Refresh skills from disk
#[tauri::command]
pub fn refresh_skills(state: tauri::State<'_, AppState>) -> Result<Vec<SkillInfo>, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager.refresh_skills().map_err(|e| e.to_string())?;
    Ok(manager.skills().iter().map(SkillInfo::from).collect())
}

/// Delete a skill
#[tauri::command]
pub fn delete_skill(state: tauri::State<'_, AppState>, name: String) -> Result<(), String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager.delete_skill(&name).map_err(|e| e.to_string())
}

/// Get manager statistics
#[tauri::command]
pub fn get_stats(state: tauri::State<'_, AppState>) -> Result<StatsInfo, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let stats = manager.stats();
    Ok(StatsInfo {
        total_skills: stats.total_skills,
        valid_skills: stats.valid_skills,
        invalid_skills: stats.invalid_skills,
        total_targets: stats.total_targets,
        enabled_targets: stats.enabled_targets,
        is_watching: stats.is_watching,
    })
}

/// Get the raw content of a skill's SKILL.md file
#[tauri::command]
pub fn get_skill_content(state: tauri::State<'_, AppState>, name: String) -> Result<String, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager
        .get_skill_content(&name)
        .map_err(|e| e.to_string())
}

/// Save content to a skill's SKILL.md file
#[tauri::command]
pub fn save_skill_content(
    state: tauri::State<'_, AppState>,
    name: String,
    content: String,
) -> Result<SkillInfo, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager
        .save_skill_content(&name, &content)
        .map_err(|e| e.to_string())?;

    let skill = manager
        .get_skill(&name)
        .ok_or_else(|| format!("Skill not found: {name}"))?;
    Ok(SkillInfo::from(skill))
}
