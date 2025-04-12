mod configuration;
mod handlers;
mod models;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use lode_core::telemetry::{get_stdout_subscriber, init_subscriber};
use tracing::info;
use tracing_actix_web::TracingLogger;

use configuration::Settings;
use handlers::{health_check, run_load_test, AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let subscriber = get_stdout_subscriber("lode-api".into(), "info".into());
    init_subscriber(subscriber);

    let settings = Settings::get_configuration().expect("Failed to load configuration");
    let address = format!("{}:{}", settings.server.host, settings.server.port);

    let app_state = web::Data::new(AppState::new().expect("Failed to create app state"));

    info!("Starting Lode API server on {}", address);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&settings.server.cors_origin)
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(TracingLogger::default())
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check))
            .route("/load-test", web::post().to(run_load_test))
    })
    .bind(&address)?
    .run()
    .await
}
