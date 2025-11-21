use std::{collections::HashMap, sync::Arc};
use std::path::PathBuf; // Added this

use crate::{
    agents::registry::AgentRegistry, config::ServerConfig, global_registry::GlobalProjectRegistry,
    llm::LlmRegistry, profiles::ProfileCatalog, project_sessions::ProjectSession,
    sessions::SessionStore,
    agents::spawner::AgentSpawner,
};
use parking_lot::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub config: ServerConfig,
    pub sessions: Arc<SessionStore>,
    pub profiles: Arc<ProfileCatalog>,
    pub llms: Arc<LlmRegistry>,
    pub global_registry: Arc<RwLock<GlobalProjectRegistry>>,
    pub project_sessions: Arc<RwLock<HashMap<String, ProjectSession>>>,
    pub agents: AgentRegistry,
    pub agent_spawner: AgentSpawner,
    pub server_root_dir: PathBuf, // Added this
}
