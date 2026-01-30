//! Validation engine for skills
//!
//! Validates skills against agentskills.io specification:
//! - name: 1-64 chars, lowercase alphanumeric + hyphens, no leading/trailing/consecutive hyphens
//! - name must match parent directory name
//! - description: 1-1024 chars
//! - compatibility: max 500 chars (if provided)
//! - Content presence (optional)
//!
//! See https://agentskills.io/specification for full spec.

use crate::error::{Error, Result};
use crate::skill::{Skill, ValidationStatus};

/// Maximum length for skill name (per agentskills.io spec)
const MAX_NAME_LENGTH: usize = 64;

/// Maximum length for skill description (per agentskills.io spec)
const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Maximum length for compatibility field (per agentskills.io spec)
const MAX_COMPATIBILITY_LENGTH: usize = 500;

/// Validator for skills
pub struct Validator {
    /// Whether to require non-empty content
    pub require_content: bool,
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            require_content: true,
        }
    }
}

impl Validator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate a skill, updating its validation status
    pub fn validate(&self, skill: &mut Skill) -> Result<()> {
        // Preserve any loading errors (these indicate frontmatter issues)
        let loading_errors: Vec<String> = skill
            .validation_errors
            .iter()
            .filter(|e| e.contains("frontmatter") || e.contains("auto-fixed"))
            .cloned()
            .collect();

        let mut errors = Vec::new();

        // Check required fields
        if skill.meta.name.is_empty() {
            errors.push("name is required".to_string());
        } else {
            // Validate name length (max 64 chars)
            if skill.meta.name.len() > MAX_NAME_LENGTH {
                errors.push(format!(
                    "name exceeds {} characters (has {})",
                    MAX_NAME_LENGTH,
                    skill.meta.name.len()
                ));
            }

            // Validate name format (kebab-case)
            if !is_kebab_case(&skill.meta.name) {
                errors.push(format!(
                    "name '{}' must be kebab-case (lowercase letters, numbers, hyphens; no leading/trailing/consecutive hyphens)",
                    skill.meta.name
                ));
            }

            // Validate name matches parent directory (per agentskills.io spec)
            let folder_name = skill.folder_name();
            if folder_name != skill.meta.name {
                errors.push(format!(
                    "name '{}' must match parent directory '{}' (per agentskills.io spec)",
                    skill.meta.name, folder_name
                ));
            }
        }

        if skill.meta.description.is_empty() {
            errors.push("description is required".to_string());
        } else if skill.meta.description.len() > MAX_DESCRIPTION_LENGTH {
            errors.push(format!(
                "description exceeds {} characters (has {})",
                MAX_DESCRIPTION_LENGTH,
                skill.meta.description.len()
            ));
        }

        // Validate optional compatibility field (max 500 chars per spec)
        if let Some(ref compat) = skill.meta.compatibility {
            if compat.len() > MAX_COMPATIBILITY_LENGTH {
                errors.push(format!(
                    "compatibility exceeds {} characters (has {})",
                    MAX_COMPATIBILITY_LENGTH,
                    compat.len()
                ));
            }
        }

        // Check content presence
        if self.require_content && skill.content.trim().is_empty() {
            errors.push("skill must have content".to_string());
        }

        // Combine loading errors with validation errors
        let all_errors: Vec<String> = loading_errors
            .into_iter()
            .chain(errors.into_iter())
            .collect();

        // Update skill status
        if all_errors.is_empty() {
            skill.validation_status = ValidationStatus::Valid;
            skill.validation_errors.clear();
            Ok(())
        } else {
            skill.validation_status = ValidationStatus::Invalid;
            let message = all_errors.join("; ");
            skill.validation_errors = all_errors;
            Err(Error::ValidationFailed {
                name: skill.meta.name.clone(),
                message,
            })
        }
    }

    /// Validate multiple skills, returning all results
    pub fn validate_all(&self, skills: &mut [Skill]) -> Vec<Result<()>> {
        skills.iter_mut().map(|s| self.validate(s)).collect()
    }
}

