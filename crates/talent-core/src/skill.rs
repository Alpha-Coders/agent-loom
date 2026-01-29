//! Skill model with frontmatter parsing and discovery
//!
//! Skills are Markdown files with YAML frontmatter containing metadata.
//! Format:
//! ```markdown
//! ---
//! name: my-skill
//! description: What this skill does
//! ---
//!
//! Skill content here...
//! ```

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// The filename we look for in skill directories
pub const SKILL_FILE_NAME: &str = "SKILL.md";

/// Skill metadata parsed from YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillMeta {
    /// Unique skill identifier (required)
    pub name: String,

    /// Human-readable description (required)
    pub description: String,

    /// Optional list of tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Optional version string
    #[serde(default)]
    pub version: Option<String>,

    /// Optional author
    #[serde(default)]
    pub author: Option<String>,
}

/// Validation status of a skill
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationStatus {
    /// Not yet validated
    #[default]
    Unknown,
    /// Passed validation
    Valid,
    /// Failed validation
    Invalid,
}

/// A complete skill with metadata, content, and status
#[derive(Debug, Clone)]
pub struct Skill {
    /// Parsed frontmatter metadata
    pub meta: SkillMeta,

    /// Markdown content (after frontmatter)
    pub content: String,

    /// Path to the skill directory
    pub path: PathBuf,

    /// Current validation status
    pub validation_status: ValidationStatus,

    /// Validation errors (if any)
    pub validation_errors: Vec<String>,
}

impl Skill {
    /// Load a skill from a directory containing SKILL.md
    pub fn load(skill_dir: &Path) -> Result<Self> {
        let skill_file = skill_dir.join(SKILL_FILE_NAME);

        if !skill_file.exists() {
            return Err(Error::MissingSkillFile(skill_dir.to_path_buf()));
        }

        let contents = fs::read_to_string(&skill_file).map_err(|e| Error::io(&skill_file, e))?;

        let (meta, content) = Self::parse_frontmatter(&contents, &skill_file)?;

        Ok(Self {
            meta,
            content,
            path: skill_dir.to_path_buf(),
            validation_status: ValidationStatus::Unknown,
            validation_errors: Vec::new(),
        })
    }

    /// Parse YAML frontmatter from content
    fn parse_frontmatter(contents: &str, path: &Path) -> Result<(SkillMeta, String)> {
        let trimmed = contents.trim_start();

        if !trimmed.starts_with("---") {
            return Err(Error::InvalidFrontmatter {
                path: path.to_path_buf(),
                message: "File must start with YAML frontmatter (---)".to_string(),
            });
        }

        // Find the closing ---
        let after_first = &trimmed[3..];
        let end_idx = after_first.find("\n---").ok_or_else(|| Error::InvalidFrontmatter {
            path: path.to_path_buf(),
            message: "Could not find closing frontmatter delimiter (---)".to_string(),
        })?;

        let yaml_content = &after_first[..end_idx].trim();
        let content = after_first[end_idx + 4..].trim().to_string();

        let meta: SkillMeta =
            serde_yaml::from_str(yaml_content).map_err(|e| Error::InvalidFrontmatter {
                path: path.to_path_buf(),
                message: format!("Invalid YAML: {e}"),
            })?;

        Ok((meta, content))
    }

    /// Create a new skill with the given name and write to disk
    pub fn create(skills_dir: &Path, name: &str, description: &str) -> Result<Self> {
        let skill_dir = skills_dir.join(name);
        let skill_file = skill_dir.join(SKILL_FILE_NAME);

        // Create directory
        fs::create_dir_all(&skill_dir).map_err(|e| Error::create_dir(&skill_dir, e))?;

        let meta = SkillMeta {
            name: name.to_string(),
            description: description.to_string(),
            ..Default::default()
        };

        let content = Self::generate_template(&meta);

        fs::write(&skill_file, &content).map_err(|e| Error::io(&skill_file, e))?;

        Self::load(&skill_dir)
    }

