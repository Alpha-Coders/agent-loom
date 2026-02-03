//! Skill importer for migrating skills from target CLIs
//!
//! Scans target CLI directories for importable skills and copies them
//! to the central skills storage.

use crate::config::Config;
use crate::error::{Error, Result};
use crate::skill::{normalize_frontmatter, to_kebab_case, Skill, SKILL_FILE_NAME};
use crate::target::{Target, TargetKind};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

/// A skill discovered in an external folder (not a target CLI)
#[derive(Debug, Clone, Serialize)]
pub struct ScannedSkill {
    /// Skill name (from frontmatter, normalized to kebab-case)
    pub name: String,

    /// Skill description (from frontmatter)
    pub description: String,

    /// Path to the skill directory
    pub source_path: PathBuf,

    /// Whether the skill has fixable issues
    pub needs_fixes: bool,

    /// Preview of fixes that will be applied (empty if no fixes needed)
    pub fixes_preview: Vec<String>,

    /// Conflict information if skill already exists in Talent
    pub conflict: Option<ConflictInfo>,
}

/// User's selection for importing a skill from an external folder
#[derive(Debug, Clone, Deserialize)]
pub struct FolderImportSelection {
    /// Skill name (kebab-case, used as folder name in ~/.agents/skills/)
    pub name: String,

    /// Path to the source skill directory
    pub source_path: PathBuf,

    /// Whether to apply normalization fixes during import
    pub apply_fixes: bool,

    /// How to handle conflicts
    pub resolution: ConflictResolution,
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
        // Skip custom folder targets (they don't have a TargetKind)
        let target_kind = match target.kind {
            Some(kind) => kind,
            None => return Ok(Vec::new()),
        };

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
                    source_target: target_kind,
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
                        path.parent().map(|p| p.join(&target)).unwrap_or(target)
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

    /// Scan an external folder for skills
    ///
    /// Recursively searches for SKILL.md files up to max_depth levels deep.
    /// Returns information about each discovered skill including normalization preview.
    pub fn scan_folder(&self, path: &Path) -> Result<Vec<ScannedSkill>> {
        const MAX_DEPTH: usize = 5;

        if !path.exists() {
            return Err(Error::io(
                path,
                std::io::Error::new(std::io::ErrorKind::NotFound, "Path does not exist"),
            ));
        }

        if !path.is_dir() {
            return Err(Error::io(
                path,
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Path is not a directory"),
            ));
        }

        let mut skills = Vec::new();

        for entry in WalkDir::new(path)
            .max_depth(MAX_DEPTH)
            .follow_links(false) // Avoid circular symlinks
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();

            // Look for SKILL.md files
            if entry_path.file_name().is_some_and(|n| n == SKILL_FILE_NAME) {
                if let Some(skill_dir) = entry_path.parent() {
                    // Skip if this is inside our own skills directory
                    if let (Ok(skill_abs), Ok(talent_abs)) =
                        (skill_dir.canonicalize(), self.skills_dir.canonicalize())
                    {
                        if skill_abs.starts_with(&talent_abs) {
                            continue;
                        }
                    }

                    if let Some(scanned) = self.scan_single_skill(skill_dir) {
                        skills.push(scanned);
                    }
                }
            }
        }

