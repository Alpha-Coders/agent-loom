//! Skill Manager - Integration layer for all Talent components
//!
//! The SkillManager is the main entry point for interacting with Talent.
//! It integrates config, skills, targets, validation, and syncing.

use crate::config::Config;
use crate::error::{Error, Result};
use crate::skill::{discover_skills, Skill, ValidationStatus};
use crate::syncer::{SyncResult, Syncer};
use crate::target::Target;
use crate::validator::Validator;
use std::path::PathBuf;

/// Main manager for Talent operations
pub struct SkillManager {
    /// Application configuration
    config: Config,

    /// Discovered skills
    skills: Vec<Skill>,

    /// Detected/configured targets
    targets: Vec<Target>,

    /// Symlink syncer
    syncer: Syncer,

    /// Skill validator
    validator: Validator,
}

impl SkillManager {
    /// Create a new SkillManager with default configuration
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Self::with_config(config)
    }

    /// Create a new SkillManager with the given configuration
    pub fn with_config(config: Config) -> Result<Self> {
        // Ensure skills directory exists
        config.ensure_skills_dir()?;

        // Discover skills
        let skills = discover_skills(&config.skills_dir)?;

        // Detect targets and merge with config
        let targets = Self::load_targets_with_config(&config);

        Ok(Self {
            config,
            skills,
            targets,
            syncer: Syncer::new(),
            validator: Validator::new(),
        })
    }

    /// Get all discovered skills
    pub fn skills(&self) -> &[Skill] {
        &self.skills
    }

    /// Get all detected targets
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get a skill by name
    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.name() == name)
    }

    /// Get a mutable skill by name
    pub fn get_skill_mut(&mut self, name: &str) -> Option<&mut Skill> {
        self.skills.iter_mut().find(|s| s.name() == name)
    }

    /// Refresh the list of skills by re-scanning the directory
    pub fn refresh_skills(&mut self) -> Result<()> {
        self.skills = discover_skills(&self.config.skills_dir)?;
        Ok(())
    }

    /// Refresh the list of targets by re-detecting installed CLIs
    pub fn refresh_targets(&mut self) {
        self.targets = Self::load_targets_with_config(&self.config);
    }

    /// Load targets, merging auto-detected with config settings
    fn load_targets_with_config(config: &Config) -> Vec<Target> {
        use crate::target::TargetKind;

        let mut targets = Vec::new();

        // First, add all auto-detected targets
        for mut target in Target::detect_all() {
            // Check if config has settings for this target
            if let Some(target_config) = config.targets.get(target.id()) {
                target.enabled = target_config.enabled;
                // Override path if specified in config
                if let Some(ref custom_path) = target_config.skills_path {
                    target.skills_path = custom_path.clone();
                }
            }
            targets.push(target);
        }

        // Then, add custom targets from config that weren't auto-detected
        for (id, target_config) in &config.targets {
            // Skip if already in the list
            if targets.iter().any(|t| t.id() == id) {
                continue;
            }

            // Only add if it has a custom path (otherwise it's just a disabled auto-detected one)
            if let Some(ref custom_path) = target_config.skills_path {
                // Try to parse the id as a known TargetKind, or treat as custom
                if let Some(kind) = TargetKind::all().iter().find(|k| k.id() == id) {
                    let mut target = Target::new(*kind, custom_path.clone());
                    target.enabled = target_config.enabled;
                    target.auto_detected = false;
                    targets.push(target);
                }
                // Note: For truly custom targets (not in TargetKind), we'd need to extend the system
            }
        }

        targets
    }

    /// Toggle a target's enabled state
    pub fn toggle_target(&mut self, target_id: &str) -> Result<bool> {
        // Find the target
        let target = self
            .targets
            .iter_mut()
            .find(|t| t.id() == target_id)
            .ok_or_else(|| Error::TargetError(format!("Target not found: {}", target_id)))?;

        // Toggle
        target.enabled = !target.enabled;
        let new_state = target.enabled;

        // Update config and save
        if target.enabled {
            self.config.enable_target(target_id);
        } else {
            self.config.disable_target(target_id);
        }
        self.config.save()?;

        Ok(new_state)
    }

    /// Set a target's enabled state explicitly
    pub fn set_target_enabled(&mut self, target_id: &str, enabled: bool) -> Result<()> {
        // Find the target
        let target = self
            .targets
            .iter_mut()
            .find(|t| t.id() == target_id)
            .ok_or_else(|| Error::TargetError(format!("Target not found: {}", target_id)))?;

        target.enabled = enabled;

        // Update config and save
        if enabled {
            self.config.enable_target(target_id);
        } else {
            self.config.disable_target(target_id);
        }
        self.config.save()?;

        Ok(())
    }

    /// Add a custom target with a specific path
    pub fn add_custom_target(&mut self, target_id: &str, skills_path: PathBuf) -> Result<()> {
        use crate::target::TargetKind;

        // Validate the target_id is a known kind
        let kind = TargetKind::all()
            .iter()
            .find(|k| k.id() == target_id)
            .ok_or_else(|| Error::UnknownTargetType(target_id.to_string()))?;

        // Check if target already exists
        if self.targets.iter().any(|t| t.id() == target_id) {
            return Err(Error::TargetAlreadyExists(target_id.to_string()));
        }

        // Create the target
        let mut target = Target::new(*kind, skills_path.clone());
        target.enabled = true;
        target.auto_detected = false;

        // Ensure the skills directory exists
        target.ensure_skills_dir()?;

        // Add to targets list
        self.targets.push(target);

        // Update config and save
        let target_config = self.config.get_or_create_target(target_id);
        target_config.enabled = true;
        target_config.skills_path = Some(skills_path);
        self.config.save()?;

        Ok(())
    }

    /// Add a folder as a sync target
    ///
    /// Creates a custom target from any folder path. The ID and name are derived
    /// from the path. Returns the created target.
    pub fn add_folder_as_target(&mut self, path: PathBuf) -> Result<Target> {
        // Generate ID from path (use folder name, make it lowercase and kebab-case)
        let folder_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("folder");

        // Create a unique ID: folder-<name>
        let base_id = format!("folder-{}", folder_name.to_lowercase().replace(' ', "-"));

        // Check if this exact path is already a target
        if self.targets.iter().any(|t| t.skills_path == path) {
            return Err(Error::TargetAlreadyExists(path.display().to_string()));
        }

        // Make ID unique if needed
        let mut target_id = base_id.clone();
        let mut counter = 1;
        while self.targets.iter().any(|t| t.id() == target_id) {
            target_id = format!("{}-{}", base_id, counter);
            counter += 1;
        }

        // Derive display name from path
        let display_name = folder_name.to_string();

        // Create the target
        let mut target = Target::new_folder(path.clone(), target_id.clone(), display_name);
        target.enabled = true;

        // Ensure the directory exists
        target.ensure_skills_dir()?;

        // Add to targets list
        self.targets.push(target.clone());

        // Update config and save
        let target_config = self.config.get_or_create_target(&target_id);
        target_config.enabled = true;
        target_config.skills_path = Some(path);
        self.config.save()?;

        Ok(target)
    }

    /// Remove a custom target (only works for non-auto-detected targets)
    pub fn remove_custom_target(&mut self, target_id: &str) -> Result<()> {
        // Find the target
        let target = self
            .targets
            .iter()
            .find(|t| t.id() == target_id)
            .ok_or_else(|| Error::TargetError(format!("Target not found: {}", target_id)))?;

        // Only allow removing non-auto-detected targets
        if target.auto_detected {
            return Err(Error::TargetError(
                "Cannot remove auto-detected target. Use disable instead.".to_string(),
            ));
        }

        // Remove from targets list
        self.targets.retain(|t| t.id() != target_id);

        // Remove from config and save
        self.config.targets.remove(target_id);
        self.config.save()?;

        Ok(())
    }

    /// Validate a specific skill by name
    pub fn validate_skill(&mut self, name: &str) -> Result<()> {
        let skill = self
            .skills
            .iter_mut()
            .find(|s| s.name() == name)
            .ok_or_else(|| Error::SkillNotFound(self.config.skills_dir.join(name)))?;

        self.validator.validate(skill)
    }

    /// Validate all skills
    pub fn validate_all(&mut self) -> Vec<Result<()>> {
        self.validator.validate_all(&mut self.skills)
    }

    /// Sync all skills to all targets
    pub fn sync_all(&self) -> Vec<SyncResult> {
        // Filter to only valid skills if validate_on_sync is enabled
        let skills_to_sync: Vec<_> = if self.config.preferences.validate_on_sync {
            self.skills
                .iter()
                .filter(|s| s.validation_status == ValidationStatus::Valid)
                .cloned()
                .collect()
        } else {
            self.skills.clone()
        };

        self.syncer.sync_all(&self.targets, &skills_to_sync)
    }

    /// Sync all skills to a specific target
    pub fn sync_target(&self, target_id: &str) -> Option<SyncResult> {
        let target = self.targets.iter().find(|t| t.id() == target_id)?;

        // Filter to only valid skills if validate_on_sync is enabled
        let skills_to_sync: Vec<_> = if self.config.preferences.validate_on_sync {
            self.skills
                .iter()
                .filter(|s| s.validation_status == ValidationStatus::Valid)
                .cloned()
                .collect()
        } else {
            self.skills.clone()
        };

        Some(self.syncer.sync_target(target, &skills_to_sync))
    }

    /// Create a new skill (name is automatically converted to kebab-case)
    pub fn create_skill(&mut self, name: &str, description: &str) -> Result<&Skill> {
        // Normalize name to kebab-case
        let normalized_name = crate::skill::to_kebab_case(name);

        // Check if skill already exists
        let skill_path = self.config.skills_dir.join(&normalized_name);
        if skill_path.exists() {
            return Err(Error::SkillAlreadyExists(normalized_name));
        }

        // Create the skill with normalized name
        let skill = Skill::create(&self.config.skills_dir, &normalized_name, description)?;
        self.skills.push(skill);

        // Return a reference to the newly created skill
        Ok(self.skills.last().unwrap())
    }

    /// Delete a skill (removes symlinks first, then the skill directory)
    pub fn delete_skill(&mut self, name: &str) -> Result<()> {
        let skill_path = self.config.skills_dir.join(name);

        if !skill_path.exists() {
            return Err(Error::SkillNotFound(skill_path));
        }

        // Remove symlinks from all targets FIRST
        for target in &self.targets {
            if target.enabled {
                let symlink = target.skill_link_path(name);
                if symlink.exists() || symlink.is_symlink() {
                    let _ = std::fs::remove_file(&symlink);
                }
            }
        }

        // Remove the skill directory
        std::fs::remove_dir_all(&skill_path).map_err(|e| Error::io(&skill_path, e))?;

        // Remove from our list
        self.skills.retain(|s| s.name() != name);

        Ok(())
    }

    /// Rename a skill (renames the skill directory, updates YAML frontmatter, and updates symlinks)
    pub fn rename_skill(&mut self, old_name: &str, new_name: &str) -> Result<&Skill> {
        // Validate new name
        if !crate::skill::is_valid_skill_name(new_name) {
            return Err(Error::InvalidSkillName(new_name.to_string()));
        }

        let old_path = self.config.skills_dir.join(old_name);
        let new_path = self.config.skills_dir.join(new_name);

        if !old_path.exists() {
            return Err(Error::SkillNotFound(old_path));
        }

        if new_path.exists() {
            return Err(Error::SkillAlreadyExists(new_name.to_string()));
        }

        // Remove old symlinks from all targets first
        for target in &self.targets {
            if target.enabled {
                let old_symlink = target.skill_link_path(old_name);
                if old_symlink.is_symlink() {
                    let _ = std::fs::remove_file(&old_symlink);
                }
            }
        }

        // Rename the directory
        std::fs::rename(&old_path, &new_path).map_err(|e| Error::io(&old_path, e))?;

        // Load the skill from the new path (may have old name in YAML)
        let mut skill = Skill::load(&new_path)?;

        // Update the YAML frontmatter to have the new name if it differs
        if skill.name() != new_name {
            let content = skill.raw_content()?;
            let updated_content = Self::update_name_in_content(&content, new_name);
            skill.save_content(&updated_content)?;
        }

        // Update in our list - find by folder_name since YAML name may have been different
        if let Some(existing) = self.skills.iter_mut().find(|s| s.folder_name() == old_name) {
            *existing = skill;
        }

        // Create new symlinks
        for target in &self.targets {
            if target.enabled {
                let new_symlink = target.skill_link_path(new_name);
                #[cfg(unix)]
                let _ = std::os::unix::fs::symlink(&new_path, &new_symlink);
                #[cfg(windows)]
                let _ = std::os::windows::fs::symlink_dir(&new_path, &new_symlink);
            }
        }

        // Return reference to renamed skill - find by folder_name which is now new_name
        self.skills
            .iter()
            .find(|s| s.folder_name() == new_name)
            .ok_or_else(|| Error::SkillNotFound(new_path))
    }

    /// Update the name field in YAML frontmatter content
    fn update_name_in_content(content: &str, new_name: &str) -> String {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return content.to_string();
        }

        let after_first = &trimmed[3..];
        let Some(end_idx) = after_first.find("\n---") else {
            return content.to_string();
        };

        let yaml_section = &after_first[..end_idx];
        let rest = &after_first[end_idx..];

        // Replace the name field in the YAML section
        let mut updated_yaml = String::new();
        let mut found_name = false;
        for line in yaml_section.lines() {
            if line.trim_start().starts_with("name:") && !found_name {
                updated_yaml.push_str(&format!("name: {}", new_name));
                found_name = true;
            } else {
                updated_yaml.push_str(line);
            }
            updated_yaml.push('\n');
        }

        // Remove trailing newline since we'll add the rest
        if updated_yaml.ends_with('\n') {
            updated_yaml.pop();
        }

        format!("---{}{}", updated_yaml, rest)
    }

    /// Get enabled targets
    pub fn enabled_targets(&self) -> impl Iterator<Item = &Target> {
        self.targets.iter().filter(|t| t.enabled)
    }

    /// Get valid skills
    pub fn valid_skills(&self) -> impl Iterator<Item = &Skill> {
        self.skills
            .iter()
            .filter(|s| s.validation_status == ValidationStatus::Valid)
    }

    /// Get invalid skills
    pub fn invalid_skills(&self) -> impl Iterator<Item = &Skill> {
        self.skills
            .iter()
            .filter(|s| s.validation_status == ValidationStatus::Invalid)
    }

    /// Get the raw content of a skill's SKILL.md file
    pub fn get_skill_content(&self, name: &str) -> Result<String> {
        let skill = self
            .get_skill(name)
            .ok_or_else(|| Error::SkillNotFound(self.config.skills_dir.join(name)))?;

        skill.raw_content()
    }

    /// Save content to a skill's SKILL.md file
    /// Save skill content and auto-rename folder if frontmatter name changed
    ///
    /// Returns the (possibly new) skill name after save
    pub fn save_skill_content(&mut self, name: &str, content: &str) -> Result<String> {
        // Extract the name from frontmatter to check if it changed
        let new_name = Self::extract_name_from_content(content);

        // If name changed in frontmatter, rename the skill first
        if let Some(ref new_name) = new_name {
            if new_name != name {
                // Rename will update the folder and symlinks
                self.rename_skill(name, new_name)?;

                // Now save content to the renamed skill
                let skill_path = self.config.skills_dir.join(new_name);
                let skill = self
                    .get_skill_mut(new_name)
                    .ok_or_else(|| Error::SkillNotFound(skill_path))?;
                skill.save_content(content)?;

                return Ok(new_name.clone());
            }
        }

        // No rename needed, just save
        let skill_path = self.config.skills_dir.join(name);
        let skill = self
            .get_skill_mut(name)
            .ok_or_else(|| Error::SkillNotFound(skill_path))?;

        skill.save_content(content)?;
        Ok(name.to_string())
    }

    /// Extract the name field from YAML frontmatter and normalize to kebab-case
    fn extract_name_from_content(content: &str) -> Option<String> {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return None;
        }

        let after_first = &trimmed[3..];
        let end_idx = after_first.find("\n---")?;
        let yaml_content = &after_first[..end_idx];

        // Simple regex-free extraction of name field
        for line in yaml_content.lines() {
            let line = line.trim();
            if line.starts_with("name:") {
                let value = line.strip_prefix("name:")?.trim();
                // Remove quotes if present
                let value = value.trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    // Normalize to kebab-case
                    return Some(crate::skill::to_kebab_case(value));
                }
            }
        }

        None
    }

    /// Fix a skill's frontmatter issues automatically
    ///
    /// Returns the list of fixes applied
    pub fn fix_skill(&mut self, name: &str) -> Result<Vec<String>> {
        let skill_path = self.config.skills_dir.join(name);
        let skill = self
            .get_skill_mut(name)
            .ok_or_else(|| Error::SkillNotFound(skill_path))?;

        skill.fix_frontmatter()
    }

    /// Fix all skills with frontmatter issues
    ///
    /// Returns a map of skill name to fixes applied
    pub fn fix_all_skills(&mut self) -> Vec<(String, Vec<String>)> {
        let names_to_fix: Vec<String> = self
            .skills
            .iter()
            .filter(|s| s.has_fixable_errors())
            .map(|s| s.name().to_string())
            .collect();

        let mut results = Vec::new();
        for name in names_to_fix {
            match self.fix_skill(&name) {
                Ok(fixes) if !fixes.is_empty() => {
                    results.push((name, fixes));
                }
                _ => {}
            }
        }
        results
    }

    /// Get summary statistics
    pub fn stats(&self) -> ManagerStats {
        ManagerStats {
            total_skills: self.skills.len(),
            valid_skills: self
                .skills
                .iter()
                .filter(|s| s.validation_status == ValidationStatus::Valid)
                .count(),
            invalid_skills: self
                .skills
                .iter()
                .filter(|s| s.validation_status == ValidationStatus::Invalid)
                .count(),
            total_targets: self.targets.len(),
            enabled_targets: self.targets.iter().filter(|t| t.enabled).count(),
        }
    }
}

