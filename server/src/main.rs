mod api;
mod config;
mod llm;
mod profiles;
mod sessions;
mod state;
mod ws;

use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use config::ServerConfig;
use llm::LlmRegistry;
use profiles::ProfileCatalog;
use sessions::SessionStore;
use state::AppState;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();

  let config = ServerConfig::from_env()?;
  let profiles = Arc::new(ProfileCatalog::load(&config.prompt_profile_dir)?);
  let sessions = Arc::new(SessionStore::new());
  let llms = Arc::new(LlmRegistry::new());
  let state = AppState {
    config: config.clone(),
    sessions,
    profiles,
    llms,
  };

  let http_app = api::router(state.clone());
  let ws_app = ws::router(state.clone());

  let http_addr = SocketAddr::new(config.host, config.http_port);
  let ws_addr = SocketAddr::new(config.host, config.ws_port);

  let http_listener = tokio::net::TcpListener::bind(http_addr).await?;
  let ws_listener = tokio::net::TcpListener::bind(ws_addr).await?;

  tracing::info!("HTTP server listening on {}", http_addr);
  tracing::info!("WS server listening on {}", ws_addr);

  tokio::try_join!(
    serve(http_listener, http_app),
    serve(ws_listener, ws_app)
  )?;

  Ok(())
}

async fn serve(listener: tokio::net::TcpListener, app: Router) -> anyhow::Result<()> {
  axum::serve(listener, app).await?;
  Ok(())
}
