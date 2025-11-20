use std::{collections::HashMap, net::IpAddr, str::FromStr, sync::Arc};

use agent_hub_server::{
    config::ServerConfig,
    global_registry::GlobalProjectRegistry,
    llm::{LlmConfig, LlmRegistry, ProviderKind},
    profiles::ProfileCatalog,
    project_sessions::{create_or_get_session_for_project, list_sessions, ProjectSessionStatus},
    sessions::SessionStore,
    state::AppState,
    ws,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use parking_lot::RwLock;
use tempfile::tempdir;
use tower::ServiceExt;

#[test]
fn create_session_for_project_creates_entry() {
    let state = test_state();
    let project_root = "/tmp/vibe-project";
    let session = create_or_get_session_for_project(&state, project_root, "Vibe Project");
    assert_eq!(session.project_root, project_root);
    assert_eq!(session.status, ProjectSessionStatus::Active);

    let sessions = list_sessions(&state);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, session.session_id);
}

#[test]
fn create_or_get_reuses_active_session() {
    let state = test_state();
    let project_root = "/tmp/vibe-project";

    let first = create_or_get_session_for_project(&state, project_root, "Vibe Project");
    let second = create_or_get_session_for_project(&state, project_root, "Vibe Project");
    assert_eq!(
        first.session_id, second.session_id,
        "should reuse active session"
    );

    let other = create_or_get_session_for_project(&state, "/tmp/another", "Another");
    assert_ne!(
        first.session_id, other.session_id,
        "different project roots should create new sessions"
    );
}

#[tokio::test]
async fn project_session_ws_route_is_mounted() {
    let state = test_state();
    let app = ws::router(state);
    let request = Request::builder()
        .method("GET")
        .uri("/ws/project-session/test-session")
        .header("host", "localhost")
        .header("upgrade", "websocket")
        .header("connection", "Upgrade")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .header("sec-websocket-version", "13")
        .body(Body::empty())
        .expect("failed to build request");
    let response = app
        .oneshot(request)
        .await
        .expect("router should handle request without error");
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "expected WS router to serve /ws/project-session/:session_id"
    );
}

fn test_state() -> AppState {
    let prompt_temp = tempdir().expect("temp dir");
    let prompt_dir = prompt_temp.path().to_path_buf();
    let profiles =
        Arc::new(ProfileCatalog::load(&prompt_dir).expect("empty profile directory should load"));
    AppState {
        config: ServerConfig {
            host: IpAddr::from_str("127.0.0.1").unwrap(),
            http_port: 4110,
            ws_port: 4111,
            shared_secret: None,
            prompt_profile_dir: prompt_dir,
            default_llm: LlmConfig {
                provider: ProviderKind::Dummy,
                model: "dummy".into(),
                temperature: 0.2,
            },
        },
        sessions: Arc::new(SessionStore::new()),
        profiles,
        llms: Arc::new(LlmRegistry::new()),
        global_registry: Arc::new(RwLock::new(GlobalProjectRegistry::empty())),
        project_sessions: Arc::new(RwLock::new(HashMap::new())),
    }
}
