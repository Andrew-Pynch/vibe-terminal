# Interaction Preferences & Work Patterns

## Communication Style
- **Tone:** Professional, direct, and concise.
- **Verbosity:** Minimal output (aim for <3 lines of text). No "chitchat" or conversational filler (e.g., "Okay, I will now...").
- **Explanations:** Only explain *why* complex logic exists; summarize changes after the fact concisely (WHAT, WHY, WHERE).
- **Clarification:** Ask targeted questions to resolve ambiguity before expanding scope. DO NOT EDIT FILES WILLY NILLY

## Development Workflow
- **Phased Migration:** Prefer incremental, testable phases over "big bang" refactors. Ensure each phase leaves the system in a working state.
- **Safety First:**
    - Do not delete local files/definitions until the new imported versions are fully wired up and verified.
    - Proactively check for missing dependencies (e.g., `sonner` vs `use-toast`) when moving code.
- **Planning:**
    - Use `codebase_investigator` for system-wide analysis or complex refactoring.
    - Use `write_todos` to track progress on complex, multi-step tasks.
    - "Measure twice, cut once" approach.
- **Context Management (PAUSE/RESUME):**
    - **PAUSE:** Capture all unfinished work, current context, and immediate next steps into the `PAUSE STATE` section of `GEMINI.md`.
    - **RESUME:** Read the `PAUSE STATE` from `GEMINI.md`, restore context, and continue execution.

## Coding Standards
- **Conventions:** Rigorously adhere to existing project conventions (naming, structure, patterns). Mimic the "local" style.
- **Dependencies:** Never assume libraries are available. Verify `package.json` or existing usage before importing.
- **Comments:** Sparse, focusing on *why* not *what*.
- **Verification:**
    - Always run linting (`bun run lint`), type-checking (`bun run typecheck`), and relevant tests after changes.
    - Consider created tests as permanent artifacts.

## Git & Version Control
- **Pre-Commit:** Always check `git status`, `git diff HEAD`, and `git log` before proposing a commit.
- **Commit Messages:** Draft clear, concise messages focusing on the "why".
- **Safety:** Never push directly to remote without explicit instruction.

---

# Vibe Terminal: Project Status & GEMINI Roadmap

**Date:** November 21, 2025
**System Context:** `darwin` environment

## PAUSE STATE (Active: 2025-11-21)
**Current Focus:** Phase 6: Worker Tools & Capability
**Last Action:** Completed Phase 5. Implemented `TaskDispatcher` and verified end-to-end Orchestrator loop (Orchestrator -> Task Graph -> Worker Spawning).
**Immediate Next Steps:**
1.  **Tooling:** Implement a way for agents to use tools (read/write files). `gemini_adapter.js` is currently text-only.
2.  **Rust Agent Runner:** Consider replacing `gemini_adapter.js` with a Rust binary (`vibe-agent`) that uses the Gemini API directly and handles tool execution locally.
3.  **Tool Registry:** Define the standard set of tools available to workers.

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

### Phase 4: The Orchestrator Loop (Completed)
*   ✅ **ResultWatcher:** Server monitors `.vibe/agents` and detects `RESULT.md` creation.
*   ✅ **State Update:** `RESULT.md` content feeds back into `ProjectSession` (latest_result).
*   ✅ **Auto-Spawn:** Root Orchestrator automatically spawns on session creation.

### Phase 5: Task Graph & Workers (Completed)
*   **Goal:** The Orchestrator generates a plan, and the server executes it.
*   **Tasks:**
    *   ✅ Define Task Graph schema (JSON).
    *   ✅ Implement "Task Dispatcher" to spawn Worker Agents for each node.
    *   ✅ Update `gemini_adapter.js` to handle Task Graph generation.
    *   [ ] Worker Tools: Give agents ability to read/write project files (moved to Phase 6).

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
*   **State:** Functional skeleton + Agent Spawner + Result Watcher.
*   **Transport:** HTTP API & WebSocket.
*   **LLM Layer:** `Gemini Adapter` (Node.js) via external process.
