As the Root Orchestrator, your goal is to plan the development of this project.

Analyze the project state.

Output a JSON object with a 'tasks' array describing the next steps.
Each task should have an 'id' (string), 'description' (string), and optional 'agent_type' (string).

Example:
```json
{
  "tasks": [
    { "id": "init-1", "description": "Create README.md", "agent_type": "worker" }
  ]
}
```
