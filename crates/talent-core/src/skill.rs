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

/// Check if a skill name is valid (kebab-case)
///
/// Valid names:
/// - Start with a lowercase letter
/// - Contain only lowercase letters, digits, and hyphens
/// - No leading, trailing, or consecutive hyphens
pub fn is_valid_skill_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let chars: Vec<char> = name.chars().collect();

    // Must start with lowercase letter
    if !chars[0].is_ascii_lowercase() {
        return false;
    }

    // Must end with alphanumeric
    if !chars.last().unwrap().is_ascii_alphanumeric() {
        return false;
    }

    // Check all characters and no consecutive hyphens
    let mut prev_was_hyphen = false;
    for c in &chars {
        if *c == '-' {
            if prev_was_hyphen {
                return false; // No consecutive hyphens
            }
            prev_was_hyphen = true;
        } else if c.is_ascii_lowercase() || c.is_ascii_digit() {
            prev_was_hyphen = false;
        } else {
            return false; // Invalid character
        }
    }

    true
}

/// Convert any string to valid kebab-case
///
/// Examples:
/// - "Test Skill" -> "test-skill"
/// - "TEST" -> "test"
/// - "My_Cool_Skill" -> "my-cool-skill"
/// - "hello world 123" -> "hello-world-123"
/// - "  Multiple   Spaces  " -> "multiple-spaces"
pub fn to_kebab_case(input: &str) -> String {
    let mut result = String::new();
    let mut prev_was_separator = true; // Start true to skip leading separators

    for c in input.chars() {
        if c.is_ascii_alphanumeric() {
            // Convert uppercase to lowercase
            if c.is_ascii_uppercase() {
                // If previous char was lowercase, add hyphen before (camelCase handling)
                if !prev_was_separator && !result.is_empty() {
                    let last_char = result.chars().last().unwrap();
                    if last_char.is_ascii_lowercase() {
                        result.push('-');
                    }
                }
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
            prev_was_separator = false;
        } else if !prev_was_separator && !result.is_empty() {
            // Non-alphanumeric becomes hyphen (but not consecutive)
            result.push('-');
            prev_was_separator = true;
        }
    }

    // Remove trailing hyphen
    while result.ends_with('-') {
        result.pop();
    }

    // Ensure it starts with a letter (prepend 'skill-' if starts with digit)
    if result.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        result = format!("skill-{}", result);
    }

    // If empty, return a default
    if result.is_empty() {
        return "unnamed-skill".to_string();
    }

    result
}

/// Result of normalizing frontmatter
#[derive(Debug, Clone)]
pub struct NormalizeResult {
    /// The normalized YAML content
    pub yaml: String,
    /// List of fixes that were applied
    pub fixes: Vec<String>,
    /// Whether any fixes were made
    pub was_modified: bool,
}

