//! Symlink synchronization for skills
//!
//! Creates symlinks from target CLI skills directories to the central skills storage.
//! For example: `~/.claude/commands/my-skill` -> `~/.agents/skills/my-skill`

use crate::error::{Error, Result};
use crate::skill::Skill;
use crate::target::Target;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Result of syncing skills to a target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Target identifier
    pub target_id: String,

    /// Target display name
    pub target_name: String,

    /// Skills that were newly linked
    pub created: Vec<String>,

    /// Skills that were unlinked (no longer exist in source)
    pub removed: Vec<String>,

    /// Skills that already had valid symlinks
    pub unchanged: Vec<String>,

    /// Errors encountered during sync
    pub errors: Vec<SyncError>,
}

/// An error encountered during sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    /// Skill name (if applicable)
    pub skill: Option<String>,

    /// Error message
    pub message: String,
}

impl SyncResult {
    /// Create a new empty sync result for a target
    pub fn new(target: &Target) -> Self {
        Self {
            target_id: target.id().to_string(),
            target_name: target.name().to_string(),
            created: Vec::new(),
            removed: Vec::new(),
            unchanged: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Check if the sync was successful (no errors)
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get total number of synced skills (created + unchanged)
    pub fn synced_count(&self) -> usize {
        self.created.len() + self.unchanged.len()
    }

    /// Add an error to the result
    pub fn add_error(&mut self, skill: Option<&str>, message: impl Into<String>) {
        self.errors.push(SyncError {
            skill: skill.map(|s| s.to_string()),
            message: message.into(),
        });
    }
}

/// Syncer for managing symlinks between Talent skills and target CLIs
pub struct Syncer {
    /// Whether to remove stale symlinks (links to skills that no longer exist)
    pub remove_stale: bool,

    /// Whether to create target directories if they don't exist
    pub create_dirs: bool,

    /// Whether to automatically migrate non-symlink folders to symlinks
    /// When true, if a real folder exists where a symlink should be, it will be
    /// removed (since the skill already exists in Talent) and replaced with a symlink
    pub auto_migrate: bool,
}

impl Default for Syncer {
    fn default() -> Self {
        Self {
            remove_stale: true,
            create_dirs: true,
            auto_migrate: true,
        }
    }
}

impl Syncer {
    /// Create a new syncer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sync skills to a single target
    pub fn sync_target(&self, target: &Target, skills: &[Skill]) -> SyncResult {
        let mut result = SyncResult::new(target);

        // Skip disabled targets
        if !target.enabled {
            return result;
        }

        // Ensure target skills directory exists
        if self.create_dirs {
            if let Err(e) = target.ensure_skills_dir() {
                result.add_error(None, format!("Failed to create skills directory: {e}"));
                return result;
            }
        } else if !target.skills_dir_exists() {
            result.add_error(None, "Skills directory does not exist");
            return result;
        }

        // Get set of skill names we want to sync
        let skill_names: HashSet<&str> = skills.iter().map(|s| s.name()).collect();

        // Create symlinks for each skill
        for skill in skills {
            let link_path = target.skill_link_path(skill.name());
            let target_path = &skill.path;

            match self.create_symlink(&link_path, target_path, self.auto_migrate) {
                Ok(SymlinkAction::Created) => {
                    result.created.push(skill.name().to_string());
                }
                Ok(SymlinkAction::Migrated) => {
                    result.created.push(skill.name().to_string());
                    eprintln!(
                        "Migrated '{}': removed original folder, created symlink",
                        skill.name()
                    );
                }
                Ok(SymlinkAction::Unchanged) => {
                    result.unchanged.push(skill.name().to_string());
                }
                Err(e) => {
                    result.add_error(Some(skill.name()), e.to_string());
                }
            }
        }

        // Remove stale symlinks if enabled
        if self.remove_stale {
            if let Err(e) = self.remove_stale_symlinks(target, &skill_names, &mut result) {
                result.add_error(None, format!("Failed to clean stale symlinks: {e}"));
            }
        }

        result
    }

    /// Sync skills to all targets
    pub fn sync_all(&self, targets: &[Target], skills: &[Skill]) -> Vec<SyncResult> {
        targets
            .iter()
            .map(|target| self.sync_target(target, skills))
            .collect()
    }

    /// Create a symlink, handling existing paths
    ///
    /// If `auto_migrate` is true and a non-symlink exists at `link_path`,
    /// it will be removed (since the skill exists in Talent) and replaced with a symlink.
    fn create_symlink(
        &self,
        link_path: &Path,
        target_path: &Path,
        auto_migrate: bool,
    ) -> Result<SymlinkAction> {
        // Check if something already exists at the link path
        if link_path.exists() || link_path.symlink_metadata().is_ok() {
            // Check if it's already a symlink or junction
            let metadata = link_path
                .symlink_metadata()
                .map_err(|e| Error::io(link_path, e))?;

            if self.is_link(&metadata, link_path) {
                // Check if it points to the correct target
                let current_target = self
                    .read_link_target(link_path)
                    .map_err(|e| Error::io(link_path, e))?;

                if current_target == target_path {
                    return Ok(SymlinkAction::Unchanged);
                }

                // Points to wrong target, remove and recreate
                self.remove_link(link_path)
                    .map_err(|e| Error::SymlinkRemove {
                        path: link_path.to_path_buf(),
                        message: e.to_string(),
                    })?;
            } else if auto_migrate {
                // Not a symlink, but auto_migrate is enabled
                // Remove the existing folder/file since the skill exists in Talent
                if metadata.is_dir() {
                    fs::remove_dir_all(link_path).map_err(|e| Error::SymlinkRemove {
                        path: link_path.to_path_buf(),
                        message: format!("Failed to remove existing folder for migration: {e}"),
                    })?;
                } else {
                    fs::remove_file(link_path).map_err(|e| Error::SymlinkRemove {
                        path: link_path.to_path_buf(),
                        message: format!("Failed to remove existing file for migration: {e}"),
                    })?;
                }

                // Create the symlink after removing
                self.do_create_symlink(link_path, target_path)?;
                return Ok(SymlinkAction::Migrated);
            } else {
                // Not a symlink and auto_migrate is disabled - this is an error
                return Err(Error::NotASymlink(link_path.to_path_buf()));
            }
        }

        // Create the symlink
        self.do_create_symlink(link_path, target_path)?;
        Ok(SymlinkAction::Created)
    }

    /// Actually create the symlink (platform-specific)
    ///
    /// On Unix, creates a symbolic link.
    /// On Windows, creates a directory junction (no admin privileges required).
    fn do_create_symlink(&self, link_path: &Path, target_path: &Path) -> Result<()> {
        #[cfg(unix)]
        std::os::unix::fs::symlink(target_path, link_path).map_err(|e| Error::SymlinkCreate {
            link_source: target_path.to_path_buf(),
            link_target: link_path.to_path_buf(),
            message: e.to_string(),
        })?;

        #[cfg(windows)]
        {
            // On Windows, use NTFS directory junctions instead of symlinks.
            // Junctions don't require SeCreateSymbolicLinkPrivilege (admin/Developer Mode).
            // Note: Junctions only work for directories and require absolute paths.
            if target_path.is_dir() {
                // Convert to absolute paths as junctions require them
                let abs_target = if target_path.is_absolute() {
                    target_path.to_path_buf()
                } else {
                    std::env::current_dir()
                        .map_err(|e| Error::SymlinkCreate {
                            link_source: target_path.to_path_buf(),
                            link_target: link_path.to_path_buf(),
                            message: format!("Failed to get current directory: {e}"),
                        })?
                        .join(target_path)
                };

                junction::create(&abs_target, link_path).map_err(|e| Error::SymlinkCreate {
                    link_source: target_path.to_path_buf(),
                    link_target: link_path.to_path_buf(),
                    message: e.to_string(),
                })?;
            } else {
                // For files, fall back to copy since junctions only work for directories
                // and symlinks require elevated privileges
                std::fs::copy(target_path, link_path).map_err(|e| Error::SymlinkCreate {
                    link_source: target_path.to_path_buf(),
                    link_target: link_path.to_path_buf(),
                    message: format!("Failed to copy file (symlinks require admin): {e}"),
                })?;
            }
        }

        Ok(())
    }

    /// Check if a path is a symlink or junction (platform-specific)
    #[cfg(unix)]
    fn is_link(&self, metadata: &std::fs::Metadata, _path: &Path) -> bool {
        metadata.file_type().is_symlink()
    }

    #[cfg(windows)]
    fn is_link(&self, metadata: &std::fs::Metadata, path: &Path) -> bool {
        // On Windows, is_symlink() returns true for both symlinks and junctions
        // Also check junction::exists for junctions not detected as symlinks
        metadata.file_type().is_symlink() || junction::exists(path).unwrap_or(false)
    }

    /// Read the target of a symlink or junction (platform-specific)
    #[cfg(unix)]
    fn read_link_target(&self, path: &Path) -> std::io::Result<std::path::PathBuf> {
        fs::read_link(path)
    }

    #[cfg(windows)]
    fn read_link_target(&self, path: &Path) -> std::io::Result<std::path::PathBuf> {
        // Try junction first, then symlink
        if junction::exists(path).unwrap_or(false) {
            junction::get_target(path)
        } else {
            fs::read_link(path)
        }
    }

    /// Remove a symlink or junction (platform-specific)
    #[cfg(unix)]
    fn remove_link(&self, path: &Path) -> std::io::Result<()> {
        fs::remove_file(path)
    }

    #[cfg(windows)]
    fn remove_link(&self, path: &Path) -> std::io::Result<()> {
        // On Windows, junctions are created by junction::create and can be removed
        // with fs::remove_dir. Try multiple strategies for robustness.

        // Strategy 1: remove_dir (works for junctions)
        if fs::remove_dir(path).is_ok() && !path.exists() {
            return Ok(());
        }

        // Strategy 2: remove_file (works for symlinks to directories)
        if fs::remove_file(path).is_ok() && !path.exists() {
            return Ok(());
        }

        // Strategy 3: junction::delete as fallback
        if junction::delete(path).is_ok() && !path.exists() {
            return Ok(());
        }

        // If path still exists, report error
        if path.exists() {
            return Err(std::io::Error::other(
                "Failed to remove link - path still exists after removal attempts",
            ));
        }

        Ok(())
    }

    /// Remove symlinks/junctions for skills that no longer exist
    fn remove_stale_symlinks(
        &self,
        target: &Target,
        current_skills: &HashSet<&str>,
        result: &mut SyncResult,
    ) -> Result<()> {
        let entries = fs::read_dir(&target.skills_path)
            .map_err(|e| Error::read_dir(&target.skills_path, e))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            // Only process symlinks/junctions
            let metadata = match path.symlink_metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if !self.is_link(&metadata, &path) {
                continue;
            }

            // Get the skill name from the symlink name
            let skill_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name,
                None => continue,
            };

            // If this skill is not in our current set, remove the symlink
            if !current_skills.contains(skill_name) {
                match self.remove_link(&path) {
                    Ok(()) => {
                        result.removed.push(skill_name.to_string());
                    }
                    Err(e) => {
                        result.add_error(
                            Some(skill_name),
                            format!("Failed to remove stale symlink: {e}"),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Remove all symlinks/junctions for a target (used when disabling a target)
    pub fn remove_all_symlinks(&self, target: &Target) -> Result<Vec<String>> {
        if !target.skills_path.exists() {
            return Ok(Vec::new());
        }

        let mut removed = Vec::new();

        let entries = fs::read_dir(&target.skills_path)
            .map_err(|e| Error::read_dir(&target.skills_path, e))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            // Only process symlinks/junctions
            let metadata = match path.symlink_metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if !self.is_link(&metadata, &path) {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();

            self.remove_link(&path).map_err(|e| Error::SymlinkRemove {
                path: path.clone(),
                message: e.to_string(),
            })?;

            removed.push(name);
        }

        Ok(removed)
    }
}

/// Action taken when creating a symlink
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymlinkAction {
    /// New symlink was created
    Created,
    /// Symlink already existed and pointed to correct target
    Unchanged,
    /// Existing non-symlink was removed and replaced with symlink (auto-migration)
    Migrated,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::Skill;
    use crate::target::TargetKind;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_skill(skills_dir: &Path, name: &str) -> Skill {
        Skill::create(skills_dir, name, "Test skill").unwrap()
    }

    fn create_test_target(temp_dir: &Path) -> Target {
        let skills_path = temp_dir.join("target-skills");
        Target::new(TargetKind::ClaudeCode, skills_path)
    }

    #[test]
    fn sync_creates_symlinks() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill = create_test_skill(&skills_dir, "my-skill");
        let target = create_test_target(temp.path());

        let syncer = Syncer::new();
        let result = syncer.sync_target(&target, &[skill]);

        assert!(result.is_success());
        assert_eq!(result.created.len(), 1);
        assert_eq!(result.created[0], "my-skill");
        assert!(target.skill_link_path("my-skill").exists());
    }

    #[test]
    fn sync_detects_unchanged_symlinks() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill = create_test_skill(&skills_dir, "my-skill");
        let target = create_test_target(temp.path());

        let syncer = Syncer::new();

        // First sync - creates symlink
        let result1 = syncer.sync_target(&target, std::slice::from_ref(&skill));
        assert_eq!(result1.created.len(), 1);

        // Second sync - should be unchanged
        let result2 = syncer.sync_target(&target, &[skill]);
        assert!(result2.created.is_empty());
        assert_eq!(result2.unchanged.len(), 1);
        assert_eq!(result2.unchanged[0], "my-skill");
    }

    #[test]
    fn sync_removes_stale_symlinks() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill1 = create_test_skill(&skills_dir, "skill-one");
        let skill2 = create_test_skill(&skills_dir, "skill-two");
        let target = create_test_target(temp.path());

        let syncer = Syncer::new();

        // Sync both skills
        let result1 = syncer.sync_target(&target, &[skill1.clone(), skill2]);
        assert_eq!(result1.created.len(), 2);

        // Sync with only one skill - should remove the other
        let result2 = syncer.sync_target(&target, &[skill1]);
        assert_eq!(result2.removed.len(), 1);
        assert_eq!(result2.removed[0], "skill-two");
        assert!(!target.skill_link_path("skill-two").exists());
        assert!(target.skill_link_path("skill-one").exists());
    }

    #[test]
    fn sync_errors_on_non_symlink_when_auto_migrate_disabled() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill = create_test_skill(&skills_dir, "my-skill");
        let target = create_test_target(temp.path());

        // Create target directory and a regular file where symlink should go
        std::fs::create_dir_all(&target.skills_path).unwrap();
        let conflict_path = target.skill_link_path("my-skill");
        std::fs::create_dir_all(&conflict_path).unwrap(); // Create a directory instead

        let mut syncer = Syncer::new();
        syncer.auto_migrate = false; // Disable auto-migration
        let result = syncer.sync_target(&target, &[skill]);

        assert!(!result.is_success());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("not a symlink"));
    }

    #[test]
    fn sync_auto_migrates_non_symlink() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill = create_test_skill(&skills_dir, "my-skill");
        let target = create_test_target(temp.path());

        // Create target directory and a regular folder where symlink should go
        std::fs::create_dir_all(&target.skills_path).unwrap();
        let conflict_path = target.skill_link_path("my-skill");
        std::fs::create_dir_all(&conflict_path).unwrap();

        // Create a file inside to simulate existing skill
        std::fs::write(conflict_path.join("test.txt"), "test content").unwrap();

        let syncer = Syncer::new(); // auto_migrate is true by default
        let result = syncer.sync_target(&target, &[skill]);

        assert!(result.is_success());
        assert_eq!(result.created.len(), 1);

        // The path should now be a symlink
        let link_path = target.skill_link_path("my-skill");
        assert!(link_path
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn sync_skips_disabled_target() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill = create_test_skill(&skills_dir, "my-skill");
        let mut target = create_test_target(temp.path());
        target.enabled = false;

        let syncer = Syncer::new();
        let result = syncer.sync_target(&target, &[skill]);

        assert!(result.is_success());
        assert!(result.created.is_empty());
        assert!(!target.skill_link_path("my-skill").exists());
    }

    #[test]
    fn sync_all_targets() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill = create_test_skill(&skills_dir, "my-skill");

        let target1 = Target::new(TargetKind::ClaudeCode, temp.path().join("target1"));
        let target2 = Target::new(TargetKind::Codex, temp.path().join("target2"));

        let syncer = Syncer::new();
        let results = syncer.sync_all(&[target1, target2], &[skill]);

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_success()));
        assert!(results.iter().all(|r| r.created.len() == 1));
    }

    #[test]
    fn remove_all_symlinks() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill1 = create_test_skill(&skills_dir, "skill-one");
        let skill2 = create_test_skill(&skills_dir, "skill-two");
        let target = create_test_target(temp.path());

        let syncer = Syncer::new();

        // Sync skills first
        syncer.sync_target(&target, &[skill1, skill2]);
        assert!(target.skill_link_path("skill-one").exists());
        assert!(target.skill_link_path("skill-two").exists());

        // Remove all
        let removed = syncer.remove_all_symlinks(&target).unwrap();
        assert_eq!(removed.len(), 2);
        assert!(!target.skill_link_path("skill-one").exists());
        assert!(!target.skill_link_path("skill-two").exists());
    }

    #[test]
    fn synced_count_calculation() {
        let target = Target::new(TargetKind::ClaudeCode, PathBuf::from("/test"));
        let mut result = SyncResult::new(&target);

        result.created.push("skill-1".to_string());
        result.created.push("skill-2".to_string());
        result.unchanged.push("skill-3".to_string());

        assert_eq!(result.synced_count(), 3);
    }
}
