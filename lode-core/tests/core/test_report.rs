use lode_core::{
    metrics::{RequestMetrics, TestMetrics},
    report::Report,
};
use reqwest::StatusCode;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_report_generation() {
    let mut metrics = TestMetrics::new().unwrap();

    // Record successful requests
    for _ in 0..5 {
        let req_metrics = RequestMetrics::new().complete(StatusCode::OK);
        metrics.record_request(req_metrics);
    }

    // Record failed requests
    for _ in 0..3 {
        let req_metrics = RequestMetrics::new().complete(StatusCode::NOT_FOUND);
        metrics.record_request(req_metrics);
    }

    metrics.finalize(Duration::from_secs(2)).await.unwrap();
    let metrics = Arc::new(Mutex::new(metrics));

    let report = Report::from_metrics(metrics).await.unwrap();

    assert_eq!(report.total_requests, 8);
    assert_eq!(report.successful_requests, 5);
    assert_eq!(report.failed_requests, 3);
    assert_eq!(report.requests_per_second, 4.0);
}

#[tokio::test]
async fn test_report_with_errors() {
    let mut metrics = TestMetrics::new().unwrap();

    // Record different types of errors
    metrics.record_request(RequestMetrics::new().complete(StatusCode::NOT_FOUND));
    metrics.record_request(RequestMetrics::new().complete(StatusCode::NOT_FOUND));
    metrics.record_request(RequestMetrics::new().complete(StatusCode::INTERNAL_SERVER_ERROR));

    metrics.finalize(Duration::from_secs(1)).await.unwrap();
    let metrics = Arc::new(Mutex::new(metrics));

    let report = Report::from_metrics(metrics).await.unwrap();

    assert!(report.error_stats.is_some());
    let error_stats = report.error_stats.unwrap();
    assert_eq!(error_stats.error_counts.get("HTTP 404").unwrap(), &2u64);
    assert_eq!(error_stats.error_counts.get("HTTP 500").unwrap(), &1u64);
}

#[tokio::test]
async fn test_report_response_times() {
    let mut metrics = TestMetrics::new().unwrap();

    // Record requests with different response times
    let req_metrics = RequestMetrics::new();
    tokio::time::sleep(Duration::from_millis(100)).await;
    metrics.record_request(req_metrics.complete(StatusCode::OK));

    let req_metrics = RequestMetrics::new();
    tokio::time::sleep(Duration::from_millis(200)).await;
    metrics.record_request(req_metrics.complete(StatusCode::OK));

    metrics.finalize(Duration::from_secs(1)).await.unwrap();
    let metrics = Arc::new(Mutex::new(metrics));

    let report = Report::from_metrics(metrics).await.unwrap();

    assert!(report.min_response_time_ms >= 100.0);
    assert!(report.max_response_time_ms >= 200.0);
    assert!(report.mean_response_time_ms >= 150.0);
}
