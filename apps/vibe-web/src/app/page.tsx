"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { SERVER_BASE_URL } from "../lib/serverConfig";

type Project = {
  project_root: string;
  project_name: string;
  last_seen: string;
};

type SessionResponse = {
  session?: {
    session_id: string;
  };
  error?: string;
};

type ProjectsResponse = {
  projects?: Project[];
};

export default function ProjectsPage() {
  const router = useRouter();
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [startingSessions, setStartingSessions] = useState<Record<string, boolean>>({});

  useEffect(() => {
    let cancelled = false;

    async function loadProjects() {
      try {
        const res = await fetch(`${SERVER_BASE_URL}/projects`, {
          cache: "no-store",
        });
        if (!res.ok) {
          throw new Error(`Failed to load projects: ${res.status}`);
        }
        const data = (await res.json()) as ProjectsResponse;
        if (!cancelled) {
          setProjects(data.projects ?? []);
          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Unknown error");
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    loadProjects();

    return () => {
      cancelled = true;
    };
  }, []);

  async function handleStartSession(project_root: string) {
    setStartingSessions((prev) => ({ ...prev, [project_root]: true }));
    try {
      const res = await fetch(`${SERVER_BASE_URL}/project-sessions`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ project_root }),
      });
      const data = (await res.json().catch(() => ({}))) as SessionResponse;
      if (!res.ok) {
        throw new Error(data.error ?? `Failed to start session (${res.status})`);
      }
      const sessionId = data.session?.session_id;
      if (!sessionId) {
        throw new Error("Session ID missing in response");
      }
      router.push(`/sessions/${sessionId}`);
    } catch (err) {
      alert(err instanceof Error ? err.message : "Failed to start session");
    } finally {
      setStartingSessions((prev) => {
        const next = { ...prev };
        delete next[project_root];
        return next;
      });
    }
  }

  if (loading) {
    return <p>Loading projects...</p>;
  }

  if (error) {
    return <p style={{ color: "red" }}>Failed to load projects: {error}</p>;
  }

  if (projects.length === 0) {
    return <p>No projects available. Start the Rust server to register projects.</p>;
  }

  return (
    <section style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
      {projects.map((project) => {
        const isStarting = Boolean(startingSessions[project.project_root]);
        return (
          <article
            key={project.project_root}
            style={{
              border: "1px solid #ddd",
              borderRadius: 8,
              padding: 16,
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
            }}
          >
            <div>
              <div style={{ fontWeight: 600 }}>{project.project_name}</div>
              <div style={{ fontSize: 14, color: "#555" }}>{project.project_root}</div>
              {project.last_seen ? (
                <div style={{ fontSize: 12, color: "#777" }}>
                  Last seen: {new Date(project.last_seen).toLocaleString()}
                </div>
              ) : null}
            </div>
            <button onClick={() => handleStartSession(project.project_root)} disabled={isStarting}>
              {isStarting ? "Starting..." : "Start Session"}
            </button>
          </article>
        );
      })}
    </section>
  );
}
