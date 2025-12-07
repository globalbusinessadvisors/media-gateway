//! Security headers middleware for OWASP-compliant HTTP headers

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddleware { service }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            let headers = res.headers_mut();

            // HSTS - Force HTTPS for 1 year including subdomains
            headers.insert(
                actix_web::http::header::STRICT_TRANSPORT_SECURITY,
                "max-age=31536000; includeSubDomains".parse().unwrap(),
            );

            // Prevent clickjacking
            headers.insert(
                actix_web::http::header::X_FRAME_OPTIONS,
                "DENY".parse().unwrap(),
            );

            // Prevent MIME type sniffing
            headers.insert(
                actix_web::http::header::X_CONTENT_TYPE_OPTIONS,
                "nosniff".parse().unwrap(),
            );

            // Content Security Policy for API responses
            headers.insert(
                actix_web::http::header::CONTENT_SECURITY_POLICY,
                "default-src 'none'; frame-ancestors 'none'".parse().unwrap(),
            );

            // Referrer Policy
            headers.insert(
                actix_web::http::header::REFERRER_POLICY,
                "strict-origin-when-cross-origin".parse().unwrap(),
            );

            // Permissions Policy - restrict browser features
            headers.insert(
                "Permissions-Policy".parse().unwrap(),
                "geolocation=(), microphone=(), camera=()".parse().unwrap(),
            );

            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().body("test")
    }

    #[actix_web::test]
    async fn test_security_headers_applied() {
        let app = test::init_service(
            App::new()
                .wrap(SecurityHeaders)
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.headers().contains_key("strict-transport-security"));
        assert!(resp.headers().contains_key("x-frame-options"));
        assert!(resp.headers().contains_key("x-content-type-options"));
        assert!(resp.headers().contains_key("content-security-policy"));
        assert!(resp.headers().contains_key("referrer-policy"));
        assert!(resp.headers().contains_key("permissions-policy"));
    }
}
