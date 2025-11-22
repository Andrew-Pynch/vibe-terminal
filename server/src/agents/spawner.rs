use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;
use std::process::Stdio; // Use tokio's Command
use crate::agents::registry::{Agent, AgentRegistry, AgentStatus};
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt}; // For async file I/O
use tracing::{info, error};

#[derive(Clone)]
pub struct AgentSpawner {
    registry: AgentRegistry,
    base_dir: PathBuf,
}

impl AgentSpawner {
    pub fn new(registry: AgentRegistry, base_dir: PathBuf) -> Self {
        Self { registry, base_dir }
    }

    /// Spawns a new agent for a given session.
    pub async fn spawn_agent( // Make this async
        &self, 
        session_id: String, 
        agent_type: String, 
        instruction: String,
        command_str: String, 
        args: Vec<String>,
        env_vars: HashMap<String, String>
    ) -> Result<String, String> {
        let mut agent = Agent::new(session_id.clone(), agent_type.clone());
        let agent_id = agent.id.clone();
        
        let agent_dir = self.base_dir.join(".vibe").join("agents").join(&session_id).join(&agent_id);
        if let Err(e) = tokio::fs::create_dir_all(&agent_dir).await { // Use tokio::fs
            return Err(format!("Failed to create agent directory {:?}: {}", agent_dir, e));
        }

        let instruction_path = agent_dir.join("INSTRUCTION.md");
        if let Err(e) = tokio::fs::write(&instruction_path, &instruction).await { // Use tokio::fs
            return Err(format!("Failed to write INSTRUCTION.md: {}", e));
        }

        // 4. Spawn Process and capture stdout/stderr
        let debug_log_path = agent_dir.join("debug_log.txt");
        let _debug_log_file = tokio::fs::File::create(&debug_log_path) // _ to suppress unused warning
            .await
            .map_err(|e| format!("Failed to create debug_log.txt at {:?}: {}", debug_log_path, e))?;

        let child_result = Command::new(&command_str) // Use tokio::process::Command
            .args(&args)
            .envs(&env_vars)
            .current_dir(&agent_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match child_result {
            Ok(mut child) => {
                agent.pid = child.id(); // Assign Option<u32> directly
                agent.status = AgentStatus::Running;
                self.registry.register_agent(agent);

                let stdout = child.stdout.take().expect("Failed to capture stdout");
                let stderr = child.stderr.take().expect("Failed to capture stderr");
                
                let agent_id_for_log = agent_id.clone();
                let registry_clone = self.registry.clone(); // Clone registry for the spawned task

                tokio::spawn(async move {
                    let mut reader_stdout = BufReader::new(stdout);
                    let mut reader_stderr = BufReader::new(stderr);
                    let mut stdout_line = String::new(); // Separate buffer for stdout
                    let mut stderr_line = String::new(); // Separate buffer for stderr

                    let mut log_file = tokio::fs::OpenOptions::new()
                        .append(true)
                        .create(false) // file should already exist
                        .open(&debug_log_path)
                        .await
                        .expect("Failed to open debug log file for appending");

                    loop {
                        tokio::select! {
                            result_stdout = reader_stdout.read_line(&mut stdout_line) => {
                                match result_stdout {
                                    Ok(0) => break, // EOF
                                    Ok(_) => {
                                        let log_entry = format!("[{}] STDOUT: {}", agent_id_for_log, stdout_line.trim_end());
                                        log_file.write_all(log_entry.as_bytes()).await.expect("Failed to write to log");
                                        log_file.write_all(b"\n").await.expect("Failed to write newline to log");
                                        info!("{}", log_entry); // Also log to server's info stream
                                        stdout_line.clear();
                                    },
                                    Err(e) => {
                                        error!("Error reading stdout for agent {}: {}", agent_id_for_log, e);
                                        break;
                                    }
                                }
                            }
                            result_stderr = reader_stderr.read_line(&mut stderr_line) => { // Use stderr_line
                                match result_stderr {
                                    Ok(0) => break, // EOF
                                    Ok(_) => {
                                        let log_entry = format!("[{}] STDERR: {}", agent_id_for_log, stderr_line.trim_end());
                                        log_file.write_all(log_entry.as_bytes()).await.expect("Failed to write to log");
                                        log_file.write_all(b"\n").await.expect("Failed to write newline to log");
                                        error!("{}", log_entry); // Also log to server's error stream
                                        stderr_line.clear();
                                    },
                                    Err(e) => {
                                        error!("Error reading stderr for agent {}: {}", agent_id_for_log, e);
                                        break;
                                    }
                                }
                            }
                            // Also await child process exit to update its status
                            exit_status = child.wait() => {
                                match exit_status {
                                    Ok(status) => {
                                        let final_status = if status.success() {
                                            AgentStatus::Completed
                                        } else {
                                            AgentStatus::Failed(format!("Exited with status: {:?}", status))
                                        };
                                        if let Err(e) = registry_clone.update_status(&agent_id_for_log, final_status) {
                                            error!("Failed to update agent {} status after exit: {}", agent_id_for_log, e);
                                        }
                                        info!("Agent {} process exited with status: {:?}.", agent_id_for_log, status);
                                    },
                                    Err(e) => {
                                        error!("Error waiting for agent {} process: {}", agent_id_for_log, e);
                                        if let Err(e) = registry_clone.update_status(&agent_id_for_log, AgentStatus::Failed(format!("Process wait error: {}", e))) {
                                            error!("Failed to update agent {} status after wait error: {}", agent_id_for_log, e);
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                });
                
                Ok(agent_id)
            }
            Err(e) => {
                agent.status = AgentStatus::Failed(e.to_string());
                self.registry.register_agent(agent);
                Err(format!("Failed to spawn command '{}': {}", command_str, e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::runtime::Runtime; // For running async test

    #[test]
    fn test_spawn_agent_creates_files() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { // Run async test in block_on
            let registry = AgentRegistry::new();
            let temp_dir = tempdir().unwrap();
            let spawner = AgentSpawner::new(registry.clone(), temp_dir.path().to_path_buf());

            let session_id = "test-session-1";
            let agent_id_result = spawner.spawn_agent(
                session_id.to_string(),
                "worker".to_string(),
                "Do the work".to_string(),
                "echo".to_string(), // Use echo for test command
                vec!["Hello from agent".to_string()],
                HashMap::new(),
            ).await; // Await the async call

            assert!(agent_id_result.is_ok());
            let agent_id = agent_id_result.unwrap();

            // Verify Directory
            let agent_dir = temp_dir.path().join(".vibe").join("agents").join(session_id).join(&agent_id); // Fixed path
            assert!(agent_dir.exists());

            // Verify Instruction File
            let instruction_path = agent_dir.join("INSTRUCTION.md");
            assert!(instruction_path.exists());
            let content = tokio::fs::read_to_string(instruction_path).await.unwrap(); // Use tokio::fs
            assert_eq!(content, "Do the work");

            // Verify debug_log.txt (give some time for async task)
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let debug_log_path = agent_dir.join("debug_log.txt");
            assert!(debug_log_path.exists());
            let debug_content = tokio::fs::read_to_string(debug_log_path).await.unwrap();
            assert!(debug_content.contains("Hello from agent"));

            // Verify Registry
            let agent = registry.get_agent(&agent_id).unwrap();
            assert_eq!(agent.status, AgentStatus::Completed); // Should be completed for echo
        });
    }
}