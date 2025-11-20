import { SERVER_BASE_URL } from "../../../lib/serverConfig";

type Session = {
  session_id: string;
  project_name: string;
  project_root: string;
  created_at: string;
  last_active_at: string;
  status: string;
};

type SessionsResponse = {
  sessions?: Session[];
  error?: string;
};

async function loadSessions(): Promise<Session[] | null> {
  try {
    const res = await fetch(`${SERVER_BASE_URL}/project-sessions`, {
      cache: "no-store",
    });
    if (!res.ok) {
      throw new Error(`Failed to fetch sessions: ${res.status}`);
    }
    const data = (await res.json()) as SessionsResponse;
    return data.sessions ?? [];
  } catch (err) {
    console.error(err);
    return null;
  }
}

export default async function SessionDetailsPage({
  params,
}: {
  params: { sessionId: string };
}) {
  const sessions = await loadSessions();

  if (!sessions) {
    return <p>Unable to load sessions. Please try again.</p>;
  }

  const session = sessions.find((s) => s.session_id === params.sessionId);

  if (!session) {
    return (
      <p>
        Session not found. Try starting a new one from the project list.
      </p>
    );
  }

  return (
    <article style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      <div>
        <strong>Session ID:</strong> {session.session_id}
      </div>
      <div>
        <strong>Project:</strong> {session.project_name}
      </div>
      <div>
        <strong>Project Root:</strong> {session.project_root}
      </div>
      <div>
        <strong>Created:</strong> {new Date(session.created_at).toLocaleString()}
      </div>
      <div>
        <strong>Last Active:</strong>{" "}
        {new Date(session.last_active_at).toLocaleString()}
      </div>
      <div>
        <strong>Status:</strong> {session.status}
      </div>
    </article>
  );
}
