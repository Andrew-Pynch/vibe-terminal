use std::env;
use std::sync::Arc;
use crate::tasks::{TaskGraph, Task};
use crate::agents::spawner::AgentSpawner;
use crate::config::ServerConfig;
use std::collections::{HashMap, HashSet};
use tracing::{info, error, warn};
use std::path::PathBuf;
use parking_lot::Mutex;

#[derive(Clone)]
pub struct TaskDispatcher {
    spawner: AgentSpawner,
    config: Arc<ServerConfig>,
    base_dir: PathBuf,
    // State management
    task_queue: Arc<Mutex<Vec<(String, Task)>>>, // (session_id, Task)
    running_agent: Arc<Mutex<Option<String>>>, // Some(agent_id) if busy
    dispatched_tasks: Arc<Mutex<HashSet<(String, String)>>>, // (session_id, task_id) for idempotency
}

impl TaskDispatcher {
    pub fn new(spawner: AgentSpawner, config: Arc<ServerConfig>, base_dir: PathBuf) -> Self {
        Self { 
            spawner, 
            config, 
            base_dir,
            task_queue: Arc::new(Mutex::new(Vec::new())),
            running_agent: Arc::new(Mutex::new(None)),
            dispatched_tasks: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Called by ResultWatcher when a new TASK_GRAPH.json is found.
    pub async fn dispatch(&self, session_id: String, task_graph: TaskGraph) { 
        info!("Analyzing {} tasks for dispatch in session: {}", task_graph.tasks.len(), session_id);

        {
            let mut queue = self.task_queue.lock();
            let mut dispatched = self.dispatched_tasks.lock();
            let mut added_count = 0;

            for task in task_graph.tasks {
                // Idempotency check
                if dispatched.contains(&(session_id.clone(), task.id.clone())) {
                    continue;
                }

                dispatched.insert((session_id.clone(), task.id.clone()));
                queue.push((session_id.clone(), task));
                added_count += 1;
            }
            info!("Queued {} new tasks. Queue size: {}", added_count, queue.len());
        } // Guards dropped here

        self.process_queue().await;
    }

    /// Called by ResultWatcher when it detects an agent has completed (RESULT.md).
    pub async fn on_agent_complete(&self, agent_id: &str) {
        info!("Agent {} completed. Checking queue...", agent_id);
        
        {
            let mut running = self.running_agent.lock();
            if let Some(current_id) = running.as_ref() {
                if current_id == agent_id {
                    *running = None;
                } else {
                    warn!("Completed agent {} does not match expected running agent {}", agent_id, current_id);
                }
            }
        } // Guard dropped

        self.process_queue().await;
    }

    async fn process_queue(&self) {
        {
            let running = self.running_agent.lock();
            if running.is_some() {
                return;
            }
        } // Drop running lock

        let next_task = {
            let mut queue = self.task_queue.lock();
            if queue.is_empty() {
                None
            } else {
                // FIFO
                Some(queue.remove(0))
            }
        }; // Drop queue lock

        if let Some((session_id, task)) = next_task {
            info!("Starting execution for task: {} - {}", task.id, task.description);

            let agent_type = task.agent_type.clone().unwrap_or_else(|| "worker".to_string());
            let agent_id = uuid::Uuid::new_v4().to_string(); 

            {
                let mut running = self.running_agent.lock();
                *running = Some(agent_id.clone());
            } // Drop running lock

        // Construct the dynamic prompt content for the agent
        let prompt_content = format!(
            r#"You are a Vibe agent named {}. Your task is to {}.

You have access to the following Vibe utilities, which are executable binaries in your PATH:
- `vibe-report --agent-id {} --session-id {} --progress <percentage> --thought "<message>"`
- `vibe-ask --agent-id {} --session-id {} --question "<question>"` (Blocks until user replies)
- `vibe-complete --agent-id {} --session-id {} --result "<summary>"`

**IMPORTANT:** To use these utilities, you MUST use the `run_shell_command` tool. 
For example: `run_shell_command(command="vibe-complete --agent-id ...")`

Do NOT try to call these as direct tool functions.

You should use your tools to perform the task. When you believe you have successfully completed the task, use `vibe-complete`.

Task: {}
"#,
            agent_id, task.description, agent_id, session_id, agent_id, session_id, agent_id, session_id, task.description
        );

        let mut env_vars = HashMap::new();
        if let Ok(key) = env::var("GEMINI_API_KEY") {
            env_vars.insert("GEMINI_API_KEY".to_string(), key);
        }
        let server_url = format!("http://{}:{}", self.config.host, self.config.http_port);
        env_vars.insert("VIBE_SERVER_URL".to_string(), server_url);
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

        let adapter = crate::llm::adapters::get_adapter(&self.config.default_llm.provider);
        let command = adapter.get_command();
        let args = adapter.get_args(
            "INSTRUCTION.md", 
            &self.config.default_llm.model
        );

                    // Await the async spawn_agent call
                    match self.spawner.spawn_agent(
                        session_id.clone(),
                        agent_type,
                        prompt_content, 
                        command.clone(),
                        args.clone(),
                        env_vars.clone(),
                        Some(agent_id.clone()), // Pass the agent ID we generated
                    ).await { 
                        Ok(spawned_agent_id) => {
                            info!("Successfully spawned agent {} for task {}. Worker will read INSTRUCTION.md.", spawned_agent_id, task.id);
                        },
                        Err(e) => {
                            error!("Failed to spawn agent for task {}: {}", task.id, e);
                            // If spawn fails, we should probably clear the running flag so the next task can try
                            let mut running = self.running_agent.lock();
                            *running = None;
                            // Optionally re-trigger process_queue logic?
                        }
                    }
                }
            }
        }
