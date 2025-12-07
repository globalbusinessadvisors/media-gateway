use crate::middleware::auth::{get_user_context, AuthMiddleware};
use crate::proxy::{ProxyRequest, ServiceProxy};
use crate::rate_limit::RateLimiter;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sync")
            .wrap(AuthMiddleware::optional())
            .route("/watchlist", web::post().to(sync_watchlist))
            .route("/progress", web::post().to(sync_progress))
            .route("/devices", web::get().to(list_devices))
            .route("/devices/handoff", web::post().to(device_handoff)),
    );
}

async fn sync_watchlist(
    req: HttpRequest,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let user_ctx = get_user_context(&req);
    let user_id = user_ctx
        .as_ref()
        .map(|u| u.user_id.as_str())
        .unwrap_or("anonymous");
    let tier = user_ctx
        .as_ref()
        .map(|u| u.tier.as_str())
        .unwrap_or("anonymous");

    match rate_limiter.check_rate_limit(user_id, tier).await {
        Ok(rate_info) => {
            // Convert actix-web headers to reqwest headers
            let mut headers = reqwest::header::HeaderMap::new();
            for (key, value) in req.headers() {
                if let Ok(value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
                    headers.insert(key.clone(), value);
                }
            }

            let proxy_req = ProxyRequest {
                service: "sync".to_string(),
                path: "/api/v1/sync/watchlist".to_string(),
                method: req.method().clone(),
                headers,
                body: Some(body),
                query: req.uri().query().map(String::from),
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response
                        .insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                })),
            }
        }
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn sync_progress(
    req: HttpRequest,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let user_ctx = get_user_context(&req);
    let user_id = user_ctx
        .as_ref()
        .map(|u| u.user_id.as_str())
        .unwrap_or("anonymous");
    let tier = user_ctx
        .as_ref()
        .map(|u| u.tier.as_str())
        .unwrap_or("anonymous");

    match rate_limiter.check_rate_limit(user_id, tier).await {
        Ok(rate_info) => {
            // Convert actix-web headers to reqwest headers
            let mut headers = reqwest::header::HeaderMap::new();
            for (key, value) in req.headers() {
                if let Ok(value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
                    headers.insert(key.clone(), value);
                }
            }

            let proxy_req = ProxyRequest {
                service: "sync".to_string(),
                path: "/api/v1/sync/progress".to_string(),
                method: req.method().clone(),
                headers,
                body: Some(body),
                query: req.uri().query().map(String::from),
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response
                        .insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                })),
            }
        }
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn list_devices(
    req: HttpRequest,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let user_ctx = get_user_context(&req);
    let user_id = user_ctx
        .as_ref()
        .map(|u| u.user_id.as_str())
        .unwrap_or("anonymous");
    let tier = user_ctx
        .as_ref()
        .map(|u| u.tier.as_str())
        .unwrap_or("anonymous");

    match rate_limiter.check_rate_limit(user_id, tier).await {
        Ok(rate_info) => {
            // Convert actix-web headers to reqwest headers
            let mut headers = reqwest::header::HeaderMap::new();
            for (key, value) in req.headers() {
                if let Ok(value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
                    headers.insert(key.clone(), value);
                }
            }

            let proxy_req = ProxyRequest {
                service: "sync".to_string(),
                path: "/api/v1/devices".to_string(),
                method: req.method().clone(),
                headers,
                body: None,
                query: req.uri().query().map(String::from),
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response
                        .insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                })),
            }
        }
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn device_handoff(
    req: HttpRequest,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let user_ctx = get_user_context(&req);
    let user_id = user_ctx
        .as_ref()
        .map(|u| u.user_id.as_str())
        .unwrap_or("anonymous");
    let tier = user_ctx
        .as_ref()
        .map(|u| u.tier.as_str())
        .unwrap_or("anonymous");

    match rate_limiter.check_rate_limit(user_id, tier).await {
        Ok(rate_info) => {
            // Convert actix-web headers to reqwest headers
            let mut headers = reqwest::header::HeaderMap::new();
            for (key, value) in req.headers() {
                if let Ok(value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
                    headers.insert(key.clone(), value);
                }
            }

            let proxy_req = ProxyRequest {
                service: "sync".to_string(),
                path: "/api/v1/devices/handoff".to_string(),
                method: req.method().clone(),
                headers,
                body: Some(body),
                query: req.uri().query().map(String::from),
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response
                        .insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                })),
            }
        }
        Err(err) => HttpResponse::from_error(err),
    }
}
