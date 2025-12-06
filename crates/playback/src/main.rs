//! Playback Service - Device Management and Deep Linking
//!
//! Port: 8086
//! SLA: 99.5% availability

mod session;

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use session::{
    SessionManager, PlaybackSession, CreateSessionRequest,
    UpdatePositionRequest, SessionError,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Application state
struct AppState {
    session_manager: Arc<SessionManager>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("Starting Playback Service on port 8086");

    let session_manager = SessionManager::from_env()
        .expect("Failed to connect to Redis");

    let state = web::Data::new(AppState {
        session_manager: Arc::new(session_manager),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/health", web::get().to(health_check))
            .route("/ready", web::get().to(readiness_check))
            .service(
                web::scope("/api/v1")
                    .route("/sessions", web::post().to(create_session))
                    .route("/sessions/{id}", web::get().to(get_session))
                    .route("/sessions/{id}", web::delete().to(delete_session))
                    .route("/sessions/{id}/position", web::patch().to(update_position))
                    .route("/users/{user_id}/sessions", web::get().to(get_user_sessions))
            )
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

async fn readiness_check(state: web::Data<AppState>) -> HttpResponse {
    let redis_healthy = state.session_manager.is_healthy().await;

    if redis_healthy {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "redis": "connected"
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "redis": "disconnected"
        }))
    }
}

async fn create_session(
    state: web::Data<AppState>,
    request: web::Json<CreateSessionRequest>,
) -> Result<HttpResponse, SessionError> {
    let session = state.session_manager.create(request.into_inner()).await?;
    Ok(HttpResponse::Created().json(session))
}

async fn get_session(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, SessionError> {
    let session_id = path.into_inner();
    let session = state.session_manager.get(session_id).await?
        .ok_or(SessionError::NotFound)?;
    Ok(HttpResponse::Ok().json(session))
}

async fn delete_session(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, SessionError> {
    let session_id = path.into_inner();
    state.session_manager.delete(session_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn update_position(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<UpdatePositionRequest>,
) -> Result<HttpResponse, SessionError> {
    let session_id = path.into_inner();
    let session = state.session_manager
        .update_position(session_id, request.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(session))
}

async fn get_user_sessions(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, SessionError> {
    let user_id = path.into_inner();
    let sessions = state.session_manager.get_user_sessions(user_id).await?;
    Ok(HttpResponse::Ok().json(sessions))
}
