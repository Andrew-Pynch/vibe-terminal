use std::env;

use agent_hub_server::vibe_project::{
    init_vibe_project, load_project_config, InitStatus, VIBE_SCHEMA_VERSION,
};
use anyhow::Context;
use clap::Command;

fn main() -> anyhow::Result<()> {
    let matches = Command::new("vibe")
        .about("Vibe orchestrator CLI")
        .subcommand(Command::new("init").about("Initialize .vibe/ in the current project"))
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => handle_init(),
        _ => {
            eprintln!("Use `vibe init` to initialize a project.");
            Ok(())
        }
    }
}

fn handle_init() -> anyhow::Result<()> {
    let cwd = env::current_dir().context("failed to determine current working directory")?;

    match init_vibe_project(&cwd)? {
        InitStatus::Created => {
            let config = load_project_config(&cwd)?;
            println!(
                "Initialized .vibe/ for project \"{}\" (schema_version={}).",
                config.project_name, config.schema_version
            );
        }
        InitStatus::AlreadyInitializedUpToDate => {
            let config = load_project_config(&cwd)?;
            println!(
                ".vibe/ already initialized (schema_version={}). Nothing to do.",
                config.schema_version
            );
        }
        InitStatus::AlreadyInitializedOlderSchema { existing } => {
            eprintln!(
        "Found .vibe/ with schema_version={} (older than this CLI: {}). No changes were made.",
        existing, VIBE_SCHEMA_VERSION
      );
        }
        InitStatus::AlreadyInitializedNewerSchema { existing } => {
            eprintln!(
        "Found .vibe/ with newer schema_version={} than this CLI supports ({}). Please upgrade the vibe CLI.",
        existing, VIBE_SCHEMA_VERSION
      );
        }
    }

    Ok(())
}
