use std::env;
use std::sync::Arc;
use crate::tasks::TaskGraph;
use crate::agents::spawner::AgentSpawner;
use crate::config::ServerConfig;
use std::collections::HashMap;
use tracing::{info, error};

pub struct TaskDispatcher {
    spawner: AgentSpawner,
    config: Arc<ServerConfig>,
}

impl TaskDispatcher {
    pub fn new(spawner: AgentSpawner, config: Arc<ServerConfig>) -> Self {
        Self { spawner, config }
    }

    pub fn dispatch(&self, session_id: String, task_graph: TaskGraph) {
        info!("Dispatching {} tasks for session: {}", task_graph.tasks.len(), session_id);
        
        // Determine the path to the Gemini adapter script
        // Assuming we are running from the repo root or server root
        let base_dir = match env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to get current directory: {}", e);
                return;
            }
        };

        // Logic to find the adapter script.
        let adapter_path = base_dir.join("server/tests/scripts/gemini_adapter.js");
        let final_adapter_path = if adapter_path.exists() {
            adapter_path
        } else {
            base_dir.join("tests/scripts/gemini_adapter.js")
        };
        
        if !final_adapter_path.exists() {
            error!("Could not find gemini_adapter.js at {:?}", final_adapter_path);
        }

        let mut env_vars = HashMap::new();
        if let Ok(key) = env::var("GEMINI_API_KEY") {
            env_vars.insert("GEMINI_API_KEY".to_string(), key);
        }

        for task in task_graph.tasks {
            info!("Spawning agent for task: {} - {}", task.id, task.description);
            
            // Default to "worker" if not specified
            let agent_type = task.agent_type.clone().unwrap_or_else(|| "worker".to_string());
            
            // Wrap instruction for worker
            let instruction = format!(
                "You are a worker agent. Your task is:\n\n{}\n\nPlease provide the code or result requested.",
                task.description
            );

            // Construct command: node <adapter_path>
            let command = "node".to_string();
            let args = vec![
                final_adapter_path.to_string_lossy().to_string(),
                // We might want to pass other args later, like model override
            ];

            match self.spawner.spawn_agent(
                session_id.clone(),
                agent_type,
                instruction,
                command.clone(),
                args.clone(),
                env_vars.clone(),
            ) {
                Ok(agent_id) => {
                    info!("Successfully spawned agent {} for task {}", agent_id, task.id);
                },
                Err(e) => {
                    error!("Failed to spawn agent for task {}: {}", task.id, e);
                }
            }
        }
    }
}
