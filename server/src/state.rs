use std::sync::Arc;

use crate::{config::ServerConfig, llm::LlmRegistry, profiles::ProfileCatalog, sessions::SessionStore};

#[derive(Clone)]
pub struct AppState {
  pub config: ServerConfig,
  pub sessions: Arc<SessionStore>,
  pub profiles: Arc<ProfileCatalog>,
  pub llms: Arc<LlmRegistry>,
}
