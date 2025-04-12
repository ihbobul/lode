# Lode CLI

A command-line interface for the Lode load testing tool, providing a simple and efficient way to perform load tests on
HTTP endpoints.

## Features

- Simple command-line interface for load testing
- Support for various HTTP methods (GET, POST, PUT, etc.)
- Configurable concurrency (defaults to number of CPU cores)
- Custom headers support
- JSON request body support
- Multiple output formats (text/json)
- Request timeout configuration
- Debug logging option

## Usage

Basic usage:

```bash
cargo run -p lode-cli -- --url https://example.com --requests 100 --method GET
```

### Command Line Arguments

- `-u, --url`: Target URL to load test (required)
- `-r, --requests`: Number of requests to send (required)
- `-c, --concurrency`: Number of concurrent requests (default: number of CPU cores)
- `-m, --method`: HTTP method to use (GET, POST, etc.) (required)
- `-t, --timeout`: Request timeout in seconds (default: 30)
- `-b, --body`: JSON body for POST/PUT requests
- `-H, --headers`: Custom headers (format: "key:value", comma-separated)
- `-f, --format`: Output format (text or json) (default: text)
- `--no-capture`: Show debug logs

### Examples

1. Basic GET request:

```bash
lode-cli --url https://api.example.com/data --requests 1000 --method GET
```

2. POST request with JSON body:

```bash
lode-cli --url https://api.example.com/create --requests 500 --method POST --body '{"name": "test", "value": 123}'
```

3. Custom headers and timeout:

```bash
lode-cli --url https://api.example.com/protected --requests 200 --method GET --headers "Authorization:Bearer token123,Content-Type:application/json" --timeout 60
```

4. JSON output format:

```bash
lode-cli --url https://api.example.com/data --requests 100 --method GET --format json
```

5. Debug mode:

```bash
lode-cli --url https://api.example.com/data --requests 50 --method GET --no-capture
```

## Output

The tool provides detailed statistics about the load test, including:

- Total requests
- Successful/failed requests
- Requests per second (RPS)
- Response time statistics (min, max, mean, median, p95, p99)
- Total duration

## Development

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building from Source

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Running Tests with Coverage

```bash
cargo tarpaulin
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 