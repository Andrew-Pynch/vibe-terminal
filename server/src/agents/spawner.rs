use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::agents::registry::{Agent, AgentRegistry, AgentStatus};

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
    /// 
    /// This function:
    /// 1. Creates a new Agent entry in the registry.
    /// 2. Creates the agent's working directory on disk.
    /// 3. Writes an initial INSTRUCTION.md file.
    /// 4. Spawns the specified process in that directory.
    pub fn spawn_agent(
        &self, 
        session_id: String, 
        agent_type: String, 
        instruction: String,
        command: String,
        args: Vec<String>,
        env_vars: HashMap<String, String>
    ) -> Result<String, String> {
        // 1. Create Agent Entry
        let mut agent = Agent::new(session_id.clone(), agent_type.clone());
        let agent_id = agent.id.clone();
        
        // 2. Create Working Directory: .vibe/agents/<session_id>/<agent_id>/
        let agent_dir = self.base_dir.join(".vibe").join("agents").join(&session_id).join(&agent_id);
        if let Err(e) = fs::create_dir_all(&agent_dir) {
            return Err(format!("Failed to create agent directory {:?}: {}", agent_dir, e));
        }

        // 3. Write INSTRUCTION.md
        let instruction_path = agent_dir.join("INSTRUCTION.md");
        if let Err(e) = fs::write(&instruction_path, &instruction) {
            return Err(format!("Failed to write INSTRUCTION.md: {}", e));
        }

        // 4. Spawn Process
        // We set the current directory to the agent's workspace so it knows where to read files.
        let child_result = Command::new(&command)
            .args(&args)
            .envs(&env_vars)
            .current_dir(&agent_dir)
            .stdout(Stdio::piped()) // Capture stdout/stderr if needed later, or let it inherit
            .stderr(Stdio::piped())
            .spawn();

        match child_result {
            Ok(child) => {
                let pid = child.id();
                agent.status = AgentStatus::Running;
                agent.pid = Some(pid);
                
                // Register the agent with the PID
                self.registry.register_agent(agent);
                
                Ok(agent_id)
            }
            Err(e) => {
                agent.status = AgentStatus::Failed(e.to_string());
                self.registry.register_agent(agent);
                Err(format!("Failed to spawn command '{}': {}", command, e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_spawn_agent_creates_files() {
        let registry = AgentRegistry::new();
        let temp_dir = tempdir().unwrap();
        let spawner = AgentSpawner::new(registry.clone(), temp_dir.path().to_path_buf());

        let session_id = "test-session-1";
        let agent_id_result = spawner.spawn_agent(
            session_id.to_string(),
            "worker".to_string(),
            "Do the work".to_string()
        );

        assert!(agent_id_result.is_ok());
        let agent_id = agent_id_result.unwrap();

        // Verify Directory
        let agent_dir = temp_dir.path().join("agents").join(session_id).join(&agent_id);
        assert!(agent_dir.exists());

        // Verify Instruction File
        let instruction_path = agent_dir.join("INSTRUCTION.md");
        assert!(instruction_path.exists());
        let content = fs::read_to_string(instruction_path).unwrap();
        assert_eq!(content, "Do the work");

        // Verify Registry
        let agent = registry.get_agent(&agent_id).unwrap();
        assert_eq!(agent.status, AgentStatus::Running);
    }
}
