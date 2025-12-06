use crate::proxy::{ProxyRequest, ServiceProxy};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/playback")
            .route("/sessions", web::post().to(create_session))
            .route("/sessions/{id}", web::get().to(get_session))
            .route("/sessions/{id}", web::delete().to(delete_session))
            .route("/sessions/{id}/position", web::patch().to(update_position))
            .route("/users/{user_id}/sessions", web::get().to(get_user_sessions))
    );
}

async fn create_session(
    req: HttpRequest,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    let proxy_req = ProxyRequest {
        service: "playback".to_string(),
        path: "/api/v1/sessions".to_string(),
        method: req.method().clone(),
        headers: req.headers().clone(),
        body: Some(body),
        query: req.uri().query().map(String::from),
    };

    match proxy.forward(proxy_req).await {
        Ok(response) => HttpResponse::build(response.status)
            .body(response.body),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn get_session(
    req: HttpRequest,
    path: web::Path<String>,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    let session_id = path.into_inner();
    let proxy_req = ProxyRequest {
        service: "playback".to_string(),
        path: format!("/api/v1/sessions/{}", session_id),
        method: req.method().clone(),
        headers: req.headers().clone(),
        body: None,
        query: req.uri().query().map(String::from),
    };

    match proxy.forward(proxy_req).await {
        Ok(response) => HttpResponse::build(response.status)
            .body(response.body),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn delete_session(
    req: HttpRequest,
    path: web::Path<String>,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    let session_id = path.into_inner();
    let proxy_req = ProxyRequest {
        service: "playback".to_string(),
        path: format!("/api/v1/sessions/{}", session_id),
        method: req.method().clone(),
        headers: req.headers().clone(),
        body: None,
        query: req.uri().query().map(String::from),
    };

    match proxy.forward(proxy_req).await {
        Ok(response) => HttpResponse::build(response.status)
            .body(response.body),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn update_position(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    let session_id = path.into_inner();
    let proxy_req = ProxyRequest {
        service: "playback".to_string(),
        path: format!("/api/v1/sessions/{}/position", session_id),
        method: req.method().clone(),
        headers: req.headers().clone(),
        body: Some(body),
        query: req.uri().query().map(String::from),
    };

    match proxy.forward(proxy_req).await {
        Ok(response) => HttpResponse::build(response.status)
            .body(response.body),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn get_user_sessions(
    req: HttpRequest,
    path: web::Path<String>,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    let user_id = path.into_inner();
    let proxy_req = ProxyRequest {
        service: "playback".to_string(),
        path: format!("/api/v1/users/{}/sessions", user_id),
        method: req.method().clone(),
        headers: req.headers().clone(),
        body: None,
        query: req.uri().query().map(String::from),
    };

    match proxy.forward(proxy_req).await {
        Ok(response) => HttpResponse::build(response.status)
            .body(response.body),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}
