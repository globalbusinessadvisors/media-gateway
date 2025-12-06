//! Discovery Service - Natural Language Search and Content Discovery
//!
//! Port: 8081
//! SLA: 99.9% availability
//! Latency target: <500ms p95

use actix_web::{web, App, HttpServer, HttpResponse};
use tracing::{info, warn};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("Starting Discovery Service on port 8081");

    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
            .route("/ready", web::get().to(readiness_check))
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "discovery-service",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn readiness_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ready"
    }))
}
