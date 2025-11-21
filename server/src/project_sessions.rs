use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap; // Added this

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSession {
    pub session_id: String,
    pub project_root: String,
    pub project_name: String,
    pub created_at: String,
    pub last_active_at: String,
    pub status: ProjectSessionStatus,
    pub latest_result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectSessionStatus {
    Active,
    Closed,
}

pub fn list_sessions(state: &AppState) -> Vec<ProjectSession> {
    let sessions = state.project_sessions.read();
    sessions.values().cloned().collect()
}

pub fn create_or_get_session_for_project(
    state: &AppState,
    project_root: &str,
    project_name: &str,
) -> ProjectSession {
    if let Some(existing) = find_active_session(state, project_root) {
        return update_last_active(state, &existing.session_id).unwrap_or(existing);
    }

    let mut sessions = state.project_sessions.write();
    if let Some(existing) = sessions
        .values()
        .find(|session| {
            session.status == ProjectSessionStatus::Active && session.project_root == project_root
        })
        .cloned()
    {
        drop(sessions);
        return update_last_active(state, &existing.session_id).unwrap_or(existing);
    }

    let now = Utc::now().to_rfc3339();
    let session_id = Uuid::new_v4().to_string();
    let session = ProjectSession {
        session_id: session_id.clone(),
        project_root: project_root.to_string(),
        project_name: project_name.to_string(),
        created_at: now.clone(),
        last_active_at: now,
        status: ProjectSessionStatus::Active,
        latest_result: None, // Initialize latest_result
    };
    sessions.insert(session_id.clone(), session.clone()); 

    // Spawn Root Orchestrator
    // Assuming 'node' is in PATH and gemini_adapter.js exists
    let command = "node".to_string();
    let adapter_path = state.config.project_root.join("server").join("tests").join("scripts").join("gemini_adapter.js");
    let args = vec![
        adapter_path.to_str().unwrap_or_default().to_string(),
    ];
    let instruction = "As the Root Orchestrator, your first task is to initialize the project and decide on the next steps.".to_string();
    let env_vars = HashMap::new(); // For now, no specific env vars

    match state.agent_spawner.spawn_agent(
        session_id.clone(),
        "orchestrator".to_string(),
        instruction,
        command,
        args,
        env_vars,
    ) {
        Ok(agent_id) => tracing::info!("Root Orchestrator agent {} spawned for session {}", agent_id, session_id),
        Err(e) => tracing::error!("Failed to spawn Root Orchestrator for session {}: {}", session_id, e),
    }

    session
}

pub fn get_session(state: &AppState, session_id: &str) -> Option<ProjectSession> {
    let sessions = state.project_sessions.read();
    sessions.get(session_id).cloned()
}

pub fn update_last_active(state: &AppState, session_id: &str) -> Option<ProjectSession> {
    let mut sessions = state.project_sessions.write();
    if let Some(session) = sessions.get_mut(session_id) {
        session.last_active_at = Utc::now().to_rfc3339();
        return Some(session.clone());
    }
    None
}

fn find_active_session(state: &AppState, project_root: &str) -> Option<ProjectSession> {
    let sessions = state.project_sessions.read();
    sessions
        .values()
        .find(|session| {
            session.status == ProjectSessionStatus::Active && session.project_root == project_root
        })
        .cloned()
}
