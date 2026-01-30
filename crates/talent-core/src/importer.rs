//! Skill importer for migrating skills from target CLIs
//!
//! Scans target CLI directories for importable skills and copies them
//! to Talent's central storage.

use crate::config::Config;
use crate::error::{Error, Result};
use crate::skill::{Skill, SKILL_FILE_NAME};
use crate::target::{Target, TargetKind};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// A skill discovered in a target CLI's skills directory
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredSkill {
    /// Skill name (from frontmatter or directory name)
    pub name: String,

    /// Skill description (from frontmatter)
    pub description: String,

    /// Path to the skill directory in the target
    pub source_path: PathBuf,

    /// Which target CLI this skill was found in
    pub source_target: TargetKind,

    /// Conflict information if skill already exists in Talent
    pub conflict: Option<ConflictInfo>,
}

/// Information about a conflicting existing skill
#[derive(Debug, Clone, Serialize)]
pub struct ConflictInfo {
    /// Path to the existing skill in Talent's storage
    pub existing_path: PathBuf,

    /// Description of the existing skill
    pub existing_description: String,
}

/// User's selection for importing a skill
#[derive(Debug, Clone, Deserialize)]
pub struct ImportSelection {
    /// Skill name
    pub name: String,

    /// Path to the source skill directory
    pub source_path: PathBuf,

    /// How to handle this skill
    pub resolution: ConflictResolution,
}

/// How to resolve a conflict when importing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictResolution {
    /// No conflict, just import
    Import,
    /// Skip this skill (keep existing)
    Skip,
    /// Overwrite existing with incoming
    Overwrite,
}

/// Result of an import operation
#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    /// Names of successfully imported skills
    pub imported: Vec<String>,

    /// Names of skipped skills
    pub skipped: Vec<String>,

    /// Errors that occurred (skill name, error message)
    pub errors: Vec<(String, String)>,

    /// Number of targets skills were synced to
    pub synced_to: usize,
}

/// Importer for migrating skills from target CLIs
pub struct Importer {
    /// Path to Talent's central skills directory
    skills_dir: PathBuf,
}

impl Importer {
    /// Create a new importer
    pub fn new(skills_dir: PathBuf) -> Self {
        Self { skills_dir }
    }

    /// Create an importer using the default config
    pub fn from_config(config: &Config) -> Self {
        Self::new(config.skills_dir.clone())
    }

    /// Discover importable skills from all detected targets
    pub fn discover_importable_skills(&self, targets: &[Target]) -> Vec<DiscoveredSkill> {
        let mut discovered = Vec::new();

        for target in targets {
            if let Ok(skills) = self.scan_target(target) {
                discovered.extend(skills);
            }
        }

        discovered
    }

    /// Scan a single target for importable skills
    fn scan_target(&self, target: &Target) -> Result<Vec<DiscoveredSkill>> {
        let skills_path = &target.skills_path;

        if !skills_path.exists() {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();

        // Read entries in the skills directory (depth 1)
        let entries = fs::read_dir(skills_path).map_err(|e| Error::read_dir(skills_path, e))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            // Skip if not a directory
            if !path.is_dir() {
                continue;
            }

            // Check if this is a symlink
            if self.is_symlink_to_talent(&path) {
                // Already managed by Talent, skip
                continue;
            }

            // Check for SKILL.md
            let skill_file = path.join(SKILL_FILE_NAME);
            if !skill_file.exists() {
                continue;
            }

            // Try to load and parse the skill
            if let Ok(skill) = Skill::load(&path) {
                let conflict = self.check_conflict(&skill.meta.name);

                skills.push(DiscoveredSkill {
                    name: skill.meta.name,
                    description: skill.meta.description,
                    source_path: path,
                    source_target: target.kind,
                    conflict,
                });
            }
        }

        Ok(skills)
    }

    /// Check if a path is a symlink pointing to Talent's skills directory
    fn is_symlink_to_talent(&self, path: &Path) -> bool {
        // Use symlink_metadata to not follow the symlink
        if let Ok(metadata) = fs::symlink_metadata(path) {
            if metadata.file_type().is_symlink() {
                // Read the symlink target
                if let Ok(target) = fs::read_link(path) {
                    // Check if it points to our skills directory
                    let target_canonical = if target.is_absolute() {
                        target
                    } else {
                        path.parent()
                            .map(|p| p.join(&target))
                            .unwrap_or(target)
                    };

                    // Check if the target is under our skills directory
                    if let (Ok(target_abs), Ok(talent_abs)) = (
                        target_canonical.canonicalize(),
                        self.skills_dir.canonicalize(),
                    ) {
                        return target_abs.starts_with(talent_abs);
                    }
                }
            }
        }
        false
    }

