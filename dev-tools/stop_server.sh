#!/bin/bash
# dev-tools/stop_server.sh

if [ -f server.pid ]; then
    PID=$(cat server.pid)
    if ps -p $PID > /dev/null; then
        echo "Killing server process $PID..."
        kill $PID
        echo "Server stopped."
    else
        echo "Process $PID not found."
    fi
    rm server.pid
else
    echo "No server.pid file found."
fi
