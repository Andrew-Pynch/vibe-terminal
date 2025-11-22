use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use tracing::{info, error};

use crate::state::AppState;
use std::env; 

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

pub async fn create_or_get_session_for_project(
    state: &AppState,
    project_root: &str,
    project_name: &str,
) -> ProjectSession {
    // 1. Optimistic read check (sync)
    if let Some(existing) = find_active_session(state, project_root) {
        return update_last_active(state, &existing.session_id).unwrap_or(existing);
    }

    // 2. Write lock scope
    let (session, should_spawn) = {
        let mut sessions = state.project_sessions.write();
        
        // Double check inside lock
        if let Some(existing) = sessions
            .values()
            .find(|session| {
                session.status == ProjectSessionStatus::Active && session.project_root == project_root
            })
            .cloned()
        {
            // Found existing, update timestamp inline
            let mut updated = existing.clone();
            updated.last_active_at = Utc::now().to_rfc3339();
            if let Some(entry) = sessions.get_mut(&existing.session_id) {
                entry.last_active_at = updated.last_active_at.clone();
            }
            (updated, false)
        } else {
            // Create new
            let now = Utc::now().to_rfc3339();
            let session_id = Uuid::new_v4().to_string();
            let session = ProjectSession {
                session_id: session_id.clone(),
                project_root: project_root.to_string(),
                project_name: project_name.to_string(),
                created_at: now.clone(),
                last_active_at: now,
                status: ProjectSessionStatus::Active,
                latest_result: None,
            };
            sessions.insert(session_id.clone(), session.clone());
            (session, true)
        }
    }; // Lock is dropped here

    // 3. Async operations (Spawn) - Lock is released
    if should_spawn {
        let command = "gemini".to_string();
        let args = vec![
            "-p".to_string(),
            "INSTRUCTION.md".to_string(),
            "--yolo".to_string(),
            "-m".to_string(),
            state.config.default_llm.model.clone(),
        ];
        
        let agent_id = Uuid::new_v4().to_string();
        let server_url = format!("http://{}:{}", state.config.host, state.config.http_port);
        let session_id = session.session_id.clone();

        let instruction = format!(
            r#"You are the Root Orchestrator Vibe agent (ID: {}). Your goal is to plan the development of this project: '{}'.

You have the following tools available via shell commands:
- `vibe-report --agent-id {} --session-id {} --progress <percentage> --thought "<message>"`: Report your current progress and thought process to the server.
- `vibe-complete --agent-id {} --session-id {} --result "<summary>"`: Signal that you have completed your task, optionally with a summary.
- All other standard shell commands, including `read_file`, `write_file`, `glob`, `search_file_content`, etc.

Your first task is to analyze the project state (currently empty or initialized) and outline the next development steps.
Output a JSON object with a 'tasks' array describing the next steps.
Each task should have an 'id' (string), 'description' (string), and optional 'agent_type' (string).

Example:
```json
{{
  "tasks": [
    {{ "id": "init-1", "description": "Create README.md", "agent_type": "worker" }}
  ]
}}
```
"#,
            agent_id,
            project_name,
            agent_id, session_id,
            agent_id, session_id
        );

        let mut env_vars = HashMap::new();
        if let Ok(key) = env::var("GEMINI_API_KEY") {
            env_vars.insert("GEMINI_API_KEY".to_string(), key);
        }
        env_vars.insert("VIBE_SERVER_URL".to_string(), server_url.clone());
        env_vars.insert("AGENT_ID".to_string(), agent_id.clone());
        env_vars.insert("SESSION_ID".to_string(), session_id.clone());

        match state.agent_spawner.spawn_agent(
            session_id.clone(),
            "orchestrator".to_string(),
            instruction,
            command,
            args,
            env_vars,
        ).await {
            Ok(spawned_agent_id) => info!("Root Orchestrator agent {} spawned for session {}", spawned_agent_id, session_id),
            Err(e) => error!("Failed to spawn Root Orchestrator for session {}: {}", session_id, e),
        }
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
