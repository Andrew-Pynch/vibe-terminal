use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use std::env;
use std::path::PathBuf;
use anyhow::Context;

use agent_hub_server::{
    agents::registry::AgentRegistry,
    api,
    config::ServerConfig,
    global_registry::{load_or_init_registry, GlobalProjectRegistry, RegistryError},
    llm::LlmRegistry,
    profiles::ProfileCatalog,
    project_sessions::ProjectSession,
    sessions::SessionStore,
    state::AppState,
    ws,
    agents::spawner::AgentSpawner,
    agents::watcher::ResultWatcher,
    agents::dispatcher::TaskDispatcher,
};
use axum::Router;
use parking_lot::RwLock;
use tracing::error;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok(); // Load .env file if it exists
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let server_root_dir = env::current_dir().context("Failed to get current directory")?;

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
    let agents = AgentRegistry::new();
    let agent_spawner = AgentSpawner::new(agents.clone(), server_root_dir.clone());
    
    let task_dispatcher = TaskDispatcher::new(agent_spawner.clone(), Arc::new(config.clone()));

    let state = AppState {
        config: config.clone(),
        sessions,
        profiles,
        llms,
        global_registry,
        project_sessions,
        agents: agents.clone(),
        agent_spawner,
        server_root_dir: server_root_dir.clone(),
    };

    // Initialize and spawn ResultWatcher
    let result_watcher = ResultWatcher::new(
        state.agents.clone(),
        state.project_sessions.clone(),
        server_root_dir.clone(),
        task_dispatcher,
    );

    tokio::spawn(async move {
        if let Err(e) = result_watcher.start().await {
            error!("ResultWatcher failed: {:?}", e);
        }
    });

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