/// Normalize YAML frontmatter to fix common issues
///
/// Fixes applied:
/// - Converts non-string metadata values to strings (arrays become comma-separated)
/// - Ensures name exists (uses folder_name fallback)
/// - Ensures description exists (uses placeholder)
/// - Converts name to kebab-case if needed
pub fn normalize_frontmatter(yaml_content: &str, folder_name: &str) -> NormalizeResult {
    let mut fixes = Vec::new();

    // Parse as generic YAML Value
    let mut value: serde_yaml::Value = match serde_yaml::from_str(yaml_content) {
        Ok(v) => v,
        Err(_) => {
            // Can't even parse as generic YAML - return minimal valid frontmatter
            fixes.push("Replaced unparseable YAML with minimal frontmatter".to_string());
            return NormalizeResult {
                yaml: format!(
                    "name: {}\ndescription: Skill imported with invalid frontmatter",
                    folder_name
                ),
                fixes,
                was_modified: true,
            };
        }
    };

    let map = match value.as_mapping_mut() {
        Some(m) => m,
        None => {
            fixes.push("YAML root was not a mapping, replaced with minimal frontmatter".to_string());
            return NormalizeResult {
                yaml: format!(
                    "name: {}\ndescription: Skill imported with invalid frontmatter",
                    folder_name
                ),
                fixes,
                was_modified: true,
            };
        }
    };

    // Fix: Ensure name exists and is kebab-case
    let name_key = serde_yaml::Value::String("name".to_string());
    let current_name = map
        .get(&name_key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match current_name {
        Some(name) if !is_valid_skill_name(&name) => {
            let fixed_name = to_kebab_case(&name);
            fixes.push(format!("Converted name '{}' to kebab-case '{}'", name, fixed_name));
            map.insert(name_key.clone(), serde_yaml::Value::String(fixed_name));
        }
        None => {
            fixes.push(format!("Added missing name field: '{}'", folder_name));
            map.insert(name_key.clone(), serde_yaml::Value::String(folder_name.to_string()));
        }
        _ => {}
    }

    // Fix: Ensure description exists
    let desc_key = serde_yaml::Value::String("description".to_string());
    if !map.contains_key(&desc_key) {
        fixes.push("Added missing description field".to_string());
        map.insert(
            desc_key,
            serde_yaml::Value::String("No description provided".to_string()),
        );
    }

    // Fix: Convert metadata values to strings
    let metadata_key = serde_yaml::Value::String("metadata".to_string());
    if let Some(metadata) = map.get_mut(&metadata_key) {
        if let Some(meta_map) = metadata.as_mapping_mut() {
            let keys_to_fix: Vec<_> = meta_map
                .iter()
                .filter(|(_, v)| !v.is_string())
                .map(|(k, _)| k.clone())
                .collect();

            for key in keys_to_fix {
                if let Some(val) = meta_map.get(&key) {
                    let key_str = key.as_str().unwrap_or("unknown");
                    let string_val = yaml_value_to_string(val);
                    fixes.push(format!(
                        "Converted metadata.{} from complex type to string",
                        key_str
                    ));
                    meta_map.insert(key.clone(), serde_yaml::Value::String(string_val));
                }
            }
        }
    }

    let was_modified = !fixes.is_empty();

    // Re-serialize
    let yaml = serde_yaml::to_string(&value).unwrap_or_else(|_| yaml_content.to_string());
    // Remove trailing newline that serde_yaml adds
    let yaml = yaml.trim_end().to_string();

    NormalizeResult {
        yaml,
        fixes,
        was_modified,
    }
}

/// Convert any YAML value to a string representation
fn yaml_value_to_string(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::Null => String::new(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Sequence(seq) => {
            // Convert array to comma-separated string
            seq.iter()
                .filter_map(|v| {
                    if v.is_string() {
                        v.as_str().map(|s| s.to_string())
                    } else {
                        Some(yaml_value_to_string(v))
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        }
        serde_yaml::Value::Mapping(map) => {
            // Convert mapping to JSON-like string
            let pairs: Vec<String> = map
                .iter()
                .filter_map(|(k, v)| {
                    let key = k.as_str()?;
                    let val = yaml_value_to_string(v);
                    Some(format!("{}={}", key, val))
                })
                .collect();
            pairs.join("; ")
        }
        serde_yaml::Value::Tagged(tagged) => yaml_value_to_string(&tagged.value),
    }
}

/// Skill metadata parsed from YAML frontmatter
/// See https://agentskills.io/specification for the full spec
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillMeta {
    /// Unique skill identifier (required, 1-64 chars, kebab-case)
    pub name: String,

    /// Human-readable description (required, 1-1024 chars)
    pub description: String,

    /// License name or reference to bundled license file (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Environment requirements: intended product, system packages, network access (optional, max 500 chars)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,

    /// Arbitrary key-value metadata (optional)
    /// Spec recommends: author, version, etc.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub metadata: std::collections::HashMap<String, String>,

    /// Pre-approved tools the skill may use (experimental, space-delimited)
    /// Example: "Bash(git:*) Bash(jq:*) Read"
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "allowed-tools")]
    pub allowed_tools: Option<String>,

    // --- Legacy fields (not in spec, kept for backward compatibility) ---

    /// Optional list of tags for categorization (legacy, not in spec)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Optional version string (legacy - spec puts this in metadata)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Optional author (legacy - spec puts this in metadata)
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

    /// Load a skill leniently - always returns a Skill, capturing errors instead of failing.
    /// This allows displaying broken skills to users with error badges.
    pub fn load_lenient(skill_dir: &Path) -> Self {
        let skill_file = skill_dir.join(SKILL_FILE_NAME);
        let folder_name = skill_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Try to read the file
        let contents = match fs::read_to_string(&skill_file) {
            Ok(c) => c,
            Err(e) => {
                return Self {
                    meta: SkillMeta {
                        name: folder_name.clone(),
                        description: format!("Failed to read: {}", e),
                        ..Default::default()
                    },
                    content: String::new(),
                    path: skill_dir.to_path_buf(),
                    validation_status: ValidationStatus::Invalid,
                    validation_errors: vec![format!("Cannot read SKILL.md: {}", e)],
                };
            }
        };

        // Try to parse frontmatter
        match Self::parse_frontmatter(&contents, &skill_file) {
            Ok((meta, content)) => Self {
                meta,
                content,
                path: skill_dir.to_path_buf(),
                validation_status: ValidationStatus::Unknown,
                validation_errors: Vec::new(),
            },
            Err(e) => {
                // Parsing failed - try normalizing the frontmatter
                let (partial_meta, raw_content, normalize_result) =
                    Self::parse_with_normalization(&contents, &folder_name);

                let mut errors = vec![e.to_string()];
                if normalize_result.was_modified {
                    errors.push(format!(
                        "Frontmatter can be auto-fixed: {}",
                        normalize_result.fixes.join(", ")
                    ));
                }

                Self {
                    meta: partial_meta,
                    content: raw_content,
                    path: skill_dir.to_path_buf(),
                    validation_status: ValidationStatus::Invalid,
                    validation_errors: errors,
                }
            }
        }
    }

    /// Try to parse frontmatter, normalizing if needed
    fn parse_with_normalization(
        contents: &str,
        folder_name: &str,
    ) -> (SkillMeta, String, NormalizeResult) {
        let trimmed = contents.trim_start();

        // Extract YAML section
        if !trimmed.starts_with("---") {
            let result = NormalizeResult {
                yaml: format!("name: {}\ndescription: No frontmatter found", folder_name),
                fixes: vec!["Added missing frontmatter".to_string()],
                was_modified: true,
            };
            return (
                SkillMeta {
                    name: folder_name.to_string(),
                    description: "No frontmatter found".to_string(),
                    ..Default::default()
                },
                contents.to_string(),
                result,
            );
        }

        let after_first = &trimmed[3..];
        let (yaml_content, content) = match after_first.find("\n---") {
            Some(end_idx) => (
                after_first[..end_idx].trim().to_string(),
                after_first[end_idx + 4..].trim().to_string(),
            ),
            None => (after_first.trim().to_string(), String::new()),
        };

        // Try normalizing the YAML
        let normalize_result = normalize_frontmatter(&yaml_content, folder_name);

        // Try parsing the normalized YAML
        let meta = serde_yaml::from_str::<SkillMeta>(&normalize_result.yaml).unwrap_or_else(|_| {
            // Even normalized YAML failed - use lenient extraction
            let mut meta = SkillMeta {
                name: folder_name.to_string(),
                description: String::new(),
                ..Default::default()
            };
            if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&yaml_content) {
                if let Some(map) = value.as_mapping() {
                    if let Some(name) = map.get("name").and_then(|v| v.as_str()) {
                        meta.name = name.to_string();
                    }
                    if let Some(desc) = map.get("description").and_then(|v| v.as_str()) {
                        meta.description = desc.to_string();
                    }
                }
            }
            meta
        });

        (meta, content, normalize_result)
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

    /// Get the skill name (from frontmatter)
    pub fn name(&self) -> &str {
        &self.meta.name
    }

    /// Get the folder name (actual directory name on disk)
    /// This may differ from name() if frontmatter name doesn't match folder
    pub fn folder_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&self.meta.name)
    }

    /// Get the skill description (shorthand for meta.description)
    pub fn description(&self) -> &str {
        &self.meta.description
    }

    /// Check if this skill has been validated successfully
    pub fn is_valid(&self) -> bool {
        self.validation_status == ValidationStatus::Valid
    }

    /// Get the raw content of the SKILL.md file
    pub fn raw_content(&self) -> Result<String> {
        let skill_file = self.path.join(SKILL_FILE_NAME);
        fs::read_to_string(&skill_file).map_err(|e| Error::io(&skill_file, e))
    }

    /// Save content to the SKILL.md file and reload the skill
    ///
    /// Always saves the file, even if frontmatter is invalid.
    /// If parsing fails, keeps the old metadata and marks as invalid.
    pub fn save_content(&mut self, content: &str) -> Result<()> {
        let skill_file = self.path.join(SKILL_FILE_NAME);

        // Always write the file first - user should be able to save work-in-progress
        fs::write(&skill_file, content).map_err(|e| Error::io(&skill_file, e))?;

        // Try to re-parse the frontmatter to update metadata
        // If parsing fails, keep old metadata but mark as invalid
        match Self::parse_frontmatter(content, &skill_file) {
            Ok((meta, parsed_content)) => {
                self.meta = meta;
                self.content = parsed_content;
                // Reset validation status since content changed
                self.validation_status = ValidationStatus::Unknown;
                self.validation_errors.clear();
            }
            Err(e) => {
                // Save succeeded but parsing failed - mark as invalid
                self.content = content.to_string();
                self.validation_status = ValidationStatus::Invalid;
                self.validation_errors = vec![e.to_string()];
            }
        }

        Ok(())
    }

    /// Fix frontmatter issues and save the skill
    ///
    /// Returns the list of fixes applied, or empty if no fixes were needed.
    pub fn fix_frontmatter(&mut self) -> Result<Vec<String>> {
        let skill_file = self.path.join(SKILL_FILE_NAME);
        let contents = fs::read_to_string(&skill_file).map_err(|e| Error::io(&skill_file, e))?;
        let trimmed = contents.trim_start();

        // Extract current frontmatter boundaries
        if !trimmed.starts_with("---") {
            // No frontmatter - create one
            let new_content = format!(
                "---\nname: {}\ndescription: {}\n---\n\n{}",
                self.folder_name(),
                if self.meta.description.is_empty() {
                    "No description provided"
                } else {
                    &self.meta.description
                },
                contents
            );
            self.save_content(&new_content)?;
            return Ok(vec!["Added missing frontmatter".to_string()]);
        }

        let after_first = &trimmed[3..];
        let Some(end_idx) = after_first.find("\n---") else {
            // Unclosed frontmatter - can't safely fix
            return Ok(Vec::new());
        };

        let yaml_content = after_first[..end_idx].trim();
        let body_content = after_first[end_idx + 4..].trim();

        // Normalize the YAML
        let normalize_result = normalize_frontmatter(yaml_content, self.folder_name());

        if !normalize_result.was_modified {
            return Ok(Vec::new());
        }

        // Rebuild the file with normalized frontmatter
        let new_content = format!("---\n{}\n---\n\n{}", normalize_result.yaml, body_content);

        self.save_content(&new_content)?;

        Ok(normalize_result.fixes)
    }

    /// Check if this skill has fixable frontmatter issues
    pub fn has_fixable_errors(&self) -> bool {
        !self.validation_errors.is_empty()
            && self
                .validation_errors
                .iter()
                .any(|e| e.contains("can be auto-fixed"))
    }
}

