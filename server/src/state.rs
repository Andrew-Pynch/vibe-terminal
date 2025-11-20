use std::{collections::HashMap, sync::Arc};

use crate::{
    config::ServerConfig, global_registry::GlobalProjectRegistry, llm::LlmRegistry,
    profiles::ProfileCatalog, project_sessions::ProjectSession, sessions::SessionStore,
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
}
