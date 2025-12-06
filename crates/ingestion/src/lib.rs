//! Media Gateway Ingestion Pipeline
//!
//! This crate provides the data ingestion pipeline for the Media Gateway platform,
//! including platform normalizers, entity resolution, and content enrichment.

pub mod aggregator;
pub mod deep_link;
pub mod embedding;
pub mod entity_resolution;
pub mod genre_mapping;
pub mod normalizer;
pub mod pipeline;
pub mod rate_limit;
pub mod repository;

// Re-export main types
pub use pipeline::{IngestionPipeline, IngestionSchedule};
pub use normalizer::PlatformNormalizer;
pub use entity_resolution::EntityResolver;
pub use genre_mapping::GenreMapper;
pub use embedding::EmbeddingGenerator;
pub use deep_link::{DeepLinkGenerator, DeepLinkResult};
pub use rate_limit::RateLimitManager;
pub use repository::{ContentRepository, PostgresContentRepository, ExpiringContent};

/// Common error type for the ingestion pipeline
#[derive(Debug, thiserror::Error)]
pub enum IngestionError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Rate limit exceeded for {platform}")]
    RateLimitExceeded { platform: String },

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("Entity resolution failed: {0}")]
    EntityResolutionFailed(String),

    #[error("Normalization failed: {0}")]
    NormalizationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, IngestionError>;
