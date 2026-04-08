use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

use skilleton::conflict::detect_policy_overlaps;
use skilleton::render::render_skill;
use skilleton::storage::{SkillLoader, SkillWriter};
use skilleton::types::{ItemId, ItemMeta, Skill, SkillMeta};
use skilleton::validate::{
    validate_criterion_references, validate_invocation_references, validate_type_prefixes,
};

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
    let exit_code = match cli.command {
        Commands::Init { path } => cmd_init(&path),
        Commands::Check { path } => cmd_check(&path),
        Commands::Build { path } => cmd_build(&path),
    };
    process::exit(exit_code);
}

fn cmd_init(path: &PathBuf) -> i32 {
    if path.join("skill.toml").exists() {
        eprintln!("error: skill already exists at {}", path.display());
        return 1;
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "my-skill".to_string());

    let skill_id = match ItemId::parse(&format!("skill:{name}")) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("error: invalid skill name '{}': {}", name, e);
            return 1;
        }
    };

    let skill = Skill {
        meta: ItemMeta {
            id: skill_id,
            conditions: vec![],
        },
        metadata: SkillMeta {
            name: name.clone(),
            description: String::new(),
        },
        procedures: vec![],
        policies: vec![],
        criteria: vec![],
    };

    // SkillWriter::write(root, skill) creates <root>/<slug>/...
    // We want files at <path>/, so root = path.parent()
    let root = path.parent().unwrap_or(path);
    if let Err(e) = SkillWriter::write(root, &skill) {
        eprintln!("error: failed to initialize skill: {}", e);
        return 1;
    }

    eprintln!("initialized skill '{}' at {}", name, path.display());
    0
}

fn cmd_check(path: &PathBuf) -> i32 {
    let skill = match SkillLoader::load(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to load skill: {}", e);
            return 1;
        }
    };

    let mut errors = 0;

    if let Err(ref_errors) = validate_invocation_references(&skill) {
        for e in &ref_errors {
            eprintln!("error: {}", e);
            errors += ref_errors.len();
        }
    }

    if let Err(crit_errors) = validate_criterion_references(&skill) {
        for e in &crit_errors {
            eprintln!("error: {}", e);
            errors += crit_errors.len();
        }
    }

    if let Err(prefix_errors) = validate_type_prefixes(&skill) {
        for e in &prefix_errors {
            eprintln!("error: {}", e);
            errors += prefix_errors.len();
        }
    }

    let overlaps = detect_policy_overlaps(&skill);
    if !overlaps.is_empty() {
        for overlap in &overlaps {
            eprintln!("warning: policy overlap at {}", overlap.target_scope.as_str());
        }
    }

    if errors > 0 {
        eprintln!("{} error(s) found", errors);
        1
    } else {
        eprintln!("check passed");
        0
    }
}

fn cmd_build(path: &PathBuf) -> i32 {
    let skill = match SkillLoader::load(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to load skill: {}", e);
            return 1;
        }
    };

    // Validate before rendering
    let mut errors = 0;

    if let Err(ref_errors) = validate_invocation_references(&skill) {
        for e in &ref_errors {
            eprintln!("error: {}", e);
        }
        errors += ref_errors.len();
    }

    if let Err(crit_errors) = validate_criterion_references(&skill) {
        for e in &crit_errors {
            eprintln!("error: {}", e);
        }
        errors += crit_errors.len();
    }

    if let Err(prefix_errors) = validate_type_prefixes(&skill) {
        for e in &prefix_errors {
            eprintln!("error: {}", e);
        }
        errors += prefix_errors.len();
    }

    if errors > 0 {
        eprintln!("{} error(s) found — build aborted", errors);
        return 1;
    }

    let md = render_skill(&skill);
    print!("{}", md);
    0
}
