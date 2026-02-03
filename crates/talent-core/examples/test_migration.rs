//! Test the migration from ~/.agentloom to ~/.agents

use agentloom_core::{has_legacy_skills, migrate_if_needed};

fn main() {
    println!("=== Migration Test ===\n");

    println!("Checking for legacy skills...");
    let has_legacy = has_legacy_skills();
    println!("Has legacy skills in ~/.agentloom: {}\n", has_legacy);

    if !has_legacy {
        println!("No legacy skills found. Nothing to migrate.");
        return;
    }

    println!("Running migration...");
    match migrate_if_needed() {
        Ok(result) => {
            println!("\nMigration result:");
            println!("  Migrated: {}", result.migrated);
            println!("  Skills count: {}", result.skills_count);

            if result.migrated {
                println!("\n  Migrated skills:");
                for name in &result.skill_names {
                    println!("    - {}", name);
                }
            }

            if let Some(from) = &result.from_path {
                println!("\n  From: {}", from.display());
            }
            if let Some(to) = &result.to_path {
                println!("  To: {}", to.display());
            }

            if !result.errors.is_empty() {
                println!("\n  Errors:");
                for err in &result.errors {
                    println!("    - {}", err);
                }
            }

            if result.migrated {
                println!("\n✓ Migration completed successfully!");
                if result.errors.is_empty() {
                    println!("The old ~/.agentloom directory has been deleted.");
                }
            } else {
                println!("\n• No migration needed (destination already has skills)");
            }
        }
        Err(e) => {
            eprintln!("\n✗ Migration failed: {}", e);
            std::process::exit(1);
        }
    }
}
