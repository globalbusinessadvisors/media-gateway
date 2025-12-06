use crate::middleware::auth::{get_user_context, require_user_context, AuthMiddleware};
use crate::proxy::{ProxyRequest, ServiceProxy};
use crate::rate_limit::RateLimiter;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .wrap(AuthMiddleware::required())
            .route("/profile", web::get().to(get_profile))
            .route("/preferences", web::put().to(update_preferences))
            .route("/watchlist", web::get().to(get_watchlist))
            .route("/watchlist", web::post().to(add_to_watchlist))
            .route("/watchlist/{id}", web::delete().to(remove_from_watchlist))
            .route("/history", web::get().to(get_history)),
    );
}

async fn get_profile(
    req: HttpRequest,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let user_ctx = match require_user_context(&req) {
        Ok(ctx) => ctx,
        Err(err) => return HttpResponse::from_error(err),
    };

    match rate_limiter.check_rate_limit(&user_ctx.user_id, &user_ctx.tier).await {
        Ok(rate_info) => {
            let proxy_req = ProxyRequest {
                service: "auth".to_string(),
                path: "/api/v1/user/profile".to_string(),
                method: req.method().clone(),
                headers: req.headers().clone(),
                body: None,
                query: None,
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response.insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(err) => HttpResponse::from_error(err.into()),
            }
        }
        Err(err) => HttpResponse::from_error(err.into()),
    }
}

async fn update_preferences(
    req: HttpRequest,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let user_ctx = match require_user_context(&req) {
        Ok(ctx) => ctx,
        Err(err) => return HttpResponse::from_error(err),
    };

    match rate_limiter.check_rate_limit(&user_ctx.user_id, &user_ctx.tier).await {
        Ok(rate_info) => {
            let proxy_req = ProxyRequest {
                service: "auth".to_string(),
                path: "/api/v1/user/preferences".to_string(),
                method: req.method().clone(),
                headers: req.headers().clone(),
                body: Some(body),
                query: None,
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response.insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(err) => HttpResponse::from_error(err.into()),
            }
        }
        Err(err) => HttpResponse::from_error(err.into()),
    }
}

async fn get_watchlist(
    req: HttpRequest,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let user_ctx = match require_user_context(&req) {
        Ok(ctx) => ctx,
        Err(err) => return HttpResponse::from_error(err),
    };

    match rate_limiter.check_rate_limit(&user_ctx.user_id, &user_ctx.tier).await {
        Ok(rate_info) => {
            let proxy_req = ProxyRequest {
                service: "sync".to_string(),
                path: "/api/v1/user/watchlist".to_string(),
                method: req.method().clone(),
                headers: req.headers().clone(),
                body: None,
                query: req.query_string().is_empty().then(|| req.query_string().to_string()),
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response.insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(err) => HttpResponse::from_error(err.into()),
            }
        }
        Err(err) => HttpResponse::from_error(err.into()),
    }
}

async fn add_to_watchlist(
    req: HttpRequest,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let user_ctx = match require_user_context(&req) {
        Ok(ctx) => ctx,
        Err(err) => return HttpResponse::from_error(err),
    };

    match rate_limiter.check_rate_limit(&user_ctx.user_id, &user_ctx.tier).await {
        Ok(rate_info) => {
            let proxy_req = ProxyRequest {
                service: "sync".to_string(),
                path: "/api/v1/user/watchlist".to_string(),
                method: req.method().clone(),
                headers: req.headers().clone(),
                body: Some(body),
                query: None,
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response.insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(err) => HttpResponse::from_error(err.into()),
            }
        }
        Err(err) => HttpResponse::from_error(err.into()),
    }
}

async fn remove_from_watchlist(
    req: HttpRequest,
    path: web::Path<String>,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let user_ctx = match require_user_context(&req) {
        Ok(ctx) => ctx,
        Err(err) => return HttpResponse::from_error(err),
    };

    let content_id = path.into_inner();

    match rate_limiter.check_rate_limit(&user_ctx.user_id, &user_ctx.tier).await {
        Ok(rate_info) => {
            let proxy_req = ProxyRequest {
                service: "sync".to_string(),
                path: format!("/api/v1/user/watchlist/{}", content_id),
                method: req.method().clone(),
                headers: req.headers().clone(),
                body: None,
                query: None,
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response.insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(err) => HttpResponse::from_error(err.into()),
            }
        }
        Err(err) => HttpResponse::from_error(err.into()),
    }
}

async fn get_history(
    req: HttpRequest,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let user_ctx = match require_user_context(&req) {
        Ok(ctx) => ctx,
        Err(err) => return HttpResponse::from_error(err),
    };

    match rate_limiter.check_rate_limit(&user_ctx.user_id, &user_ctx.tier).await {
        Ok(rate_info) => {
            let proxy_req = ProxyRequest {
                service: "sync".to_string(),
                path: "/api/v1/user/history".to_string(),
                method: req.method().clone(),
                headers: req.headers().clone(),
                body: None,
                query: req.query_string().is_empty().then(|| req.query_string().to_string()),
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response.insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(err) => HttpResponse::from_error(err.into()),
            }
        }
        Err(err) => HttpResponse::from_error(err.into()),
    }
}