/// Summary statistics for the manager
#[derive(Debug, Clone)]
pub struct ManagerStats {
    pub total_skills: usize,
    pub valid_skills: usize,
    pub invalid_skills: usize,
    pub total_targets: usize,
    pub enabled_targets: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir) -> Config {
        Config {
            skills_dir: temp_dir.path().join("skills"),
            preferences: crate::config::Preferences {
                validate_on_sync: true,
            },
            ..Default::default()
        }
    }

    #[test]
    fn manager_creates_with_config() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);

        let manager = SkillManager::with_config(config).unwrap();
        assert!(manager.skills().is_empty());
    }

    #[test]
    fn manager_creates_skill() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut manager = SkillManager::with_config(config).unwrap();

        let skill = manager.create_skill("test-skill", "A test skill").unwrap();
        assert_eq!(skill.name(), "test-skill");
        assert_eq!(manager.skills().len(), 1);
    }

    #[test]
    fn manager_validates_skill() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut manager = SkillManager::with_config(config).unwrap();

        manager.create_skill("valid-skill", "Valid").unwrap();

        let result = manager.validate_skill("valid-skill");
        assert!(result.is_ok());

        let skill = manager.get_skill("valid-skill").unwrap();
        assert_eq!(skill.validation_status, ValidationStatus::Valid);
    }

    #[test]
    fn manager_deletes_skill() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut manager = SkillManager::with_config(config).unwrap();

        manager
            .create_skill("doomed-skill", "Will be deleted")
            .unwrap();
        assert_eq!(manager.skills().len(), 1);

        manager.delete_skill("doomed-skill").unwrap();
        assert!(manager.skills().is_empty());
    }

    #[test]
    fn manager_refreshes_skills() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut manager = SkillManager::with_config(config.clone()).unwrap();

        // Create skill directly on disk
        Skill::create(&config.skills_dir, "external-skill", "Created externally").unwrap();

        // Manager doesn't know about it yet
        assert!(manager.skills().is_empty());

        // Refresh picks it up
        manager.refresh_skills().unwrap();
        assert_eq!(manager.skills().len(), 1);
    }

    #[test]
    fn manager_get_skill_by_name() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut manager = SkillManager::with_config(config).unwrap();

        manager.create_skill("find-me", "Test").unwrap();
        manager.create_skill("other-skill", "Test").unwrap();

        assert!(manager.get_skill("find-me").is_some());
        assert!(manager.get_skill("not-found").is_none());
    }

    #[test]
    fn manager_stats() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut manager = SkillManager::with_config(config).unwrap();

        manager.create_skill("skill-one", "Test").unwrap();
        manager.create_skill("skill-two", "Test").unwrap();
        manager.validate_skill("skill-one").unwrap();

        let stats = manager.stats();
        assert_eq!(stats.total_skills, 2);
        assert_eq!(stats.valid_skills, 1);
    }

    #[test]
    fn validate_all_validates_all_skills() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut manager = SkillManager::with_config(config).unwrap();

        manager.create_skill("skill-one", "Test").unwrap();
        manager.create_skill("skill-two", "Test").unwrap();

        let results = manager.validate_all();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn rename_skill_updates_folder_and_yaml() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut manager = SkillManager::with_config(config.clone()).unwrap();

        manager
            .create_skill("old-name", "A skill to rename")
            .unwrap();
        assert_eq!(manager.skills().len(), 1);

        // Rename the skill
        let renamed = manager.rename_skill("old-name", "new-name").unwrap();
        assert_eq!(renamed.name(), "new-name");
        assert_eq!(renamed.folder_name(), "new-name");

        // Old folder should not exist, new folder should
        assert!(!config.skills_dir.join("old-name").exists());
        assert!(config.skills_dir.join("new-name").exists());

        // Should be findable by new name
        assert!(manager.get_skill("new-name").is_some());
        assert!(manager.get_skill("old-name").is_none());
    }

    #[test]
    fn save_skill_content_with_name_change_renames_folder() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut manager = SkillManager::with_config(config.clone()).unwrap();

        manager
            .create_skill("original-skill", "Original description")
            .unwrap();

        // Edit content to change the name
        let new_content = r#"---
name: renamed-skill
description: Updated description
---

# Renamed Skill

New content here.
"#;

        let result_name = manager
            .save_skill_content("original-skill", new_content)
            .unwrap();
        assert_eq!(result_name, "renamed-skill");

        // Folder should be renamed
        assert!(!config.skills_dir.join("original-skill").exists());
        assert!(config.skills_dir.join("renamed-skill").exists());

        // Skill should be accessible by new name
        let skill = manager.get_skill("renamed-skill").unwrap();
        assert_eq!(skill.name(), "renamed-skill");
        assert_eq!(skill.description(), "Updated description");
    }

    #[test]
    fn update_name_in_content_replaces_name_field() {
        let content = r#"---
name: old-name
description: A test skill
---

# Content
"#;
        let updated = SkillManager::update_name_in_content(content, "new-name");
        assert!(updated.contains("name: new-name"));
        assert!(!updated.contains("name: old-name"));
        assert!(updated.contains("description: A test skill"));
    }
}
