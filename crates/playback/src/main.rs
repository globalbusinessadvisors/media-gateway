//! Playback Service - Device Management and Deep Linking
//!
//! Port: 8086
//! SLA: 99.5% availability

use actix_web::{web, App, HttpServer, HttpResponse};
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("Starting Playback Service on port 8086");

    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", 8086))?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "playback-service",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