/// Check if a string is valid kebab-case
fn is_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Must start and end with alphanumeric
    let chars: Vec<char> = s.chars().collect();
    if !chars.first().is_some_and(|c| c.is_ascii_lowercase()) {
        return false;
    }
    if !chars.last().is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
        return false;
    }

    // Only lowercase letters, digits, and hyphens allowed
    // No consecutive hyphens
    let mut prev_hyphen = false;
    for c in chars {
        if c == '-' {
            if prev_hyphen {
                return false; // No consecutive hyphens
            }
            prev_hyphen = true;
        } else if c.is_ascii_lowercase() || c.is_ascii_digit() {
            prev_hyphen = false;
        } else {
            return false; // Invalid character
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::SkillMeta;
    use std::path::PathBuf;

    /// Create a test skill with matching folder name (valid by default)
    fn create_test_skill(name: &str, description: &str, content: &str) -> Skill {
        // Path ends with the skill name so folder_name() matches meta.name
        Skill {
            meta: SkillMeta {
                name: name.to_string(),
                description: description.to_string(),
                ..Default::default()
            },
            content: content.to_string(),
            path: PathBuf::from(format!("/test/skills/{}", name)),
            validation_status: ValidationStatus::Unknown,
            validation_errors: Vec::new(),
        }
    }

    /// Create a test skill with mismatched folder name
    fn create_test_skill_with_path(
        name: &str,
        description: &str,
        content: &str,
        path: &str,
    ) -> Skill {
        Skill {
            meta: SkillMeta {
                name: name.to_string(),
                description: description.to_string(),
                ..Default::default()
            },
            content: content.to_string(),
            path: PathBuf::from(path),
            validation_status: ValidationStatus::Unknown,
            validation_errors: Vec::new(),
        }
    }

    /// Create a test skill with compatibility field
    fn create_test_skill_with_compatibility(
        name: &str,
        description: &str,
        content: &str,
        compatibility: Option<&str>,
    ) -> Skill {
        let mut meta = SkillMeta {
            name: name.to_string(),
            description: description.to_string(),
            ..Default::default()
        };
        meta.compatibility = compatibility.map(String::from);
        Skill {
            meta,
            content: content.to_string(),
            path: PathBuf::from(format!("/test/skills/{}", name)),
            validation_status: ValidationStatus::Unknown,
            validation_errors: Vec::new(),
        }
    }

    #[test]
    fn valid_skill_passes() {
        let validator = Validator::new();
        let mut skill = create_test_skill("my-skill", "A test skill", "Some content here");

        let result = validator.validate(&mut skill);
        assert!(result.is_ok());
        assert_eq!(skill.validation_status, ValidationStatus::Valid);
        assert!(skill.validation_errors.is_empty());
    }

    #[test]
    fn missing_name_fails() {
        let validator = Validator::new();
        let mut skill = create_test_skill("", "A description", "Content");

        let result = validator.validate(&mut skill);
        assert!(result.is_err());
        assert_eq!(skill.validation_status, ValidationStatus::Invalid);
        assert!(skill.validation_errors.iter().any(|e| e.contains("name is required")));
    }

    #[test]
    fn missing_description_fails() {
        let validator = Validator::new();
        let mut skill = create_test_skill("my-skill", "", "Content");

        let result = validator.validate(&mut skill);
        assert!(result.is_err());
        assert_eq!(skill.validation_status, ValidationStatus::Invalid);
        assert!(skill
            .validation_errors
            .iter()
            .any(|e| e.contains("description is required")));
    }

    #[test]
    fn empty_content_fails_by_default() {
        let validator = Validator::new();
        let mut skill = create_test_skill("my-skill", "Description", "   ");

        let result = validator.validate(&mut skill);
        assert!(result.is_err());
        assert!(skill
            .validation_errors
            .iter()
            .any(|e| e.contains("must have content")));
    }

    #[test]
    fn empty_content_allowed_when_disabled() {
        let validator = Validator {
            require_content: false,
        };
        let mut skill = create_test_skill("my-skill", "Description", "");

        let result = validator.validate(&mut skill);
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_name_format_fails() {
        let validator = Validator::new();

        // Uppercase
        let mut skill = create_test_skill("MySkill", "Desc", "Content");
        assert!(validator.validate(&mut skill).is_err());
        assert!(skill.validation_errors.iter().any(|e| e.contains("kebab-case")));

        // Underscores
        let mut skill = create_test_skill("my_skill", "Desc", "Content");
        assert!(validator.validate(&mut skill).is_err());

        // Spaces
        let mut skill = create_test_skill("my skill", "Desc", "Content");
        assert!(validator.validate(&mut skill).is_err());

        // Starting with hyphen
        let mut skill = create_test_skill("-my-skill", "Desc", "Content");
        assert!(validator.validate(&mut skill).is_err());

        // Ending with hyphen
        let mut skill = create_test_skill("my-skill-", "Desc", "Content");
        assert!(validator.validate(&mut skill).is_err());

        // Consecutive hyphens
        let mut skill = create_test_skill("my--skill", "Desc", "Content");
        assert!(validator.validate(&mut skill).is_err());
    }

    #[test]
    fn valid_kebab_case_names() {
        let validator = Validator::new();

        let valid_names = [
            "skill",
            "my-skill",
            "skill-with-many-parts",
            "skill1",
            "my-skill-2",
            "a",
            "ab",
            "a1",
        ];

        for name in valid_names {
            let mut skill = create_test_skill(name, "Desc", "Content");
            let result = validator.validate(&mut skill);
            assert!(result.is_ok(), "Expected '{}' to be valid", name);
        }
    }

    #[test]
    fn validate_all_returns_all_results() {
        let validator = Validator::new();
        let mut skills = vec![
            create_test_skill("valid-skill", "Desc", "Content"),
            create_test_skill("", "Desc", "Content"), // Invalid
            create_test_skill("another-valid", "Desc", "Content"),
        ];

        let results = validator.validate_all(&mut skills);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());
    }

    #[test]
    fn is_kebab_case_function() {
        assert!(is_kebab_case("valid"));
        assert!(is_kebab_case("valid-name"));
        assert!(is_kebab_case("valid-name-123"));
        assert!(is_kebab_case("a"));

        assert!(!is_kebab_case(""));
        assert!(!is_kebab_case("Invalid"));
        assert!(!is_kebab_case("invalid_name"));
        assert!(!is_kebab_case("-invalid"));
        assert!(!is_kebab_case("invalid-"));
        assert!(!is_kebab_case("in--valid"));
        assert!(!is_kebab_case("1invalid")); // Starts with digit
    }

    #[test]
    fn name_must_match_directory() {
        let validator = Validator::new();

        // Skill with mismatched folder name
        let mut skill =
            create_test_skill_with_path("my-skill", "Desc", "Content", "/test/skills/wrong-folder");

        let result = validator.validate(&mut skill);
        assert!(result.is_err());
        assert_eq!(skill.validation_status, ValidationStatus::Invalid);
        assert!(skill
            .validation_errors
            .iter()
            .any(|e| e.contains("must match parent directory")));
    }

    #[test]
    fn matching_name_and_directory_passes() {
        let validator = Validator::new();

        // Skill with matching folder name
        let mut skill =
            create_test_skill_with_path("my-skill", "Desc", "Content", "/test/skills/my-skill");

        let result = validator.validate(&mut skill);
        assert!(result.is_ok());
        assert_eq!(skill.validation_status, ValidationStatus::Valid);
    }

    #[test]
    fn compatibility_field_max_length() {
        let validator = Validator::new();

        // Valid compatibility (under 500 chars)
        let mut skill = create_test_skill_with_compatibility(
            "my-skill",
            "Desc",
            "Content",
            Some("Requires git, docker, and network access"),
        );
        assert!(validator.validate(&mut skill).is_ok());

        // Invalid compatibility (over 500 chars)
        let long_compat = "a".repeat(501);
        let mut skill =
            create_test_skill_with_compatibility("my-skill", "Desc", "Content", Some(&long_compat));

        let result = validator.validate(&mut skill);
        assert!(result.is_err());
        assert!(skill
            .validation_errors
            .iter()
            .any(|e| e.contains("compatibility exceeds")));
    }

    #[test]
    fn compatibility_at_exactly_500_chars_passes() {
        let validator = Validator::new();

        let compat_500 = "a".repeat(500);
        let mut skill =
            create_test_skill_with_compatibility("my-skill", "Desc", "Content", Some(&compat_500));

        let result = validator.validate(&mut skill);
        assert!(result.is_ok());
    }

    #[test]
    fn no_compatibility_field_passes() {
        let validator = Validator::new();

        let mut skill = create_test_skill_with_compatibility("my-skill", "Desc", "Content", None);

        let result = validator.validate(&mut skill);
        assert!(result.is_ok());
    }
}
