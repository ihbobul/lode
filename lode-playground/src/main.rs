use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    message: String,
    timestamp: String,
    processing_time_ms: u64,
    request_info: Option<RequestInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestInfo {
    method: String,
    headers: Vec<(String, String)>,
    body_size: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserData {
    name: String,
    email: String,
    data: String,
}

async fn handle_request(req: HttpRequest, body: Option<web::Json<UserData>>) -> impl Responder {
    let start = Instant::now();

    println!("Received request: {} {}", req.method(), req.path());
    println!("Headers: {:?}", req.headers());
    if let Some(ref body) = body {
        println!("Body: {:?}", body.0);
    }

    let headers: Vec<(String, String)> = req
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                value.to_str().unwrap_or("invalid").to_string(),
            )
        })
        .collect();

    let request_info = RequestInfo {
        method: req.method().to_string(),
        headers,
        body_size: body
            .as_ref()
            .map(|b| serde_json::to_string(&b.0).unwrap().len()),
    };

    let response = Response {
        message: "Request processed".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        processing_time_ms: start.elapsed().as_millis() as u64,
        request_info: Some(request_info),
    };

    HttpResponse::Ok().json(response)
}

async fn auth_required(req: HttpRequest) -> impl Responder {
    let start = Instant::now();

    println!("Auth request received");
    println!("Auth header: {:?}", req.headers().get("Authorization"));

    match req.headers().get("Authorization") {
        Some(auth_header) => {
            if let Ok(auth_str) = auth_header.to_str() {
                println!("Auth string: {}", auth_str);
                if auth_str.starts_with("Bearer ") {
                    let token = auth_str.trim_start_matches("Bearer ");
                    println!("Token: {}", token);
                    if token == "test-token" {
                        let response = Response {
                            message: "Authorized access granted".to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            processing_time_ms: start.elapsed().as_millis() as u64,
                            request_info: None,
                        };
                        return HttpResponse::Ok().json(response);
                    }
                }
            }
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid authorization header format"
            }))
        }
        None => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Authorization header required"
        })),
    }
}

async fn slow_post(body: web::Json<UserData>) -> impl Responder {
    let start = Instant::now();

    println!("Received POST request with body: {:?}", body.0);

    let processing_time = std::cmp::min(body.data.len() as u64, 500);
    tokio::time::sleep(Duration::from_millis(processing_time)).await;

    let response = Response {
        message: format!("Processed data for user: {}", body.name),
        timestamp: chrono::Utc::now().to_rfc3339(),
        processing_time_ms: start.elapsed().as_millis() as u64,
        request_info: Some(RequestInfo {
            method: "POST".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body_size: Some(serde_json::to_string(&body.0).unwrap().len()),
        }),
    };

    HttpResponse::Ok().json(response)
}

async fn error_simulation(req: HttpRequest) -> impl Responder {
    let start = Instant::now();

    println!("Error simulation request received");
    println!("Error type header: {:?}", req.headers().get("X-Error-Type"));

    match req.headers().get("X-Error-Type") {
        Some(error_type) => match error_type.to_str().unwrap_or("500") {
            "404" => HttpResponse::NotFound().json(serde_json::json!({
                "error": "Not Found",
                "message": "The requested resource was not found on this server",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "processing_time_ms": start.elapsed().as_millis()
            })),
            "429" => HttpResponse::TooManyRequests().json(serde_json::json!({
                "error": "Too Many Requests",
                "message": "Rate limit exceeded. Please try again later.",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "processing_time_ms": start.elapsed().as_millis()
            })),
            "503" => HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "Service Unavailable",
                "message": "The server is temporarily unable to handle your request",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "processing_time_ms": start.elapsed().as_millis()
            })),
            _ => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal Server Error",
                "message": "An unexpected error occurred on the server",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "processing_time_ms": start.elapsed().as_millis()
            })),
        },
        None => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Internal Server Error",
            "message": "No error type specified. Defaulting to internal server error.",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "processing_time_ms": start.elapsed().as_millis()
        })),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting lode-playground server on http://localhost:8080");
    HttpServer::new(|| {
        App::new()
            .route("/api/v1/data", web::post().to(handle_request))
            .route("/api/v1/data", web::get().to(handle_request))
            .route("/api/v1/auth", web::get().to(auth_required))
            .route("/api/v1/process", web::post().to(slow_post))
            .route("/api/v1/error", web::get().to(error_simulation))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
