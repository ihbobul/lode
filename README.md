<img src="https://raw.githubusercontent.com/ihbobul/lode/master/logo.svg?sanitize=true" alt="Logos logo" width="250" align="right">

# 🏎️ Lode - ⚡ High-Performance API Load Testing 🛠️

Lode is an 🌍 open-source, 🚀 high-performance API 🔥 load testing 🧪 tool designed to 📏 benchmark 📊 and 🔍 analyze API ⚙️ performance efficiently. Built in 🦀 Rust, Lode provides both a 🖥️ CLI for local load testing and a 🌐 REST API (🐳 Docker container) for remote test execution, ensuring 🎯 flexibility and 🤹 ease of use.

---

## ✨ Features

- **🖥️ CLI Interface** – Run 🏋️ tests directly from the 🏗️ command line.
- **🌐 REST API** – Deploy a 🏗️ containerized API to handle 📡 remote load testing.
- **🚀 High Concurrency** – Utilizes asynchronous execution with `tokio`.
- **📊 Detailed Metrics** – Tracks ⏳ response times, ❌ error rates, and 🚦 throughput.
- **⚙️ Configurable Tests** – Supports 🎭 custom headers, 📜 request payloads, and 🔑 authentication.
- **❌ Robust Error Handling** – Provides 📄 structured JSON reports with 🛠️ diagnostic details.

---

## 📦 Installation

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
lode --url https://api.example.com --requests 1000 --concurrency 50
```

## 🌐 REST API Example
```sh
curl -X POST http://localhost:8080/load-test 
-H "Content-Type: application/json" 
-d '{
  "url": "https://api.example.com",
  "requests": 1000,
  "concurrency": 50
}'
```

### 📊 Output Format
```json
{
  "total_requests": 1000,
  "successful_requests": 980,
  "failed_requests": 20,
  "avg_response_time_ms": 200,
  "p95_response_time_ms": 400,
  "throughput_rps": 50,
  "errors": {
    "timeout": 10,
    "500_internal_server_error": 5,
    "connection_failed": 5
  }
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
