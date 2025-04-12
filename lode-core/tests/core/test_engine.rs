use crate::common::error_simulation::setup_error_mock_server;
use crate::common::mock_server::setup_mock_server;

use lode_core::engine::LoadTestEngine;
use lode_core::http::DefaultHttpClient;
use reqwest::Method;
use std::time::Duration;

#[tokio::test]
async fn test_load_test_success() {
    let mock_server = setup_mock_server(200, "/test", None).await;
    let client = DefaultHttpClient::new().unwrap();
    let engine = LoadTestEngine::new(client).unwrap();

    let metrics = engine
        .run(
            Method::GET,
            format!("{}/test", mock_server.uri()),
            10,
            2,
            Duration::from_secs(1),
            vec![],
            None,
            None,
        )
        .await
        .unwrap();

    let metrics = metrics.lock().await;
    assert_eq!(metrics.total_requests(), 10);
    assert_eq!(metrics.successful_requests(), 10);
    assert_eq!(metrics.failed_requests(), 0);
}

#[tokio::test]
async fn test_load_test_with_errors() {
    let mock_server = setup_error_mock_server("/error").await;
    let client = DefaultHttpClient::new().unwrap();
    let engine = LoadTestEngine::new(client).unwrap();

    let metrics = engine
        .run(
            Method::GET,
            format!("{}/error", mock_server.uri()),
            5,
            2,
            Duration::from_secs(1),
            vec![("X-Error-Type".to_string(), "429".to_string())],
            None,
            None,
        )
        .await
        .unwrap();

    let metrics = metrics.lock().await;
    assert_eq!(metrics.total_requests(), 5);
    assert_eq!(metrics.successful_requests(), 0);
    assert_eq!(metrics.failed_requests(), 5);

    let (error_counts, error_messages) = metrics.error_stats().unwrap();
    assert_eq!(error_counts.get("HTTP 429").unwrap(), &5u64);
    assert!(error_messages
        .iter()
        .all(|msg| msg.contains("Too Many Requests")));
}
