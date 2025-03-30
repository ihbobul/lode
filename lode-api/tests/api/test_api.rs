use crate::common::utils::{setup_mock_server, setup_test_app};
use actix_web::http::Method;
use actix_web::{test, web, App};
use lode_api::handlers::{health_check, run_load_test, AppState};
use lode_api::models::LoadTestRequest;
use lode_api::LoadTestResponse;
use std::collections::HashMap;
use std::env;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[actix_web::test]
async fn test_app_setup() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::new().unwrap()))
            .route("/health", web::get().to(health_check))
            .route("/load-test", web::post().to(run_load_test)),
    )
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::post().uri("/load-test").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400); // Bad request because no body
}

#[actix_web::test]
async fn test_environment_variables() {
    env::remove_var("PORT");
    env::remove_var("HOST");
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::new().unwrap()))
            .route("/health", web::get().to(health_check)),
    )
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    env::set_var("PORT", "8081");
    env::set_var("HOST", "0.0.0.0");
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::new().unwrap()))
            .route("/health", web::get().to(health_check)),
    )
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_cors_configuration() {
    let app = test::init_service(
        App::new()
            .wrap(
                actix_cors::Cors::default()
                    .allowed_origin("http://example.com")
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .app_data(web::Data::new(AppState::new().unwrap()))
            .route("/health", web::get().to(health_check)),
    )
    .await;

    let req = test::TestRequest::with_uri("/health")
        .method(Method::OPTIONS)
        .insert_header(("Origin", "http://example.com"))
        .insert_header(("Access-Control-Request-Method", "GET"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert_eq!(
        resp.headers().get("access-control-allow-origin").unwrap(),
        "http://example.com"
    );
}

#[actix_web::test]
async fn test_invalid_port() {
    env::set_var("PORT", "invalid");
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::new().unwrap()))
            .route("/health", web::get().to(health_check)),
    )
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_health_check() {
    let app = setup_test_app().await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_run_load_test() {
    let mock_server = setup_mock_server(200).await;

    let app = setup_test_app().await;

    let request = LoadTestRequest {
        url: format!("{}/test", mock_server.uri()),
        method: "GET".to_string(),
        requests: 10,
        concurrency: 2,
        timeout_ms: Some(30000),
        headers: None,
        body: None,
    };

    let req = test::TestRequest::post()
        .uri("/load-test")
        .set_json(&request)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let response: LoadTestResponse = test::read_body_json(resp).await;
    assert_eq!(response.total_requests, 10);
    assert_eq!(response.successful_requests, 10);
    assert_eq!(response.failed_requests, 0);
}

#[actix_web::test]
async fn test_run_load_test_with_error() {
    let mock_server = setup_mock_server(500).await;

    let app = setup_test_app().await;

    let request = LoadTestRequest {
        url: format!("{}/test", mock_server.uri()),
        method: "GET".to_string(),
        requests: 10,
        concurrency: 2,
        timeout_ms: Some(30000),
        headers: None,
        body: None,
    };

    let req = test::TestRequest::post()
        .uri("/load-test")
        .set_json(&request)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let response: LoadTestResponse = test::read_body_json(resp).await;
    assert_eq!(response.total_requests, 10);
    assert_eq!(response.successful_requests, 0);
    assert_eq!(response.failed_requests, 10);
}

#[actix_web::test]
async fn test_run_load_test_with_invalid_url() {
    let app = setup_test_app().await;

    let request = LoadTestRequest {
        url: "invalid-url".to_string(),
        method: "GET".to_string(),
        requests: 10,
        concurrency: 2,
        timeout_ms: Some(30000),
        headers: None,
        body: None,
    };

    let req = test::TestRequest::post()
        .uri("/load-test")
        .set_json(&request)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_run_load_test_with_invalid_method() {
    let app = setup_test_app().await;

    let request = LoadTestRequest {
        url: "http://example.com".to_string(),
        method: "INVALID".to_string(),
        requests: 10,
        concurrency: 2,
        timeout_ms: Some(30000),
        headers: None,
        body: None,
    };

    let req = test::TestRequest::post()
        .uri("/load-test")
        .set_json(&request)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_run_load_test_with_invalid_headers() {
    let app = setup_test_app().await;

    let mut headers = HashMap::new();
    headers.insert("Invalid@Header".to_string(), "value".to_string());

    let request = LoadTestRequest {
        url: "http://example.com".to_string(),
        method: "GET".to_string(),
        requests: 10,
        concurrency: 2,
        timeout_ms: Some(30000),
        headers: Some(headers),
        body: None,
    };

    let req = test::TestRequest::post()
        .uri("/load-test")
        .set_json(&request)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_app_state_initialization_error() {
    let result = lode_api::handlers::AppState::new();
    assert!(result.is_ok());
}

#[actix_web::test]
async fn test_run_load_test_with_headers() {
    let mock_server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::header("X-Error-Type", "429"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": "Too Many Requests",
            "message": "Rate limit exceeded. Please try again later."
        })))
        .mount(&mock_server)
        .await;

    let app = setup_test_app().await;

    let mut headers = HashMap::new();
    headers.insert("X-Error-Type".to_string(), "429".to_string());

    let request = LoadTestRequest {
        url: format!("{}/test", mock_server.uri()),
        method: "GET".to_string(),
        requests: 10,
        concurrency: 2,
        timeout_ms: Some(30000),
        headers: Some(headers),
        body: None,
    };

    let req = test::TestRequest::post()
        .uri("/load-test")
        .set_json(&request)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let response: LoadTestResponse = test::read_body_json(resp).await;
    assert_eq!(response.total_requests, 10);
    assert_eq!(response.successful_requests, 0);
    assert_eq!(response.failed_requests, 10);

    let error_stats = response.error_stats.unwrap();
    assert_eq!(error_stats.error_counts.get("HTTP 429").unwrap(), &10u64);
    assert!(error_stats
        .error_messages
        .iter()
        .all(|msg| msg.contains("Too Many Requests")));
}
