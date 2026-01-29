use clap::Parser;

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
    List,
    /// Sync skills to all targets
    Sync,
    /// Show diagnostic information
    Doctor,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List) => {
            println!("Listing skills... (not yet implemented)");
        }
        Some(Commands::Sync) => {
            println!("Syncing skills... (not yet implemented)");
        }
        Some(Commands::Doctor) => {
            println!("Running diagnostics... (not yet implemented)");
        }
        None => {
            println!("Talent - Agent Skills Manager");
            println!("Run 'talent --help' for usage");
        }
    }
}
