pub mod error;
pub mod jwt;
pub mod middleware;
pub mod oauth;
pub mod rbac;
pub mod scopes;
pub mod server;
pub mod session;
pub mod token;

#[cfg(test)]
mod tests;

pub use error::{AuthError, Result};
pub use jwt::{Claims, JwtManager};
pub use middleware::AuthMiddleware;
pub use oauth::{OAuthConfig, OAuthManager};
pub use rbac::{Permission, Role, RbacManager};
pub use scopes::{Scope, ScopeManager};
pub use server::start_server;
pub use session::{Session, SessionManager};
pub use token::{TokenManager, TokenType};
