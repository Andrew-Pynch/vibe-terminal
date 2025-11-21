# High-Level Plan: Hypervisor & Agents Architecture

This document outlines the high-level architecture for integrating external AI agents (like `gemini-cli`) into the Vibe Terminal server. The core idea is to transform the Rust server into an "Hypervisor" that orchestrates and monitors these agents, rather than directly implementing LLM logic.

## 1. Core Principles

*   **Decoupling:** The Rust server manages agent lifecycles and communication; agents perform the "thinking" and specific tasks.
*   **Leverage Existing Tools:** Utilize the capabilities of robust CLI agents (e.g., `gemini-cli` with its built-in tool use, context management, and MCP support) to avoid re-implementing complex agentic behaviors.
*   **File-Based Communication (Initial):** Agents primarily communicate with the Hypervisor and each other via structured files within dedicated directories. This provides a clear, persistent audit trail and simplifies inter-process communication.
*   **MCP for Advanced Communication (Future):** Implement the Model Context Protocol (MCP) to enable more structured, real-time communication and tool calling between the Hypervisor (as an MCP server) and agents (as MCP clients).

## 2. Architectural Components

### 2.1. Rust Server (The Hypervisor)

*   **Process Manager:** Responsible for spawning, monitoring, and terminating external agent processes.
*   **Agent Registry:** Maintains state and metadata for all active agents (PID, assigned session, current status).
*   **Env Var Injection:** Securely passes secrets (e.g., `GEMINI_API_KEY`) to agents via process environment.
*   **File Watcher:** Monitors agent-specific directories for changes in output files (e.g., logs, results).
*   **Orchestration Logic:** Implements the core Vibe Terminal session flow.

### 2.2. External AI Agents (The Workers)

These are separate processes spawned by the Rust server.

*   **Gemini Adapter (Node.js):**
    *   A lightweight wrapper script that bridges the file-based protocol (`INSTRUCTION.md` -> `RESULT.md`) with the Google Generative AI SDK.
    *   Uses `gemini-2.0-flash` (or configured model) for intelligence.
    *   Allows us to leverage official SDKs without embedding them in the Rust server.

*   **Orchestrator Agent:**
    *   **Input:** Project context (PRDs, docs), current task graph state.
    *   **Task:** Analyze, plan, generate new tasks (`todos/`).
    *   **Output:** Updated task files, summaries, potential questions for the user.
*   **Worker Agent:**
    *   **Input:** Specific task definition, file boundaries (`must_touch`, `may_touch`).
    *   **Task:** Execute a single task (e.g., "Implement feature X," "Fix bug Y").
    *   **Output:** Modified files, logs of actions, diff summaries, documentation notes.
*   **Doc Scibe Agent (Future):**
    *   **Input:** Queued documentation notes.
    *   **Task:** Update project documentation.

## 3. Communication Mechanisms

### 3.1. File-Based Bus

Each agent operates within its own dedicated working directory (`.vibe/agents/<session_id>/<agent_id>/`).

*   **`INSTRUCTION.md` (Hypervisor -> Agent):** The specific goal or command for the agent.
*   **`CONTEXT.md` (Hypervisor -> Agent):** Additional contextual information (e.g., current file content, project overview).
*   **`ACTIVITY_LOG.md` (Agent -> Hypervisor):** A running log of the agent's internal thoughts, actions, and observations.
*   **`RESULT.json` (Agent -> Hypervisor):** Structured output summarizing the agent's work, including status, modified files, and any errors.
*   **`STDIN/STDOUT/STDERR` (Basic Process I/O):** For initial basic communication and debugging, but primarily for passing initial instructions and capturing final structured results.

### 3.2. Model Context Protocol (MCP) (Future)

*   The Rust server will expose an HTTP/WebSocket endpoint acting as an MCP server.
*   Agents will be configured to connect to this MCP server.
*   Agents can then "call tools" exposed by the Hypervisor (e.g., `filesystem.read_file`, `filesystem.write_file_with_constraints`, `user_feedback.ask_question`) directly through MCP, allowing for fine-grained control and structured interactions. This will eventually replace some of the file-based communication for dynamic operations.

## 4. Phased Implementation Plan

This high-level plan will be broken down into smaller, testable chunks in `TODOS/sub-plans/`.

---
_This plan will be iteratively refined based on development and testing._
