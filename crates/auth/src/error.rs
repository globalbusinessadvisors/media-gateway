use actix_web::{HttpResponse, ResponseError};
use std::fmt;

pub type Result<T> = std::result::Result<T, AuthError>;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid PKCE verifier")]
    InvalidPkceVerifier,

    #[error("Invalid authorization code")]
    InvalidAuthCode,

    #[error("Authorization code already used")]
    AuthCodeReused,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    #[error("Session not found")]
    SessionNotFound,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid redirect URI")]
    InvalidRedirectUri,

    #[error("Invalid client")]
    InvalidClient,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Device code not found")]
    DeviceCodeNotFound,

    #[error("Authorization pending")]
    AuthorizationPending,

    #[error("Access denied")]
    AccessDenied,

    #[error("Device code expired")]
    DeviceCodeExpired,
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::InvalidCredentials => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "invalid_credentials",
                    "error_description": "Invalid username or password"
                }))
            }
            AuthError::InvalidToken(_) | AuthError::TokenExpired => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "invalid_token",
                    "error_description": self.to_string()
                }))
            }
            AuthError::InvalidPkceVerifier => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "invalid_grant",
                    "error_description": "Invalid code verifier"
                }))
            }
            AuthError::InvalidAuthCode | AuthError::AuthCodeReused => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "invalid_grant",
                    "error_description": self.to_string()
                }))
            }
            AuthError::InsufficientPermissions => {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "insufficient_scope",
                    "error_description": "Insufficient permissions"
                }))
            }
            AuthError::InvalidScope(scope) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "invalid_scope",
                    "error_description": format!("Invalid scope: {}", scope)
                }))
            }
            AuthError::InvalidRedirectUri => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "invalid_request",
                    "error_description": "Invalid redirect URI"
                }))
            }
            AuthError::InvalidClient => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "invalid_client",
                    "error_description": "Invalid client ID"
                }))
            }
            AuthError::RateLimitExceeded => {
                HttpResponse::TooManyRequests().json(serde_json::json!({
                    "error": "rate_limit_exceeded",
                    "error_description": "Too many requests"
                }))
            }
            AuthError::AuthorizationPending => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "authorization_pending",
                    "error_description": "User has not yet completed authorization"
                }))
            }
            AuthError::AccessDenied => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "access_denied",
                    "error_description": "User denied authorization"
                }))
            }
            AuthError::DeviceCodeExpired => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "expired_token",
                    "error_description": "Device code expired"
                }))
            }
            _ => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "server_error",
                    "error_description": "Internal server error"
                }))
            }
        }
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::Database(err.to_string())
    }
}

impl From<redis::RedisError> for AuthError {
    fn from(err: redis::RedisError) -> Self {
        AuthError::Redis(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AuthError::InvalidToken(err.to_string())
    }
}
