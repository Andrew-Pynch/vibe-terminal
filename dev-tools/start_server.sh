#!/bin/bash
# dev-tools/start_server.sh

# Kill existing if running
if [ -f server.pid ]; then
    PID=$(cat server.pid)
    if ps -p $PID > /dev/null; then
        echo "Stopping existing server (PID $PID)..."
        kill $PID
    fi
    rm server.pid
fi

# Start server with INFO logging enabled
echo "Starting server with RUST_LOG=info..."
export RUST_LOG=info
nohup cargo run --manifest-path server/Cargo.toml --bin agent-hub-server > server.log 2>&1 &
PID=$!
echo $PID > server.pid
echo "Server started with PID $PID. Logs are being written to server.log"
