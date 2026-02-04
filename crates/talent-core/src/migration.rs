//! Migration module for upgrading from previous versions
//!
//! Handles automatic migration of skills from old directory locations.

use crate::error::{Error, Result};
use crate::skill::SKILL_FILE_NAME;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Legacy directory name (pre-v1.0.4)
const LEGACY_DIR_NAME: &str = ".agentloom";

/// Current directory name (v1.0.4+)
const CURRENT_DIR_NAME: &str = ".agents";

/// Skills subdirectory name
const SKILLS_DIR: &str = "skills";

/// Result of a migration operation
#[derive(Debug, Clone, Default, Serialize)]
pub struct MigrationResult {
    /// Whether migration was performed
    pub migrated: bool,

    /// Number of skills migrated
    pub skills_count: usize,

    /// Names of migrated skills
    pub skill_names: Vec<String>,

    /// Path skills were migrated from
    pub from_path: Option<PathBuf>,

    /// Path skills were migrated to
    pub to_path: Option<PathBuf>,

    /// Errors encountered (non-fatal)
    pub errors: Vec<String>,
}

/// Check if migration from legacy directory is needed and perform it
///
/// This function:
/// 1. Checks if ~/.agentloom/skills/ exists with skills
/// 2. Checks if ~/.agents/skills/ is empty or doesn't exist
/// 3. If migration is needed, copies skills from old to new location
/// 4. Deletes the old ~/.agentloom directory after successful migration
pub fn migrate_if_needed() -> Result<MigrationResult> {
    let home = dirs::home_dir().ok_or(Error::ConfigDirNotFound)?;

    let legacy_dir = home.join(LEGACY_DIR_NAME);
    let legacy_skills_dir = legacy_dir.join(SKILLS_DIR);
    let current_skills_dir = home.join(CURRENT_DIR_NAME).join(SKILLS_DIR);

    // Check if legacy directory exists and has skills
    if !legacy_skills_dir.exists() {
        return Ok(MigrationResult::default());
    }

    let legacy_skills = find_skills_in_dir(&legacy_skills_dir);
    if legacy_skills.is_empty() {
        return Ok(MigrationResult::default());
    }

    // Check if current directory already has skills
    let current_skills = if current_skills_dir.exists() {
        find_skills_in_dir(&current_skills_dir)
    } else {
        Vec::new()
    };

    // If current directory already has skills, don't migrate automatically
    // (user may have already set things up manually)
    if !current_skills.is_empty() {
        return Ok(MigrationResult::default());
    }

    // Perform migration
    let mut result = perform_migration(&legacy_skills_dir, &current_skills_dir, &legacy_skills)?;

    // Delete the old directory if migration was successful (no errors)
    if result.migrated && result.errors.is_empty() && result.skills_count > 0 {
        if let Err(e) = fs::remove_dir_all(&legacy_dir) {
            result.errors.push(format!(
                "Migration successful but failed to delete old directory: {}",
                e
            ));
        }
    }

    Ok(result)
}

/// Find skill directories in a given path
fn find_skills_in_dir(dir: &Path) -> Vec<String> {
    let mut skills = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                // Check if it contains a SKILL.md file
                if path.join(SKILL_FILE_NAME).exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        skills.push(name.to_string());
                    }
                }
            }
        }
    }

    skills
}

/// Perform the actual migration
fn perform_migration(
    from_dir: &Path,
    to_dir: &Path,
    skill_names: &[String],
) -> Result<MigrationResult> {
    let mut result = MigrationResult {
        migrated: true,
        skills_count: 0,
        skill_names: Vec::new(),
        from_path: Some(from_dir.to_path_buf()),
        to_path: Some(to_dir.to_path_buf()),
        errors: Vec::new(),
    };

    // Create the destination directory
    fs::create_dir_all(to_dir).map_err(|e| Error::create_dir(to_dir, e))?;

    // Copy each skill
    for skill_name in skill_names {
        let from_skill = from_dir.join(skill_name);
        let to_skill = to_dir.join(skill_name);

        match copy_dir_recursive(&from_skill, &to_skill) {
            Ok(()) => {
                result.skills_count += 1;
                result.skill_names.push(skill_name.clone());
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Failed to migrate '{}': {}", skill_name, e));
            }
        }
    }

    Ok(result)
}

/// Recursively copy a directory and its contents
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).map_err(|e| Error::create_dir(dst, e))?;

    for entry in fs::read_dir(src).map_err(|e| Error::read_dir(src, e))? {
        let entry = entry.map_err(|e| Error::read_dir(src, e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| Error::io(&src_path, e))?;
        }
    }

    Ok(())
}

/// Get the legacy skills directory path
pub fn legacy_skills_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(LEGACY_DIR_NAME).join(SKILLS_DIR))
}

/// Check if legacy directory exists and has skills
pub fn has_legacy_skills() -> bool {
    legacy_skills_dir()
        .map(|dir| dir.exists() && !find_skills_in_dir(&dir).is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_skill(dir: &Path, name: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join(SKILL_FILE_NAME),
            format!(
                "---\nname: {}\ndescription: Test skill\n---\n\n# {}\n",
                name, name
            ),
        )
        .unwrap();
    }

    #[test]
    fn find_skills_finds_valid_skills() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        create_skill(&skills_dir, "skill-one");
        create_skill(&skills_dir, "skill-two");

        // Create an empty directory (not a skill)
        fs::create_dir_all(skills_dir.join("not-a-skill")).unwrap();

        let skills = find_skills_in_dir(&skills_dir);
        assert_eq!(skills.len(), 2);
        assert!(skills.contains(&"skill-one".to_string()));
        assert!(skills.contains(&"skill-two".to_string()));
    }

    #[test]
    fn find_skills_empty_dir() {
        let temp = TempDir::new().unwrap();
        let skills = find_skills_in_dir(temp.path());
        assert!(skills.is_empty());
    }

    #[test]
    fn find_skills_nonexistent_dir() {
        let skills = find_skills_in_dir(Path::new("/nonexistent/path"));
        assert!(skills.is_empty());
    }

    #[test]
    fn perform_migration_copies_skills() {
        let temp = TempDir::new().unwrap();
        let from_dir = temp.path().join("old");
        let to_dir = temp.path().join("new");

        fs::create_dir_all(&from_dir).unwrap();
        create_skill(&from_dir, "my-skill");
        create_skill(&from_dir, "another-skill");

        let skill_names = vec!["my-skill".to_string(), "another-skill".to_string()];
        let result = perform_migration(&from_dir, &to_dir, &skill_names).unwrap();

        assert!(result.migrated);
        assert_eq!(result.skills_count, 2);
        assert!(result.errors.is_empty());

        // Verify skills were copied
        assert!(to_dir.join("my-skill").join(SKILL_FILE_NAME).exists());
        assert!(to_dir.join("another-skill").join(SKILL_FILE_NAME).exists());

        // Verify originals still exist
        assert!(from_dir.join("my-skill").join(SKILL_FILE_NAME).exists());
    }

    #[test]
    fn migration_result_default() {
        let result = MigrationResult::default();
        assert!(!result.migrated);
        assert_eq!(result.skills_count, 0);
        assert!(result.skill_names.is_empty());
    }
}
