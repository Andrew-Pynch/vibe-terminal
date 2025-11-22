```json
{
  "tasks": [
    { "id": "shim-1", "description": "Create vibe-report CLI shim", "agent_type": "worker" },
    { "id": "shim-2", "description": "Create vibe-complete CLI shim", "agent_type": "worker" },
    { "id": "api-1", "description": "Update Rust Server API to receive shim signals", "agent_type": "worker" },
    { "id": "refactor-1", "description": "Refactor TaskDispatcher to spawn gemini CLI with dynamic prompts", "agent_type": "worker" },
    { "id": "verify-1", "description": "Verify end-to-end \"Rambo\" mode", "agent_type": "worker" }
  ]
}
```