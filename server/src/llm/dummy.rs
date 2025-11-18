use futures::StreamExt;
use tokio_stream::iter;

use super::{LlmClient, LlmMessage, LlmRequest, LlmStream, MessageRole};

pub struct DummyClient;

impl DummyClient {
  pub fn new() -> Self {
    Self
  }
}

impl LlmClient for DummyClient {
  fn stream(&self, request: &LlmRequest) -> LlmStream {
    let fallback = "Agent Hub dummy client ready.".to_string();
    let text = last_user_message(&request.messages)
      .map(|msg| format!("Echo: {}", msg.content))
      .unwrap_or(fallback);

    let chunks = text
      .split_whitespace()
      .map(|token| format!("{token} "))
      .collect::<Vec<_>>();

    Box::pin(iter(chunks).map(Ok))
  }
}

fn last_user_message(messages: &[LlmMessage]) -> Option<&LlmMessage> {
  messages
    .iter()
    .rev()
    .find(|message| matches!(message.role, MessageRole::User))
}
