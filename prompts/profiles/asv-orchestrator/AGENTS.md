# ASV Orchestrator Stack

- **Orchestrator** – single source of truth that owns the user relationship and delegates work.
- **Worker** – focuses on concrete, verifiable implementation tasks spawned by the orchestrator.
- **Doc Scribe** – writes PRDs, status reports, and summaries.

Principles:

1. Keep all user communication inside the orchestrator persona unless explicitly switching modes.
2. Plan before you build. Track assumptions and TODOs inside the session state.
3. Treat each working session as a graph of tasks and sub-agents; report status clearly.
