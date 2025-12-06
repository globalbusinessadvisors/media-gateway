//! Main ingestion pipeline with scheduling and orchestration

use crate::{
    normalizer::{PlatformNormalizer, RawContent},
    entity_resolution::EntityResolver,
    genre_mapping::GenreMapper,
    embedding::EmbeddingGenerator,
    rate_limit::RateLimitManager,
    Result, IngestionError,
};
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error, debug};

/// Ingestion schedule configuration
#[derive(Debug, Clone)]
pub struct IngestionSchedule {
    /// Full catalog refresh interval (6 hours)
    pub catalog_refresh: Duration,
    /// Availability sync interval (1 hour)
    pub availability_sync: Duration,
    /// Expiring content check interval (15 minutes)
    pub expiring_content: Duration,
    /// Metadata enrichment interval (24 hours)
    pub metadata_enrichment: Duration,
}

impl Default for IngestionSchedule {
    fn default() -> Self {
        Self {
            catalog_refresh: Duration::from_secs(6 * 3600),
            availability_sync: Duration::from_secs(3600),
            expiring_content: Duration::from_secs(900),
            metadata_enrichment: Duration::from_secs(24 * 3600),
        }
    }
}

/// Main ingestion pipeline orchestrator
pub struct IngestionPipeline {
    normalizers: Vec<Arc<dyn PlatformNormalizer>>,
    entity_resolver: Arc<EntityResolver>,
    genre_mapper: Arc<GenreMapper>,
    embedding_generator: Arc<EmbeddingGenerator>,
    rate_limiter: Arc<RateLimitManager>,
    schedule: IngestionSchedule,
    regions: Vec<String>,
}

impl IngestionPipeline {
    /// Create a new ingestion pipeline
    pub fn new(
        normalizers: Vec<Arc<dyn PlatformNormalizer>>,
        entity_resolver: EntityResolver,
        genre_mapper: GenreMapper,
        embedding_generator: EmbeddingGenerator,
        rate_limiter: RateLimitManager,
        schedule: IngestionSchedule,
        regions: Vec<String>,
    ) -> Self {
        Self {
            normalizers,
            entity_resolver: Arc::new(entity_resolver),
            genre_mapper: Arc::new(genre_mapper),
            embedding_generator: Arc::new(embedding_generator),
            rate_limiter: Arc::new(rate_limiter),
            schedule,
            regions,
        }
    }

    /// Start the ingestion pipeline with all scheduled tasks
    pub async fn start(&self) -> Result<()> {
        info!("Starting ingestion pipeline with {} platforms", self.normalizers.len());

        // Spawn concurrent tasks for different schedules
        let catalog_handle = self.spawn_catalog_refresh_task();
        let availability_handle = self.spawn_availability_sync_task();
        let expiring_handle = self.spawn_expiring_content_task();
        let enrichment_handle = self.spawn_metadata_enrichment_task();

        // Wait for all tasks (they run indefinitely)
        tokio::select! {
            result = catalog_handle => {
                error!("Catalog refresh task terminated: {:?}", result);
            }
            result = availability_handle => {
                error!("Availability sync task terminated: {:?}", result);
            }
            result = expiring_handle => {
                error!("Expiring content task terminated: {:?}", result);
            }
            result = enrichment_handle => {
                error!("Metadata enrichment task terminated: {:?}", result);
            }
        }

        Ok(())
    }

    /// Spawn catalog refresh task (every 6 hours)
    fn spawn_catalog_refresh_task(&self) -> tokio::task::JoinHandle<()> {
        let normalizers = self.normalizers.clone();
        let entity_resolver = self.entity_resolver.clone();
        let genre_mapper = self.genre_mapper.clone();
        let embedding_generator = self.embedding_generator.clone();
        let rate_limiter = self.rate_limiter.clone();
        let regions = self.regions.clone();
        let schedule_duration = self.schedule.catalog_refresh;

        tokio::spawn(async move {
            let mut interval = interval(schedule_duration);
            loop {
                interval.tick().await;
                info!("Starting catalog refresh cycle");

                for normalizer in &normalizers {
                    for region in &regions {
                        if let Err(e) = Self::process_catalog_refresh(
                            normalizer.clone(),
                            &entity_resolver,
                            &genre_mapper,
                            &embedding_generator,
                            &rate_limiter,
                            region,
                        ).await {
                            error!("Catalog refresh failed for {} in {}: {}",
                                normalizer.platform_id(), region, e);
                        }
                    }
                }

                info!("Catalog refresh cycle completed");
            }
        })
    }

    /// Spawn availability sync task (every 1 hour)
    fn spawn_availability_sync_task(&self) -> tokio::task::JoinHandle<()> {
        let normalizers = self.normalizers.clone();
        let rate_limiter = self.rate_limiter.clone();
        let regions = self.regions.clone();
        let schedule_duration = self.schedule.availability_sync;

        tokio::spawn(async move {
            let mut interval = interval(schedule_duration);
            loop {
                interval.tick().await;
                info!("Starting availability sync cycle");

                for normalizer in &normalizers {
                    for region in &regions {
                        if let Err(e) = Self::sync_availability(
                            normalizer.clone(),
                            &rate_limiter,
                            region,
                        ).await {
                            error!("Availability sync failed for {} in {}: {}",
                                normalizer.platform_id(), region, e);
                        }
                    }
                }

                info!("Availability sync cycle completed");
            }
        })
    }

