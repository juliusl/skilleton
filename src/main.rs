use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "skilleton", about = "Build and validate agent skills")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new skill directory layout
    Init {
        /// Path to create the skill directory
        path: PathBuf,
    },
    /// Validate a skill's structure and references
    Check {
        /// Path to the skill directory
        path: PathBuf,
    },
    /// Build a skill into Markdown output
    Build {
        /// Path to the skill directory
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { path } => {
            eprintln!("init: {:?} (stub)", path);
        }
        Commands::Check { path } => {
            eprintln!("check: {:?} (stub)", path);
        }
        Commands::Build { path } => {
            eprintln!("build: {:?} (stub)", path);
        }
    }
}
