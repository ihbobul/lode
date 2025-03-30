# API Request Examples

This directory contains example JSON request files that can be used with the Lode API. Each file corresponds to a
different test scenario from the CLI test commands.

## Usage

To try these requests, send a POST request to the `/load-test` endpoint against `lode-playground` dummy server with the
contents of the desired JSON file. For example:

## Available Requests

1. `200_basic_api_test.json` - Basic API endpoint test
2. `200_auth_valid_token.json` - Authentication test with valid token
3. `200_post_small_payload.json` - POST request with small payload
4. `200_mixed_load_test.json` - Mixed load test with multiple headers and body
5. `401_auth_invalid_token.json` - Authentication test with invalid token
6. `404_error_404.json` - 404 Not Found error test
7. `429_error_429.json` - 429 Too Many Requests error test
8. `503_error_503.json` - 503 Service Unavailable error test

Each request file follows the same structure:

- `url`: Target URL for the load test
- `method`: HTTP method (GET, POST, etc.)
- `requests`: Number of requests to make
- `concurrency`: Number of concurrent requests
- `timeout_ms`: Request timeout in milliseconds
- `headers`: Optional HTTP headers
- `body`: Optional request body 