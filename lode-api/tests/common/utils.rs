use actix_web::{test, web, App};
use lode_api::handlers::AppState;
use wiremock::{Mock, MockServer, ResponseTemplate};

pub async fn setup_test_app() -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::new().unwrap()))
            .route("/health", web::get().to(lode_api::handlers::health_check))
            .route(
                "/load-test",
                web::post().to(lode_api::handlers::run_load_test),
            ),
    )
    .await
}

pub async fn setup_mock_server(status: u16) -> MockServer {
    let mock_server = MockServer::start().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(status))
        .mount(&mock_server)
        .await;
    mock_server
}
