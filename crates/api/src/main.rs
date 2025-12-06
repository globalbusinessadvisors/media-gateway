//! API Gateway - HTTP Gateway and Request Router
//!
//! Port: 8080
//! SLA: 99.9% availability
//! Latency target: <100ms p95

use actix_web::{web, App, HttpServer, HttpResponse};
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("Starting API Gateway on port 8080");

    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
            .route("/api/v1/status", web::get().to(api_status))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "api-gateway",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn api_status() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "api_version": "v1",
        "platform": "Media Gateway",
        "status": "operational"
    }))
}
