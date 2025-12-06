use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub mod filters;
pub mod keyword;
pub mod vector;

pub use filters::SearchFilters;
pub use keyword::KeywordSearch;
pub use vector::VectorSearch;

use crate::config::DiscoveryConfig;
use crate::intent::{IntentParser, ParsedIntent};

/// Hybrid search service orchestrator
pub struct HybridSearchService {
    config: Arc<DiscoveryConfig>,
    intent_parser: Arc<IntentParser>,
    vector_search: Arc<vector::VectorSearch>,
    keyword_search: Arc<keyword::KeywordSearch>,
    db_pool: sqlx::PgPool,
}

/// Search request
#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub filters: Option<SearchFilters>,
    pub page: u32,
    pub page_size: u32,
    pub user_id: Option<Uuid>,
}

/// Search response
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: usize,
    pub page: u32,
    pub page_size: u32,
    pub query_parsed: ParsedIntent,
    pub search_time_ms: u64,
}

/// Individual search result
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub content: ContentSummary,
    pub relevance_score: f32,
    pub match_reasons: Vec<String>,
    pub vector_similarity: Option<f32>,
    pub graph_score: Option<f32>,
    pub keyword_score: Option<f32>,
}

/// Content summary for search results
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContentSummary {
    pub id: Uuid,
    pub title: String,
    pub overview: String,
    pub release_year: i32,
    pub genres: Vec<String>,
    pub platforms: Vec<String>,
    pub popularity_score: f32,
}

impl HybridSearchService {
    /// Create new hybrid search service
    pub fn new(
        config: Arc<DiscoveryConfig>,
        intent_parser: Arc<IntentParser>,
        vector_search: Arc<vector::VectorSearch>,
        keyword_search: Arc<keyword::KeywordSearch>,
        db_pool: sqlx::PgPool,
    ) -> Self {
        Self {
            config,
            intent_parser,
            vector_search,
            keyword_search,
            db_pool,
        }
    }

    /// Execute hybrid search
    pub async fn search(&self, request: SearchRequest) -> anyhow::Result<SearchResponse> {
        let start_time = std::time::Instant::now();

        // Phase 1: Parse intent
        let intent = self.intent_parser.parse(&request.query).await?;

        // Phase 2: Execute parallel search strategies
        let (vector_results, keyword_results) = tokio::join!(
            self.vector_search.search(&request.query, request.filters.clone()),
            self.keyword_search.search(&request.query, request.filters.clone())
        );

        // Phase 3: Merge results using Reciprocal Rank Fusion
        let merged_results = self.reciprocal_rank_fusion(
            vector_results?,
            keyword_results?,
            self.config.search.rrf_k,
        );

        // Phase 4: Apply personalization if user_id provided
        let ranked_results = if let Some(_user_id) = request.user_id {
            // TODO: Apply user preference scoring
            merged_results
        } else {
            merged_results
        };

        // Phase 5: Paginate
        let total_count = ranked_results.len();
        let start = ((request.page - 1) * request.page_size) as usize;
        let end = std::cmp::min(start + request.page_size as usize, total_count);
        let page_results = ranked_results[start..end].to_vec();

        let search_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(SearchResponse {
            results: page_results,
            total_count,
            page: request.page,
            page_size: request.page_size,
            query_parsed: intent,
            search_time_ms,
        })
    }

    /// Vector-only search
    pub async fn vector_search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        _limit: Option<usize>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        self.vector_search.search(query, filters).await
    }

    /// Keyword-only search
    pub async fn keyword_search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        _limit: Option<usize>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        self.keyword_search.search(query, filters).await
    }

    /// Get content by ID
    pub async fn get_content_by_id(&self, id: Uuid) -> anyhow::Result<Option<ContentSummary>> {
        let result = sqlx::query_as::<_, ContentSummary>(
            r#"
            SELECT
                id,
                title,
                overview,
                release_year,
                genres,
                ARRAY[]::text[] as "platforms!: Vec<String>",
                popularity_score
            FROM content
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result)
    }

    /// Reciprocal Rank Fusion (RRF) algorithm
    /// Merges results from multiple search strategies
    fn reciprocal_rank_fusion(
        &self,
        vector_results: Vec<SearchResult>,
        keyword_results: Vec<SearchResult>,
        k: f32,
    ) -> Vec<SearchResult> {
        let mut scores: HashMap<Uuid, (f32, SearchResult)> = HashMap::new();

        // Process vector results
        for (rank, result) in vector_results.iter().enumerate() {
            let rrf_score = self.config.search.weights.vector / (k + (rank + 1) as f32);
            scores
                .entry(result.content.id)
                .and_modify(|(score, _)| *score += rrf_score)
                .or_insert((rrf_score, result.clone()));
        }

        // Process keyword results
        for (rank, result) in keyword_results.iter().enumerate() {
            let rrf_score = self.config.search.weights.keyword / (k + (rank + 1) as f32);
            scores
                .entry(result.content.id)
                .and_modify(|(score, _)| *score += rrf_score)
                .or_insert((rrf_score, result.clone()));
        }

        // Sort by combined score
        let mut merged: Vec<(f32, SearchResult)> = scores
            .into_iter()
            .map(|(_, (score, mut result))| {
                result.relevance_score = score;
                (score, result)
            })
            .collect();

        merged.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        merged.into_iter().map(|(_, result)| result).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reciprocal_rank_fusion() {
        // Create mock results
        let content1 = ContentSummary {
            id: Uuid::new_v4(),
            title: "Movie 1".to_string(),
            overview: "Description".to_string(),
            release_year: 2020,
            genres: vec!["action".to_string()],
            platforms: vec![],
            popularity_score: 0.8,
        };

        let content2 = ContentSummary {
            id: Uuid::new_v4(),
            title: "Movie 2".to_string(),
            overview: "Description".to_string(),
            release_year: 2021,
            genres: vec!["drama".to_string()],
            platforms: vec![],
            popularity_score: 0.7,
        };

        let vector_results = vec![
            SearchResult {
                content: content1.clone(),
                relevance_score: 0.9,
                match_reasons: vec![],
                vector_similarity: Some(0.9),
                graph_score: None,
                keyword_score: None,
            },
            SearchResult {
                content: content2.clone(),
                relevance_score: 0.8,
                match_reasons: vec![],
                vector_similarity: Some(0.8),
                graph_score: None,
                keyword_score: None,
            },
        ];

        let keyword_results = vec![SearchResult {
            content: content2.clone(),
            relevance_score: 0.85,
            match_reasons: vec![],
            vector_similarity: None,
            graph_score: None,
            keyword_score: Some(0.85),
        }];

        // Mock config
        let config = Arc::new(DiscoveryConfig::default());

        // Create mock service (simplified for test)
        let db_pool = sqlx::PgPool::connect("postgresql://localhost/test")
            .await
            .expect("Failed to connect");

        let service = HybridSearchService {
            config,
            intent_parser: Arc::new(IntentParser::new(String::new(), String::new())),
            vector_search: Arc::new(vector::VectorSearch::new(
                String::new(),
                String::new(),
                768,
            )),
            keyword_search: Arc::new(keyword::KeywordSearch::new(String::new())),
            db_pool,
        };

        let merged = service.reciprocal_rank_fusion(vector_results, keyword_results, 60.0);

        // content2 should rank higher (appears in both results)
        assert_eq!(merged[0].content.id, content2.id);
    }
}