        Ok(skills)
    }

    /// Scan a single skill directory and return ScannedSkill info
    fn scan_single_skill(&self, skill_dir: &Path) -> Option<ScannedSkill> {
        let skill_file = skill_dir.join(SKILL_FILE_NAME);
        let contents = fs::read_to_string(&skill_file).ok()?;

        let folder_name = skill_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Extract frontmatter for normalization check
        let trimmed = contents.trim_start();
        let (yaml_content, _body) = if let Some(after_first) = trimmed.strip_prefix("---") {
            match after_first.find("\n---") {
                Some(end_idx) => (
                    after_first[..end_idx].trim().to_string(),
                    after_first[end_idx + 4..].trim().to_string(),
                ),
                None => (String::new(), contents.clone()),
            }
        } else {
            (String::new(), contents.clone())
        };

        // Run normalization to preview fixes
        let normalize_result = normalize_frontmatter(&yaml_content, folder_name);

        // Load skill leniently to get metadata
        let skill = Skill::load_lenient(skill_dir);

        // Determine final name (kebab-case)
        let name = if normalize_result.was_modified {
            // Use normalized name from fixes
            to_kebab_case(&skill.meta.name)
        } else {
            skill.meta.name.clone()
        };

        // Check for conflicts with existing skills
        let conflict = self.check_conflict(&name);

        Some(ScannedSkill {
            name,
            description: skill.meta.description.clone(),
            source_path: skill_dir.to_path_buf(),
            needs_fixes: normalize_result.was_modified || !skill.validation_errors.is_empty(),
            fixes_preview: normalize_result.fixes,
            conflict,
        })
    }

    /// Import a skill from an external folder
    ///
    /// Unlike import_skill(), this does NOT remove the source directory.
    /// Optionally applies normalization fixes during import.
    pub fn import_from_external(
        &self,
        source: &Path,
        name: &str,
        apply_fixes: bool,
        overwrite: bool,
    ) -> Result<PathBuf> {
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

        // Optionally apply normalization fixes
        if apply_fixes {
            let mut skill = Skill::load_lenient(&dest);
            let _ = skill.fix_frontmatter();
        }

        // Note: We do NOT remove the source directory for external imports
        // This is intentional - external imports are copies, not migrations

        Ok(dest)
    }

    /// Import multiple skills from external folders based on user selections
    pub fn import_folder_selections(&self, selections: &[FolderImportSelection]) -> ImportResult {
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
                    match self.import_from_external(
                        &selection.source_path,
                        &selection.name,
                        selection.apply_fixes,
                        overwrite,
                    ) {
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

    // =========================================================================
    // Folder Import Tests (scan_folder, import_from_external, etc.)
    // =========================================================================

    fn create_skill_with_content(dir: &Path, name: &str, content: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join(SKILL_FILE_NAME), content).unwrap();
    }

    #[test]
    fn scan_folder_empty_returns_empty_list() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        let importer = Importer::new(talent_skills);
        let scanned = importer.scan_folder(&external_folder).unwrap();

        assert!(scanned.is_empty());
    }

    #[test]
    fn scan_folder_finds_single_skill() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        create_skill_in_dir(&external_folder, "my-skill", "A test skill");

        let importer = Importer::new(talent_skills);
        let scanned = importer.scan_folder(&external_folder).unwrap();

        assert_eq!(scanned.len(), 1);
        assert_eq!(scanned[0].name, "my-skill");
        assert_eq!(scanned[0].description, "A test skill");
        assert!(!scanned[0].needs_fixes);
        assert!(scanned[0].conflict.is_none());
    }

    #[test]
    fn scan_folder_finds_nested_skills() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        // Create skills at different nesting levels
        let nested1 = external_folder.join("project-a");
        let nested2 = external_folder.join("project-b").join("deeper");
        fs::create_dir_all(&nested1).unwrap();
        fs::create_dir_all(&nested2).unwrap();

        create_skill_in_dir(&nested1, "skill-one", "First skill");
        create_skill_in_dir(&nested2, "skill-two", "Second skill");

        let importer = Importer::new(talent_skills);
        let scanned = importer.scan_folder(&external_folder).unwrap();

        assert_eq!(scanned.len(), 2);
        let names: Vec<&str> = scanned.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"skill-one"));
        assert!(names.contains(&"skill-two"));
    }

    #[test]
    fn scan_folder_detects_conflict_with_existing() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        // Create skill in talent (existing)
        create_skill_in_dir(&talent_skills, "existing-skill", "Original version");
        // Create skill with same name in external folder
        create_skill_in_dir(&external_folder, "existing-skill", "New version");

        let importer = Importer::new(talent_skills);
        let scanned = importer.scan_folder(&external_folder).unwrap();

        assert_eq!(scanned.len(), 1);
        assert!(scanned[0].conflict.is_some());
        let conflict = scanned[0].conflict.as_ref().unwrap();
        assert_eq!(conflict.existing_description, "Original version");
    }

    #[test]
    fn scan_folder_detects_non_kebab_name_needs_fixes() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        // Create skill with non-kebab-case name
        let content = r#"---
name: My_Skill_Name
description: A skill with non-kebab name
---

