{
  "tasks": [
    { "id": "phase6-1", "description": "Create vibe-report CLI shim (wrapper around curl to Server).", "agent_type": "worker" },
    { "id": "phase6-2", "description": "Create vibe-complete CLI shim (wrapper around curl to Server).", "agent_type": "worker" },
    { "id": "phase6-3", "description": "Update Rust Server API to handle status reports from the CLI shims.", "agent_type": "worker" },
    { "id": "phase6-4", "description": "Refactor TaskDispatcher to spawn gemini CLI with dynamic prompts.", "agent_type": "worker" },
    { "id": "phase6-5", "description": "Verify end-to-end 'Rambo' mode (native gemini CLI workers with MCP-style signaling).", "agent_type": "worker" }
  ]
}