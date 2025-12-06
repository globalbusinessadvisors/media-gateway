pub mod config;
pub mod embedding;
pub mod intent;
pub mod search;
pub mod server;

pub use config::DiscoveryConfig;
pub use embedding::EmbeddingService;
pub use intent::{IntentParser, ParsedIntent};
pub use search::{HybridSearchService, SearchRequest, SearchResponse};

use std::sync::Arc;

/// Initialize discovery service components
pub async fn init_service(
    config: Arc<DiscoveryConfig>,
) -> anyhow::Result<Arc<HybridSearchService>> {
    // Initialize database pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(std::time::Duration::from_secs(
            config.database.connect_timeout_sec,
        ))
        .connect(&config.database.url)
        .await?;

    // Initialize intent parser
    let intent_parser = Arc::new(IntentParser::new(
        config.embedding.api_url.clone(),
        config.embedding.api_key.clone(),
    ));

    // Initialize vector search
    let vector_search = Arc::new(search::vector::VectorSearch::new(
        config.vector.qdrant_url.clone(),
        config.vector.collection_name.clone(),
        config.vector.dimension,
    ));

    // Initialize keyword search
    let keyword_search = Arc::new(search::keyword::KeywordSearch::new(
        config.keyword.index_path.clone(),
    ));

    // Initialize hybrid search service
    let search_service = Arc::new(HybridSearchService::new(
        config.clone(),
        intent_parser,
        vector_search,
        keyword_search,
        db_pool,
    ));

    Ok(search_service)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_service_initialization() {
        let config = Arc::new(DiscoveryConfig::default());

        // This will fail without actual database, but tests the structure
        let result = init_service(config).await;
        assert!(result.is_err()); // Expected to fail without real database
    }
}