# My Skill
"#;
        create_skill_with_content(&external_folder, "My_Skill_Name", content);

        let importer = Importer::new(talent_skills);
        let scanned = importer.scan_folder(&external_folder).unwrap();

        assert_eq!(scanned.len(), 1);
        assert!(scanned[0].needs_fixes);
        assert!(!scanned[0].fixes_preview.is_empty());
        // The name should be normalized to kebab-case
        assert_eq!(scanned[0].name, "my-skill-name");
    }

    #[test]
    fn scan_folder_skips_talent_skills_directory() {
        let temp = TempDir::new().unwrap();
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&talent_skills).unwrap();

        // Create a skill directly in talent skills directory
        create_skill_in_dir(&talent_skills, "internal-skill", "Internal");

        let importer = Importer::new(talent_skills.clone());
        // Scan talent's own directory - should skip its contents
        let scanned = importer.scan_folder(&talent_skills).unwrap();

        assert!(scanned.is_empty());
    }

    #[test]
    fn scan_folder_nonexistent_path_returns_error() {
        let temp = TempDir::new().unwrap();
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&talent_skills).unwrap();

        let importer = Importer::new(talent_skills);
        let result = importer.scan_folder(&temp.path().join("nonexistent"));

        assert!(result.is_err());
    }

    #[test]
    fn scan_folder_file_path_returns_error() {
        let temp = TempDir::new().unwrap();
        let talent_skills = temp.path().join("talent");
        let file_path = temp.path().join("file.txt");

        fs::create_dir_all(&talent_skills).unwrap();
        fs::write(&file_path, "not a directory").unwrap();

        let importer = Importer::new(talent_skills);
        let result = importer.scan_folder(&file_path);

        assert!(result.is_err());
    }

    #[test]
    fn import_from_external_copies_without_removing_source() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        create_skill_in_dir(&external_folder, "my-skill", "External skill");
        let source_path = external_folder.join("my-skill");

        let importer = Importer::new(talent_skills.clone());
        let result = importer.import_from_external(&source_path, "my-skill", false, false);

        assert!(result.is_ok());

        // Skill should be copied to talent
        assert!(talent_skills.join("my-skill").exists());
        assert!(talent_skills
            .join("my-skill")
            .join(SKILL_FILE_NAME)
            .exists());

        // Source should NOT be removed (unlike regular import)
        assert!(source_path.exists());
    }

    #[test]
    fn import_from_external_renames_to_target_name() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        create_skill_in_dir(&external_folder, "Original_Name", "A skill");
        let source_path = external_folder.join("Original_Name");

        let importer = Importer::new(talent_skills.clone());
        let result = importer.import_from_external(&source_path, "new-name", false, false);

        assert!(result.is_ok());

        // Should be imported with the new name
        assert!(talent_skills.join("new-name").exists());
        assert!(!talent_skills.join("Original_Name").exists());
    }

    #[test]
    fn import_from_external_fails_on_conflict_without_overwrite() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        create_skill_in_dir(&external_folder, "conflicting", "New version");
        create_skill_in_dir(&talent_skills, "conflicting", "Existing version");

        let importer = Importer::new(talent_skills);
        let result = importer.import_from_external(
            &external_folder.join("conflicting"),
            "conflicting",
            false,
            false,
        );

        assert!(result.is_err());
    }

    #[test]
    fn import_from_external_overwrites_when_flag_set() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        create_skill_in_dir(&external_folder, "to-overwrite", "New version");
        create_skill_in_dir(&talent_skills, "to-overwrite", "Old version");

        let importer = Importer::new(talent_skills.clone());
        let result = importer.import_from_external(
            &external_folder.join("to-overwrite"),
            "to-overwrite",
            false,
            true, // overwrite = true
        );

        assert!(result.is_ok());

        // Verify content was replaced
        let skill = Skill::load(&talent_skills.join("to-overwrite")).unwrap();
        assert_eq!(skill.description(), "New version");
    }

    #[test]
    fn import_from_external_applies_fixes_when_requested() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        // Create skill with non-kebab name that needs fixing
        let content = r#"---
name: My_Bad_Name
description: Needs normalization
---

