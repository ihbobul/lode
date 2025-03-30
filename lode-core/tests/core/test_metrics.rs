use crate::common::error_simulation::setup_error_mock_server;

use lode_core::error::Result;
use lode_core::metrics::{RequestMetrics, TestMetrics};
use reqwest::StatusCode;
use std::time::Duration;

async fn create_reqwest_error(kind: &str) -> reqwest::Error {
    match kind {
        "timeout" => {
            let client = reqwest::Client::new();
            let request = client
                .get("http://0.0.0.0:1")
                .timeout(Duration::from_millis(1))
                .build()
                .unwrap();
            match client.execute(request).await {
                Ok(_) => panic!("Expected timeout error"),
                Err(e) => e,
            }
        }
        "connect" => {
            let client = reqwest::Client::new();
            let request = client.get("http://invalid.local").build().unwrap();
            match client.execute(request).await {
                Ok(_) => panic!("Expected connection error"),
                Err(e) => e,
            }
        }
        _ => reqwest::Client::new().get("not-a-url").build().unwrap_err(),
    }
}

#[tokio::test]
async fn test_metrics_recording() {
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

    assert_eq!(metrics.total_requests(), 8);
    assert_eq!(metrics.successful_requests(), 5);
    assert_eq!(metrics.failed_requests(), 3);
}

#[tokio::test]
async fn test_metrics_error_stats() {
    let mut metrics = TestMetrics::new().unwrap();

    metrics.record_request(RequestMetrics::new().complete(StatusCode::NOT_FOUND));
    metrics.record_request(RequestMetrics::new().complete(StatusCode::NOT_FOUND));
    metrics.record_request(RequestMetrics::new().complete(StatusCode::INTERNAL_SERVER_ERROR));

    let (error_counts, error_messages) = metrics.error_stats().unwrap();
    assert_eq!(error_counts.get("HTTP 404").unwrap(), &2u64);
    assert_eq!(error_counts.get("HTTP 500").unwrap(), &1u64);
    assert_eq!(error_messages.len(), 3);
}

#[tokio::test]
async fn test_metrics_response_times() {
    let mut metrics = TestMetrics::new().unwrap();

    let req_metrics = RequestMetrics::new();
    tokio::time::sleep(Duration::from_millis(100)).await;
    metrics.record_request(req_metrics.complete(StatusCode::OK));

    let req_metrics = RequestMetrics::new();
    tokio::time::sleep(Duration::from_millis(200)).await;
    metrics.record_request(req_metrics.complete(StatusCode::OK));

    assert!(metrics.min_response_time() >= Duration::from_millis(100));
    assert!(metrics.max_response_time() >= Duration::from_millis(200));
    assert!(metrics.mean_response_time() >= Duration::from_millis(150));
}

#[tokio::test]
async fn test_metrics_finalization() {
    let mut metrics = TestMetrics::new().unwrap();

    // Record some requests
    for _ in 0..10 {
        let req_metrics = RequestMetrics::new().complete(StatusCode::OK);
        metrics.record_request(req_metrics);
    }

    let duration = Duration::from_secs(2);
    metrics.finalize(duration).await.unwrap();

    assert_eq!(metrics.total_duration(), duration);
    assert_eq!(metrics.requests_per_second(), 5.0); // 10 requests / 2 seconds
}

#[tokio::test]
async fn test_metrics_with_http_errors() -> Result<()> {
    let mock_server = setup_error_mock_server("/error").await;
    let mut metrics = TestMetrics::new()?;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/error", mock_server.uri()))
        .header("X-Error-Type", "404")
        .send()
        .await?;
    metrics.record_request(RequestMetrics::new().complete(response.status()));

    let response = client
        .get(&format!("{}/error", mock_server.uri()))
        .header("X-Error-Type", "404")
        .send()
        .await?;
    metrics.record_request(RequestMetrics::new().complete(response.status()));

    let response = client
        .get(&format!("{}/error", mock_server.uri()))
        .header("X-Error-Type", "500")
        .send()
        .await?;
    metrics.record_request(RequestMetrics::new().complete(response.status()));

    let (error_counts, error_messages) = metrics.error_stats().expect("Should have error stats");

    assert_eq!(metrics.failed_requests(), 3);
    assert_eq!(error_counts.get("HTTP 404").unwrap(), &2u64);
    assert_eq!(error_counts.get("HTTP 500").unwrap(), &1u64);
    assert_eq!(error_messages.len(), 3);
    Ok(())
}