    /// Generate the default skill template
    fn generate_template(meta: &SkillMeta) -> String {
        format!(
            r#"---
name: {}
description: {}
---

# {}

{}
"#,
            meta.name, meta.description, meta.name, meta.description
        )
    }

    /// Get the skill name (shorthand for meta.name)
    pub fn name(&self) -> &str {
        &self.meta.name
    }

    /// Get the skill description (shorthand for meta.description)
    pub fn description(&self) -> &str {
        &self.meta.description
    }

    /// Check if this skill has been validated successfully
    pub fn is_valid(&self) -> bool {
        self.validation_status == ValidationStatus::Valid
    }
}

/// Discover all skills in a directory
pub fn discover_skills(skills_dir: &Path) -> Result<Vec<Skill>> {
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();

    for entry in WalkDir::new(skills_dir)
        .min_depth(1)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Look for SKILL.md files
        if path.file_name().is_some_and(|n| n == SKILL_FILE_NAME) {
            if let Some(skill_dir) = path.parent() {
                match Skill::load(skill_dir) {
                    Ok(skill) => skills.push(skill),
                    Err(e) => {
                        // Log but continue discovering other skills
                        eprintln!("Warning: Failed to load skill at {}: {}", skill_dir.display(), e);
                    }
                }
            }
        }
    }

    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_skill_file(dir: &Path, content: &str) {
        let skill_file = dir.join(SKILL_FILE_NAME);
        fs::write(skill_file, content).unwrap();
    }

    #[test]
    fn parses_valid_frontmatter() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("test-skill");
        fs::create_dir(&skill_dir).unwrap();

        create_skill_file(
            &skill_dir,
            r#"---
name: test-skill
description: A test skill
tags:
  - testing
  - example
version: "1.0"
---

# Test Skill

This is the content.
"#,
        );

        let skill = Skill::load(&skill_dir).unwrap();
        assert_eq!(skill.name(), "test-skill");
        assert_eq!(skill.description(), "A test skill");
        assert_eq!(skill.meta.tags, vec!["testing", "example"]);
        assert_eq!(skill.meta.version, Some("1.0".to_string()));
        assert!(skill.content.contains("This is the content"));
    }

    #[test]
    fn error_on_missing_frontmatter() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("bad-skill");
        fs::create_dir(&skill_dir).unwrap();

        create_skill_file(&skill_dir, "# No frontmatter here\n\nJust content.");

        let result = Skill::load(&skill_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("frontmatter"));
    }

    #[test]
    fn error_on_missing_skill_file() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("empty-skill");
        fs::create_dir(&skill_dir).unwrap();

        let result = Skill::load(&skill_dir);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::MissingSkillFile(_)));
    }

    #[test]
    fn error_on_unclosed_frontmatter() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("unclosed-skill");
        fs::create_dir(&skill_dir).unwrap();

        create_skill_file(
            &skill_dir,
            r#"---
name: test
description: No closing delimiter
"#,
        );

        let result = Skill::load(&skill_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("closing"));
    }

    #[test]
    fn create_skill_writes_template() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path();

        let skill = Skill::create(skills_dir, "new-skill", "A brand new skill").unwrap();

        assert_eq!(skill.name(), "new-skill");
        assert_eq!(skill.description(), "A brand new skill");
        assert!(skill.path.join(SKILL_FILE_NAME).exists());
    }

    #[test]
    fn discover_finds_all_skills() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path();

        // Create two skills
        Skill::create(skills_dir, "skill-one", "First skill").unwrap();
        Skill::create(skills_dir, "skill-two", "Second skill").unwrap();

        let skills = discover_skills(skills_dir).unwrap();
        assert_eq!(skills.len(), 2);

        let names: Vec<_> = skills.iter().map(|s| s.name()).collect();
        assert!(names.contains(&"skill-one"));
        assert!(names.contains(&"skill-two"));
    }

    #[test]
    fn discover_handles_empty_directory() {
        let temp = TempDir::new().unwrap();
        let skills = discover_skills(temp.path()).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn discover_handles_nonexistent_directory() {
        let skills = discover_skills(Path::new("/nonexistent/path")).unwrap();
        assert!(skills.is_empty());
    }
}
