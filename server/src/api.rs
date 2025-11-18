use axum::{
  body::Body,
  extract::{Path, State},
  http::{Request, StatusCode},
  middleware::{from_fn_with_state, Next},
  response::Response,
  routing::{delete, get, post},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  llm::ProviderKind,
  profiles::ProfileSummary,
  sessions::{SessionCreateParams, SessionDetail, SessionSummary},
  state::AppState,
};
use std::str::FromStr;

pub fn router(state: AppState) -> Router {
  Router::new()
    .route("/health", get(health))
    .route("/sessions", get(list_sessions).post(create_session))
    .route(
      "/sessions/:id",
      get(get_session).delete(delete_session),
    )
    .route("/profiles", get(list_profiles))
    .with_state(state.clone())
    .layer(from_fn_with_state(state, guard_shared_secret))
}

async fn health() -> &'static str {
  "ok"
}

async fn list_sessions(
  State(state): State<AppState>,
) -> Json<SessionListResponse> {
  let sessions = state.sessions.list().await;
  Json(SessionListResponse { sessions })
}

async fn get_session(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> Result<Json<SessionDetailResponse>, StatusCode> {
  let session = state.sessions.detail(&id).await.ok_or(StatusCode::NOT_FOUND)?;
  Ok(Json(SessionDetailResponse { session }))
}

async fn delete_session(
  State(state): State<AppState>,
  Path(id): Path<String>,
) -> StatusCode {
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

async fn list_profiles(
  State(state): State<AppState>,
) -> Json<ProfileListResponse> {
  Json(ProfileListResponse {
    profiles: state.profiles.summaries(),
  })
}

async fn guard_shared_secret(
  State(state): State<AppState>,
  mut request: Request<Body>,
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
