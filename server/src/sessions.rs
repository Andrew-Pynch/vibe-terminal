use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::llm::{LlmConfig, MessageRole};

#[derive(Clone)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<WsEvent>>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn list(&self) -> Vec<SessionSummary> {
        let store = self.sessions.read().await;
        store.values().map(Session::summary).collect()
    }

    pub async fn detail(&self, id: &str) -> Option<SessionDetail> {
        let store = self.sessions.read().await;
        store.get(id).map(Session::detail)
    }

    pub async fn create(&self, params: SessionCreateParams) -> SessionDetail {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();
        let session = Session {
            id: id.clone(),
            name: params.name,
            profile: params.profile,
            created_at: now,
            updated_at: now,
            llm_config: params.llm_config,
            messages: vec![],
            meta: params.meta.unwrap_or(Value::Object(Default::default())),
        };
        let detail = session.detail();
        sessions.insert(id.clone(), session);
        drop(sessions);
        self.ensure_channel(&id).await;
        detail
    }

    pub async fn delete(&self, id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        let removed = sessions.remove(id).is_some();
        drop(sessions);
        if removed {
            let mut channels = self.channels.write().await;
            channels.remove(id);
        }
        removed
    }

    pub async fn exists(&self, id: &str) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(id)
    }

    pub async fn append_message(
        &self,
        session_id: &str,
        role: MessageRole,
        content: String,
    ) -> Option<SessionMessage> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)?;
        let message = SessionMessage {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            timestamp: Utc::now(),
            meta: Value::Null,
        };
        session.messages.push(message.clone());
        session.updated_at = Utc::now();
        Some(message)
    }

    pub async fn update_assistant_message(
        &self,
        session_id: &str,
        message_id: &str,
        content: &str,
    ) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(message) = session
                .messages
                .iter_mut()
                .find(|m| m.id == message_id && matches!(m.role, MessageRole::Assistant))
            {
                message.content = content.to_string();
                session.updated_at = Utc::now();
            } else {
                session.messages.push(SessionMessage {
                    id: message_id.to_string(),
                    role: MessageRole::Assistant,
                    content: content.to_string(),
                    timestamp: Utc::now(),
                    meta: Value::Null,
                });
            }
        }
    }

    pub async fn ensure_channel(&self, session_id: &str) -> broadcast::Sender<WsEvent> {
        let mut map = self.channels.write().await;
        map.entry(session_id.to_string())
            .or_insert_with(|| {
                let (tx, _rx) = broadcast::channel(128);
                tx
            })
            .clone()
    }

    pub async fn subscribe(&self, session_id: &str) -> Option<broadcast::Receiver<WsEvent>> {
        let sender = {
            let mut channels = self.channels.write().await;
            channels
                .entry(session_id.to_string())
                .or_insert_with(|| {
                    let (tx, _rx) = broadcast::channel(128);
                    tx
                })
                .clone()
        };
        Some(sender.subscribe())
    }

    pub async fn publish(&self, session_id: &str, event: WsEvent) {
        let sender = {
            let channels = self.channels.read().await;
            channels.get(session_id).cloned()
        };
        if let Some(sender) = sender {
            let _ = sender.send(event);
        }
    }

    pub async fn summary(&self, session_id: &str) -> Option<SessionSummary> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(Session::summary)
    }

    pub async fn messages(&self, session_id: &str) -> Option<Vec<SessionMessage>> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|session| session.messages.clone())
    }
}

#[derive(Clone)]
pub struct SessionCreateParams {
    pub name: String,
    pub profile: String,
    pub llm_config: LlmConfig,
    pub meta: Option<Value>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub name: String,
    pub profile: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub llm_config: LlmConfig,
    pub meta: Value,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub id: String,
    pub name: String,
    pub profile: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub llm_config: LlmConfig,
    pub meta: Value,
    pub messages: Vec<SessionMessage>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub meta: Value,
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WsEvent {
    SessionJoined {
        session_id: String,
    },
    AssistantMessageStart {
        session_id: String,
        message_id: String,
    },
    AssistantMessageChunk {
        session_id: String,
        message_id: String,
        text_chunk: String,
    },
    AssistantMessageComplete {
        session_id: String,
        message_id: String,
    },
    SessionUpdated {
        session: SessionSummary,
    },
    Error {
        code: String,
        message: String,
    },
}

impl Session {
    fn summary(&self) -> SessionSummary {
        SessionSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            profile: self.profile.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            llm_config: self.llm_config.clone(),
            meta: self.meta.clone(),
        }
    }

    fn detail(&self) -> SessionDetail {
        SessionDetail {
            id: self.id.clone(),
            name: self.name.clone(),
            profile: self.profile.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            llm_config: self.llm_config.clone(),
            meta: self.meta.clone(),
            messages: self.messages.clone(),
        }
    }
}

#[derive(Clone)]
struct Session {
    pub id: String,
    pub name: String,
    pub profile: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub llm_config: LlmConfig,
    pub messages: Vec<SessionMessage>,
    pub meta: Value,
}
