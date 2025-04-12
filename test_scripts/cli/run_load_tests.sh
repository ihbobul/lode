#!/bin/bash

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Create output directory if it doesn't exist
mkdir -p "$SCRIPT_DIR/output"

# Function to run a load test and save results
run_test() {
    local name=$1
    local url=$2
    local requests=$3
    local concurrency=$4
    local method=${5:-"GET"}
    local headers=${6:-""}
    local body=${7:-""}
    local output_file="$SCRIPT_DIR/output/${name}_$(date +%Y%m%d_%H%M%S).json"
    
    echo "Running $name test..."
    echo "Method: $method"
    echo "Requests: $requests, Concurrency: $concurrency"
    [ ! -z "$headers" ] && echo "Headers: $headers"
    [ ! -z "$body" ] && echo "Body: $body"
    
    # Build the command with proper escaping
    cmd="cargo run -p lode-cli -- --url \"$url\" --requests $requests --concurrency $concurrency --format json"
    [ ! -z "$method" ] && cmd="$cmd --method $method"
    
    # Handle headers
    if [ ! -z "$headers" ]; then
        # Split headers and properly escape each one
        IFS=' ' read -ra HEADER_ARRAY <<< "$headers"
        for header in "${HEADER_ARRAY[@]}"; do
            if [ ! -z "$header" ]; then
                cmd="$cmd $header"
            fi
        done
    fi
    
    # Handle body with proper escaping
    if [ ! -z "$body" ]; then
        # Escape single quotes in the body
        escaped_body=$(echo "$body" | sed "s/'/'\"'\"'/g")
        cmd="$cmd --body '$escaped_body'"
    fi
    
    echo "Executing command:"
    echo "$cmd"
    echo "---"
    
    eval "$cmd" > "$output_file"
    
    if [ $? -eq 0 ]; then
        echo "Test completed successfully. Results saved to $output_file"
        echo "Summary:"
        grep -E "total_requests|successful_requests|failed_requests|requests_per_second" "$output_file" | sed 's/^[[:space:]]*//'
        echo "---"
    else
        echo "Test failed"
        return 1
    fi
}

echo "Starting load tests..."

# 1. Basic API endpoint test (GET)
run_test "get_data" "http://localhost:8080/api/v1/data" 100 10

# 2. Authentication test
run_test "auth_valid" "http://localhost:8080/api/v1/auth" 50 5 "GET" "-H 'Authorization: Bearer test-token'"
run_test "auth_invalid" "http://localhost:8080/api/v1/auth" 20 5 "GET" "-H 'Authorization: Invalid'"

# 3. Data processing with different payload sizes
small_payload='{"name":"test","email":"test@example.com","data":"small"}'
run_test "post_small" "http://localhost:8080/api/v1/process" 50 5 "POST" "-H 'Content-Type: application/json'" "$small_payload"

large_payload='{"name":"test","email":"test@example.com","data":"'$(printf 'a%.0s' {1..10000})'"}'
run_test "post_large" "http://localhost:8080/api/v1/process" 30 5 "POST" "-H 'Content-Type: application/json'" "$large_payload"

# 4. Error handling scenarios
run_test "error_404" "http://localhost:8080/api/v1/error" 20 5 "GET" "-H 'X-Error-Type: 404'"
run_test "error_429" "http://localhost:8080/api/v1/error" 20 5 "GET" "-H 'X-Error-Type: 429'"
run_test "error_503" "http://localhost:8080/api/v1/error" 20 5 "GET" "-H 'X-Error-Type: 503'"

# 5. Mixed load test (longer duration)
echo "Running mixed load test (60 seconds)..."
mixed_payload='{"name":"test","email":"test@example.com","data":"mixed"}'
run_test "mixed_load" "http://localhost:8080/api/v1/data" 200 20 "POST" "-H 'Content-Type: application/json' -H 'Authorization: Bearer test-token'" "$mixed_payload"

echo "All tests completed." 