mod api;
mod config;
mod global_registry;
mod llm;
mod profiles;
mod project_sessions;
mod sessions;
mod state;
mod ws;

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::Router;
use config::ServerConfig;
use global_registry::{load_or_init_registry, GlobalProjectRegistry, RegistryError};
use llm::LlmRegistry;
use parking_lot::RwLock;
use profiles::ProfileCatalog;
use project_sessions::ProjectSession;
use sessions::SessionStore;
use state::AppState;
use tracing::error;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let config = ServerConfig::from_env()?;
    let profiles = Arc::new(ProfileCatalog::load(&config.prompt_profile_dir)?);
    let sessions = Arc::new(SessionStore::new());
    let llms = Arc::new(LlmRegistry::new());
    let registry = match load_or_init_registry() {
        Ok(registry) => registry,
        Err(RegistryError::Parse(err)) => {
            error!("Failed to parse global project registry, using empty registry: {err}");
            GlobalProjectRegistry::empty()
        }
        Err(err) => {
            error!("Unable to initialize global project registry: {err}");
            return Err(err.into());
        }
    };
    let global_registry = Arc::new(RwLock::new(registry));
    let project_sessions = Arc::new(RwLock::new(HashMap::<String, ProjectSession>::new()));
    let state = AppState {
        config: config.clone(),
        sessions,
        profiles,
        llms,
        global_registry,
        project_sessions,
    };

    let http_app = api::router(state.clone());
    let ws_app = ws::router(state.clone());

    let http_addr = SocketAddr::new(config.host, config.http_port);
    let ws_addr = SocketAddr::new(config.host, config.ws_port);

    let http_listener = tokio::net::TcpListener::bind(http_addr).await?;
    let ws_listener = tokio::net::TcpListener::bind(ws_addr).await?;

    tracing::info!("HTTP server listening on http://{}", http_addr);
    tracing::info!("WebSocket server listening on ws://{}", ws_addr);

    tokio::try_join!(serve(http_listener, http_app), serve(ws_listener, ws_app))?;

    Ok(())
}

async fn serve(listener: tokio::net::TcpListener, app: Router) -> anyhow::Result<()> {
    axum::serve(listener, app).await?;
    Ok(())
}
