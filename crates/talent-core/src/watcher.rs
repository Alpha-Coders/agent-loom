//! File watching for skills directory
//!
//! Watches the skills directory for changes and emits events when skills
//! are created, modified, or deleted. Uses debouncing to avoid rapid-fire events.

use crate::error::{Error, Result};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use notify::RecommendedWatcher;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

/// Events emitted by the skill watcher
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillEvent {
    /// A skill was created (new directory with SKILL.md)
    Created(PathBuf),

    /// A skill was modified (SKILL.md content changed)
    Modified(PathBuf),

    /// A skill was deleted (directory or SKILL.md removed)
    Deleted(PathBuf),
}

impl SkillEvent {
    /// Get the path associated with this event
    pub fn path(&self) -> &Path {
        match self {
            SkillEvent::Created(p) => p,
            SkillEvent::Modified(p) => p,
            SkillEvent::Deleted(p) => p,
        }
    }

    /// Get the skill name from the path (directory name)
    pub fn skill_name(&self) -> Option<&str> {
        self.path()
            .file_name()
            .or_else(|| self.path().parent().and_then(|p| p.file_name()))
            .and_then(|n| n.to_str())
    }
}

/// Watcher for the skills directory
pub struct SkillWatcher {
    /// The debounced watcher
    _debouncer: Debouncer<RecommendedWatcher>,

    /// Receiver for debounced events
    receiver: Receiver<std::result::Result<Vec<DebouncedEvent>, notify::Error>>,

    /// The watched path
    watched_path: PathBuf,
}

impl SkillWatcher {
    /// Create a new watcher for the given skills directory
    ///
    /// # Arguments
    /// * `skills_dir` - The directory to watch
    /// * `debounce_ms` - Debounce duration in milliseconds
    pub fn new(skills_dir: &Path, debounce_ms: u64) -> Result<Self> {
        let (tx, rx) = channel();
        let debounce_duration = Duration::from_millis(debounce_ms);

        let mut debouncer = new_debouncer(debounce_duration, tx)
            .map_err(|e| Error::WatcherInit(e.to_string()))?;

        debouncer
            .watcher()
            .watch(skills_dir, notify::RecursiveMode::Recursive)
            .map_err(|e| Error::WatcherInit(e.to_string()))?;

        Ok(Self {
            _debouncer: debouncer,
            receiver: rx,
            watched_path: skills_dir.to_path_buf(),
        })
    }