    /// Check if a skill with the given name already exists
    pub fn check_conflict(&self, name: &str) -> Option<ConflictInfo> {
        let existing_path = self.skills_dir.join(name);

        if existing_path.exists() {
            // Try to load the existing skill to get its description
            let description = Skill::load(&existing_path)
                .map(|s| s.meta.description)
                .unwrap_or_else(|_| "Unknown".to_string());

            Some(ConflictInfo {
                existing_path,
                existing_description: description,
            })
        } else {
            None
        }
    }

    /// Import a single skill
    ///
    /// Copies the skill to Talent's central storage and removes the source
    /// directory so that sync can create a symlink in its place.
    pub fn import_skill(&self, source: &Path, name: &str, overwrite: bool) -> Result<PathBuf> {
        let dest = self.skills_dir.join(name);

        if dest.exists() {
            if !overwrite {
                return Err(Error::ValidationFailed {
                    name: name.to_string(),
                    message: "Skill already exists".to_string(),
                });
            }
            // Remove existing directory
            fs::remove_dir_all(&dest).map_err(|e| Error::io(&dest, e))?;
        }

        // Copy the entire directory recursively
        copy_dir_recursive(source, &dest)?;

        // Remove the source directory so sync can create a symlink
        // Only do this if source is different from dest (not importing from Talent itself)
        if source != dest && source.exists() {
            if let Err(e) = fs::remove_dir_all(source) {
                // Log but don't fail - the import succeeded, cleanup is best-effort
                eprintln!(
                    "Warning: Could not remove source directory {:?}: {}",
                    source, e
                );
            }
        }

        Ok(dest)
    }

    /// Import multiple skills based on user selections
    pub fn import_selections(&self, selections: &[ImportSelection]) -> ImportResult {
        let mut result = ImportResult {
            imported: Vec::new(),
            skipped: Vec::new(),
            errors: Vec::new(),
            synced_to: 0,
        };

        for selection in selections {
            match selection.resolution {
                ConflictResolution::Skip => {
                    result.skipped.push(selection.name.clone());
                }
                ConflictResolution::Import | ConflictResolution::Overwrite => {
                    let overwrite = selection.resolution == ConflictResolution::Overwrite;
                    match self.import_skill(&selection.source_path, &selection.name, overwrite) {
                        Ok(_) => {
                            result.imported.push(selection.name.clone());
                        }
                        Err(e) => {
                            result.errors.push((selection.name.clone(), e.to_string()));
                        }
                    }
                }
            }
        }

        result
    }
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

/// Check if FileMerge (opendiff) is available on the system
pub fn check_filemerge_available() -> bool {
    std::process::Command::new("which")
        .arg("opendiff")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Open FileMerge to compare two skill files
pub fn open_filemerge(existing: &Path, incoming: &Path) -> Result<()> {
    let existing_file = existing.join(SKILL_FILE_NAME);
    let incoming_file = incoming.join(SKILL_FILE_NAME);

    std::process::Command::new("opendiff")
        .arg(&existing_file)
        .arg(&incoming_file)
        .spawn()
        .map_err(|e| Error::io(&existing_file, e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_skill_in_dir(dir: &Path, name: &str, description: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();

        let content = format!(
            r#"---
name: {name}
description: {description}
---

# {name}

{description}
"#
        );

        fs::write(skill_dir.join(SKILL_FILE_NAME), content).unwrap();
    }

    #[test]
    fn discover_finds_skills_in_target() {
        let temp = TempDir::new().unwrap();
        let target_skills = temp.path().join("target_skills");
        let talent_skills = temp.path().join("talent_skills");

        fs::create_dir_all(&target_skills).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        // Create a skill in the target directory
        create_skill_in_dir(&target_skills, "test-skill", "A test skill");

        let target = Target::new(TargetKind::Codex, target_skills);
        let importer = Importer::new(talent_skills);

        let discovered = importer.discover_importable_skills(&[target]);

        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].name, "test-skill");
        assert_eq!(discovered[0].description, "A test skill");
        assert_eq!(discovered[0].source_target, TargetKind::Codex);
        assert!(discovered[0].conflict.is_none());
    }

    #[test]
    fn discover_filters_symlinks_to_talent() {
        let temp = TempDir::new().unwrap();
        let target_skills = temp.path().join("target_skills");
        let talent_skills = temp.path().join("talent_skills");

        fs::create_dir_all(&target_skills).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        // Create a skill in talent directory
        create_skill_in_dir(&talent_skills, "managed-skill", "Managed by Talent");

        // Create a symlink in the target pointing to the talent skill
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                talent_skills.join("managed-skill"),
                target_skills.join("managed-skill"),
            )
            .unwrap();
        }

        let target = Target::new(TargetKind::Codex, target_skills);
        let importer = Importer::new(talent_skills);

        let discovered = importer.discover_importable_skills(&[target]);

        // The symlink should be filtered out
        assert!(discovered.is_empty());
    }

