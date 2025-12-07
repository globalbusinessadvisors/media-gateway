//! OpenTelemetry OTLP Integration Example
//!
//! This example demonstrates how to use the complete OTLP tracing integration
//! with a simple Actix Web service that exports traces to Jaeger.
//!
//! # Prerequisites
//!
//! 1. Start Jaeger with OTLP support:
//!    ```bash
//!    docker run -d --name jaeger \
//!      -e COLLECTOR_OTLP_ENABLED=true \
//!      -p 4317:4317 \
//!      -p 16686:16686 \
//!      jaegertracing/all-in-one:1.51
//!    ```
//!
//! 2. Set environment variables:
//!    ```bash
//!    export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
//!    export SERVICE_NAME=example-service
//!    export RUST_ENV=development
//!    ```
//!
//! 3. Run the example:
//!    ```bash
//!    cargo run --example telemetry_otlp_example
//!    ```
//!
//! 4. Generate traces:
//!    ```bash
//!    curl http://localhost:8080/api/users/123
//!    curl http://localhost:8080/api/sessions/abc123
//!    ```
//!
//! 5. View traces in Jaeger UI: http://localhost:16686

use actix_web::{web, App, HttpResponse, HttpServer, Result};
use media_gateway_core::telemetry::{
    create_span, db_query_span, external_api_span, init_tracing, redis_op_span,
    shutdown_tracing, TracingConfig, TracingMiddleware,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, instrument};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i64,
    username: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Session {
    session_id: String,
    user_id: i64,
    expires_at: i64,
}

/// Health check endpoint
#[instrument]
async fn health_check() -> Result<HttpResponse> {
    info!("Health check requested");
    Ok(HttpResponse::Ok().body("OK"))
}

/// Get user by ID - demonstrates database query tracing
#[instrument(name = "get_user", skip(user_id))]
async fn get_user(user_id: web::Path<i64>) -> Result<HttpResponse> {
    let user_id = user_id.into_inner();
    info!(user_id = %user_id, "Fetching user");

    // Simulate database query with custom span
    let user = fetch_user_from_db(user_id).await?;

    // Simulate external API call
    notify_user_access(user_id).await?;

    Ok(HttpResponse::Ok().json(user))
}

/// Get session - demonstrates Redis operation tracing
#[instrument(name = "get_session", skip(session_id))]
async fn get_session(session_id: web::Path<String>) -> Result<HttpResponse> {
    let session_id = session_id.into_inner();
    info!(session_id = %session_id, "Fetching session");

    let session = fetch_session_from_redis(&session_id).await?;

    Ok(HttpResponse::Ok().json(session))
}

/// Simulate database query with tracing
async fn fetch_user_from_db(user_id: i64) -> Result<User> {
    // Create database query span
    let _span = db_query_span(
        &format!("SELECT * FROM users WHERE id = {}", user_id),
        "users",
    );

    // Simulate database latency
    tokio::time::sleep(Duration::from_millis(50)).await;

    info!(user_id = %user_id, "User fetched from database");

    Ok(User {
        id: user_id,
        username: format!("user_{}", user_id),
        email: format!("user_{}@example.com", user_id),
    })
}

/// Simulate Redis operation with tracing
async fn fetch_session_from_redis(session_id: &str) -> Result<Session> {
    // Create Redis operation span
    let _span = redis_op_span("GET", &format!("session:{}", session_id));

    // Simulate Redis latency
    tokio::time::sleep(Duration::from_millis(10)).await;

    info!(session_id = %session_id, "Session fetched from Redis");

    Ok(Session {
        session_id: session_id.to_string(),
        user_id: 123,
        expires_at: chrono::Utc::now().timestamp() + 3600,
    })
}

/// Simulate external API call with tracing
async fn notify_user_access(user_id: i64) -> Result<()> {
    // Create external API span
    let _span = external_api_span(
        "POST",
        "https://api.analytics.example.com/events",
        "analytics-service",
    );

    // Simulate API latency
    tokio::time::sleep(Duration::from_millis(100)).await;

    info!(user_id = %user_id, "User access notification sent");

    Ok(())
}

