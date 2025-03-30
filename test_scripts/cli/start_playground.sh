#!/bin/bash

# Kill any existing process on port 8080
lsof -ti:8080 | xargs kill -9 2>/dev/null || true

# Start the playground server
cargo run -p lode-playground &

# Wait for the server to start
sleep 2

# Check if the server is running
if curl -s http://localhost:8080/hello > /dev/null; then
    echo "Playground server started successfully on http://localhost:8080"
else
    echo "Failed to start playground server"
    exit 1
fi 