/// Discover all skills in a directory
/// Uses lenient loading to include skills with errors (for UI display with error badges)
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
                // Use lenient loading to capture errors instead of skipping
                let skill = Skill::load_lenient(skill_dir);
                if !skill.validation_errors.is_empty() {
                    eprintln!(
                        "Warning: Skill '{}' has errors: {}",
                        skill.folder_name(),
                        skill.validation_errors.join(", ")
                    );
                }
                skills.push(skill);
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

    #[test]
    fn raw_content_returns_full_file() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path();

        let skill = Skill::create(skills_dir, "content-skill", "A skill").unwrap();
        let content = skill.raw_content().unwrap();

        assert!(content.contains("---"));
        assert!(content.contains("name: content-skill"));
        assert!(content.contains("description: A skill"));
    }

    #[test]
    fn save_content_updates_skill() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path();

        let mut skill = Skill::create(skills_dir, "editable-skill", "Original").unwrap();
        assert_eq!(skill.description(), "Original");

        let new_content = r#"---
name: editable-skill
description: Updated description
tags:
  - new-tag
---

# New Content

This is the new content.
"#;

        skill.save_content(new_content).unwrap();

        assert_eq!(skill.description(), "Updated description");
        assert_eq!(skill.meta.tags, vec!["new-tag"]);
        assert!(skill.content.contains("This is the new content"));
        assert_eq!(skill.validation_status, ValidationStatus::Unknown);
    }

    #[test]
    fn load_lenient_captures_yaml_errors() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("broken-skill");
        fs::create_dir(&skill_dir).unwrap();

        // Create skill with invalid YAML (metadata.triggers is array, but expected string)
        create_skill_file(
            &skill_dir,
            r#"---
name: broken-skill
description: A skill with invalid metadata
metadata:
  triggers:
    - item1
    - item2
---

# Broken Skill

Content here.
"#,
        );

        // Strict load should fail
        let strict_result = Skill::load(&skill_dir);
        assert!(strict_result.is_err());

        // Lenient load should succeed with errors captured
        let skill = Skill::load_lenient(&skill_dir);
        assert_eq!(skill.name(), "broken-skill");
        assert_eq!(skill.description(), "A skill with invalid metadata");
        assert_eq!(skill.validation_status, ValidationStatus::Invalid);
        assert!(!skill.validation_errors.is_empty());
        assert!(skill.validation_errors[0].contains("metadata"));
    }

    #[test]
    fn discover_includes_broken_skills() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path();

        // Create a valid skill
        Skill::create(skills_dir, "valid-skill", "Works fine").unwrap();

        // Create a broken skill
        let broken_dir = skills_dir.join("broken-skill");
        fs::create_dir(&broken_dir).unwrap();
        create_skill_file(
            &broken_dir,
            r#"---
name: broken-skill
description: Has invalid fields
metadata:
  bad_field:
    - not
    - a
    - string
---

Content
"#,
        );

        let skills = discover_skills(skills_dir).unwrap();
        assert_eq!(skills.len(), 2);

        // Both skills should be present
        let valid = skills.iter().find(|s| s.name() == "valid-skill").unwrap();
        let broken = skills.iter().find(|s| s.name() == "broken-skill").unwrap();

        assert!(valid.validation_errors.is_empty());
        assert!(!broken.validation_errors.is_empty());
        assert_eq!(broken.validation_status, ValidationStatus::Invalid);
    }

    #[test]
    fn normalize_frontmatter_converts_array_metadata() {
        let yaml = r#"name: test-skill
description: A test
metadata:
  triggers:
    - item1
    - item2"#;

        let result = normalize_frontmatter(yaml, "test-skill");

        assert!(result.was_modified);
        assert!(result.fixes.iter().any(|f| f.contains("metadata.triggers")));

        // Verify the normalized YAML can be parsed
        let meta: SkillMeta = serde_yaml::from_str(&result.yaml).unwrap();
        assert_eq!(meta.name, "test-skill");
        assert_eq!(meta.metadata.get("triggers"), Some(&"item1, item2".to_string()));
    }

    #[test]
    fn normalize_frontmatter_adds_missing_fields() {
        let yaml = "tags:\n  - test";

        let result = normalize_frontmatter(yaml, "my-skill");

        assert!(result.was_modified);
        assert!(result.fixes.iter().any(|f| f.contains("missing name")));
        assert!(result.fixes.iter().any(|f| f.contains("missing description")));

        let meta: SkillMeta = serde_yaml::from_str(&result.yaml).unwrap();
        assert_eq!(meta.name, "my-skill");
        assert!(!meta.description.is_empty());
    }

    #[test]
    fn normalize_frontmatter_converts_name_to_kebab_case() {
        let yaml = r#"name: "Test Skill Name"
description: A test"#;

        let result = normalize_frontmatter(yaml, "folder");

        assert!(result.was_modified);
        assert!(result.fixes.iter().any(|f| f.contains("kebab-case")));

        let meta: SkillMeta = serde_yaml::from_str(&result.yaml).unwrap();
        assert_eq!(meta.name, "test-skill-name");
    }

    #[test]
    fn normalize_frontmatter_no_changes_for_valid() {
        let yaml = r#"name: valid-skill
description: A valid skill"#;

        let result = normalize_frontmatter(yaml, "valid-skill");

        assert!(!result.was_modified);
        assert!(result.fixes.is_empty());
    }

    #[test]
    fn fix_frontmatter_saves_normalized_content() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("broken-skill");
        fs::create_dir(&skill_dir).unwrap();

        create_skill_file(
            &skill_dir,
            r#"---
name: broken-skill
description: Has array metadata
metadata:
  triggers:
    - item1
    - item2
---

# Content

Body here.
"#,
        );

        let mut skill = Skill::load_lenient(&skill_dir);
        assert!(skill.has_fixable_errors());

        let fixes = skill.fix_frontmatter().unwrap();
        assert!(!fixes.is_empty());

        // Reload and verify it's now valid
        let reloaded = Skill::load(&skill_dir).unwrap();
        assert_eq!(reloaded.meta.metadata.get("triggers"), Some(&"item1, item2".to_string()));
    }
}
