use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use thiserror::Error;

use crate::global_registry::{
    load_or_init_registry, save_registry, upsert_project, ProjectSummary, RegistryError,
};

pub const VIBE_SCHEMA_VERSION: u32 = 1;
const RULES_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct VibeProjectConfig {
    pub schema_version: u32,
    pub project_root: String,
    pub project_name: String,
    pub created_at: String,
    #[serde(default)]
    pub preferred_planner_model: Option<String>,
    #[serde(default)]
    pub preferred_worker_model: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VibeRules {
    pub version: u32,
    #[serde(default = "empty_object")]
    pub file_type_overrides: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitStatus {
    Created,
    AlreadyInitializedUpToDate,
    AlreadyInitializedOlderSchema { existing: u32 },
    AlreadyInitializedNewerSchema { existing: u32 },
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unable to determine project root name at {0}")]
    InvalidProjectName(String),
}

pub fn init_vibe_project(root: &Path) -> Result<InitStatus, InitError> {
    ensure_base_layout(root)?;

    let config_path = project_config_path(root);
    let rules_path = rules_config_path(root);

    let mut created = false;
    let mut freshly_written_config = None;

    if !config_path.exists() {
        let config = default_project_config(root)?;
        write_json_pretty(&config_path, &config)?;
        freshly_written_config = Some(config);
        created = true;
    }

    if !rules_path.exists() {
        let rules = default_rules();
        write_json_pretty(&rules_path, &rules)?;
    }

    let config = match freshly_written_config {
        Some(config) => config,
        None => load_project_config(root)?,
    };

    if let Err(err) = register_project_in_global_registry(&config) {
        eprintln!("Warning: failed to update global project registry: {err}");
    }

    if created {
        return Ok(InitStatus::Created);
    }

    Ok(match config.schema_version.cmp(&VIBE_SCHEMA_VERSION) {
        Ordering::Equal => InitStatus::AlreadyInitializedUpToDate,
        Ordering::Less => InitStatus::AlreadyInitializedOlderSchema {
            existing: config.schema_version,
        },
        Ordering::Greater => InitStatus::AlreadyInitializedNewerSchema {
            existing: config.schema_version,
        },
    })
}

pub fn load_project_config(root: &Path) -> Result<VibeProjectConfig, InitError> {
    let config_path = project_config_path(root);
    let contents = fs::read_to_string(config_path)?;
    let config = serde_json::from_str(&contents)?;
    Ok(config)
}

fn ensure_base_layout(root: &Path) -> Result<(), InitError> {
    let vibe_dir = root.join(".vibe");
    let config_dir = vibe_dir.join("config");
    let modes_dir = vibe_dir.join("MODES");
    let specs_dir = vibe_dir.join("specs");
    let runtime_dir = vibe_dir.join("runtime");

    let directories = [
        vibe_dir.clone(),
        config_dir.clone(),
        modes_dir.clone(),
        specs_dir.clone(),
        specs_dir.join("prd"),
        specs_dir.join("todos"),
        runtime_dir.clone(),
        runtime_dir.join("queue"),
        runtime_dir.join("checkpoints"),
        runtime_dir.join("logs"),
        runtime_dir.join("reminders"),
    ];

    for dir in directories {
        fs::create_dir_all(dir)?;
    }

    write_if_missing(&vibe_dir.join("AGENTS.md"), AGENTS_MD)?;
    write_if_missing(&vibe_dir.join("DOCUMENTATION.md"), DOCUMENTATION_MD)?;

    for (file_name, content) in MODES_FILES {
        write_if_missing(&modes_dir.join(file_name), content)?;
    }

    for gitkeep in [
        specs_dir.join("prd").join(".gitkeep"),
        specs_dir.join("todos").join(".gitkeep"),
        runtime_dir.join("queue").join(".gitkeep"),
        runtime_dir.join("checkpoints").join(".gitkeep"),
        runtime_dir.join("logs").join(".gitkeep"),
        runtime_dir.join("reminders").join(".gitkeep"),
    ] {
        write_if_missing(&gitkeep, "")?;
    }

    Ok(())
}

fn default_project_config(root: &Path) -> Result<VibeProjectConfig, InitError> {
    let canonical = root.canonicalize()?;
    let project_root = canonical.to_string_lossy().to_string();
    let project_name = canonical
        .file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
        .or_else(|| {
            root.file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
        })
        .ok_or_else(|| InitError::InvalidProjectName(project_root.clone()))?;

    Ok(VibeProjectConfig {
        schema_version: VIBE_SCHEMA_VERSION,
        project_root,
        project_name,
        created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        preferred_planner_model: Some(String::from("gpt-5.1")),
        preferred_worker_model: Some(String::from("claude-code")),
        notes: Some(String::from(
            "Initial .vibe scaffold created by 'vibe init'.",
        )),
    })
}

fn default_rules() -> VibeRules {
    VibeRules {
        version: RULES_VERSION,
        file_type_overrides: empty_object(),
    }
}

fn project_config_path(root: &Path) -> PathBuf {
    root.join(".vibe").join("config").join("project.json")
}

fn rules_config_path(root: &Path) -> PathBuf {
    root.join(".vibe").join("config").join("rules.json")
}

fn write_if_missing(path: &Path, contents: &str) -> Result<(), InitError> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, contents)?;
    Ok(())
}

fn write_json_pretty<T: ?Sized + Serialize>(path: &Path, value: &T) -> Result<(), InitError> {
    let mut json = serde_json::to_string_pretty(value)?;
    json.push('\n');
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, json)?;
    Ok(())
}

fn register_project_in_global_registry(config: &VibeProjectConfig) -> Result<(), RegistryError> {
    let mut registry = load_or_init_registry()?;
    let summary = ProjectSummary {
        project_root: config.project_root.clone(),
        project_name: config.project_name.clone(),
        last_seen: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    };
    upsert_project(&mut registry, summary);
    save_registry(&registry)?;
    Ok(())
}

const AGENTS_MD: &str = "# Agents\n\nTODO: Define agents for this project.\n";
const DOCUMENTATION_MD: &str =
    "# Documentation\n\nTODO: Document shared context for this project.\n";

const MODES_FILES: [(&str, &str); 4] = [
    (
        "BOOT.md",
        "# BOOT\n\nTODO: Describe boot behavior for this project.\n",
    ),
    (
        "ORCHESTRATOR.md",
        "# ORCHESTRATOR\n\nTODO: Describe orchestrator behavior for this project.\n",
    ),
    (
        "WORKER.md",
        "# WORKER\n\nTODO: Describe worker behavior for this project.\n",
    ),
    (
        "DOC_SCRIBE.md",
        "# DOC_SCRIBE\n\nTODO: Describe doc scribe behavior for this project.\n",
    ),
];

fn empty_object() -> serde_json::Value {
    json!({})
}
