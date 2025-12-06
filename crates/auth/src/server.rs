use crate::{
    error::{AuthError, Result},
    jwt::JwtManager,
    oauth::{
        device::{DeviceAuthorizationResponse, DeviceCode},
        pkce::{AuthorizationCode, PkceChallenge},
        OAuthConfig, OAuthManager,
    },
    rbac::RbacManager,
    scopes::ScopeManager,
    session::SessionManager,
    storage::AuthStorage,
    token::TokenManager,
};
use actix_web::{
    get, post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Application state shared across handlers
pub struct AppState {
    pub jwt_manager: Arc<JwtManager>,
    pub session_manager: Arc<SessionManager>,
    pub oauth_manager: Arc<OAuthManager>,
    pub rbac_manager: Arc<RbacManager>,
    pub scope_manager: Arc<ScopeManager>,
    pub storage: Arc<AuthStorage>,
}

// ============================================================================
// Health Check
// ============================================================================

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "auth-service",
        "version": "0.1.0"
    }))
}

// ============================================================================
// OAuth 2.0 Authorization Endpoint
// ============================================================================

#[derive(Debug, Deserialize)]
struct AuthorizeRequest {
    client_id: String,
    redirect_uri: String,
    response_type: String,
    scope: String,
    code_challenge: String,
    code_challenge_method: String,
    state: Option<String>,
}

#[get("/auth/authorize")]
async fn authorize(
    query: web::Query<AuthorizeRequest>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    // Validate response_type
    if query.response_type != "code" {
        return Err(AuthError::InvalidClient);
    }

    // Validate code_challenge_method
    if query.code_challenge_method != "S256" {
        return Err(AuthError::InvalidPkceVerifier);
    }

    // Validate client and redirect URI
    state
        .oauth_manager
        .validate_redirect_uri(&query.client_id, &query.redirect_uri)?;

    // Parse and validate scopes
    let scopes: Vec<String> = query.scope.split_whitespace().map(|s| s.to_string()).collect();

    // Store PKCE session
    let pkce = PkceChallenge {
        code_verifier: String::new(), // Client keeps this
        code_challenge: query.code_challenge.clone(),
        code_challenge_method: query.code_challenge_method.clone(),
        state: query.state.clone().unwrap_or_else(|| "".to_string()),
    };

    let session_state = pkce.state.clone();
    state.storage.store_pkce(&session_state, &pkce).await?;

    // In a real implementation, redirect to login page
    // For now, return authorization page URL
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Authorization flow initiated",
        "state": session_state,
        "next_step": "User must complete authentication and consent"
    })))
}

// ============================================================================
// Token Exchange Endpoint
// ============================================================================

#[derive(Debug, Deserialize)]
struct TokenRequest {
    grant_type: String,
    code: Option<String>,
    code_verifier: Option<String>,
    redirect_uri: Option<String>,
    client_id: Option<String>,
    refresh_token: Option<String>,
    device_code: Option<String>,
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: i64,
    scope: String,
}

#[post("/auth/token")]
async fn token_exchange(
    form: web::Form<TokenRequest>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    match form.grant_type.as_str() {
        "authorization_code" => exchange_authorization_code(&form, &state).await,
        "refresh_token" => refresh_access_token(&form, &state).await,
        "urn:ietf:params:oauth:grant-type:device_code" => exchange_device_code(&form, &state).await,
        _ => Err(AuthError::Internal("Unsupported grant type".to_string())),
    }
}

async fn exchange_authorization_code(
    form: &TokenRequest,
    state: &AppState,
) -> Result<HttpResponse> {
    let code = form.code.as_ref().ok_or(AuthError::InvalidAuthCode)?;
    let verifier = form.code_verifier.as_ref().ok_or(AuthError::InvalidPkceVerifier)?;
    let redirect_uri = form.redirect_uri.as_ref().ok_or(AuthError::InvalidRedirectUri)?;
    let client_id = form.client_id.as_ref().ok_or(AuthError::InvalidClient)?;

    // Retrieve authorization code
    let mut auth_code = state.storage
        .get_auth_code(code)
        .await?
        .ok_or(AuthError::InvalidAuthCode)?;

    // Check if already used
    if auth_code.used {
        tracing::error!("Authorization code reuse detected: {}", code);
        return Err(AuthError::AuthCodeReused);
    }

    // Check expiration
    if auth_code.is_expired() {
        state.storage.delete_auth_code(code).await?;
        return Err(AuthError::InvalidAuthCode);
    }

    // Verify PKCE
    auth_code.verify_pkce(verifier)?;

    // Verify client_id and redirect_uri
    if &auth_code.client_id != client_id || &auth_code.redirect_uri != redirect_uri {
        return Err(AuthError::InvalidClient);
    }

    // Mark as used
    auth_code.mark_as_used();
    state.storage.update_auth_code(code, &auth_code).await?;

    // Generate tokens
    let access_token = state.jwt_manager.create_access_token(
        auth_code.user_id.clone(),
        Some(format!("user{}@example.com", auth_code.user_id)),
        vec!["free_user".to_string()],
        auth_code.scopes.clone(),
    )?;

    let refresh_token = state.jwt_manager.create_refresh_token(
        auth_code.user_id.clone(),
        Some(format!("user{}@example.com", auth_code.user_id)),
        vec!["free_user".to_string()],
        auth_code.scopes.clone(),
    )?;

    // Create session
    let refresh_claims = state.jwt_manager.verify_refresh_token(&refresh_token)?;
    state
        .session_manager
        .create_session(auth_code.user_id.clone(), refresh_claims.jti, None)
        .await?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: auth_code.scopes.join(" "),
    }))
}

