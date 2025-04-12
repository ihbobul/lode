use chrono;
use serde_json::json;
use wiremock::matchers::{header, method, path as path_matcher};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub async fn setup_error_mock_server(path: &str) -> MockServer {
    let mock_server = MockServer::start().await;

    // Setup success response
    Mock::given(method("GET"))
        .and(path_matcher(path))
        .and(header("X-Error-Type", "200"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "success",
            "message": "Request successful",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "processing_time_ms": 0
        })))
        .mount(&mock_server)
        .await;

    // Setup error responses for different status codes
    setup_error_response(
        &mock_server,
        path,
        429,
        "Too Many Requests",
        "Rate limit exceeded. Please try again later.",
    )
    .await;
    setup_error_response(
        &mock_server,
        path,
        404,
        "Not Found",
        "The requested resource was not found on this server",
    )
    .await;
    setup_error_response(
        &mock_server,
        path,
        503,
        "Service Unavailable",
        "The server is temporarily unable to handle your request",
    )
    .await;
    setup_error_response(
        &mock_server,
        path,
        500,
        "Internal Server Error",
        "An unexpected error occurred on the server",
    )
    .await;

    mock_server
}

async fn setup_error_response(
    mock_server: &MockServer,
    path: &str,
    status: u16,
    error: &str,
    message: &str,
) {
    let status_str = match status {
        429 => "429",
        404 => "404",
        503 => "503",
        500 => "500",
        _ => "500",
    };

    Mock::given(method("GET"))
        .and(path_matcher(path))
        .and(header("X-Error-Type", status_str))
        .respond_with(ResponseTemplate::new(status).set_body_json(json!({
            "error": error,
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "processing_time_ms": 0
        })))
        .mount(mock_server)
        .await;
}
