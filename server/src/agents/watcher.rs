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
        // Handle Modify as well, because sometimes files are created empty then modified
        if let EventKind::Create(_) | EventKind::Modify(_) = event.kind {
            for path in event.paths {
                let file_name = path.file_name().and_then(|n| n.to_str());

                match file_name {
                    Some("RESULT.md") => {
                        // We only want to process this if it's not empty to avoid spamming updates
                        // or reading empty files.
                        if let Ok(metadata) = fs::metadata(&path).await {
                            if metadata.len() == 0 {
                                continue;
                            }
                        }

                        info!("Detected RESULT.md update: {:?}", path);

                        let parent_dir = path.parent().context("RESULT.md has no parent directory")?;
                        let agent_id = parent_dir.file_name()
                            .context("Agent directory has no name")?
                            .to_str()
                            .context("Agent directory name is not valid UTF-8")?;

                        let session_dir = parent_dir.parent().context("Agent directory has no parent")?;
                        let session_id = session_dir.file_name()
                            .context("Session directory has no name")?
                            .to_str()
                            .context("Session directory name is not valid UTF-8")?;

                        // info!("Extracted session_id: {}, agent_id: {}", session_id, agent_id);

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
                            // info!("Updated status and result for agent {}", agent_id);
                        }

                                                                                                // Update ProjectSession

                                                                                                let mut project_sessions_guard = self.project_sessions.write();

                                                                                                if let Some(session) = project_sessions_guard.get_mut(session_id) {

                                                                                                    // info!("Updating latest_result for session {}: {}", session_id, result_content);

                                                                                                    session.latest_result = Some(result_content.clone());

                                                                                                }

                                                                                                

                                                                                                // Try to parse task graph from RESULT.md if it contains one

                                                                                                // This is a fallback/heuristic if the agent writes it to RESULT.md instead of TASK_GRAPH.json

                                                                                                 if result_content.contains("\"tasks\"") {

                                                                                                    // Try extracting JSON block

                                                                                                     let json_str = if let Some(start) = result_content.find("```json") {

                                                                                                         if let Some(end) = result_content[start..].find("```") {

                                                                                                             // Be careful with indices here, finding the SECOND ```

                                                                                                             let block = &result_content[start..];

                                                                                                              if let Some(end_block) = block[7..].find("```") {

                                                                                                                  &block[7..7+end_block]

                                                                                                              } else {

                                                                                                                  ""

                                                                                                              }

                                                                                                         } else { "" }

                                                                                                     } else {

                                                                                                         &result_content

                                                                                                     };

                                                                        

                                                                                                     if let Ok(task_graph) = serde_json::from_str::<TaskGraph>(json_str.trim()) {

                                                                                                         info!("Detected embedded TaskGraph in RESULT.md. Dispatching...");

                                                                                                         let dispatcher_clone = self.dispatcher.clone();

                                                                                                         let session_id_string = session_id.to_string();

                                                                                                         tokio::spawn(async move {

                                                                                                             dispatcher_clone.dispatch(session_id_string, task_graph).await;

                                                                                                         });

                                                                                                     }

                                                                                                 }

                                                                                            },
                    Some("TASK_GRAPH.json") => {
                        info!("Detected TASK_GRAPH.json update: {:?}", path);
                        
                        if let Ok(metadata) = fs::metadata(&path).await {
                            if metadata.len() == 0 {
                                continue;
                            }
                        }

                        let parent_dir = path.parent().context("TASK_GRAPH.json has no parent directory")?;
                        let session_dir = parent_dir.parent().context("Agent directory has no parent")?;
                        let session_id = session_dir.file_name()
                            .context("Session directory has no name")?
                            .to_str()
                            .context("Session directory name is not valid UTF-8")?;

                        let content = fs::read_to_string(&path)
                            .await
                            .context("Failed to read TASK_GRAPH.json")?;

                        match serde_json::from_str::<TaskGraph>(&content) {
                            Ok(task_graph) => {
                                info!("Parsed TaskGraph with {} tasks. Dispatching...", task_graph.tasks.len());
                                let dispatcher_clone = self.dispatcher.clone();
                                let session_id_string = session_id.to_string();
                                tokio::spawn(async move {
                                    dispatcher_clone.dispatch(session_id_string, task_graph).await;
                                });
                            },
                            Err(e) => error!("Failed to parse TASK_GRAPH.json: {}", e),
                        }
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }
}