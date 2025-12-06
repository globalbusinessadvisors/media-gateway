use media_gateway_auth::{
    jwt::JwtManager,
    oauth::OAuthConfig,
    session::SessionManager,
    server::start_server,
};
use std::{env, fs, sync::Arc};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    tracing::info!("Starting Media Gateway Auth Service");

    // Load configuration
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8084".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Load JWT keys
    let private_key_path = env::var("JWT_PRIVATE_KEY_PATH")
        .unwrap_or_else(|_| "/secrets/jwt_private_key.pem".to_string());
    let public_key_path = env::var("JWT_PUBLIC_KEY_PATH")
        .unwrap_or_else(|_| "/secrets/jwt_public_key.pem".to_string());

    let private_key = fs::read(&private_key_path)
        .expect("Failed to read JWT private key");
    let public_key = fs::read(&public_key_path)
        .expect("Failed to read JWT public key");

    let jwt_issuer = env::var("JWT_ISSUER")
        .unwrap_or_else(|_| "https://api.mediagateway.io".to_string());
    let jwt_audience = env::var("JWT_AUDIENCE")
        .unwrap_or_else(|_| "mediagateway-users".to_string());

    // Initialize JWT manager
    let jwt_manager = Arc::new(
        JwtManager::new(&private_key, &public_key, jwt_issuer, jwt_audience)
            .expect("Failed to initialize JWT manager"),
    );

    // Initialize session manager
    let session_manager = Arc::new(
        SessionManager::new(&redis_url)
            .expect("Failed to initialize session manager"),
    );

    // Initialize OAuth config (load from environment)
    let oauth_config = OAuthConfig {
        providers: std::collections::HashMap::new(),
    };

    // Start server
    start_server(&bind_address, jwt_manager, session_manager, oauth_config).await
}
