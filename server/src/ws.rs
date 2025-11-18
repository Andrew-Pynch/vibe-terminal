use std::sync::Arc;

use axum::{
  extract::{
    ws::{Message, WebSocket, WebSocketUpgrade},
    State,
  },
  http::{HeaderMap, StatusCode},
  response::IntoResponse,
  routing::get,
  Router,
};
use futures::{stream::SplitSink, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
  llm::{LlmMessage, LlmRequest, MessageRole},
  sessions::WsEvent,
  state::AppState,
};

pub fn router(state: AppState) -> Router {
  Router::new()
    .route("/sessions", get(ws_handler))
    .with_state(state)
}

async fn ws_handler(
  headers: HeaderMap,
  ws: WebSocketUpgrade,
  State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
  if let Some(secret) = &state.config.shared_secret {
    let provided = headers
      .get("x-agent-hub-auth")
      .and_then(|value| value.to_str().ok())
      .map(|value| value.to_string());
    if provided.as_deref() != Some(secret.as_str()) {
      return Err(StatusCode::UNAUTHORIZED);
    }
  }

  Ok(ws.on_upgrade(move |socket| handle_socket(state, socket)))
}

async fn handle_socket(state: AppState, socket: WebSocket) {
  let (sender, mut receiver) = socket.split();
  let mut active_session: Option<String> = None;
  let mut rx_task: Option<tokio::task::JoinHandle<()>> = None;
  let sender = Arc::new(tokio::sync::Mutex::new(sender));

  while let Some(Ok(message)) = receiver.next().await {
    if let Message::Text(text) = message {
      match serde_json::from_str::<ClientWsMessage>(&text) {
        Ok(ClientWsMessage::JoinSession { session_id }) => {
          if !state.sessions.exists(&session_id).await {
            let _ = send_event(&sender, WsEvent::Error {
              code: "session-not-found".into(),
              message: format!("Session {session_id} not found"),
            })
            .await;
            continue;
          }
          active_session = Some(session_id.clone());
          if let Some(task) = rx_task.take() {
            task.abort();
          }
          if let Some(mut subscriber) = state.sessions.subscribe(&session_id).await {
            let sender_clone = sender.clone();
            rx_task = Some(tokio::spawn(async move {
              loop {
                match subscriber.recv().await {
                  Ok(event) => {
                    if send_event(&sender_clone, event).await.is_err() {
                      break;
                    }
                  }
                  Err(broadcast::error::RecvError::Lagged(_)) => continue,
                  Err(_) => break,
                }
              }
            }));
          }
          let _ = send_event(
            &sender,
            WsEvent::SessionJoined {
              session_id: session_id.clone(),
            },
          )
          .await;
          if let Some(summary) = state.sessions.summary(&session_id).await {
            let _ = send_event(&sender, WsEvent::SessionUpdated { session: summary }).await;
          }
        }
        Ok(ClientWsMessage::UserMessage { session_id, content, .. }) => {
          if Some(session_id.clone()) != active_session {
            continue;
          }
          handle_user_message(state.clone(), session_id, content).await;
        }
        Ok(ClientWsMessage::Ping { .. }) => {}
        Err(error) => {
          let _ = send_event(
            &sender,
            WsEvent::Error {
              code: "invalid-payload".into(),
              message: format!("Failed to parse: {error}"),
            },
          )
          .await;
        }
      }
    }
  }

  if let Some(task) = rx_task {
    task.abort();
  }
}

async fn send_event(
  sender: &Arc<tokio::sync::Mutex<SplitSink<WebSocket, Message>>>,
  event: WsEvent,
) -> Result<(), ()> {
  let serialized =
    serde_json::to_string(&event).map_err(|_| ())?;
  let mut guard = sender.lock().await;
  guard.send(Message::Text(serialized)).await.map_err(|_| ())
}

async fn handle_user_message(state: AppState, session_id: String, content: String) {
  if state
    .sessions
    .append_message(&session_id, MessageRole::User, content.clone())
    .await
    .is_none()
  {
    return;
  }
  if let Some(summary) = state.sessions.summary(&session_id).await {
    state
      .sessions
      .publish(&session_id, WsEvent::SessionUpdated { session: summary })
      .await;
  }
  tokio::spawn(run_orchestrator(state, session_id, content));
}

async fn run_orchestrator(state: AppState, session_id: String, user_content: String) {
  let Some(messages) = state.sessions.messages(&session_id).await else {
    return;
  };
  let mut llm_messages: Vec<LlmMessage> = messages
    .into_iter()
    .map(|message| LlmMessage {
      role: message.role,
      content: message.content,
    })
    .collect();
  if llm_messages.is_empty() {
    llm_messages.push(LlmMessage {
      role: MessageRole::System,
      content: format!("Agent Hub orchestrator ready for session {session_id}"),
    });
    llm_messages.push(LlmMessage {
      role: MessageRole::User,
      content: user_content,
    });
  }

  let Some(summary) = state.sessions.summary(&session_id).await else {
    return;
  };

  let request = LlmRequest {
    config: summary.llm_config.clone(),
    messages: llm_messages,
  };
  let message_id = Uuid::new_v4().to_string();
  state
    .sessions
    .publish(
      &session_id,
      WsEvent::AssistantMessageStart {
        session_id: session_id.clone(),
        message_id: message_id.clone(),
      },
    )
    .await;

  match state.llms.stream(request) {
    Ok(mut stream) => {
      let mut buffer = String::new();
      while let Some(chunk) = stream.next().await {
        match chunk {
          Ok(text) => {
            buffer.push_str(&text);
            state
              .sessions
              .publish(
                &session_id,
                WsEvent::AssistantMessageChunk {
                  session_id: session_id.clone(),
                  message_id: message_id.clone(),
                  text_chunk: text,
                },
              )
              .await;
          }
          Err(error) => {
            state
              .sessions
              .publish(
                &session_id,
                WsEvent::Error {
                  code: "llm-error".into(),
                  message: error.to_string(),
                },
              )
              .await;
            break;
          }
        }
      }
      state
        .sessions
        .update_assistant_message(&session_id, &message_id, &buffer)
        .await;
      if let Some(summary) = state.sessions.summary(&session_id).await {
        state
          .sessions
          .publish(&session_id, WsEvent::SessionUpdated { session: summary })
          .await;
      }
      state
        .sessions
        .publish(
          &session_id,
          WsEvent::AssistantMessageComplete {
            session_id,
            message_id,
          },
        )
        .await;
    }
    Err(error) => {
      state
        .sessions
        .publish(
          &session_id,
          WsEvent::Error {
            code: "llm-init-error".into(),
            message: error.to_string(),
          },
        )
        .await;
    }
  }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ClientWsMessage {
  JoinSession { session_id: String },
  UserMessage {
    session_id: String,
    content: String,
    #[serde(default)]
    meta: Option<Value>,
  },
  Ping { timestamp: i64 },
}
