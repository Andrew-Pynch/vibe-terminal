# Vibe Terminal: Project Status & GEMINI Roadmap

**Date:** November 20, 2025
**System Context:** `darwin` environment

## Important Development Directives:
*   Always present plans for approval before raw implementation.
*   Break down tasks into manageable, testable chunks.

## 1. High-Level Concept
**vibe-terminal** is a "project mind" server designed to orchestrate software development. It sits between human intent (goals, vibes) and worker agents (AI models modifying code).

*   **Core Philosophy:** Orchestrator vs. Worker. The server plans; agents execute.
*   **Mechanism:** Structured "Project Sessions" tracking PRDs, task graphs, and file boundaries.
*   **Modes:** BOOT (read-only), ORCHESTRATOR (planning), WORKER (execution), DOC_SCRIBE (documentation).

## 2. Strategic Pivot: Gemini First
**Decision:** All AI components (Orchestrator and Workers) will default to **Gemini** models.
*   **Orchestrator:** Gemini Flash/Pro for high-speed context understanding and planning.
*   **Workers:** Gemini models for code generation and task execution.
*   **Goal:** Fully commit to a Gemini-powered stack for the next month of development.

## 3. Current Implementation Status

### Backend (`server/`)
*   **Language:** Rust.
*   **State:** Functional skeleton.
*   **Transport:**
    *   HTTP API: Health, Session CRUD.
    *   WebSocket: `ws://.../ws/project-session/:id` (Echo implemented), `ws://.../ws/:id` (Chat streaming implemented).
*   **Session Models:**
    *   `ProjectSession`: In-memory store, basic metadata.
    *   `Session` (Chat): In-memory, event-driven pub/sub for streaming.
*   **LLM Layer:** `LlmClient` trait exists. Only `DummyClient` is implemented. **No real AI integration yet.**

### Frontend & Protocol
*   **`packages/protocol`:** Shared TypeScript definitions for all WS messages and config.
*   **Clients:**
    *   `apps/cli`: Basic setup.
    *   `apps/vibe-web`: Next.js dashboard (skeleton).
    *   `apps/mobile`: Expo app (skeleton).

### Observations
*   **Code Quality:** Clean separation of concerns in Rust (`ws.rs`, `sessions.rs`, `llm/`).
*   **Architecture:** Monorepo structure (Turborepo) is well-configured.
*   **Missing:** Real persistence (currently in-memory), real LLM providers, orchestration logic.

## 4. Roadmap: The Gemini Era

### High-Level Plan
The detailed high-level plan for implementing the "Hypervisor & Agents" architecture can be found in `TODOS/high-level-plan.md`.

### Next Actions (Current Focus)
The immediate next step is to implement a core utility function for spawning and capturing the output of external processes, which is foundational for managing agent CLIs.
