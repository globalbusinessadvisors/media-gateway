use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::DiscoveryConfig;
use crate::search::{HybridSearchService, SearchFilters, SearchRequest};

/// Application state shared across all handlers
pub struct AppState {
    pub config: Arc<DiscoveryConfig>,
    pub search_service: Arc<HybridSearchService>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

/// Health check endpoint
async fn health() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        service: "discovery-service".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Hybrid search request payload
#[derive(Debug, Deserialize)]
pub struct HybridSearchRequest {
    pub query: String,
    pub filters: Option<SearchFiltersPayload>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Search filters payload
#[derive(Debug, Deserialize)]
pub struct SearchFiltersPayload {
    pub genres: Option<Vec<String>>,
    pub platforms: Option<Vec<String>>,
    pub year_range: Option<YearRange>,
    pub rating_range: Option<RatingRange>,
}

#[derive(Debug, Deserialize)]
pub struct YearRange {
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Deserialize)]
pub struct RatingRange {
    pub min: f32,
    pub max: f32,
}

/// POST /api/v1/search - Hybrid search endpoint
async fn hybrid_search(
    data: web::Data<AppState>,
    payload: web::Json<HybridSearchRequest>,
) -> impl Responder {
    let request = SearchRequest {
        query: payload.query.clone(),
        filters: payload.filters.as_ref().map(|f| SearchFilters {
            genres: f.genres.clone().unwrap_or_default(),
            platforms: f.platforms.clone().unwrap_or_default(),
            year_range: f.year_range.as_ref().map(|r| (r.min, r.max)),
            rating_range: f.rating_range.as_ref().map(|r| (r.min, r.max)),
        }),
        page: payload.page.unwrap_or(1),
        page_size: payload.page_size.unwrap_or(data.config.search.page_size as u32),
        user_id: None, // TODO: Extract from auth context
    };

    match data.search_service.search(request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            tracing::error!("Search error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Search failed",
                "message": e.to_string()
            }))
        }
    }
}

/// Semantic search request (vector-only)
#[derive(Debug, Deserialize)]
pub struct SemanticSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub filters: Option<SearchFiltersPayload>,
}

/// POST /api/v1/search/semantic - Vector-only search
async fn semantic_search(
    data: web::Data<AppState>,
    payload: web::Json<SemanticSearchRequest>,
) -> impl Responder {
    let filters = payload.filters.as_ref().map(|f| SearchFilters {
        genres: f.genres.clone().unwrap_or_default(),
        platforms: f.platforms.clone().unwrap_or_default(),
        year_range: f.year_range.as_ref().map(|r| (r.min, r.max)),
        rating_range: f.rating_range.as_ref().map(|r| (r.min, r.max)),
    });

    match data
        .search_service
        .vector_search(&payload.query, filters, payload.limit)
        .await
    {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => {
            tracing::error!("Semantic search error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Semantic search failed",
                "message": e.to_string()
            }))
        }
    }
}

/// Keyword search request (BM25-only)
#[derive(Debug, Deserialize)]
pub struct KeywordSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub filters: Option<SearchFiltersPayload>,
}

/// POST /api/v1/search/keyword - Keyword-only search
async fn keyword_search(
    data: web::Data<AppState>,
    payload: web::Json<KeywordSearchRequest>,
) -> impl Responder {
    let filters = payload.filters.as_ref().map(|f| SearchFilters {
        genres: f.genres.clone().unwrap_or_default(),
        platforms: f.platforms.clone().unwrap_or_default(),
        year_range: f.year_range.as_ref().map(|r| (r.min, r.max)),
        rating_range: f.rating_range.as_ref().map(|r| (r.min, r.max)),
    });

    match data
        .search_service
        .keyword_search(&payload.query, filters, payload.limit)
        .await
    {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => {
            tracing::error!("Keyword search error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Keyword search failed",
                "message": e.to_string()
            }))
        }
    }
}

/// Content lookup response
#[derive(Debug, Serialize)]
pub struct ContentResponse {
    pub id: Uuid,
    pub title: String,
    pub overview: String,
    pub release_year: i32,
    pub genres: Vec<String>,
}

/// GET /api/v1/content/{id} - Content lookup endpoint
async fn get_content(
    data: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let content_id = path.into_inner();

    match data.search_service.get_content_by_id(content_id).await {
        Ok(Some(content)) => HttpResponse::Ok().json(content),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Content not found",
            "id": content_id
        })),
        Err(e) => {
            tracing::error!("Content lookup error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Content lookup failed",
                "message": e.to_string()
            }))
        }
    }
}

/// Configure application routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health))
            .route("/search", web::post().to(hybrid_search))
            .route("/search/semantic", web::post().to(semantic_search))
            .route("/search/keyword", web::post().to(keyword_search))
            .route("/content/{id}", web::get().to(get_content)),
    );
}

/// Start HTTP server
pub async fn start_server(
    config: Arc<DiscoveryConfig>,
    search_service: Arc<HybridSearchService>,
) -> anyhow::Result<()> {
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);

    tracing::info!("Starting Discovery Service on {}", bind_addr);

    let app_state = web::Data::new(AppState {
        config: config.clone(),
        search_service,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes)
            .wrap(actix_web::middleware::Logger::default())
    })
    .workers(config.server.workers.unwrap_or_else(num_cpus::get))
    .bind(&bind_addr)?
    .run()
    .await?;

    Ok(())
}
