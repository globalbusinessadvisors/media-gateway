pub mod handlers;

pub use handlers::get_analytics;

use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::analytics::SearchAnalytics;

/// Create the analytics router
pub fn analytics_router(analytics: Arc<SearchAnalytics>) -> Router {
    Router::new()
        .route("/api/v1/admin/search/analytics", get(handlers::get_analytics))
        .with_state(analytics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_router_creation() {
        // This test just ensures the router can be created
        // Actual functionality is tested in handlers/analytics.rs
        let analytics = Arc::new(SearchAnalytics::new(
            sqlx::PgPool::connect_lazy("postgres://localhost/test")
                .expect("Failed to create lazy pool")
        ));

        let _router = analytics_router(analytics);
    }
}
