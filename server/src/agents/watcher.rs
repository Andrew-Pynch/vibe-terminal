use std::path::{PathBuf, Path};
use std::sync::Arc;
use notify::{Watcher, RecursiveMode, RecommendedWatcher, Config, EventKind, Event};
use anyhow::{Result, Context};
use tokio::fs; // For async file operations
use tracing::{info, error}; // For logging
use parking_lot::RwLock;
use std::collections::HashMap;

use crate::agents::registry::{AgentRegistry, AgentStatus};
use crate::project_sessions::ProjectSession;
use crate::agents::dispatcher::TaskDispatcher;
use crate::tasks::TaskGraph;

pub struct ResultWatcher {
    registry: AgentRegistry,
    project_sessions: Arc<RwLock<HashMap<String, ProjectSession>>>,
    base_dir: PathBuf,
    dispatcher: TaskDispatcher,
}

impl ResultWatcher {
    pub fn new(
        registry: AgentRegistry,
        project_sessions: Arc<RwLock<HashMap<String, ProjectSession>>>,
        base_dir: PathBuf,
        dispatcher: TaskDispatcher,
    ) -> Self {
        ResultWatcher {
            registry,
            project_sessions,
            base_dir,
            dispatcher,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("ResultWatcher starting...");

        let (tx, mut rx) = tokio::sync::mpsc::channel(100); // Use tokio's mpsc channel
        let mut watcher = RecommendedWatcher::new(move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.expect("Failed to send watch event");
            });
        }, Config::default())?;

        let agents_dir = self.base_dir.join(".vibe").join("agents");
        watcher.watch(Path::new(&agents_dir), RecursiveMode::Recursive)
            .context(format!("Failed to watch directory: {:?}", &agents_dir))?;

        info!("Watching for changes in: {:?}", agents_dir);

        while let Some(res) = rx.recv().await {
            match res {
                Ok(event) => {
                    if let Err(e) = self.handle_event(event).await {
                        error!("Error handling watch event: {:?}", e);
                    }
                },
                Err(e) => error!("watch error: {:?}", e),
            }
        }
        Ok(())
    }

    async fn handle_event(&self, event: Event) -> Result<()> {
        if let EventKind::Create(notify::event::CreateKind::File) = event.kind {
            for path in event.paths {
                let file_name = path.file_name().and_then(|n| n.to_str());

                match file_name {
                    Some("RESULT.md") => {
                        info!("Detected RESULT.md created: {:?}", path);

                        let parent_dir = path.parent().context("RESULT.md has no parent directory")?;
                        let agent_id = parent_dir.file_name()
                            .context("Agent directory has no name")?
                            .to_str()
                            .context("Agent directory name is not valid UTF-8")?
                            .to_string();

                        let session_dir = parent_dir.parent().context("Agent directory has no parent")?;
                        let session_id = session_dir.file_name()
                            .context("Session directory has no name")?
                            .to_str()
                            .context("Session directory name is not valid UTF-8")?
                            .to_string();

                        info!("Extracted session_id: {}, agent_id: {}", session_id, agent_id);

                        let result_content = fs::read_to_string(&path)
                            .await
                            .context(format!("Failed to read RESULT.md from {:?}", path))?;

                        // Update AgentRegistry
                        if let Err(e) = self.registry.update_status_and_result(
                            &agent_id,
                            AgentStatus::Completed,
                            Some(result_content.clone()), // Clone for project_sessions
                        ) {
                            error!("Failed to update status and result for agent {}: {}", agent_id, e);
                        } else {
                            info!("Updated status and result for agent {}", agent_id);
                        }


                        // Update ProjectSession
                        let mut project_sessions_guard = self.project_sessions.write();
                        if let Some(session) = project_sessions_guard.get_mut(&session_id) {
                            info!("Updating latest_result for session {}: {}", session_id, result_content);
                            session.latest_result = Some(result_content);
                        } else {
                            error!("ProjectSession with ID {} not found for agent {}.", session_id, agent_id);
                        }
                    },
                    Some("TASK_GRAPH.json") => {
                        info!("Detected TASK_GRAPH.json created: {:?}", path);
                        
                        // Parent dir is the agent dir that created the task graph (e.g. Orchestrator)
                        let parent_dir = path.parent().context("TASK_GRAPH.json has no parent directory")?;
                        let session_dir = parent_dir.parent().context("Agent directory has no parent")?;
                        let session_id = session_dir.file_name()
                            .context("Session directory has no name")?
                            .to_str()
                            .context("Session directory name is not valid UTF-8")?
                            .to_string();

                        let content = fs::read_to_string(&path)
                            .await
                            .context(format!("Failed to read TASK_GRAPH.json from {:?}", path))?;

                        let task_graph: TaskGraph = serde_json::from_str(&content)
                            .context("Failed to parse TASK_GRAPH.json")?;
                        
                        info!("Parsed TaskGraph with {} tasks. Dispatching...", task_graph.tasks.len());
                        
                        self.dispatcher.dispatch(session_id, task_graph);
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }
}