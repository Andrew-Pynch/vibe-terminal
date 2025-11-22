use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Represents the status of an agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Starting,
    Running,
    Completed,
    Failed(String),
    Terminated,
}

/// Represents a single agent instance.
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub session_id: String,
    pub agent_type: String, // e.g., "orchestrator", "worker"
    pub status: AgentStatus,
    pub pid: Option<u32>,
    pub result: Option<String>,
    pub progress: Option<u8>,
    pub last_thought: Option<String>,
}

impl Agent {
    pub fn new(session_id: String, agent_type: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            session_id,
            agent_type,
            status: AgentStatus::Starting,
            pid: None,
            result: None,
            progress: Some(0),
            last_thought: None,
        }
    }
}

/// A thread-safe registry for managing active agents.
#[derive(Clone, Default)]
pub struct AgentRegistry {
    agents: Arc<Mutex<HashMap<String, Agent>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers a new agent.
    pub fn register_agent(&self, agent: Agent) {
        let mut agents = self.agents.lock().unwrap();
        agents.insert(agent.id.clone(), agent);
    }

    /// Retrieves an agent by its ID.
    pub fn get_agent(&self, agent_id: &str) -> Option<Agent> {
        let agents = self.agents.lock().unwrap();
        agents.get(agent_id).cloned()
    }

    /// Updates the status of an existing agent.
    pub fn update_status(&self, agent_id: &str, status: AgentStatus) -> Result<(), String> {
        let mut agents = self.agents.lock().unwrap();
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.status = status;
            Ok(())
        } else {
            Err(format!("Agent with ID {} not found", agent_id))
        }
    }

    /// Updates the PID of an existing agent.
    pub fn update_pid(&self, agent_id: &str, pid: u32) -> Result<(), String> {
        let mut agents = self.agents.lock().unwrap();
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.pid = Some(pid);
            Ok(())
        } else {
            Err(format!("Agent with ID {} not found", agent_id))
        }
    }

    /// Removes an agent from the registry.
    pub fn remove_agent(&self, agent_id: &str) -> Option<Agent> {
        let mut agents = self.agents.lock().unwrap();
        agents.remove(agent_id)
    }

    /// Lists all agents for a specific session.
    pub fn list_agents_by_session(&self, session_id: &str) -> Vec<Agent> {
        let agents = self.agents.lock().unwrap();
        agents
            .values()
            .filter(|a| a.session_id == session_id)
            .cloned()
            .collect()
    }

    /// Updates the status and result of an existing agent.
    pub fn update_status_and_result(
        &self,
        agent_id: &str,
        status: AgentStatus,
        result: Option<String>,
    ) -> Result<(), String> {
        let mut agents = self.agents.lock().unwrap();
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.status = status;
            agent.result = result;
            Ok(())
        } else {
            Err(format!("Agent with ID {} not found", agent_id))
        }
    }

    /// Updates the status, progress, and last thought of an existing agent.
    pub fn update_status_and_progress(
        &self,
        agent_id: &str,
        status: AgentStatus,
        progress: u8,
        thought: Option<String>,
    ) -> Result<(), String> {
        let mut agents = self.agents.lock().unwrap();
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.status = status;
            agent.progress = Some(progress);
            agent.last_thought = thought;
            Ok(())
        } else {
            Err(format!("Agent with ID {} not found", agent_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get_agent() {
        let registry = AgentRegistry::new();
        let agent = Agent::new("session-123".to_string(), "worker".to_string());
        let agent_id = agent.id.clone();

        registry.register_agent(agent);

        let retrieved_agent = registry.get_agent(&agent_id);
        assert!(retrieved_agent.is_some());
        assert_eq!(retrieved_agent.unwrap().session_id, "session-123");
    }

    #[test]
    fn test_update_status() {
        let registry = AgentRegistry::new();
        let agent = Agent::new("session-123".to_string(), "worker".to_string());
        let agent_id = agent.id.clone();

        registry.register_agent(agent);
        registry
            .update_status(&agent_id, AgentStatus::Running)
            .expect("Failed to update status");

        let retrieved_agent = registry.get_agent(&agent_id).unwrap();
        assert_eq!(retrieved_agent.status, AgentStatus::Running);
    }

    #[test]
    fn test_remove_agent() {
        let registry = AgentRegistry::new();
        let agent = Agent::new("session-123".to_string(), "worker".to_string());
        let agent_id = agent.id.clone();

        registry.register_agent(agent);
        let removed = registry.remove_agent(&agent_id);
        assert!(removed.is_some());
        assert!(registry.get_agent(&agent_id).is_none());
    }

    #[test]
    fn test_list_agents_by_session() {
        let registry = AgentRegistry::new();
        let agent1 = Agent::new("session-A".to_string(), "worker".to_string());
        let agent2 = Agent::new("session-A".to_string(), "orchestrator".to_string());
        let agent3 = Agent::new("session-B".to_string(), "worker".to_string());

        registry.register_agent(agent1);
        registry.register_agent(agent2);
        registry.register_agent(agent3);

        let session_a_agents = registry.list_agents_by_session("session-A");
        assert_eq!(session_a_agents.len(), 2);
    }
}
