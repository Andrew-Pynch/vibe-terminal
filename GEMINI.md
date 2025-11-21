# Vibe Terminal: Project Status & GEMINI Roadmap

**Date:** November 21, 2025
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

## 3. Implementation Roadmap & Status

### Phase 1: Foundation (Completed)
*   ✅ **Process Utility:** `spawn_and_capture_output` for generic process execution.
*   ✅ **Agent Registry:** In-memory tracking of active agent processes (`uuid`, `pid`, `status`).

### Phase 2: The Spawner (Completed)
*   ✅ **AgentSpawner:** Logic to create `.vibe/agents/<session>/<id>/` directories.
*   ✅ **File Bus:** Establishes `INSTRUCTION.md` -> `RESULT.md` protocol.
*   ✅ **Debug API:** `POST /debug/spawn` to manually trigger agents.

### Phase 3: Gemini Integration (Completed)
*   ✅ **Node.js Adapter:** `gemini_adapter.js` bridges the gap between file protocol and Google AI SDK.
*   ✅ **Env Var Injection:** Server securely passes `GEMINI_API_KEY` to agents via `.env`.
*   ✅ **Proof of Life:** Validated end-to-end flow with `gemini-2.0-flash` telling jokes.

### Phase 4: The Orchestrator Loop (Next Up)
*   **Goal:** Close the loop. Server watches for results and auto-spawns agents.
*   **Tasks:**
    *   [ ] Implement `ResultWatcher` (File watcher or polling).
    *   [ ] Feed `RESULT.md` back into `ProjectSession` state.
    *   [ ] Auto-spawn "Root Orchestrator" on session creation.

### Phase 5: Task Graph & Workers (Planned)
*   **Goal:** The Orchestrator generates a plan, and the server executes it.
*   **Tasks:**
    *   [ ] Define Task Graph schema (JSON).
    *   [ ] Implement "Task Dispatcher" to spawn Worker Agents for each node.
    *   [ ] Worker Tools: Give agents ability to read/write project files (not just their sandbox).

## 4. Future Architecture: Initiatives
*   **Concept:** A higher-level abstraction above "Sessions".
*   **Initiative:** Represents a long-running feature map or high-level goal (e.g., "Add User Auth").
*   **Relation:** One Initiative -> Many Project Sessions (Planning, Coding, Testing).
*   **Workflow:**
    1.  User creates an **Initiative** (Mind Map / PRD).
    2.  Orchestrator converts Initiative items into concrete **Project Sessions**.
    3.  Agents execute sessions.

## 5. Current Implementation Status (Backend)
*   **Language:** Rust.
*   **State:** Functional skeleton + Agent Spawner.
*   **Transport:** HTTP API & WebSocket.
*   **LLM Layer:** `Gemini Adapter` (Node.js) via external process.
