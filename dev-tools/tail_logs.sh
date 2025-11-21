#!/bin/bash
# dev-tools/tail_logs.sh

if [ ! -f server.log ]; then
    echo "server.log not found. Has the server been started?"
    exit 1
fi

tail -f server.log
