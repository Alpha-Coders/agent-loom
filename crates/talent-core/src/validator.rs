//! Validation engine for skills
//!
//! Validates skills against agentskills.io specification:
//! - name: 1-64 chars, lowercase alphanumeric + hyphens, no leading/trailing/consecutive hyphens
//! - description: 1-1024 chars
//! - Content presence (optional)
//!
//! See https://agentskills.io/specification for full spec.

use crate::error::{Error, Result};
use crate::skill::{Skill, ValidationStatus};

/// Maximum length for skill name (per agentskills.io spec)
const MAX_NAME_LENGTH: usize = 64;

/// Maximum length for skill description (per agentskills.io spec)
const MAX_DESCRIPTION_LENGTH: usize = 1024;

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

        // Check content presence
        if self.require_content && skill.content.trim().is_empty() {
            errors.push("skill must have content".to_string());
        }

        // Update skill status
        if errors.is_empty() {
            skill.validation_status = ValidationStatus::Valid;
            skill.validation_errors.clear();
            Ok(())
        } else {
            skill.validation_status = ValidationStatus::Invalid;
            skill.validation_errors = errors.clone();
            Err(Error::ValidationFailed {
                name: skill.meta.name.clone(),
                message: errors.join("; "),
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

    fn create_test_skill(name: &str, description: &str, content: &str) -> Skill {
        Skill {
            meta: SkillMeta {
                name: name.to_string(),
                description: description.to_string(),
                tags: Vec::new(),
                version: None,
                author: None,
            },
            content: content.to_string(),
            path: PathBuf::from("/test"),
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
}
