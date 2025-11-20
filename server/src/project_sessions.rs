use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSession {
    pub session_id: String,
    pub project_root: String,
    pub project_name: String,
    pub created_at: String,
    pub last_active_at: String,
    pub status: ProjectSessionStatus,
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
    };
    sessions.insert(session_id, session.clone());
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
