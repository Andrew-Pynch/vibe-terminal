#!/bin/bash
# dev-tools/test_ask.sh

SERVER_URL=${VIBE_SERVER_URL:-http://localhost:4110}

if [ "$1" == "list" ]; then
    echo "Fetching pending interactions..."
    curl -s "$SERVER_URL/agent/interactions/pending" | jq .
elif [ "$1" == "reply" ]; then
    if [ -z "$2" ] || [ -z "$3" ]; then
        echo "Usage: $0 reply <interaction_id> <answer>"
        exit 1
    fi
    ID=$2
    ANSWER=$3
    echo "Replying to $ID with '$ANSWER'..."
    curl -X POST "$SERVER_URL/interactions/$ID/reply" \
        -H "Content-Type: application/json" \
        -d "{\"answer\": \"$ANSWER\"}"
else
    echo "Usage: $0 [list | reply <id> <answer>]"
fi

