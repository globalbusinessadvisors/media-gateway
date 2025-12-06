//! Discovery Service - Natural Language Search and Content Discovery
//!
//! Port: 8081
//! SLA: 99.9% availability
//! Latency target: <500ms p95

use actix_web::{web, App, HttpServer, HttpResponse};
use tracing::info;
use std::sync::Arc;
use media_gateway_discovery::{config, server};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("Starting Discovery Service on port 8081");

    // Load configuration
    let config = Arc::new(config::DiscoveryConfig::load()?);
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);

    info!("Discovery Service listening on {}", bind_addr);

    // Initialize service components
    let search_service = media_gateway_discovery::init_service(config.clone()).await?;

    // Create application state
    let app_state = web::Data::new(server::AppState {
        config: config.clone(),
        search_service,
    });

    // Start HTTP server with routes
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check))
            .route("/ready", web::get().to(readiness_check))
            .configure(server::configure_routes)
            .wrap(actix_web::middleware::Logger::default())
    })
    .workers(config.server.workers.unwrap_or_else(num_cpus::get))
    .bind(&bind_addr)?
    .run()
    .await?;

    Ok(())
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
