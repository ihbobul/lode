use std::time::Duration;
use wiremock::matchers::{method, path as path_matcher};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub async fn setup_mock_server(status: u16, path: &str, delay: Option<Duration>) -> MockServer {
    let mock_server = MockServer::start().await;
    let mut response = ResponseTemplate::new(status);

    if let Some(delay) = delay {
        response = response.set_delay(delay);
    }

    // Handle both GET and POST requests
    Mock::given(method("GET"))
        .and(path_matcher(path))
        .respond_with(response.clone())
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path_matcher(path))
        .respond_with(response)
        .mount(&mock_server)
        .await;

    mock_server
}
