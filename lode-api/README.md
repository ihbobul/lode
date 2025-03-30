# Lode API

REST API service for the Lode load testing tool.

## Features

- Run load tests via HTTP endpoints
- Support for all HTTP methods (GET, POST, PUT, DELETE, PATCH)
- Configurable concurrency and request count
- Detailed performance metrics
- CORS support
- Health check endpoint

## API Endpoints

### Health Check

```
GET /health
```

Response:

```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

### Run Load Test

```
POST /load-test
```

Request body:

```json
{
  "url": "https://httpbin.test.k6.io/get",
  "method": "GET",
  "requests": 1000,
  "concurrency": 100,
  "timeout_ms": 15000
}
```

Response:

```json
{
  "id": "9e2a6d4e-7add-4f5e-a5e9-fd70700efa7d",
  "status": "completed",
  "total_requests": 1000,
  "successful_requests": 1000,
  "failed_requests": 0,
  "requests_per_second": 125.82281040471598,
  "min_response_time_ms": 122.56,
  "max_response_time_ms": 2072.5750000000003,
  "mean_response_time_ms": 680.902,
  "median_response_time_ms": 551.423,
  "p95_response_time_ms": 1553.407,
  "p99_response_time_ms": 1753.0869999999998,
  "total_duration_seconds": 7.9476845,
  "error_stats": null
}
```

## Running the Service

### Local Development

```bash
cargo run -p lode-api
```

### Docker

```bash
docker build -t lode-api .
docker run -p 8081:8081 lode-api
```

## Environment Variables

- `PORT`: Server port (default: 8081)
- `HOST`: Server host (default: 127.0.0.1)

## Testing

```bash
cargo test
```

## License

MIT 