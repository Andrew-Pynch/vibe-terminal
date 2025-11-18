import { useCallback, useEffect, useMemo, useState } from "react";
import {
  ActivityIndicator,
  FlatList,
  RefreshControl,
  StyleSheet,
  Text,
  TouchableOpacity,
  View
} from "react-native";
import { useRouter } from "expo-router";
import { SessionSummary, SessionListResponse } from "@agent-hub/protocol";
import { AgentHubConfig, resolveConfig } from "@agent-hub/config";

function makeHttpUrl(config: AgentHubConfig, path: string) {
  return `${config.protocol}://${config.host}:${config.httpPort}${path}`;
}

export default function SessionListScreen() {
  const config = useMemo(() => resolveConfig(), []);
  const router = useRouter();
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  const fetchSessions = useCallback(async () => {
    setError(null);
    try {
      const headers: Record<string, string> = {};
      if (config.sharedSecret) {
        headers["X-Agent-Hub-Auth"] = config.sharedSecret;
      }
      const response = await fetch(makeHttpUrl(config, "/sessions"), { headers });
      if (!response.ok) {
        throw new Error(`Server responded with ${response.status}`);
      }
      const json = (await response.json()) as SessionListResponse;
      setSessions(json.sessions);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [config]);

  useEffect(() => {
    fetchSessions();
  }, [fetchSessions]);

  const onRefresh = useCallback(() => {
    setRefreshing(true);
    fetchSessions();
  }, [fetchSessions]);

  if (loading) {
    return (
      <View style={styles.centered}>
        <ActivityIndicator />
        <Text style={styles.subtle}>Connecting to Agent Hub...</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      {error ? <Text style={styles.error}>{error}</Text> : null}
      <FlatList
        data={sessions}
        keyExtractor={(item) => item.id}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
        contentContainerStyle={sessions.length === 0 && styles.centered}
        ListEmptyComponent={
          <Text style={styles.subtle}>No sessions yet. Create one via the CLI.</Text>
        }
        renderItem={({ item }) => (
          <TouchableOpacity
            style={styles.card}
            onPress={() => router.push(`/${item.id}`)}
          >
            <Text style={styles.cardTitle}>{item.name}</Text>
            <Text style={styles.cardSubtitle}>
              {item.profile} • {item.llmConfig.provider}
            </Text>
          </TouchableOpacity>
        )}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#0f172a",
    padding: 16
  },
  centered: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
    gap: 8
  },
  subtle: {
    color: "#94a3b8"
  },
  card: {
    backgroundColor: "#1e293b",
    padding: 16,
    borderRadius: 12,
    marginBottom: 12
  },
  cardTitle: {
    fontSize: 18,
    fontWeight: "600",
    color: "#f8fafc"
  },
  cardSubtitle: {
    color: "#cbd5f5",
    marginTop: 4
  },
  error: {
    color: "#f87171",
    marginBottom: 8
  }
});
