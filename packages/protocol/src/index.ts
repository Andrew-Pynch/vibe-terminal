export type Role = "system" | "user" | "assistant";

export type ProviderKind = "openai" | "anthropic" | "dummy";

export interface LlmConfig {
  provider: ProviderKind;
  model: string;
  temperature?: number;
}

export interface SessionMessage {
  id: string;
  role: Role;
  content: string;
  timestamp: string;
  meta?: Record<string, unknown>;
}

export interface SessionSummary {
  id: string;
  name: string;
  profile: string;
  createdAt: string;
  updatedAt: string;
  llmConfig: LlmConfig;
  meta?: Record<string, unknown>;
}

export interface SessionDetail extends SessionSummary {
  messages: SessionMessage[];
}

export interface SessionListResponse {
  sessions: SessionSummary[];
}

export interface SessionDetailResponse {
  session: SessionDetail;
}

export interface CreateSessionRequest {
  name: string;
  profile: string;
  llmConfig?: Partial<LlmConfig>;
  meta?: Record<string, unknown>;
}

export interface CreateSessionResponse {
  session: SessionDetail;
}

export interface DeleteSessionResponse {
  sessionId: string;
}

export interface ProfileSummary {
  id: string;
  name: string;
  description?: string;
  modes: string[];
}

export interface ProfileListResponse {
  profiles: ProfileSummary[];
}

export interface TokenChunk {
  messageId: string;
  text: string;
  index: number;
  final: boolean;
}

export type ClientToServerWsMessage =
  | {
      type: "JoinSession";
      sessionId: string;
    }
  | {
      type: "UserMessage";
      sessionId: string;
      content: string;
      meta?: Record<string, unknown>;
    }
  | {
      type: "Ping";
      timestamp: number;
    };

export type ServerToClientWsMessage =
  | {
      type: "SessionJoined";
      sessionId: string;
    }
  | {
      type: "AssistantMessageStart";
      messageId: string;
      sessionId: string;
    }
  | {
      type: "AssistantMessageChunk";
      messageId: string;
      sessionId: string;
      textChunk: string;
    }
  | {
      type: "AssistantMessageComplete";
      messageId: string;
      sessionId: string;
    }
  | {
      type: "SessionUpdated";
      session: SessionSummary;
    }
  | {
      type: "Error";
      code: string;
      message: string;
    };
