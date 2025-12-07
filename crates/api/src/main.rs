//! API Gateway - HTTP Gateway and Request Router
//!
//! Port: 8080
//! SLA: 99.9% availability
//! Latency target: <100ms p95

use actix_web::{web, App, HttpResponse, HttpServer};
use media_gateway_api::middleware::SecurityHeaders;
use media_gateway_api::routes;
use media_gateway_core::{metrics_handler, MetricsMiddleware};
use tokio::signal;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Media Gateway API",
        version = "1.0.0",
        description = "Unified media content gateway with intelligent search, personalization, and cross-platform synchronization",
        contact(
            name = "Media Gateway Team",
            email = "api@mediagateway.com"
        ),
        license(
            name = "MIT"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.mediagateway.com", description = "Production server")
    ),
    tags(
        (name = "health", description = "Health check and system status endpoints"),
        (name = "user", description = "User profile, preferences, watchlist, and history management"),
        (name = "content", description = "Content metadata, availability, and trending information"),
        (name = "search", description = "Search functionality including semantic and autocomplete"),
        (name = "discovery", description = "Content discovery for movies and TV shows"),
        (name = "playback", description = "Playback session management and progress tracking"),
        (name = "sona", description = "SONA AI recommendations and personalization"),
        (name = "sync", description = "Cross-device synchronization for watchlist, progress, and handoff")
    ),
    components(
        schemas(),
        responses(),
        security_schemes(
            ("bearer_auth" = (
                type = ApiKey,
                in = Header,
                name = "Authorization",
                description = "JWT bearer token authentication. Format: 'Bearer {token}'"
            ))
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    // Validate JWT_SECRET is set before starting server
    std::env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set - cannot start with default secret");

    info!("Starting API Gateway on port 8080");

    let openapi = ApiDoc::openapi();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(SecurityHeaders)
            .wrap(MetricsMiddleware)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone())
            )
            .route("/health", web::get().to(health_check))
            .route("/ready", web::get().to(readiness_check))
            .route("/liveness", web::get().to(liveness_check))
            .route("/metrics", web::get().to(metrics_handler))
            .route("/api/v1/status", web::get().to(api_status))
            .configure(routes::configure)
    })
    .bind(("0.0.0.0", 8080))?
    .shutdown_timeout(30) // 30 second graceful shutdown
    .run();

    let server_handle = server.handle();

    // Spawn a task to handle shutdown signals
    let shutdown_handle = server_handle.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        tracing::info!("Shutdown signal received, initiating graceful shutdown");
        shutdown_handle.stop(true).await;
    });

    server.await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
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

async fn readiness_check() -> HttpResponse {
    // Check dependencies - for standalone main.rs, we do basic checks
    // In production, this would check database, Redis, etc.
    let redis_ok = std::env::var("REDIS_URL").is_ok();
    let db_ok = std::env::var("DATABASE_URL").is_ok();

    let ready = redis_ok && db_ok;
    let status_code = if ready {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(serde_json::json!({
        "ready": ready,
        "checks": {
            "database": db_ok,
            "redis": redis_ok
        }
    }))
}

async fn liveness_check() -> HttpResponse {
    // Liveness is lightweight - just confirm the service is alive
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
