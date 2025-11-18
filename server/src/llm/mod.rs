use std::{pin::Pin, str::FromStr, sync::Arc};

use futures::Stream;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod dummy;

pub type LlmStream = Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
  OpenAi,
  Anthropic,
  Dummy,
}

impl FromStr for ProviderKind {
  type Err = LlmError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "openai" => Ok(ProviderKind::OpenAi),
      "anthropic" => Ok(ProviderKind::Anthropic),
      "dummy" => Ok(ProviderKind::Dummy),
      other => Err(LlmError::UnknownProvider(other.to_string())),
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
  pub provider: ProviderKind,
  pub model: String,
  #[serde(default = "default_temperature")]
  pub temperature: f32,
}

fn default_temperature() -> f32 {
  0.2
}

#[derive(Clone, Debug)]
pub struct LlmMessage {
  pub role: MessageRole,
  pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
  System,
  User,
  Assistant,
}

#[derive(Clone, Debug)]
pub struct LlmRequest {
  pub config: LlmConfig,
  pub messages: Vec<LlmMessage>,
}

pub trait LlmClient: Send + Sync {
  fn stream(&self, request: &LlmRequest) -> LlmStream;
}

#[derive(Debug, Error)]
pub enum LlmError {
  #[error("provider not configured")]
  ProviderNotConfigured,
  #[error("unknown provider {0}")]
  UnknownProvider(String),
  #[error("stream failure: {0}")]
  StreamFailure(String),
}

#[derive(Clone)]
pub struct LlmRegistry {
  dummy: Arc<dummy::DummyClient>,
}

impl LlmRegistry {
  pub fn new() -> Self {
    Self {
      dummy: Arc::new(dummy::DummyClient::new()),
    }
  }

  pub fn stream(&self, request: LlmRequest) -> Result<LlmStream, LlmError> {
    match request.config.provider {
      ProviderKind::Dummy => Ok(self.dummy.stream(&request)),
      ProviderKind::OpenAi | ProviderKind::Anthropic => Err(LlmError::ProviderNotConfigured),
    }
  }
}
