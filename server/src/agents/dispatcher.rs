use std::env;
use std::sync::Arc;
use crate::tasks::TaskGraph;
use crate::agents::spawner::AgentSpawner;
use crate::config::ServerConfig;
use std::collections::HashMap;
use tracing::{info, error};
use std::path::PathBuf;

#[derive(Clone)]
pub struct TaskDispatcher {
    spawner: AgentSpawner,
    config: Arc<ServerConfig>, // config is currently unused, but kept for future expansion
    base_dir: PathBuf, // Add base_dir to TaskDispatcher to correctly resolve paths
}

impl TaskDispatcher {
    pub fn new(spawner: AgentSpawner, config: Arc<ServerConfig>, base_dir: PathBuf) -> Self {
        Self { spawner, config, base_dir }
    }

    pub async fn dispatch(&self, session_id: String, task_graph: TaskGraph) { // Make dispatch async
        info!("Dispatching {} tasks for session: {}", task_graph.tasks.len(), session_id);

        let server_url = format!("http://{}:{}", self.config.host, self.config.http_port);

        for task in task_graph.tasks {
            info!("Spawning agent for task: {} - {}", task.id, task.description);

            // Default to "worker" if not specified
            let agent_type = task.agent_type.clone().unwrap_or_else(|| "worker".to_string());
            let agent_id = uuid::Uuid::new_v4().to_string(); 

            // Construct the dynamic prompt content for the agent
            let prompt_content = format!(
                r#"You are a Vibe agent named {}. Your task is to {}.

You have the following tools available via shell commands:
- `vibe-report --agent-id {} --session-id {} --progress <percentage> --thought "<message>"`: Report your current progress and thought process to the server.
- `vibe-complete --agent-id {} --session-id {} --result "<summary>"`: Signal that you have completed your task, optionally with a summary.
- All other standard shell commands, including `read_file`, `write_file`, `glob`, `search_file_content`, etc.

You should use your tools to perform the task. When you believe you have successfully completed the task, use `vibe-complete`.

Task: {}
"#,
                agent_id, task.description, agent_id, session_id, agent_id, session_id, task.description
            );

            let mut env_vars = HashMap::new();
            if let Ok(key) = env::var("GEMINI_API_KEY") {
                env_vars.insert("GEMINI_API_KEY".to_string(), key);
            }
            env_vars.insert("VIBE_SERVER_URL".to_string(), server_url.clone());
            env_vars.insert("AGENT_ID".to_string(), agent_id.clone());
            env_vars.insert("SESSION_ID".to_string(), session_id.clone());

            // Inject PATH to include shim binaries
            let shim_dir = if self.base_dir.join("server/Cargo.toml").exists() {
                self.base_dir.join("server/target/debug")
            } else {
                self.base_dir.join("target/debug")
            };
            
            if let Ok(current_path) = env::var("PATH") {
                let new_path = format!("{}:{}", shim_dir.to_string_lossy(), current_path);
                env_vars.insert("PATH".to_string(), new_path);
            } else {
                env_vars.insert("PATH".to_string(), shim_dir.to_string_lossy().to_string());
            }

            let command = "gemini".to_string();
            let args = vec![
                "-p".to_string(),
                "INSTRUCTION.md".to_string(),
                "--yolo".to_string(),
                "-m".to_string(),
                self.config.default_llm.model.clone(),
            ];
            // Await the async spawn_agent call
            match self.spawner.spawn_agent(
                session_id.clone(),
                agent_type,
                prompt_content, 
                command.clone(),
                args.clone(),
                env_vars.clone(),
            ).await { // Await here
                Ok(spawned_agent_id) => {
                    info!("Successfully spawned agent {} for task {}. Worker will read INSTRUCTION.md.", spawned_agent_id, task.id);
                },
                Err(e) => {
                    error!("Failed to spawn agent for task {}: {}", task.id, e);
                }
            }
            // Add a delay between spawns to avoid hitting API rate limits
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}