#[tokio::test]
async fn test_metrics_with_client_errors() -> Result<()> {
    let mut metrics = TestMetrics::new()?;

    // Record different client errors
    let timeout_error = create_reqwest_error("timeout").await;
    let connect_error = create_reqwest_error("connect").await;
    let timeout_error2 = create_reqwest_error("timeout").await;

    metrics.record_request(RequestMetrics::new().record_error(timeout_error));
    metrics.record_request(RequestMetrics::new().record_error(connect_error));
    metrics.record_request(RequestMetrics::new().record_error(timeout_error2));

    let (error_counts, error_messages) = metrics.error_stats().unwrap();

    assert_eq!(metrics.failed_requests(), 3);
    assert!(error_counts.values().sum::<u64>() == 3);
    assert_eq!(error_messages.len(), 3);
    Ok(())
}

#[tokio::test]
async fn test_metrics_with_mixed_errors() -> Result<()> {
    let mock_server = setup_error_mock_server("/error").await;
    let mut metrics = TestMetrics::new()?;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/error", mock_server.uri()))
        .header("X-Error-Type", "404")
        .send()
        .await?;
    metrics.record_request(RequestMetrics::new().complete(response.status()));

    // Record a timeout error
    let timeout_error = create_reqwest_error("timeout").await;
    metrics.record_request(RequestMetrics::new().record_error(timeout_error));

    let response = client
        .get(&format!("{}/error", mock_server.uri()))
        .header("X-Error-Type", "404")
        .send()
        .await?;
    metrics.record_request(RequestMetrics::new().complete(response.status()));

    // Record a successful request
    let response = client
        .get(&format!("{}/error", mock_server.uri()))
        .header("X-Error-Type", "200")
        .send()
        .await?;
    metrics.record_request(RequestMetrics::new().complete(response.status()));

    // Record another timeout error
    let timeout_error2 = create_reqwest_error("timeout").await;
    metrics.record_request(RequestMetrics::new().record_error(timeout_error2));

    let (error_counts, error_messages) = metrics.error_stats().unwrap();

    assert_eq!(metrics.total_requests(), 5);
    assert_eq!(metrics.successful_requests(), 1);
    assert_eq!(metrics.failed_requests(), 4);
    assert_eq!(error_counts.get("HTTP 404").unwrap(), &2u64);
    assert!(error_counts.values().sum::<u64>() == 4);
    assert_eq!(error_messages.len(), 4);
    Ok(())
}

#[tokio::test]
async fn test_request_metrics_lifecycle() {
    let metrics = RequestMetrics::new();
    assert!(metrics.duration().is_none());
    assert!(metrics.status().is_none());
    assert!(metrics.error().is_none());

    let metrics = metrics.complete(StatusCode::OK);
    assert!(metrics.duration().is_some());
    assert_eq!(metrics.status(), Some(StatusCode::OK));
    assert!(metrics.error().is_none());
}

#[tokio::test]
async fn test_request_metrics_with_error() {
    let metrics = RequestMetrics::new();

    // Simulate a request error
    let client = reqwest::Client::new();
    let err = client.get("http://invalid.local").send().await.unwrap_err();

    let metrics = metrics.record_error(err);
    assert!(metrics.duration().is_some());
    assert!(metrics.status().is_none());
    assert!(metrics.error().is_some());
}

#[tokio::test]
async fn test_request_metrics_timing() {
    let metrics = RequestMetrics::new();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let metrics = metrics.complete(StatusCode::OK);

    assert!(metrics.duration().unwrap() >= Duration::from_millis(100));
}