# Skill
"#;
        create_skill_with_content(&external_folder, "My_Bad_Name", content);

        let importer = Importer::new(talent_skills.clone());
        let result = importer.import_from_external(
            &external_folder.join("My_Bad_Name"),
            "my-bad-name",
            true, // apply_fixes = true
            false,
        );

        assert!(result.is_ok());

        // Load and verify the name was fixed
        let skill = Skill::load(&talent_skills.join("my-bad-name")).unwrap();
        assert_eq!(skill.name(), "my-bad-name");
    }

    #[test]
    fn import_folder_selections_processes_all_resolutions() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        create_skill_in_dir(&external_folder, "to-import", "Will be imported");
        create_skill_in_dir(&external_folder, "to-skip", "Will be skipped");
        create_skill_in_dir(&external_folder, "to-overwrite", "New version");
        create_skill_in_dir(&talent_skills, "to-overwrite", "Old version");

        let importer = Importer::new(talent_skills.clone());

        let selections = vec![
            FolderImportSelection {
                name: "to-import".to_string(),
                source_path: external_folder.join("to-import"),
                apply_fixes: false,
                resolution: ConflictResolution::Import,
            },
            FolderImportSelection {
                name: "to-skip".to_string(),
                source_path: external_folder.join("to-skip"),
                apply_fixes: false,
                resolution: ConflictResolution::Skip,
            },
            FolderImportSelection {
                name: "to-overwrite".to_string(),
                source_path: external_folder.join("to-overwrite"),
                apply_fixes: false,
                resolution: ConflictResolution::Overwrite,
            },
        ];

        let result = importer.import_folder_selections(&selections);

        assert_eq!(result.imported, vec!["to-import", "to-overwrite"]);
        assert_eq!(result.skipped, vec!["to-skip"]);
        assert!(result.errors.is_empty());

        // Verify filesystem state
        assert!(talent_skills.join("to-import").exists());
        assert!(!talent_skills.join("to-skip").exists());
        assert!(talent_skills.join("to-overwrite").exists());

        // Verify overwrite worked
        let skill = Skill::load(&talent_skills.join("to-overwrite")).unwrap();
        assert_eq!(skill.description(), "New version");
    }

    #[test]
    fn import_folder_selections_collects_errors() {
        let temp = TempDir::new().unwrap();
        let external_folder = temp.path().join("external");
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&external_folder).unwrap();
        fs::create_dir_all(&talent_skills).unwrap();

        // Create one valid skill and one that will fail (conflict without overwrite)
        create_skill_in_dir(&external_folder, "valid-skill", "Will work");
        create_skill_in_dir(&external_folder, "will-fail", "Should fail");
        create_skill_in_dir(&talent_skills, "will-fail", "Existing");

        let importer = Importer::new(talent_skills.clone());

        let selections = vec![
            FolderImportSelection {
                name: "valid-skill".to_string(),
                source_path: external_folder.join("valid-skill"),
                apply_fixes: false,
                resolution: ConflictResolution::Import,
            },
            FolderImportSelection {
                name: "will-fail".to_string(),
                source_path: external_folder.join("will-fail"),
                apply_fixes: false,
                resolution: ConflictResolution::Import, // No overwrite, will fail
            },
        ];

        let result = importer.import_folder_selections(&selections);

        assert_eq!(result.imported, vec!["valid-skill"]);
        assert!(result.skipped.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].0, "will-fail");
    }

    #[test]
    fn import_folder_selections_empty_returns_empty_result() {
        let temp = TempDir::new().unwrap();
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&talent_skills).unwrap();

        let importer = Importer::new(talent_skills);
        let result = importer.import_folder_selections(&[]);

        assert!(result.imported.is_empty());
        assert!(result.skipped.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn check_conflict_returns_none_when_no_conflict() {
        let temp = TempDir::new().unwrap();
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&talent_skills).unwrap();

        let importer = Importer::new(talent_skills);
        let conflict = importer.check_conflict("nonexistent-skill");

        assert!(conflict.is_none());
    }

    #[test]
    fn check_conflict_returns_info_when_exists() {
        let temp = TempDir::new().unwrap();
        let talent_skills = temp.path().join("talent");

        fs::create_dir_all(&talent_skills).unwrap();
        create_skill_in_dir(&talent_skills, "existing", "Existing skill description");

        let importer = Importer::new(talent_skills);
        let conflict = importer.check_conflict("existing");

        assert!(conflict.is_some());
        let info = conflict.unwrap();
        assert_eq!(info.existing_description, "Existing skill description");
    }
}
