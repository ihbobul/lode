# Lode - ⚡ High-Performance API Load Testing

<img src="https://raw.githubusercontent.com/ihbobul/lode/master/logo.svg?sanitize=true" alt="Lode logo" width="250" align="right" style="max-width: 40vw;">

Lode is an open-source, high-performance API load testing tool designed to benchmark and analyze API performance
efficiently. Built in Rust, Lode provides both a CLI for local load testing and a REST API (Docker container) for remote
test execution, ensuring flexibility and ease of use.

<div id="toc">
  <ul align="left" style="list-style: none;">
    <summary>
      <h2>✨ Features</h2>
    </summary>
  </ul>
</div>

- **🖥️ CLI Interface** – Run 🏋️ tests directly from the 🏗️ command line.
- **🌐 REST API** – Deploy a 🏗️ containerized API to handle 📡 remote load testing.
- **🚀 High Concurrency** – Utilizes asynchronous execution with `tokio`.
- **📊 Detailed Metrics** – Tracks ⏳ response times, ❌ error rates, and 🚦 throughput.
- **⚙️ Configurable Tests** – Supports 🎭 custom headers, 📜 request payloads, and 🔑 authentication.
- **❌ Robust Error Handling** – Provides 📄 structured JSON reports with 🛠️ diagnostic details.

## 📦 Installation (TBD)

### 🖥️ CLI Usage

```sh
cargo install lode
```

### 🐳 Running as a Docker Container

```sh
docker run -p 8080:8080 ghcr.io/ihbobul/lode
```

### 🚀 Usage

## 🖥️ CLI Example

```sh
cargo run -p lode-cli -- --url "http://example.com/api/v1/data" --requests 100 --concurrency 10 --format json --method GET
```

## 🌐 REST API Example

```sh
curl -X POST http://localhost:8080/load-test 
-H "Content-Type: application/json" 
-d '{
  "url": "https://httpbin.test.k6.io/get",
  "method": "GET",
  "requests": 1000,
  "concurrency": 100,
  "timeout_ms": 15000
}'
```

### 📊 Output Format

```json
{
  "id": "1c40f3b3-acb0-4cf3-b11e-0131959c9251",
  "status": "completed",
  "total_requests": 1000,
  "successful_requests": 1000,
  "failed_requests": 0,
  "requests_per_second": 168.73390780397466,
  "min_response_time_ms": 121.85600000000001,
  "max_response_time_ms": 1498.1109999999999,
  "mean_response_time_ms": 555.3430000000001,
  "median_response_time_ms": 573.4390000000001,
  "p95_response_time_ms": 854.527,
  "p99_response_time_ms": 999.423,
  "total_duration_seconds": 5.926491083,
  "error_stats": null
}
```

### 🛠️ Development Setup

```sh
git clone https://github.com/ihbobul/lode.git
cd lode
cargo build
```

### 🤝 Contribution Guidelines

1. 🍴 Fork the repository.

2. 🌱 Create a new branch.

3. 🛠️ Implement your feature or fix a 🐛 bug.

4. 🔃 Open a Pull Request.

### 📜 License

Lode is released under the 🏛️ MIT License.
