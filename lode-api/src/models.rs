use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct LoadTestRequest {
    pub url: String,
    pub method: String,
    pub requests: u64,
    pub concurrency: u64,
    pub timeout_ms: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadTestResponse {
    pub id: String,
    pub status: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub requests_per_second: f64,
    pub min_response_time_ms: f64,
    pub max_response_time_ms: f64,
    pub mean_response_time_ms: f64,
    pub median_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub total_duration_seconds: f64,
    pub error_stats: Option<ErrorStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorStats {
    pub error_counts: HashMap<String, u64>,
    pub error_messages: Vec<String>,
}

impl From<lode_core::report::Report> for LoadTestResponse {
    fn from(report: lode_core::report::Report) -> Self {
        LoadTestResponse {
            id: report.id,
            status: report.status,
            total_requests: report.total_requests,
            successful_requests: report.successful_requests,
            failed_requests: report.failed_requests,
            requests_per_second: report.requests_per_second,
            min_response_time_ms: report.min_response_time_ms,
            max_response_time_ms: report.max_response_time_ms,
            mean_response_time_ms: report.mean_response_time_ms,
            median_response_time_ms: report.median_response_time_ms,
            p95_response_time_ms: report.p95_response_time_ms,
            p99_response_time_ms: report.p99_response_time_ms,
            total_duration_seconds: report.total_duration_seconds,
            error_stats: report.error_stats.map(|stats| ErrorStats {
                error_counts: stats.error_counts,
                error_messages: stats.error_messages,
            }),
        }
    }
}

impl From<LoadTestRequest> for lode_core::config::LoadTestConfig {
    fn from(req: LoadTestRequest) -> Self {
        let mut config = Self::new(
            req.url,
            req.method
                .parse()
                .unwrap_or(lode_core::config::HttpMethod::GET),
            req.requests as usize,
            req.concurrency as usize,
            std::time::Duration::from_millis(req.timeout_ms.unwrap_or(5000)),
        )
        .expect("Failed to create load test config");

        if let Some(headers) = req.headers {
            config.headers = headers.into_iter().collect();
        }

        if let Some(body) = req.body {
            config.body = Some(body);
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_test_request_deserialization() {
        let json = r#"{
            "url": "https://example.com",
            "method": "GET",
            "requests": 100,
            "concurrency": 10,
            "timeout_ms": 5000,
            "headers": {
                "Authorization": "Bearer token"
            },
            "body": "test"
        }"#;

        let request: LoadTestRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.method, "GET");
        assert_eq!(request.requests, 100);
        assert_eq!(request.concurrency, 10);
        assert_eq!(request.timeout_ms, Some(5000));
        assert_eq!(request.headers.unwrap()["Authorization"], "Bearer token");
        assert_eq!(request.body, Some("test".to_string()));
    }

    #[test]
    fn test_load_test_response_serialization() {
        let response = LoadTestResponse {
            id: "test-id".to_string(),
            status: "completed".to_string(),
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            requests_per_second: 10.0,
            min_response_time_ms: 100.0,
            max_response_time_ms: 500.0,
            mean_response_time_ms: 200.0,
            median_response_time_ms: 180.0,
            p95_response_time_ms: 400.0,
            p99_response_time_ms: 450.0,
            total_duration_seconds: 10.0,
            error_stats: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"id\":\"test-id\""));
        assert!(json.contains("\"status\":\"completed\""));
        assert!(json.contains("\"total_requests\":100"));
        assert!(json.contains("\"successful_requests\":95"));
        assert!(json.contains("\"failed_requests\":5"));
    }
}
