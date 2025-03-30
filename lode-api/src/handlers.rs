use actix_web::{web, HttpResponse, Responder};
use lode_core::{engine::LoadTestEngine, http::DefaultHttpClient, report::Report};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, instrument, warn};
use url;

use crate::models::{LoadTestRequest, LoadTestResponse};

pub struct AppState {
    engine: Arc<Mutex<LoadTestEngine<DefaultHttpClient>>>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let client = DefaultHttpClient::new()?;
        let engine = LoadTestEngine::new(client)?;
        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
        })
    }
}

#[instrument(skip_all)]
pub async fn health_check() -> impl Responder {
    debug!("Health check requested");
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[instrument(skip_all, fields(
    url = %data.url,
    method = %data.method,
    requests = %data.requests,
    concurrency = %data.concurrency,
))]
pub async fn run_load_test(
    data: web::Json<LoadTestRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let request = data.into_inner();

    if let Err(e) = url::Url::parse(&request.url) {
        warn!("Invalid URL provided: {}", e);
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid URL",
            "details": e.to_string()
        }));
    }

    if let Err(e) = request.method.parse::<lode_core::config::HttpMethod>() {
        warn!("Invalid HTTP method provided: {}", e);
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid HTTP method",
            "details": e.to_string()
        }));
    }

    if let Some(headers) = &request.headers {
        for (key, _) in headers {
            if !key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                warn!("Invalid header name provided: {}", key);
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid header name",
                    "details": format!("Header name '{}' contains invalid characters", key)
                }));
            }
        }
    }

    let config: lode_core::config::LoadTestConfig = request.into();

    let engine = state.engine.lock().await;
    let result = engine
        .run(
            config.method.into(),
            config.url,
            config.requests as u64,
            config.concurrency as u64,
            config.timeout,
            config.headers,
            config.body,
            None,
        )
        .await;

    match result {
        Ok(metrics) => {
            let report = Report::from_metrics(metrics).await;
            match report {
                Ok(report) => {
                    let response: LoadTestResponse = report.into();
                    HttpResponse::Ok().json(response)
                }
                Err(e) => {
                    error!("Failed to generate report: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to generate report",
                        "details": e.to_string()
                    }))
                }
            }
        }
        Err(e) => {
            error!("Failed to run load test: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Failed to run load test",
                "details": e.to_string()
            }))
        }
    }
}