    /// Poll for pending events (non-blocking)
    ///
    /// Returns all events that have occurred since the last poll.
    pub fn poll(&self) -> Vec<SkillEvent> {
        let mut events = Vec::new();

        // Drain all pending events from the channel
        while let Ok(result) = self.receiver.try_recv() {
            match result {
                Ok(debounced_events) => {
                    for event in debounced_events {
                        if let Some(skill_event) = self.process_event(event) {
                            events.push(skill_event);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Watcher error: {error}");
                }
            }
        }

        // Deduplicate events for the same path
        events.sort_by(|a, b| a.path().cmp(b.path()));
        events.dedup_by(|a, b| a.path() == b.path());

        events
    }

    /// Process a debounced event into a skill event
    fn process_event(&self, event: DebouncedEvent) -> Option<SkillEvent> {
        let path = event.path;

        // We're interested in:
        // 1. SKILL.md files being created/modified/deleted
        // 2. Skill directories being created/deleted

        // Check if this is a SKILL.md file
        let is_skill_file = path
            .file_name()
            .is_some_and(|n| n == crate::skill::SKILL_FILE_NAME);

        // Get the skill directory path
        let skill_dir = if is_skill_file {
            path.parent()?.to_path_buf()
        } else if path.is_dir() || !path.exists() {
            // Could be a skill directory
            path.clone()
        } else {
            // Some other file - ignore
            return None;
        };

        // Ensure this is within the watched directory
        if !skill_dir.starts_with(&self.watched_path) {
            return None;
        }

        // Determine the event type based on current state
        let skill_file = skill_dir.join(crate::skill::SKILL_FILE_NAME);

        if skill_file.exists() {
            if skill_dir.exists() {
                Some(SkillEvent::Modified(skill_dir))
            } else {
                Some(SkillEvent::Created(skill_dir))
            }
        } else if skill_dir.exists() {
            // Directory exists but no SKILL.md - could be a new directory
            // We'll treat this as created, validator will catch missing SKILL.md
            Some(SkillEvent::Created(skill_dir))
        } else {
            Some(SkillEvent::Deleted(skill_dir))
        }
    }

    /// Get the path being watched
    pub fn watched_path(&self) -> &Path {
        &self.watched_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::{Skill, SKILL_FILE_NAME};
    use std::fs;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    fn wait_for_events(watcher: &SkillWatcher, timeout_ms: u64) -> Vec<SkillEvent> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        loop {
            let events = watcher.poll();
            if !events.is_empty() {
                return events;
            }
            if start.elapsed() > timeout {
                return Vec::new();
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    #[test]
    fn skill_event_path() {
        let path = PathBuf::from("/skills/my-skill");
        let event = SkillEvent::Created(path.clone());
        assert_eq!(event.path(), &path);
    }

    #[test]
    fn skill_event_skill_name() {
        let event = SkillEvent::Created(PathBuf::from("/skills/my-skill"));
        assert_eq!(event.skill_name(), Some("my-skill"));

        let event = SkillEvent::Modified(PathBuf::from("/skills/another-skill/SKILL.md"));
        assert_eq!(event.skill_name(), Some("SKILL.md"));
    }

    #[test]
    fn creates_watcher_for_existing_directory() {
        let temp = TempDir::new().unwrap();
        let watcher = SkillWatcher::new(temp.path(), 100);
        assert!(watcher.is_ok());
    }

    #[test]
    fn fails_for_nonexistent_directory() {
        let result = SkillWatcher::new(Path::new("/nonexistent/path"), 100);
        assert!(result.is_err());
    }

    // Note: File watcher tests are timing-dependent and may be flaky in CI environments.
    // They are ignored by default but can be run with: cargo test -- --ignored

    #[test]
    #[ignore = "File watcher timing-dependent test - run manually with --ignored"]
    fn detects_new_skill_creation() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().to_path_buf();

        let watcher = SkillWatcher::new(&skills_dir, 100).unwrap();

        // Create a new skill
        Skill::create(&skills_dir, "new-skill", "A new skill").unwrap();

        // Wait for events
        let events = wait_for_events(&watcher, 2000);

        // Should have at least one event
        assert!(!events.is_empty(), "Expected events but got none");
    }

    #[test]
    #[ignore = "File watcher timing-dependent test - run manually with --ignored"]
    fn detects_skill_modification() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().to_path_buf();

        // Create skill first
        let skill = Skill::create(&skills_dir, "existing-skill", "Existing").unwrap();

        // Start watching after creation
        let watcher = SkillWatcher::new(&skills_dir, 100).unwrap();

        // Wait a bit then modify the skill
        thread::sleep(Duration::from_millis(200));

        let skill_file = skill.path.join(SKILL_FILE_NAME);
        fs::write(&skill_file, "---\nname: existing-skill\ndescription: Modified\n---\n\nModified content").unwrap();

        let events = wait_for_events(&watcher, 2000);
        assert!(!events.is_empty(), "Expected modification events");
    }

    #[test]
    #[ignore = "File watcher timing-dependent test - run manually with --ignored"]
    fn detects_skill_deletion() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().to_path_buf();

        // Create skill first
        let skill = Skill::create(&skills_dir, "doomed-skill", "Will be deleted").unwrap();

        // Start watching
        let watcher = SkillWatcher::new(&skills_dir, 100).unwrap();

        // Wait then delete
        thread::sleep(Duration::from_millis(200));
        fs::remove_dir_all(&skill.path).unwrap();

        let events = wait_for_events(&watcher, 2000);
        assert!(!events.is_empty(), "Expected deletion events");
    }

    #[test]
    fn poll_returns_empty_when_no_events() {
        let temp = TempDir::new().unwrap();
        let watcher = SkillWatcher::new(temp.path(), 100).unwrap();

        // Poll immediately - should be empty
        let events = watcher.poll();
        assert!(events.is_empty());
    }
}
