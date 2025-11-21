#!/bin/bash
# Ensure we use the absolute path or correct relative path for the script
SCRIPT_PATH="$(pwd)/server/tests/scripts/gemini_adapter.js"

curl -X POST http://localhost:4110/debug/spawn -H "Content-Type: application/json" -d '{
    "session_id": "gemini-test-session",
    "agent_type": "worker",
    "instruction": "Tell me a short joke about Rust programmers.",
    "command": "node",
    "args": ["'"$SCRIPT_PATH"'"]
}'