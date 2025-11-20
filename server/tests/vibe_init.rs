use std::fs;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use agent_hub_server::global_registry::{load_or_init_registry, GLOBAL_HOME_OVERRIDE_ENV};
use agent_hub_server::vibe_project::{
    init_vibe_project, load_project_config, InitStatus, VibeProjectConfig, VIBE_SCHEMA_VERSION,
};
use tempfile::tempdir;

static INIT_ENV_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn init_creates_layout_and_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    with_temp_global_home(|| -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;

        let status = init_vibe_project(dir.path())?;
        assert_eq!(status, InitStatus::Created);

        let config = load_project_config(dir.path())?;
        assert_eq!(config.schema_version, VIBE_SCHEMA_VERSION);

        let gitkeep = dir.path().join(".vibe/runtime/queue/.gitkeep");
        assert!(gitkeep.exists(), "expected {}", gitkeep.display());

        let rerun = init_vibe_project(dir.path())?;
        assert_eq!(rerun, InitStatus::AlreadyInitializedUpToDate);

        Ok(())
    })
}

#[test]
fn init_detects_schema_mismatches() -> Result<(), Box<dyn std::error::Error>> {
    with_temp_global_home(|| -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        init_vibe_project(dir.path())?;

        let config_path = dir.path().join(".vibe/config/project.json");
        let mut config: VibeProjectConfig =
            serde_json::from_str(&fs::read_to_string(&config_path)?)?;

        config.schema_version = VIBE_SCHEMA_VERSION.saturating_sub(1);
        write_config(&config_path, &config)?;
        let older = init_vibe_project(dir.path())?;
        assert!(
            matches!(older, InitStatus::AlreadyInitializedOlderSchema { existing } if existing == config.schema_version)
        );

        config.schema_version = VIBE_SCHEMA_VERSION + 1;
        write_config(&config_path, &config)?;
        let newer = init_vibe_project(dir.path())?;
        assert!(
            matches!(newer, InitStatus::AlreadyInitializedNewerSchema { existing } if existing == config.schema_version)
        );

        Ok(())
    })
}

#[test]
fn init_updates_global_registry() -> Result<(), Box<dyn std::error::Error>> {
    with_temp_global_home(|| -> Result<(), Box<dyn std::error::Error>> {
        let project_dir = tempdir()?;
        let first_status = init_vibe_project(project_dir.path())?;
        assert_eq!(first_status, InitStatus::Created);

        let config = load_project_config(project_dir.path())?;
        let registry = load_or_init_registry()?;
        assert_eq!(registry.projects.len(), 1);
        let entry = &registry.projects[0];
        assert_eq!(entry.project_root, config.project_root);
        assert_eq!(entry.project_name, config.project_name);
        let first_seen = chrono::DateTime::parse_from_rfc3339(&entry.last_seen)?;

        thread::sleep(Duration::from_secs(1));

        let rerun_status = init_vibe_project(project_dir.path())?;
        assert_eq!(rerun_status, InitStatus::AlreadyInitializedUpToDate);

        let registry = load_or_init_registry()?;
        assert_eq!(registry.projects.len(), 1);
        let updated = &registry.projects[0];
        let updated_seen = chrono::DateTime::parse_from_rfc3339(&updated.last_seen)?;
        assert!(
            updated_seen > first_seen,
            "expected newer timestamp, got {} <= {}",
            updated_seen,
            first_seen
        );

        Ok(())
    })
}

fn write_config(
    path: &std::path::Path,
    config: &VibeProjectConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut json = serde_json::to_string_pretty(&config)?;
    json.push('\n');
    fs::write(path, json)?;
    Ok(())
}

fn with_temp_global_home<F, R>(test: F) -> R
where
    F: FnOnce() -> R,
{
    let guard = INIT_ENV_GUARD
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap();
    let temp_dir = tempdir().expect("temp dir should be created");
    let home_path = temp_dir.path().join("global");
    std::env::set_var(GLOBAL_HOME_OVERRIDE_ENV, &home_path);
    let result = test();
    std::env::remove_var(GLOBAL_HOME_OVERRIDE_ENV);
    drop(temp_dir);
    drop(guard);
    result
}