async fn refresh_access_token(form: &TokenRequest, state: &AppState) -> Result<HttpResponse> {
    let refresh_token = form.refresh_token.as_ref().ok_or(AuthError::InvalidToken("Missing refresh token".to_string()))?;

    // Verify refresh token
    let claims = state.jwt_manager.verify_refresh_token(refresh_token)?;

    // Check if revoked
    if state.session_manager.is_token_revoked(&claims.jti).await? {
        return Err(AuthError::InvalidToken("Token revoked".to_string()));
    }

    // Generate new tokens
    let new_access_token = state.jwt_manager.create_access_token(
        claims.sub.clone(),
        claims.email.clone(),
        claims.roles.clone(),
        claims.scopes.clone(),
    )?;

    let new_refresh_token = state.jwt_manager.create_refresh_token(
        claims.sub.clone(),
        claims.email.clone(),
        claims.roles.clone(),
        claims.scopes.clone(),
    )?;

    // Revoke old refresh token
    state.session_manager.revoke_token(&claims.jti, 3600).await?;

    // Create new session
    let new_refresh_claims = state.jwt_manager.verify_refresh_token(&new_refresh_token)?;
    state
        .session_manager
        .create_session(claims.sub.clone(), new_refresh_claims.jti, None)
        .await?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: claims.scopes.join(" "),
    }))
}

async fn exchange_device_code(form: &TokenRequest, state: &AppState) -> Result<HttpResponse> {
    let device_code = form.device_code.as_ref().ok_or(AuthError::DeviceCodeNotFound)?;

    // Retrieve device code
    let device = state.storage
        .get_device_code(device_code)
        .await?
        .ok_or(AuthError::DeviceCodeNotFound)?;

    // Check status
    device.check_status()?;

    let user_id = device.user_id.clone().ok_or(AuthError::Internal("User ID not found".to_string()))?;

    // Generate tokens
    let access_token = state.jwt_manager.create_access_token(
        user_id.clone(),
        Some(format!("user{}@example.com", user_id)),
        vec!["free_user".to_string()],
        device.scopes.clone(),
    )?;

    let refresh_token = state.jwt_manager.create_refresh_token(
        user_id.clone(),
        Some(format!("user{}@example.com", user_id)),
        vec!["free_user".to_string()],
        device.scopes.clone(),
    )?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: device.scopes.join(" "),
    }))
}

// ============================================================================
// Token Revocation Endpoint
// ============================================================================

#[derive(Debug, Deserialize)]
struct RevokeRequest {
    token: String,
    token_type_hint: Option<String>,
}

#[post("/auth/revoke")]
async fn revoke_token(
    form: web::Form<RevokeRequest>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    // Try to decode as access or refresh token
    let claims = state
        .jwt_manager
        .verify_token(&form.token)
        .or_else(|_| state.jwt_manager.verify_refresh_token(&form.token))?;

    // Revoke token
    state.session_manager.revoke_token(&claims.jti, 3600).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Token revoked successfully"
    })))
}

// ============================================================================
// Device Authorization Endpoint (RFC 8628)
// ============================================================================

#[derive(Debug, Deserialize)]
struct DeviceAuthRequest {
    client_id: String,
    scope: Option<String>,
}

#[post("/auth/device")]
async fn device_authorization(
    form: web::Form<DeviceAuthRequest>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let scopes = form
        .scope
        .as_ref()
        .map(|s| s.split_whitespace().map(|x| x.to_string()).collect())
        .unwrap_or_default();

    let device = DeviceCode::new(
        form.client_id.clone(),
        scopes,
        "https://auth.mediagateway.io",
    );

    let response = DeviceAuthorizationResponse::from(&device);

    // Store device code
    state.storage.store_device_code(&device.device_code, &device).await?;

    Ok(HttpResponse::Ok().json(response))
}

#[get("/auth/device/poll")]
async fn device_poll(
    query: web::Query<std::collections::HashMap<String, String>>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let device_code = query
        .get("device_code")
        .ok_or(AuthError::DeviceCodeNotFound)?;

    let device = state.storage
        .get_device_code(device_code)
        .await?
        .ok_or(AuthError::DeviceCodeNotFound)?;

    device.check_status()?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "approved",
        "message": "Device authorization approved"
    })))
}

// ============================================================================
// Server Initialization
// ============================================================================

pub async fn start_server(
    bind_address: &str,
    jwt_manager: Arc<JwtManager>,
    session_manager: Arc<SessionManager>,
    oauth_config: OAuthConfig,
    storage: Arc<AuthStorage>,
) -> std::io::Result<()> {
    let app_state = Data::new(AppState {
        jwt_manager,
        session_manager,
        oauth_manager: Arc::new(OAuthManager::new(oauth_config)),
        rbac_manager: Arc::new(RbacManager::new()),
        scope_manager: Arc::new(ScopeManager::new()),
        storage,
    });

    tracing::info!("Starting auth service on {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(health_check)
            .service(authorize)
            .service(token_exchange)
            .service(revoke_token)
            .service(device_authorization)
            .service(device_poll)
    })
    .bind(bind_address)?
    .run()
    .await
}
