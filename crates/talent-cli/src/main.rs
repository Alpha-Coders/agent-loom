//! Talent CLI - Agent Skills Manager
//!
//! Command-line interface for managing skills across AI CLI tools.

use clap::Parser;
use talent_core::{Config, SkillManager, SyncResult, ValidationStatus};

#[derive(Parser)]
#[command(name = "talent")]
#[command(about = "Agent Skills Manager - Sync skills across AI CLI tools")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// List all skills
    List {
        /// Show only valid skills
        #[arg(long)]
        valid: bool,

        /// Show only invalid skills
        #[arg(long)]
        invalid: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Sync skills to all targets
    Sync {
        /// Target specific CLI (e.g., claude-code, codex)
        #[arg(long)]
        target: Option<String>,

        /// Dry run - show what would be synced without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Show diagnostic information
    Doctor,

    /// List detected targets
    Targets {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Create a new skill
    Create {
        /// Name of the skill (kebab-case)
        name: String,

        /// Description of the skill
        #[arg(short, long, default_value = "A new skill")]
        description: String,
    },

    /// Validate skills
    Validate {
        /// Specific skill to validate (validates all if not specified)
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::List { valid, invalid, json }) => cmd_list(valid, invalid, json),
        Some(Commands::Sync { target, dry_run }) => cmd_sync(target, dry_run),
        Some(Commands::Doctor) => cmd_doctor(),
        Some(Commands::Targets { json }) => cmd_targets(json),
        Some(Commands::Create { name, description }) => cmd_create(&name, &description),
        Some(Commands::Validate { name }) => cmd_validate(name),
        None => {
            println!("Talent - Agent Skills Manager");
            println!("Run 'talent --help' for usage");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// List all skills
fn cmd_list(valid_only: bool, invalid_only: bool, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = SkillManager::new()?;

    // Validate all skills first to get their status
    manager.validate_all();

    let skills: Vec<_> = manager
        .skills()
        .iter()
        .filter(|s| {
            if valid_only {
                s.validation_status == ValidationStatus::Valid
            } else if invalid_only {
                s.validation_status == ValidationStatus::Invalid
            } else {
                true
            }
        })
        .collect();

    if json {
        let output: Vec<_> = skills
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name(),
                    "description": &s.meta.description,
                    "path": s.path,
                    "status": format!("{:?}", s.validation_status),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if skills.is_empty() {
            println!("No skills found in {}", manager.config().skills_dir.display());
            println!("\nCreate a skill with: talent create <name>");
            return Ok(());
        }

        println!("Skills ({}):", skills.len());
        println!();

        for skill in skills {
            let status_icon = match skill.validation_status {
                ValidationStatus::Valid => "✓",
                ValidationStatus::Invalid => "✗",
                ValidationStatus::Unknown => "?",
            };

            let desc = if skill.meta.description.is_empty() {
                "No description"
            } else {
                &skill.meta.description
            };

            println!("  {} {} - {}", status_icon, skill.name(), desc);
        }
    }

    Ok(())
}

/// Sync skills to targets
fn cmd_sync(target: Option<String>, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = SkillManager::new()?;

    // Validate all skills first
    manager.validate_all();

    let stats = manager.stats();
    if stats.total_skills == 0 {
        println!("No skills to sync.");
        return Ok(());
    }

    if dry_run {
        println!("Dry run - no changes will be made\n");
    }

    let results: Vec<SyncResult> = if let Some(target_id) = target {
        match manager.sync_target(&target_id) {
            Some(result) => vec![result],
            None => {
                eprintln!("Target '{}' not found. Run 'talent targets' to see available targets.", target_id);
                return Ok(());
            }
        }
    } else {
        manager.sync_all()
    };

    // Display results
    for result in &results {
        println!("Target: {} ({})", result.target_name, result.target_id);

        if !result.errors.is_empty() {
            for error in &result.errors {
                let skill_info = error.skill.as_deref().unwrap_or("general");
                println!("  ✗ Error: {} - {}", skill_info, error.message);
            }
        }

        if result.created.is_empty() && result.removed.is_empty() && result.unchanged.is_empty() {
            println!("  No skills to sync");
        } else {
            if !result.created.is_empty() {
                println!("  + Created: {}", result.created.join(", "));
            }
            if !result.removed.is_empty() {
                println!("  - Removed: {}", result.removed.join(", "));
            }
            if !result.unchanged.is_empty() && results.len() == 1 {
                // Only show unchanged for single-target sync (less noisy)
                println!("  = Unchanged: {}", result.unchanged.join(", "));
            }
        }
        println!();
    }

    // Summary
    let total_created: usize = results.iter().map(|r| r.created.len()).sum();
    let total_removed: usize = results.iter().map(|r| r.removed.len()).sum();
    let total_errors: usize = results.iter().map(|r| r.errors.len()).sum();

    println!("Summary: {} created, {} removed, {} errors", total_created, total_removed, total_errors);

    Ok(())
}

/// Show diagnostic information
fn cmd_doctor() -> Result<(), Box<dyn std::error::Error>> {
    println!("Talent Doctor - Diagnostic Information\n");

    // Try to create manager
    let manager_result = SkillManager::new();

    // Config info
    println!("Configuration:");
    match &manager_result {
        Ok(manager) => {
            let config = manager.config();
            let config_path = Config::default_config_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            println!("  Skills directory: {}", config.skills_dir.display());
            println!("  Config file: {}", config_path);
            println!(
                "  Auto-sync: {}",
                if config.preferences.auto_sync { "enabled" } else { "disabled" }
            );
            println!(
                "  Validate on sync: {}",
                if config.preferences.validate_on_sync { "enabled" } else { "disabled" }
            );
        }
        Err(e) => {
            println!("  Error loading config: {e}");
        }
    }
    println!();

    // Skills info
    println!("Skills:");
    match &manager_result {
        Ok(_) => {
            let mut manager = SkillManager::new()?;
            manager.validate_all();

            let stats = manager.stats();
            println!("  Total: {}", stats.total_skills);
            println!("  Valid: {}", stats.valid_skills);
            println!("  Invalid: {}", stats.invalid_skills);

            // List invalid skills if any
            if stats.invalid_skills > 0 {
                println!("\n  Invalid skills:");
                for skill in manager.invalid_skills() {
                    println!("    - {}: {:?}", skill.name(), skill.validation_errors);
                }
            }
        }
        Err(_) => {
            println!("  Could not load skills");
        }
    }
    println!();

    // Targets info
    println!("Targets:");
    match &manager_result {
        Ok(manager) => {
            let targets = manager.targets();
            if targets.is_empty() {
                println!("  No targets detected");
            } else {
                for target in targets {
                    let status = if target.enabled {
                        if target.skills_path.exists() {
                            "✓ ready"
                        } else {
                            "! skills dir missing"
                        }
                    } else {
                        "○ disabled"
                    };
                    println!("  {} {} ({})", status, target.name(), target.id());
                }
            }
        }
        Err(_) => {
            println!("  Could not detect targets");
        }
    }
    println!();

    // File watcher status
    println!("File Watcher:");
    match &manager_result {
        Ok(manager) => {
            if manager.is_watching() {
                println!("  Status: running");
            } else {
                println!("  Status: not running (auto-sync disabled)");
            }
        }
        Err(_) => {
            println!("  Status: unknown");
        }
    }

    Ok(())
}

/// List detected targets
fn cmd_targets(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let manager = SkillManager::new()?;
    let targets = manager.targets();

    if json {
        let output: Vec<_> = targets
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id(),
                    "name": t.name(),
                    "kind": format!("{:?}", t.kind),
                    "enabled": t.enabled,
                    "skills_path": t.skills_path,
                    "exists": t.skills_path.exists(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if targets.is_empty() {
            println!("No AI CLI tools detected.");
            println!("\nSupported tools: Claude Code, Codex, Gemini, Cursor, Amp, Goose");
            return Ok(());
        }

        println!("Detected Targets ({}):", targets.len());
        println!();

        for target in targets {
            let status_icon = if target.enabled {
                if target.skills_path.exists() {
                    "✓"
                } else {
                    "!"
                }
            } else {
                "○"
            };

            println!("  {} {}", status_icon, target.name());
            println!("    ID: {}", target.id());
            println!("    Skills: {}", target.skills_path.display());
            println!(
                "    Status: {}",
                if target.enabled { "enabled" } else { "disabled" }
            );
            println!();
        }
    }

    Ok(())
}

/// Create a new skill
fn cmd_create(name: &str, description: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = SkillManager::new()?;

    // Check if skill already exists
    if manager.get_skill(name).is_some() {
        eprintln!("Skill '{}' already exists.", name);
        return Ok(());
    }

    let skill = manager.create_skill(name, description)?;
    println!("Created skill: {}", skill.name());
    println!("  Path: {}", skill.path.display());
    println!("\nEdit the SKILL.md file to add content, then run 'talent sync' to deploy.");

    Ok(())
}

/// Validate skills
fn cmd_validate(name: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = SkillManager::new()?;

    if let Some(skill_name) = name {
        // Validate specific skill
        match manager.validate_skill(&skill_name) {
            Ok(()) => {
                println!("✓ Skill '{}' is valid", skill_name);
            }
            Err(e) => {
                println!("✗ Skill '{}' is invalid: {}", skill_name, e);
            }
        }
    } else {
        // Validate all skills
        let results = manager.validate_all();

        let valid_count = results.iter().filter(|r| r.is_ok()).count();
        let invalid_count = results.len() - valid_count;

        println!("Validated {} skills: {} valid, {} invalid\n", results.len(), valid_count, invalid_count);

        for skill in manager.skills() {
            let icon = match skill.validation_status {
                ValidationStatus::Valid => "✓",
                ValidationStatus::Invalid => "✗",
                ValidationStatus::Unknown => "?",
            };
            print!("  {} {}", icon, skill.name());

            if !skill.validation_errors.is_empty() {
                println!(" - {}", skill.validation_errors.join(", "));
            } else {
                println!();
            }
        }
    }

    Ok(())
}
