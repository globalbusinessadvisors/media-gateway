pub mod api_keys;
pub mod error;
pub mod jwt;
pub mod mfa;
pub mod middleware;
pub mod oauth;
pub mod rbac;
pub mod scopes;
pub mod server;
pub mod session;
pub mod storage;
pub mod token;
pub mod token_family;

#[cfg(test)]
mod tests;

pub use api_keys::{ApiKey, ApiKeyManager, CreateApiKeyRequest};
pub use error::{AuthError, Result};
pub use jwt::{Claims, JwtManager};
pub use mfa::{MfaManager, MfaEnrollment};
pub use middleware::AuthMiddleware;
pub use oauth::{OAuthConfig, OAuthManager};
pub use rbac::{Permission, Role, RbacManager};
pub use scopes::{Scope, ScopeManager};
pub use server::start_server;
pub use session::{Session, SessionManager};
pub use storage::AuthStorage;
pub use token::{TokenManager, TokenType};
pub use token_family::{TokenFamily, TokenFamilyManager};