/// Complex business operation with custom span attributes
#[instrument(name = "process_checkout")]
async fn process_checkout(
    cart_id: web::Path<String>,
) -> Result<HttpResponse> {
    let cart_id = cart_id.into_inner();

    // Create custom span with attributes
    let _span = create_span(
        "checkout_processing",
        &[
            ("cart_id", cart_id.as_str()),
            ("payment_method", "credit_card"),
            ("amount", "99.99"),
            ("currency", "USD"),
        ],
    );

    info!(cart_id = %cart_id, "Processing checkout");

    // Simulate multiple operations
    validate_cart(&cart_id).await?;
    process_payment(&cart_id).await?;
    create_order(&cart_id).await?;

    info!(cart_id = %cart_id, "Checkout completed successfully");

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "cart_id": cart_id,
        "order_id": "order_123",
    })))
}

async fn validate_cart(cart_id: &str) -> Result<()> {
    let _span = db_query_span(
        &format!("SELECT * FROM carts WHERE id = '{}'", cart_id),
        "carts",
    );
    tokio::time::sleep(Duration::from_millis(30)).await;
    info!(cart_id = %cart_id, "Cart validated");
    Ok(())
}

async fn process_payment(cart_id: &str) -> Result<()> {
    let _span = external_api_span(
        "POST",
        "https://api.stripe.com/v1/charges",
        "stripe",
    );
    tokio::time::sleep(Duration::from_millis(200)).await;
    info!(cart_id = %cart_id, "Payment processed");
    Ok(())
}

async fn create_order(cart_id: &str) -> Result<()> {
    let _span = db_query_span(
        &format!("INSERT INTO orders (cart_id, status) VALUES ('{}', 'completed')", cart_id),
        "orders",
    );
    tokio::time::sleep(Duration::from_millis(40)).await;
    info!(cart_id = %cart_id, "Order created");
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing with OTLP exporter
    let config = TracingConfig::from_env();

    println!("=== OpenTelemetry OTLP Example ===");
    println!("Service: {}", config.service_name);
    println!("OTLP Endpoint: {}", config.otlp_endpoint);
    println!("Sampling Rate: {}%", config.sampling_rate * 100.0);
    println!("Console Logging: {}", config.enable_console);
    println!();

    init_tracing(config)
        .await
        .expect("Failed to initialize tracing");

    info!("Starting example service with OTLP tracing");

    // Start HTTP server with tracing middleware
    let server = HttpServer::new(|| {
        App::new()
            // Add tracing middleware for automatic request tracing
            .wrap(TracingMiddleware)
            // Routes
            .route("/health", web::get().to(health_check))
            .route("/api/users/{user_id}", web::get().to(get_user))
            .route("/api/sessions/{session_id}", web::get().to(get_session))
            .route("/api/checkout/{cart_id}", web::post().to(process_checkout))
    })
    .bind("0.0.0.0:8080")?
    .run();

    println!("Server running at http://localhost:8080");
    println!();
    println!("Try these endpoints:");
    println!("  curl http://localhost:8080/health");
    println!("  curl http://localhost:8080/api/users/123");
    println!("  curl http://localhost:8080/api/sessions/abc123");
    println!("  curl -X POST http://localhost:8080/api/checkout/cart_456");
    println!();
    println!("View traces in Jaeger UI: http://localhost:16686");
    println!("  Service: {}", std::env::var("SERVICE_NAME").unwrap_or_else(|_| "media-gateway".to_string()));
    println!();

    // Run server
    let result = server.await;

    // Graceful shutdown - flush remaining spans
    info!("Shutting down service");
    shutdown_tracing()
        .await
        .expect("Failed to shutdown tracing");

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_health_check() {
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = health_check().await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_fetch_user_from_db() {
        let user = fetch_user_from_db(123).await.unwrap();
        assert_eq!(user.id, 123);
        assert_eq!(user.username, "user_123");
    }

    #[actix_web::test]
    async fn test_fetch_session_from_redis() {
        let session = fetch_session_from_redis("test_session").await.unwrap();
        assert_eq!(session.session_id, "test_session");
        assert_eq!(session.user_id, 123);
    }
}
