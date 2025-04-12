use crate::common::error_simulation::setup_error_mock_server;
use crate::common::mock_server::setup_mock_server;

use lode_core::http::{DefaultHttpClient, HttpClient};
use reqwest::Method;
use std::time::Duration;

#[tokio::test]
async fn test_successful_request() {
    let mock_server = setup_mock_server(200, "/test", None).await;
    let client = DefaultHttpClient::new().unwrap();

    let response = client
        .send_request(
            Method::GET,
            format!("{}/test", mock_server.uri()),
            Duration::from_secs(1),
            vec![],
            None,
        )
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn test_error_responses() {
    let mock_server = setup_error_mock_server("/error").await;
    let client = DefaultHttpClient::new().unwrap();

    // Test 429 Too Many Requests
    let response = client
        .send_request(
            Method::GET,
            format!("{}/error", mock_server.uri()),
            Duration::from_secs(1),
            vec![("X-Error-Type".to_string(), "429".to_string())],
            None,
        )
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 429);

    // Test 404 Not Found
    let response = client
        .send_request(
            Method::GET,
            format!("{}/error", mock_server.uri()),
            Duration::from_secs(1),
            vec![("X-Error-Type".to_string(), "404".to_string())],
            None,
        )
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn test_timeout() {
    let mock_server = setup_mock_server(200, "/slow", Some(Duration::from_secs(2))).await;
    let client = DefaultHttpClient::new().unwrap();

    let result = client
        .send_request(
            Method::GET,
            format!("{}/slow", mock_server.uri()),
            Duration::from_secs(1),
            vec![],
            None,
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().is_timeout());
}
