//! Tauri commands for the frontend

use crate::{AppState, DiscoveredSkillInfo, ImportResultInfo, ImportSelectionInfo, SkillInfo, StatsInfo};
use std::path::PathBuf;
use talent_core::{check_filemerge_available, open_filemerge, Importer, SyncResult, TargetInfo};

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

/// Rename a skill
#[tauri::command]
pub fn rename_skill(state: tauri::State<'_, AppState>, old_name: String, new_name: String) -> Result<SkillInfo, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let skill = manager.rename_skill(&old_name, &new_name).map_err(|e| e.to_string())?;
    Ok(SkillInfo::from(skill))
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

    // save_skill_content now returns the (possibly new) skill name
    // if the frontmatter name changed, the folder was automatically renamed
    let final_name = manager
        .save_skill_content(&name, &content)
        .map_err(|e| e.to_string())?;

    let skill = manager
        .get_skill(&final_name)
        .ok_or_else(|| format!("Skill not found: {final_name}"))?;
    Ok(SkillInfo::from(skill))
}

/// Discover importable skills from all target CLIs
#[tauri::command]
pub fn discover_importable_skills(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<DiscoveredSkillInfo>, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let importer = Importer::from_config(manager.config());
    let discovered = importer.discover_importable_skills(manager.targets());

    Ok(discovered.iter().map(DiscoveredSkillInfo::from).collect())
}

/// Import selected skills
#[tauri::command]
pub fn import_skills(
    state: tauri::State<'_, AppState>,
    selections: Vec<ImportSelectionInfo>,
) -> Result<ImportResultInfo, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let importer = Importer::from_config(manager.config());

    // Convert frontend selections to core types
    let core_selections: Vec<_> = selections.iter().map(|s| s.to_core()).collect();

    // Import the skills
    let mut result = importer.import_selections(&core_selections);

    // Refresh skills to pick up newly imported ones
    manager.refresh_skills().map_err(|e| e.to_string())?;

    // Validate all skills
    manager.validate_all();

    // Sync to all targets
    let sync_results = manager.sync_all();
    result.synced_to = sync_results.len();

    Ok(ImportResultInfo {
        imported: result.imported,
        skipped: result.skipped,
        errors: result.errors,
        synced_to: result.synced_to,
    })
}

/// Import all skills from target CLIs automatically
///
/// Discovers all importable skills and imports them with overwrite enabled.
/// Returns the import result with sync count.
#[tauri::command]
pub fn import_all_skills(state: tauri::State<'_, AppState>) -> Result<ImportResultInfo, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let importer = Importer::from_config(manager.config());

    // Discover all importable skills
    let discovered = importer.discover_importable_skills(manager.targets());

    if discovered.is_empty() {
        return Ok(ImportResultInfo {
            imported: vec![],
            skipped: vec![],
            errors: vec![],
            synced_to: 0,
        });
    }

    // Import all with overwrite enabled
    let mut result = talent_core::ImportResult {
        imported: Vec::new(),
        skipped: Vec::new(),
        errors: Vec::new(),
        synced_to: 0,
    };

    for skill in discovered {
        match importer.import_skill(&skill.source_path, &skill.name, true) {
            Ok(_) => result.imported.push(skill.name),
            Err(e) => result.errors.push((skill.name, e.to_string())),
        }
    }

    // Refresh skills to pick up newly imported ones
    manager.refresh_skills().map_err(|e| e.to_string())?;

    // Validate all skills
    manager.validate_all();

    // Sync to all targets (this creates symlinks where originals were removed)
    let sync_results = manager.sync_all();
    result.synced_to = sync_results.len();

    Ok(ImportResultInfo {
        imported: result.imported,
        skipped: result.skipped,
        errors: result.errors,
        synced_to: result.synced_to,
    })
}

/// Check if FileMerge (opendiff) is available
#[tauri::command]
pub fn is_filemerge_available() -> bool {
    check_filemerge_available()
}

/// Open FileMerge to compare two skill directories
#[tauri::command]
pub fn launch_filemerge(existing: String, incoming: String) -> Result<(), String> {
    let existing_path = PathBuf::from(existing);
    let incoming_path = PathBuf::from(incoming);

    open_filemerge(&existing_path, &incoming_path).map_err(|e| e.to_string())
}
