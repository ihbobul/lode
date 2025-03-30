use lode_core::{engine::LoadTestEngine, http::DefaultHttpClient};
use reqwest::Method;
use std::time::Duration;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_error_handling_integration() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/error"))
        .and(header("X-Error-Type", "429"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": "Too Many Requests",
            "message": "Rate limit exceeded. Please try again later."
        })))
        .mount(&mock_server)
        .await;

    let client = DefaultHttpClient::new().unwrap();
    let engine = LoadTestEngine::new(client).unwrap();

    // Run load test with 429 error
    let metrics = engine
        .run(
            Method::GET,
            format!("{}/error", mock_server.uri()),
            5, // num_requests
            2, // concurrency
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