    #[test]
    fn discover_detects_conflicts() {
        let temp = TempDir::new().unwrap();
        let target_skills = temp.path().join("target_skills");
        let talent_skills = temp.path().join("talent_skills");

        fs::create_dir_all(&target_skills).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        // Create same skill in both directories
        create_skill_in_dir(&target_skills, "conflicting-skill", "Target version");
        create_skill_in_dir(&talent_skills, "conflicting-skill", "Talent version");

        let target = Target::new(TargetKind::Codex, target_skills);
        let importer = Importer::new(talent_skills);

        let discovered = importer.discover_importable_skills(&[target]);

        assert_eq!(discovered.len(), 1);
        assert!(discovered[0].conflict.is_some());

        let conflict = discovered[0].conflict.as_ref().unwrap();
        assert_eq!(conflict.existing_description, "Talent version");
    }

    #[test]
    fn import_copies_skill_directory_and_removes_source() {
        let temp = TempDir::new().unwrap();
        let source_dir = temp.path().join("source");
        let talent_skills = temp.path().join("talent_skills");

        fs::create_dir_all(&talent_skills).unwrap();
        create_skill_in_dir(source_dir.parent().unwrap(), "source", "Source skill");

        let importer = Importer::new(talent_skills.clone());
        let result = importer.import_skill(&source_dir, "imported-skill", false);

        assert!(result.is_ok());

        // Skill should be copied to talent directory
        let imported_path = talent_skills.join("imported-skill");
        assert!(imported_path.exists());
        assert!(imported_path.join(SKILL_FILE_NAME).exists());

        // Source directory should be removed
        assert!(!source_dir.exists());
    }

    #[test]
    fn import_fails_without_overwrite_flag() {
        let temp = TempDir::new().unwrap();
        let source_dir = temp.path().join("source");
        let talent_skills = temp.path().join("talent_skills");

        fs::create_dir_all(&talent_skills).unwrap();
        create_skill_in_dir(source_dir.parent().unwrap(), "source", "Source skill");
        create_skill_in_dir(&talent_skills, "existing-skill", "Existing");

        let importer = Importer::new(talent_skills);
        let result = importer.import_skill(&source_dir, "existing-skill", false);

        assert!(result.is_err());
    }

    #[test]
    fn import_with_overwrite_replaces_existing() {
        let temp = TempDir::new().unwrap();
        let source_dir = temp.path().join("source");
        let talent_skills = temp.path().join("talent_skills");

        fs::create_dir_all(&talent_skills).unwrap();
        create_skill_in_dir(source_dir.parent().unwrap(), "source", "New version");
        create_skill_in_dir(&talent_skills, "to-replace", "Old version");

        let importer = Importer::new(talent_skills.clone());
        let result = importer.import_skill(&source_dir, "to-replace", true);

        assert!(result.is_ok());

        // Verify the skill was replaced
        let skill = Skill::load(&talent_skills.join("to-replace")).unwrap();
        assert_eq!(skill.description(), "New version");
    }

    #[test]
    fn import_selections_processes_all() {
        let temp = TempDir::new().unwrap();
        let target_skills = temp.path().join("target");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&target_skills).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        create_skill_in_dir(&target_skills, "to-import", "Will import");
        create_skill_in_dir(&target_skills, "to-skip", "Will skip");

        let importer = Importer::new(talent_skills.clone());

        let selections = vec![
            ImportSelection {
                name: "to-import".to_string(),
                source_path: target_skills.join("to-import"),
                resolution: ConflictResolution::Import,
            },
            ImportSelection {
                name: "to-skip".to_string(),
                source_path: target_skills.join("to-skip"),
                resolution: ConflictResolution::Skip,
            },
        ];

        let result = importer.import_selections(&selections);

        assert_eq!(result.imported, vec!["to-import"]);
        assert_eq!(result.skipped, vec!["to-skip"]);
        assert!(result.errors.is_empty());
        assert!(talent_skills.join("to-import").exists());
        assert!(!talent_skills.join("to-skip").exists());
    }

    #[test]
    fn empty_target_returns_empty_list() {
        let temp = TempDir::new().unwrap();
        let target_skills = temp.path().join("empty_target");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&target_skills).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        let target = Target::new(TargetKind::Codex, target_skills);
        let importer = Importer::new(talent_skills);

        let discovered = importer.discover_importable_skills(&[target]);

        assert!(discovered.is_empty());
    }

    #[test]
    fn nonexistent_target_returns_empty_list() {
        let temp = TempDir::new().unwrap();
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&talent_skills).unwrap();

        let target = Target::new(TargetKind::Codex, temp.path().join("nonexistent"));
        let importer = Importer::new(talent_skills);

        let discovered = importer.discover_importable_skills(&[target]);

        assert!(discovered.is_empty());
    }
}
