//! Skill Manager - Integration layer for all Talent components
//!
//! The SkillManager is the main entry point for interacting with Talent.
//! It integrates config, skills, targets, validation, syncing, and file watching.

use crate::config::Config;
use crate::error::{Error, Result};
use crate::skill::{discover_skills, Skill, ValidationStatus};
use crate::syncer::{SyncResult, Syncer};
use crate::target::Target;
use crate::validator::Validator;
use crate::watcher::{SkillEvent, SkillWatcher};

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

    /// File watcher (optional - only created if watching is enabled)
    watcher: Option<SkillWatcher>,
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

        // Detect targets
        let targets = Target::detect_all();

        // Create watcher if auto-sync is enabled
        let watcher = if config.preferences.auto_sync {
            match SkillWatcher::new(&config.skills_dir, config.preferences.watch_debounce_ms) {
                Ok(w) => Some(w),
                Err(e) => {
                    eprintln!("Warning: Could not start file watcher: {e}");
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            config,
            skills,
            targets,
            syncer: Syncer::new(),
            validator: Validator::new(),
            watcher,
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
        self.targets = Target::detect_all();
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

    /// Create a new skill
    pub fn create_skill(&mut self, name: &str, description: &str) -> Result<&Skill> {
        let skill = Skill::create(&self.config.skills_dir, name, description)?;
        self.skills.push(skill);

        // Return a reference to the newly created skill
        Ok(self.skills.last().unwrap())
    }

    /// Delete a skill (removes the skill directory)
    pub fn delete_skill(&mut self, name: &str) -> Result<()> {
        let skill_path = self.config.skills_dir.join(name);

        if !skill_path.exists() {
            return Err(Error::SkillNotFound(skill_path));
        }

        // Remove the directory
        std::fs::remove_dir_all(&skill_path).map_err(|e| Error::io(&skill_path, e))?;

        // Remove from our list
        self.skills.retain(|s| s.name() != name);

        Ok(())
    }

    /// Poll for file watcher events
    ///
    /// Returns any pending events since the last poll.
    /// If the watcher is not enabled, returns an empty vector.
    pub fn poll_events(&self) -> Vec<SkillEvent> {
        match &self.watcher {
            Some(w) => w.poll(),
            None => Vec::new(),
        }
    }

    /// Process file watcher events
    ///
    /// This should be called periodically to handle file changes.
    /// It refreshes skills and optionally triggers a sync.
    pub fn process_events(&mut self) -> Result<Vec<SyncResult>> {
        let events = self.poll_events();

        if events.is_empty() {
            return Ok(Vec::new());
        }

        // Refresh skills to pick up changes
        self.refresh_skills()?;

        // Validate all skills
        self.validate_all();

        // Sync to all targets
        Ok(self.sync_all())
    }

    /// Check if the manager has a file watcher enabled
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
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

    /// Get summary statistics
    pub fn stats(&self) -> ManagerStats {
        ManagerStats {
            total_skills: self.skills.len(),
            valid_skills: self.skills
                .iter()
                .filter(|s| s.validation_status == ValidationStatus::Valid)
                .count(),
            invalid_skills: self.skills
                .iter()
                .filter(|s| s.validation_status == ValidationStatus::Invalid)
                .count(),
            total_targets: self.targets.len(),
            enabled_targets: self.targets.iter().filter(|t| t.enabled).count(),
            is_watching: self.is_watching(),
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
    pub is_watching: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir) -> Config {
        Config {
            skills_dir: temp_dir.path().join("skills"),
            preferences: crate::config::Preferences {
                auto_sync: false, // Disable watcher for tests
                validate_on_sync: true,
                watch_debounce_ms: 100,
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
        assert!(!manager.is_watching()); // Auto-sync disabled
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

        manager.create_skill("doomed-skill", "Will be deleted").unwrap();
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
        assert!(!stats.is_watching);
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
}
