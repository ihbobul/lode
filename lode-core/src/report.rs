use crate::error::Result;
use crate::metrics::TestMetrics;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// A formatted test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
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

/// Error statistics for a test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    pub error_counts: HashMap<String, u64>,
    pub error_messages: Vec<String>,
}

impl Report {
    /// Create a new report from test metrics
    pub async fn from_metrics(metrics: Arc<Mutex<TestMetrics>>) -> Result<Self> {
        let metrics = metrics.lock().await;

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            status: "completed".to_string(),
            total_requests: metrics.total_requests(),
            successful_requests: metrics.successful_requests(),
            failed_requests: metrics.failed_requests(),
            requests_per_second: metrics.requests_per_second(),
            min_response_time_ms: metrics.min_response_time().as_secs_f64() * 1000.0,
            max_response_time_ms: metrics.max_response_time().as_secs_f64() * 1000.0,
            mean_response_time_ms: metrics.mean_response_time().as_secs_f64() * 1000.0,
            median_response_time_ms: metrics.median_response_time().as_secs_f64() * 1000.0,
            p95_response_time_ms: metrics.p95_response_time().as_secs_f64() * 1000.0,
            p99_response_time_ms: metrics.p99_response_time().as_secs_f64() * 1000.0,
            total_duration_seconds: metrics.total_duration().as_secs_f64(),
            error_stats: metrics.error_stats().map(|(counts, messages)| ErrorStats {
                error_counts: counts,
                error_messages: messages,
            }),
        })
    }

    /// Format the report as JSON
    pub fn as_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::Error::Report(format!("Failed to serialize report: {}", e)))
    }

    /// Format the report as a human-readable string
    pub fn as_string(&self) -> String {
        format!(
            r#"Load Test Report
            ----------------
            Total Requests: {}
            Successful Requests: {}
            Failed Requests: {}
            Requests/second: {:.2}

            Response Time (ms)
            ----------------
            Min: {}
            Max: {}
            Mean: {}
            Median: {}
            P95: {}
            P99: {}

            Total Duration: {:.2} seconds"#,
            self.total_requests,
            self.successful_requests,
            self.failed_requests,
            self.requests_per_second,
            self.min_response_time_ms,
            self.max_response_time_ms,
            self.mean_response_time_ms,
            self.median_response_time_ms,
            self.p95_response_time_ms,
            self.p99_response_time_ms,
            self.total_duration_seconds,
        )
    }

    // Getters
    pub fn total_requests(&self) -> u64 {
        self.total_requests
    }

    pub fn successful_requests(&self) -> u64 {
        self.successful_requests
    }

    pub fn failed_requests(&self) -> u64 {
        self.failed_requests
    }

    pub fn requests_per_second(&self) -> f64 {
        self.requests_per_second
    }

    pub fn min_response_time_ms(&self) -> f64 {
        self.min_response_time_ms
    }

    pub fn max_response_time_ms(&self) -> f64 {
        self.max_response_time_ms
    }

    pub fn mean_response_time_ms(&self) -> f64 {
        self.mean_response_time_ms
    }

    pub fn median_response_time_ms(&self) -> f64 {
        self.median_response_time_ms
    }

    pub fn p95_response_time_ms(&self) -> f64 {
        self.p95_response_time_ms
    }

    pub fn p99_response_time_ms(&self) -> f64 {
        self.p99_response_time_ms
    }

    pub fn total_duration_seconds(&self) -> f64 {
        self.total_duration_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{RequestMetrics, TestMetrics};
    use reqwest::{Client, StatusCode};
    use std::time::Duration;

    #[tokio::test]
    async fn test_report_from_metrics() {
        let metrics = Arc::new(Mutex::new(TestMetrics::new().unwrap()));
        {
            let mut metrics = metrics.lock().await;

            // Record successful requests with different response times
            let request_metrics = RequestMetrics::new();
            tokio::time::sleep(Duration::from_millis(100)).await;
            let request_metrics = request_metrics.complete(StatusCode::OK);
            metrics.record_request(request_metrics);

            let request_metrics = RequestMetrics::new();
            tokio::time::sleep(Duration::from_millis(200)).await;
            let request_metrics = request_metrics.complete(StatusCode::OK);
            metrics.record_request(request_metrics);

            let request_metrics = RequestMetrics::new();
            tokio::time::sleep(Duration::from_millis(300)).await;
            let request_metrics = request_metrics.complete(StatusCode::OK);
            metrics.record_request(request_metrics);

            // Record error requests
            let request_metrics = RequestMetrics::new();
            tokio::time::sleep(Duration::from_millis(100)).await;
            let client = Client::new();
            let error = client.get("invalid://url").send().await.unwrap_err();
            let request_metrics = request_metrics.record_error(error);
            metrics.record_request(request_metrics);

            let request_metrics = RequestMetrics::new();
            tokio::time::sleep(Duration::from_millis(100)).await;
            let client = Client::new();
            let error = client
                .get("http://localhost:1")
                .timeout(Duration::from_nanos(1))
                .send()
                .await
                .unwrap_err();
            let request_metrics = request_metrics.record_error(error);
            metrics.record_request(request_metrics);

            metrics.finalize(Duration::from_secs(1)).await.unwrap();
        }

        let report = Report::from_metrics(metrics).await.unwrap();

        assert_eq!(report.total_requests(), 5);
        assert_eq!(report.successful_requests(), 3);
        assert_eq!(report.failed_requests(), 2);
        assert_eq!(report.requests_per_second(), 5.0);
        assert!(report.min_response_time_ms() >= 100.0);
        assert!(report.max_response_time_ms() >= 300.0);
        assert!(report.mean_response_time_ms() >= 160.0);
        assert!(report.median_response_time_ms() >= 100.0);
        assert!(report.p95_response_time_ms() >= 300.0);
        assert!(report.p99_response_time_ms() >= 300.0);
        assert_eq!(report.total_duration_seconds(), 1.0);

        let error_stats = report.error_stats.unwrap();
        assert_eq!(error_stats.error_messages.len(), 2);
    }

    #[test]
    fn test_report_json_serialization() {
        let report = Report {
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
            error_stats: Some(ErrorStats {
                error_counts: HashMap::from([
                    ("timeout".to_string(), 3),
                    ("connection failed".to_string(), 2),
                ]),
                error_messages: vec![
                    "Request timed out".to_string(),
                    "Connection refused".to_string(),
                ],
            }),
        };

        let json = report.as_json().unwrap();
        let deserialized: Report = serde_json::from_str(&json).unwrap();
        assert_eq!(report.id, deserialized.id);
        assert_eq!(report.total_requests, deserialized.total_requests);
        assert_eq!(
            report.error_stats.unwrap().error_counts,
            deserialized.error_stats.unwrap().error_counts
        );
    }

    #[test]
    fn test_report_string_format() {
        let report = Report {
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

        let string = report.as_string();
        assert!(string.contains("Total Requests: 100"));
        assert!(string.contains("Successful Requests: 95"));
        assert!(string.contains("Failed Requests: 5"));
        assert!(string.contains("Requests/second: 10.00"));
        assert!(string.contains("Min: 100"));
        assert!(string.contains("Max: 500"));
        assert!(string.contains("Mean: 200"));
        assert!(string.contains("Median: 180"));
        assert!(string.contains("P95: 400"));
        assert!(string.contains("P99: 450"));
        assert!(string.contains("Total Duration: 10.00 seconds"));
    }

    #[test]
    fn test_report_getters() {
        let report = Report {
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

        assert_eq!(report.total_requests(), 100);
        assert_eq!(report.successful_requests(), 95);
        assert_eq!(report.failed_requests(), 5);
        assert_eq!(report.requests_per_second(), 10.0);
        assert_eq!(report.min_response_time_ms(), 100.0);
        assert_eq!(report.max_response_time_ms(), 500.0);
        assert_eq!(report.mean_response_time_ms(), 200.0);
        assert_eq!(report.median_response_time_ms(), 180.0);
        assert_eq!(report.p95_response_time_ms(), 400.0);
        assert_eq!(report.p99_response_time_ms(), 450.0);
        assert_eq!(report.total_duration_seconds(), 10.0);
    }
}
