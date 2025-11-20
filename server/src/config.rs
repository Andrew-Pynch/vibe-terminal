use std::{env, net::IpAddr, path::PathBuf, str::FromStr};

use serde::Serialize;

use crate::llm::{LlmConfig, ProviderKind};

#[derive(Clone, Serialize)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub http_port: u16,
    pub ws_port: u16,
    pub shared_secret: Option<String>,
    pub prompt_profile_dir: PathBuf,
    pub default_llm: LlmConfig,
}

impl ServerConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let host = read_env("AGENT_HUB_HOST").unwrap_or_else(|| "0.0.0.0".into());
        let http_port = read_env("AGENT_HUB_HTTP_PORT").unwrap_or_else(|| "4110".into());
        let ws_port = read_env("AGENT_HUB_WS_PORT").unwrap_or_else(|| "4111".into());
        let protocol = read_env("AGENT_HUB_PROVIDER").unwrap_or_else(|| "dummy".into());
        let model = read_env("AGENT_HUB_MODEL").unwrap_or_else(|| "dummy-orchestrator".into());
        let temperature = read_env("AGENT_HUB_TEMPERATURE")
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(0.2);
        let shared_secret = read_env("AGENT_HUB_SECRET");
        let prompt_dir = PathBuf::from(
            read_env("AGENT_HUB_PROMPT_PROFILE_DIR").unwrap_or_else(|| "prompts/profiles".into()),
        );

        Ok(Self {
            host: host.parse()?,
            http_port: http_port.parse()?,
            ws_port: ws_port.parse()?,
            shared_secret,
            prompt_profile_dir: prompt_dir,
            default_llm: LlmConfig {
                provider: ProviderKind::from_str(&protocol).unwrap_or(ProviderKind::Dummy),
                model,
                temperature,
            },
        })
    }

    pub fn authorize(&self, header_value: Option<&str>) -> bool {
        match (&self.shared_secret, header_value) {
            (Some(expected), Some(actual)) => expected == actual,
            (None, _) => true,
            _ => false,
        }
    }
}

fn read_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .or_else(|| env::var(format!("EXPO_PUBLIC_{key}")).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, sync::Mutex};

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn default_ports_are_updated() {
        let _guard = ENV_MUTEX.lock().unwrap();
        env::remove_var("AGENT_HUB_HTTP_PORT");
        env::remove_var("EXPO_PUBLIC_AGENT_HUB_HTTP_PORT");
        env::remove_var("AGENT_HUB_WS_PORT");
        env::remove_var("EXPO_PUBLIC_AGENT_HUB_WS_PORT");

        let config = ServerConfig::from_env().expect("config loads with defaults");

        assert_eq!(config.http_port, 4110);
        assert_eq!(config.ws_port, 4111);
    }
}
