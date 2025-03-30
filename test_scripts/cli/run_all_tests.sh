#!/bin/bash

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Make scripts executable
chmod +x "$SCRIPT_DIR/start_playground.sh" "$SCRIPT_DIR/run_load_tests.sh"

# Start the playground server
"$SCRIPT_DIR/start_playground.sh"

# Wait a bit for the server to be fully ready
sleep 3

# Run the load tests
"$SCRIPT_DIR/run_load_tests.sh"

# Kill the playground server
lsof -ti:8080 | xargs kill -9 2>/dev/null || true

echo "All tests completed. Check the output directory for results." 