    /// Spawn expiring content check task (every 15 minutes)
    fn spawn_expiring_content_task(&self) -> tokio::task::JoinHandle<()> {
        let normalizers = self.normalizers.clone();
        let rate_limiter = self.rate_limiter.clone();
        let regions = self.regions.clone();
        let schedule_duration = self.schedule.expiring_content;

        tokio::spawn(async move {
            let mut interval = interval(schedule_duration);
            loop {
                interval.tick().await;
                debug!("Checking expiring content");

                for normalizer in &normalizers {
                    for region in &regions {
                        if let Err(e) = Self::check_expiring_content(
                            normalizer.clone(),
                            &rate_limiter,
                            region,
                        ).await {
                            warn!("Expiring content check failed for {} in {}: {}",
                                normalizer.platform_id(), region, e);
                        }
                    }
                }
            }
        })
    }

    /// Spawn metadata enrichment task (every 24 hours)
    fn spawn_metadata_enrichment_task(&self) -> tokio::task::JoinHandle<()> {
        let embedding_generator = self.embedding_generator.clone();
        let schedule_duration = self.schedule.metadata_enrichment;

        tokio::spawn(async move {
            let mut interval = interval(schedule_duration);
            loop {
                interval.tick().await;
                info!("Starting metadata enrichment cycle");

                if let Err(e) = Self::enrich_metadata(&embedding_generator).await {
                    error!("Metadata enrichment failed: {}", e);
                }

                info!("Metadata enrichment cycle completed");
            }
        })
    }

    /// Process full catalog refresh for a platform/region
    async fn process_catalog_refresh(
        normalizer: Arc<dyn PlatformNormalizer>,
        entity_resolver: &EntityResolver,
        genre_mapper: &GenreMapper,
        embedding_generator: &EmbeddingGenerator,
        rate_limiter: &RateLimitManager,
        region: &str,
    ) -> Result<()> {
        let platform_id = normalizer.platform_id();
        info!("Fetching catalog delta for {} in {}", platform_id, region);

        // Check rate limit
        rate_limiter.check_and_wait(platform_id).await?;

        // Calculate "since" timestamp (last successful run or 7 days ago)
        let since = Utc::now() - ChronoDuration::days(7);

        // Fetch catalog delta
        let raw_items = normalizer.fetch_catalog_delta(since, region).await?;
        info!("Fetched {} items from {} for {}", raw_items.len(), platform_id, region);

        // Process items in batches for performance (target: 500 items/s)
        const BATCH_SIZE: usize = 100;
        for batch in raw_items.chunks(BATCH_SIZE) {
            Self::process_batch(
                batch,
                normalizer.as_ref(),
                entity_resolver,
                genre_mapper,
                embedding_generator,
            ).await?;
        }

        Ok(())
    }

    /// Process a batch of raw content items
    async fn process_batch(
        batch: &[RawContent],
        normalizer: &dyn PlatformNormalizer,
        entity_resolver: &EntityResolver,
        genre_mapper: &GenreMapper,
        embedding_generator: &EmbeddingGenerator,
    ) -> Result<()> {
        for raw in batch {
            // Normalize to canonical format
            let mut canonical = normalizer.normalize(raw.clone())
                .map_err(|e| IngestionError::NormalizationFailed(e.to_string()))?;

            // Resolve entity (EIDR, external IDs, fuzzy matching)
            let entity_match = entity_resolver.resolve(&canonical).await
                .map_err(|e| IngestionError::EntityResolutionFailed(e.to_string()))?;

            if let Some(matched_entity_id) = entity_match.entity_id {
                canonical.entity_id = Some(matched_entity_id);
            }

            // Map genres to canonical taxonomy
            canonical.genres = genre_mapper.map_genres(
                &canonical.genres,
                normalizer.platform_id(),
            );

            // Generate embeddings
            canonical.embedding = Some(
                embedding_generator.generate(&canonical).await
                    .map_err(|e| IngestionError::NormalizationFailed(e.to_string()))?
            );

            // TODO: Persist to database
            debug!("Processed content: {} (entity: {:?})",
                canonical.title, canonical.entity_id);
        }

        Ok(())
    }

    /// Sync availability data (pricing, subscription status)
    async fn sync_availability(
        normalizer: Arc<dyn PlatformNormalizer>,
        rate_limiter: &RateLimitManager,
        region: &str,
    ) -> Result<()> {
        let platform_id = normalizer.platform_id();
        debug!("Syncing availability for {} in {}", platform_id, region);

        rate_limiter.check_and_wait(platform_id).await?;

        // Fetch recent availability updates (last hour)
        let since = Utc::now() - ChronoDuration::hours(1);
        let raw_items = normalizer.fetch_catalog_delta(since, region).await?;

        // TODO: Update only availability fields in database
        debug!("Updated availability for {} items from {}", raw_items.len(), platform_id);

        Ok(())
    }

    /// Check for expiring content (leaving platforms soon)
    async fn check_expiring_content(
        normalizer: Arc<dyn PlatformNormalizer>,
        rate_limiter: &RateLimitManager,
        region: &str,
    ) -> Result<()> {
        let platform_id = normalizer.platform_id();

        rate_limiter.check_and_wait(platform_id).await?;

        // TODO: Query database for content expiring in next 7 days
        // TODO: Update expiration dates from platform data

        Ok(())
    }

    /// Enrich metadata with updated embeddings
    async fn enrich_metadata(
        embedding_generator: &EmbeddingGenerator,
    ) -> Result<()> {
        // TODO: Query database for content needing enrichment
        // TODO: Regenerate embeddings for stale content
        // TODO: Update quality scores

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_schedule() {
        let schedule = IngestionSchedule::default();
        assert_eq!(schedule.catalog_refresh, Duration::from_secs(6 * 3600));
        assert_eq!(schedule.availability_sync, Duration::from_secs(3600));
        assert_eq!(schedule.expiring_content, Duration::from_secs(900));
        assert_eq!(schedule.metadata_enrichment, Duration::from_secs(24 * 3600));
    }
}
