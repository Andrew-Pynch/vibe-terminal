import { useLocalSearchParams } from "expo-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ActivityIndicator,
  FlatList,
  StyleSheet,
  Text,
  TextInput,
  TouchableOpacity,
  View
} from "react-native";
import {
  ServerToClientWsMessage,
  SessionDetailResponse
} from "@agent-hub/protocol";
import { AgentHubConfig, resolveConfig } from "@agent-hub/config";

type ChatMessage = {
  id: string;
  role: string;
  text: string;
};

function makeHttpUrl(config: AgentHubConfig, path: string) {
  return `${config.protocol}://${config.host}:${config.httpPort}${path}`;
}

function makeWsUrl(config: AgentHubConfig, path: string) {
  const protocol = config.protocol === "https" ? "wss" : "ws";
  return `${protocol}://${config.host}:${config.wsPort}${path}`;
}

export default function SessionDetailScreen() {
  const params = useLocalSearchParams();
  const sessionId = params.sessionId as string;
  const config = useMemo(() => resolveConfig(), []);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  const handleWsMessage = useCallback((message: ServerToClientWsMessage) => {
    switch (message.type) {
      case "AssistantMessageStart":
        setMessages((prev) => [
          ...prev,
          { id: message.messageId, role: "assistant", text: "" }
        ]);
        break;
      case "AssistantMessageChunk":
        setMessages((prev) =>
          prev.map((msg) =>
            msg.id === message.messageId
              ? { ...msg, text: msg.text + message.textChunk }
              : msg
          )
        );
        break;
      case "AssistantMessageComplete":
        break;
      default:
        break;
    }
  }, []);

  useEffect(() => {
    if (!sessionId) {
      return;
    }
    const headers: Record<string, string> = {};
    if (config.sharedSecret) {
      headers["X-Agent-Hub-Auth"] = config.sharedSecret;
    }
    fetch(makeHttpUrl(config, `/sessions/${sessionId}`), { headers })
      .then((res) => res.json() as Promise<SessionDetailResponse>)
      .then((json) => {
        setMessages(
          json.session.messages.map((msg) => ({
            id: msg.id,
            role: msg.role,
            text: msg.content
          }))
        );
      })
      .catch((error) => {
        setStatus(error.message);
      })
      .finally(() => setLoading(false));
  }, [config, sessionId]);

  useEffect(() => {
    if (!sessionId) {
      return;
    }
    const headers: Record<string, string> = {};
    if (config.sharedSecret) {
      headers["X-Agent-Hub-Auth"] = config.sharedSecret;
    }
    const ws = new WebSocket(makeWsUrl(config, "/sessions"), undefined, {
      headers
    });
    wsRef.current = ws;
    ws.onopen = () => {
      setStatus("Connected");
      ws.send(JSON.stringify({ type: "JoinSession", sessionId }));
    };
    ws.onmessage = (event) => {
      const payload = JSON.parse(event.data as string) as ServerToClientWsMessage;
      handleWsMessage(payload);
    };
    ws.onerror = () => setStatus("Connection error");
    ws.onclose = () => setStatus("Disconnected");
    return () => {
      wsRef.current = null;
      ws.close();
    };
  }, [config, handleWsMessage, sessionId]);

  const sendMessage = () => {
    if (!input.trim()) {
      return;
    }
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) {
      setStatus("Not connected to session");
      return;
    }
    wsRef.current.send(
      JSON.stringify({
        type: "UserMessage",
        sessionId,
        content: input.trim()
      })
    );
    setMessages((prev) => [
      ...prev,
      { id: `local-${Date.now()}`, role: "user", text: input.trim() }
    ]);
    setInput("");
  };

  if (loading) {
    return (
      <View style={styles.centered}>
        <ActivityIndicator />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <FlatList
        data={messages}
        keyExtractor={(item) => item.id}
        renderItem={({ item }) => (
          <View
            style={[
              styles.bubble,
              item.role === "user" ? styles.userBubble : styles.assistantBubble
            ]}
          >
            <Text style={styles.bubbleText}>{item.text}</Text>
          </View>
        )}
      />
      {status ? <Text style={styles.status}>{status}</Text> : null}
      <View style={styles.inputRow}>
        <TextInput
          value={input}
          onChangeText={setInput}
          placeholder="Send a message"
          placeholderTextColor="#94a3b8"
          style={styles.input}
        />
        <TouchableOpacity style={styles.sendButton} onPress={sendMessage}>
          <Text style={styles.sendButtonText}>Send</Text>
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#0f172a",
    padding: 12
  },
  centered: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center"
  },
  bubble: {
    marginBottom: 8,
    padding: 12,
    borderRadius: 12,
    maxWidth: "80%"
  },
  userBubble: {
    alignSelf: "flex-end",
    backgroundColor: "#2563eb"
  },
  assistantBubble: {
    alignSelf: "flex-start",
    backgroundColor: "#1e293b"
  },
  bubbleText: {
    color: "#f8fafc"
  },
  inputRow: {
    flexDirection: "row",
    alignItems: "center",
    gap: 8,
    marginTop: 8
  },
  input: {
    flex: 1,
    backgroundColor: "#1e293b",
    borderRadius: 10,
    paddingHorizontal: 12,
    paddingVertical: 10,
    color: "#f8fafc"
  },
  sendButton: {
    backgroundColor: "#22c55e",
    borderRadius: 10,
    paddingHorizontal: 16,
    paddingVertical: 10
  },
  sendButtonText: {
    color: "#0f172a",
    fontWeight: "600"
  },
  status: {
    textAlign: "center",
    color: "#94a3b8",
    marginBottom: 8
  }
});
