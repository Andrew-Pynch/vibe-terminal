use std::{
    fs,
    path::Path,
    sync::{Mutex, OnceLock},
};

use agent_hub_server::global_registry::{
    global_registry_path, load_or_init_registry, save_registry, upsert_project,
    GlobalProjectRegistry, ProjectSummary, GLOBAL_HOME_OVERRIDE_ENV, GLOBAL_REGISTRY_VERSION,
};
use tempfile::TempDir;

static ENV_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn load_or_init_creates_default_registry_when_missing() {
    with_temp_global_home(|_| {
        let registry = load_or_init_registry().expect("registry should load");
        assert_eq!(registry.version, GLOBAL_REGISTRY_VERSION);
        assert!(registry.projects.is_empty());

        let registry_path = global_registry_path().expect("registry path should resolve");
        assert!(
            registry_path.exists(),
            "registry file should be created automatically"
        );

        let raw = fs::read_to_string(registry_path).expect("registry file should be readable");
        let parsed: GlobalProjectRegistry =
            serde_json::from_str(&raw).expect("registry should parse as json");
        assert_eq!(parsed.version, GLOBAL_REGISTRY_VERSION);
        assert!(parsed.projects.is_empty());
    });
}

#[test]
fn load_or_init_reads_existing_registry() {
    with_temp_global_home(|home| {
        fs::create_dir_all(home).expect("global home should be creatable");
        let registry_path = home.join("projects.json");
        let existing = GlobalProjectRegistry {
            version: GLOBAL_REGISTRY_VERSION,
            projects: vec![ProjectSummary {
                project_root: "/tmp/some-project".into(),
                project_name: "some-project".into(),
                last_seen: "2025-11-18T16:00:00Z".into(),
            }],
        };
        let serialized = serde_json::to_string_pretty(&existing).expect("serialize registry");
        fs::write(&registry_path, serialized).expect("write registry file");

        let registry = load_or_init_registry().expect("registry should load");
        assert_eq!(registry.version, GLOBAL_REGISTRY_VERSION);
        assert_eq!(registry.projects.len(), 1);
        assert_eq!(registry.projects[0].project_name, "some-project");
    });
}

#[test]
fn upsert_project_updates_entries() {
    with_temp_global_home(|_| {
        let mut registry = load_or_init_registry().expect("registry should load");
        let first_seen = "2025-11-18T16:00:00Z".to_string();
        let summary = ProjectSummary {
            project_root: "/tmp/some-project".into(),
            project_name: "some-project".into(),
            last_seen: first_seen.clone(),
        };
        upsert_project(&mut registry, summary);
        assert_eq!(registry.projects.len(), 1);
        assert_eq!(registry.projects[0].last_seen, first_seen);
        save_registry(&registry).expect("registry should persist");

        let mut registry = load_or_init_registry().expect("registry should reload");
        assert_eq!(registry.projects.len(), 1);

        let updated_summary = ProjectSummary {
            project_root: "/tmp/some-project".into(),
            project_name: "renamed-project".into(),
            last_seen: "2025-11-19T12:00:00Z".into(),
        };
        upsert_project(&mut registry, updated_summary);
        assert_eq!(registry.projects.len(), 1);
        assert_eq!(registry.projects[0].project_name, "renamed-project");
        assert_eq!(registry.projects[0].last_seen, "2025-11-19T12:00:00Z");

        let second_project = ProjectSummary {
            project_root: "/tmp/another".into(),
            project_name: "another".into(),
            last_seen: "2025-11-19T12:00:01Z".into(),
        };
        upsert_project(&mut registry, second_project);
        assert_eq!(registry.projects.len(), 2);
    });
}

fn with_temp_global_home<F: FnOnce(&Path)>(test: F) {
    let guard = ENV_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let home_path = temp_dir.path().join("global");
    std::env::set_var(GLOBAL_HOME_OVERRIDE_ENV, &home_path);
    test(&home_path);
    std::env::remove_var(GLOBAL_HOME_OVERRIDE_ENV);
    drop(guard);
}
