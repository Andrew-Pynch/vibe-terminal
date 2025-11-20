use std::{
    env, fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use dirs::home_dir;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const GLOBAL_REGISTRY_VERSION: u32 = 1;
pub const GLOBAL_HOME_OVERRIDE_ENV: &str = "VIBE_GLOBAL_HOME";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSummary {
    pub project_root: String,
    pub project_name: String,
    pub last_seen: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalProjectRegistry {
    pub version: u32,
    pub projects: Vec<ProjectSummary>,
}

impl GlobalProjectRegistry {
    pub fn empty() -> Self {
        Self {
            version: GLOBAL_REGISTRY_VERSION,
            projects: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
}

pub fn global_home_dir() -> io::Result<PathBuf> {
    if let Some(override_dir) = env::var_os(GLOBAL_HOME_OVERRIDE_ENV) {
        let override_path = PathBuf::from(override_dir);
        fs::create_dir_all(&override_path)?;
        return Ok(override_path);
    }

    let home = home_dir().ok_or_else(|| {
        io::Error::new(
            ErrorKind::NotFound,
            "failed to determine the current user's home directory",
        )
    })?;

    let global_home = home.join(".vibe").join("global");
    fs::create_dir_all(&global_home)?;
    Ok(global_home)
}

pub fn global_registry_path() -> io::Result<PathBuf> {
    Ok(global_home_dir()?.join("projects.json"))
}

pub fn load_or_init_registry() -> Result<GlobalProjectRegistry, RegistryError> {
    let path = global_registry_path()?;
    if path.exists() {
        let raw = fs::read_to_string(&path)?;
        let registry = serde_json::from_str(&raw)?;
        Ok(registry)
    } else {
        let registry = GlobalProjectRegistry::empty();
        write_registry(&path, &registry)?;
        Ok(registry)
    }
}

#[allow(dead_code)]
pub fn save_registry(registry: &GlobalProjectRegistry) -> Result<(), RegistryError> {
    let path = global_registry_path()?;
    write_registry(&path, registry)?;
    Ok(())
}

#[allow(dead_code)]
pub fn upsert_project(registry: &mut GlobalProjectRegistry, summary: ProjectSummary) {
    let project_root = summary.project_root.clone();
    if let Some(existing) = registry
        .projects
        .iter_mut()
        .find(|project| project.project_root == project_root)
    {
        existing.project_name = summary.project_name;
        existing.last_seen = summary.last_seen;
    } else {
        registry.projects.push(summary);
    }
}

fn write_registry(path: &Path, registry: &GlobalProjectRegistry) -> io::Result<()> {
    let serialized = serde_json::to_vec_pretty(registry)
        .map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;
    fs::write(path, serialized)?;
    Ok(())
}
