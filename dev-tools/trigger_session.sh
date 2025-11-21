#!/bin/bash
# dev-tools/trigger_session.sh

# Default to current directory as project root if not provided
PROJECT_ROOT=${1:-$(pwd)}

echo "Triggering new session for project root: $PROJECT_ROOT"

curl -X POST http://localhost:4110/project-sessions \
  -H "Content-Type: application/json" \
  -d "{\"project_root\": \"$PROJECT_ROOT\"}"

echo -e "\nRequest sent."
