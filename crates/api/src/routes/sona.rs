use crate::middleware::auth::{get_user_context, AuthMiddleware};
use crate::proxy::{ProxyRequest, ServiceProxy};
use crate::rate_limit::RateLimiter;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sona")
            .wrap(AuthMiddleware::optional())
            .route("/recommendations", web::post().to(get_recommendations))
            .route(
                "/personalization/score",
                web::post().to(score_personalization),
            )
            .route(
                "/experiments/{id}/metrics",
                web::get().to(get_experiment_metrics),
            ),
    );
}

async fn get_recommendations(
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
                service: "sona".to_string(),
                path: "/recommendations".to_string(),
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

async fn score_personalization(
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
                service: "sona".to_string(),
                path: "/personalization/score".to_string(),
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

async fn get_experiment_metrics(
    req: HttpRequest,
    path: web::Path<String>,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let experiment_id = path.into_inner();
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
                service: "sona".to_string(),
                path: format!("/experiments/{}/metrics", experiment_id),
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
