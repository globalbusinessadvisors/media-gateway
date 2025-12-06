//! SONA Personalization Engine HTTP Server
//!
//! Actix-web server providing REST API for personalization services.
//! Runs on port 8082 as specified in SPARC architecture.

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use media_gateway_sona::{SonaEngine, SonaConfig};

/// Health check endpoint
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "sona-personalization-engine",
        "version": "0.1.0"
    }))
}

/// Recommendation request
#[derive(Debug, Deserialize)]
struct RecommendationRequest {
    user_id: Uuid,
    context: Option<RecommendationContextDto>,
    limit: Option<usize>,
    exclude_watched: Option<bool>,
    diversity_threshold: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct RecommendationContextDto {
    mood: Option<String>,
    time_of_day: Option<String>,
    device_type: Option<String>,
    viewing_with: Option<Vec<String>>,
}

/// Recommendation response
#[derive(Debug, Serialize)]
struct RecommendationResponse {
    recommendations: Vec<RecommendationDto>,
    generated_at: String,
    ttl_seconds: u32,
}

#[derive(Debug, Serialize)]
struct RecommendationDto {
    content_id: Uuid,
    confidence_score: f32,
    recommendation_type: String,
    based_on: Vec<String>,
    explanation: String,
}

/// POST /api/v1/recommendations
async fn get_recommendations(
    _req: web::Json<RecommendationRequest>,
    _engine: web::Data<Arc<SonaEngine>>,
) -> impl Responder {
    // Simulated response - in real implementation, call GenerateRecommendations
    HttpResponse::Ok().json(RecommendationResponse {
        recommendations: vec![],
        generated_at: chrono::Utc::now().to_rfc3339(),
        ttl_seconds: 3600,
    })
}

/// Similar content request
#[derive(Debug, Deserialize)]
struct SimilarContentRequest {
    content_id: Uuid,
    limit: Option<usize>,
}

/// POST /api/v1/recommendations/similar
async fn get_similar_content(
    _req: web::Json<SimilarContentRequest>,
    _engine: web::Data<Arc<SonaEngine>>,
) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "similar_content": []
    }))
}

/// Personalization score request
#[derive(Debug, Deserialize)]
struct PersonalizationScoreRequest {
    user_id: Uuid,
    content_id: Uuid,
}

/// POST /api/v1/personalization/score
async fn get_personalization_score(
    _req: web::Json<PersonalizationScoreRequest>,
    _engine: web::Data<Arc<SonaEngine>>,
) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "user_id": _req.user_id,
        "content_id": _req.content_id,
        "score": 0.75,
        "components": {
            "collaborative": 0.35,
            "content_based": 0.25,
            "graph_based": 0.30,
            "context": 0.10
        }
    }))
}

/// User profile update request
#[derive(Debug, Deserialize)]
struct ProfileUpdateRequest {
    user_id: Uuid,
    viewing_events: Vec<ViewingEventDto>,
}

#[derive(Debug, Deserialize)]
struct ViewingEventDto {
    content_id: Uuid,
    timestamp: String,
    completion_rate: f32,
    rating: Option<u8>,
    is_rewatch: bool,
    dismissed: bool,
}

/// POST /api/v1/profile/update
async fn update_profile(
    _req: web::Json<ProfileUpdateRequest>,
    _engine: web::Data<Arc<SonaEngine>>,
) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "updated",
        "user_id": _req.user_id,
        "events_processed": _req.viewing_events.len()
    }))
}

/// LoRA training request
#[derive(Debug, Deserialize)]
struct LoraTrainingRequest {
    user_id: Uuid,
    force: Option<bool>,
}

/// POST /api/v1/lora/train
async fn trigger_lora_training(
    _req: web::Json<LoraTrainingRequest>,
    _engine: web::Data<Arc<SonaEngine>>,
) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "training_queued",
        "user_id": _req.user_id,
        "estimated_duration_ms": 500
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .json()
        .init();

    tracing::info!("Starting SONA Personalization Engine on port 8082");

    // Initialize SONA engine
    let config = SonaConfig::default();
    let engine = Arc::new(SonaEngine::new(config));

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(engine.clone()))
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api/v1")
                    .route("/recommendations", web::post().to(get_recommendations))
                    .route("/recommendations/similar", web::post().to(get_similar_content))
                    .route("/personalization/score", web::post().to(get_personalization_score))
                    .route("/profile/update", web::post().to(update_profile))
                    .route("/lora/train", web::post().to(trigger_lora_training))
            )
    })
    .bind(("0.0.0.0", 8082))?
    .run()
    .await
}
