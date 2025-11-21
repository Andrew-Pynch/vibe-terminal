use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::{from_fn_with_state, Next},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::agents::spawner::AgentSpawner;

use crate::{
    global_registry::GlobalProjectRegistry,
    llm::ProviderKind,
    profiles::ProfileSummary,
    project_sessions::{
        create_or_get_session_for_project, list_sessions as list_project_sessions, ProjectSession,
    },
    sessions::{SessionCreateParams, SessionDetail, SessionSummary},
    state::AppState,
};
use std::{collections::HashMap, env, str::FromStr};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/projects", get(list_projects))
        .route(
            "/project-sessions",
            get(list_project_sessions_http).post(create_project_session),
        )
        .route("/sessions", get(list_sessions).post(create_session))
        .route("/sessions/:id", get(get_session).delete(delete_session))
        .route("/profiles", get(list_profiles))
        .route("/debug/spawn", post(debug_spawn_agent))
        .with_state(state.clone())
        .layer(from_fn_with_state(state, guard_shared_secret))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn debug_spawn_agent(
    State(state): State<AppState>,
    Json(payload): Json<DebugSpawnPayload>,
) -> Result<Json<DebugSpawnResponse>, StatusCode> {
    let base_dir = env::current_dir().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let spawner = AgentSpawner::new(state.agents.clone(), base_dir.clone());
    
    // Default to the dummy agent script if no command is provided
    let command = payload.command.unwrap_or_else(|| "bash".to_string());
    let args = payload.args.unwrap_or_else(|| {
        // Determine absolute path to the dummy script based on where we are running
        // We assume we are in the repo root or server root.
        // Best guess: try to find it relative to current dir.
        let script_path = base_dir.join("server/tests/scripts/dummy_agent.sh");
        let final_path = if script_path.exists() {
            script_path
        } else {
            base_dir.join("tests/scripts/dummy_agent.sh") // If running inside server/
        };
        
        vec![
            "-c".to_string(),
            final_path.to_string_lossy().to_string()
        ]
    });

    // Capture GEMINI_API_KEY from server environment to pass to the agent
    let mut env_vars = HashMap::new();
    if let Ok(key) = env::var("GEMINI_API_KEY") {
        env_vars.insert("GEMINI_API_KEY".to_string(), key);
    }

    let agent_id = spawner
        .spawn_agent(
            payload.session_id, 
            payload.agent_type, 
            payload.instruction,
            command,
            args,
            env_vars
        )
        .map_err(|e| {
            tracing::error!("Failed to spawn agent: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(DebugSpawnResponse { agent_id }))
}

async fn list_projects(State(state): State<AppState>) -> Json<GlobalProjectRegistry> {
    let registry = state.global_registry.read().clone();
    Json(registry)
}

async fn list_sessions(State(state): State<AppState>) -> Json<SessionListResponse> {
    let sessions = state.sessions.list().await;
    Json(SessionListResponse { sessions })
}

async fn list_project_sessions_http(
    State(state): State<AppState>,
) -> Json<ProjectSessionListResponse> {
    let sessions = list_project_sessions(&state);
    Json(ProjectSessionListResponse { sessions })
}

async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SessionDetailResponse>, StatusCode> {
    let session = state
        .sessions
        .detail(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(SessionDetailResponse { session }))
}

async fn delete_session(State(state): State<AppState>, Path(id): Path<String>) -> StatusCode {
    if state.sessions.delete(&id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn create_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateSessionPayload>,
) -> Result<Json<SessionDetailResponse>, StatusCode> {
    if state.profiles.get(&payload.profile).is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut llm_config = state.config.default_llm.clone();
    if let Some(config) = payload.llm_config {
        if let Some(provider) = config.provider {
            if let Ok(parsed) = ProviderKind::from_str(&provider) {
                llm_config.provider = parsed;
            }
        }
        if let Some(model) = config.model {
            llm_config.model = model;
        }
        if let Some(temperature) = config.temperature {
            llm_config.temperature = temperature;
        }
    }

    let session = state
        .sessions
        .create(SessionCreateParams {
            name: payload.name,
            profile: payload.profile,
            llm_config,
            meta: payload.meta,
        })
        .await;
    Ok(Json(SessionDetailResponse { session }))
}

async fn create_project_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectSessionPayload>,
) -> Result<Json<ProjectSessionResponse>, (StatusCode, Json<ProjectSessionErrorResponse>)> {
    let project_summary = {
        let registry = state.global_registry.read();
        registry
            .projects
            .iter()
            .find(|project| project.project_root == payload.project_root)
            .cloned()
    };

    let Some(project) = project_summary else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ProjectSessionErrorResponse {
                error: "PROJECT_NOT_REGISTERED",
            }),
        ));
    };

    let session =
        create_or_get_session_for_project(&state, &project.project_root, &project.project_name);
    Ok(Json(ProjectSessionResponse { session }))
}

async fn list_profiles(State(state): State<AppState>) -> Json<ProfileListResponse> {
    Json(ProfileListResponse {
        profiles: state.profiles.summaries(),
    })
}

async fn guard_shared_secret(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(secret) = &state.config.shared_secret {
        let authorized = request
            .headers()
            .get("x-agent-hub-auth")
            .and_then(|value| value.to_str().ok())
            .map(|value| value == secret)
            .unwrap_or(false);
        if !authorized {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    Ok(next.run(request).await)
}

#[derive(Serialize)]
struct SessionListResponse {
    sessions: Vec<SessionSummary>,
}

#[derive(Serialize)]
struct SessionDetailResponse {
    session: SessionDetail,
}

#[derive(Serialize)]
struct ProfileListResponse {
    profiles: Vec<ProfileSummary>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct ProjectSessionListResponse {
    sessions: Vec<ProjectSession>,
}

#[derive(Serialize)]
struct ProjectSessionResponse {
    session: ProjectSession,
}

#[derive(Serialize)]
struct ProjectSessionErrorResponse {
    error: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSessionPayload {
    name: String,
    profile: String,
    #[serde(default)]
    llm_config: Option<LlmConfigPayload>,
    #[serde(default)]
    meta: Option<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LlmConfigPayload {
    provider: Option<String>,
    model: Option<String>,
    temperature: Option<f32>,
}

#[derive(Deserialize)]
struct CreateProjectSessionPayload {
    project_root: String,
}

#[derive(Deserialize)]
struct DebugSpawnPayload {
    session_id: String,
    agent_type: String,
    instruction: String,
    command: Option<String>,
    args: Option<Vec<String>>,
}

#[derive(Serialize)]
struct DebugSpawnResponse {
    agent_id: String,
}
