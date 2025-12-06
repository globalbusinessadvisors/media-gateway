use qdrant_client::prelude::*;
use qdrant_client::qdrant::{SearchPoints, SearchResponse as QdrantSearchResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::filters::SearchFilters;
use super::{ContentSummary, SearchResult};

/// Vector search using Qdrant HNSW
pub struct VectorSearch {
    client: QdrantClient,
    collection_name: String,
    dimension: usize,
    ef_search: usize,
    top_k: usize,
    similarity_threshold: f32,
}

impl VectorSearch {
    /// Create new vector search instance
    pub fn new(
        qdrant_url: String,
        collection_name: String,
        dimension: usize,
    ) -> Self {
        let client = QdrantClient::from_url(&qdrant_url).build().unwrap();

        Self {
            client,
            collection_name,
            dimension,
            ef_search: 64,
            top_k: 50,
            similarity_threshold: 0.7,
        }
    }

    /// Execute vector similarity search
    pub async fn search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        // Generate query embedding (TODO: implement embedding service)
        let query_vector = self.generate_embedding(query).await?;

        // Determine filter strategy
        let use_pre_filter = filters
            .as_ref()
            .map(|f| f.should_pre_filter())
            .unwrap_or(false);

        let results = if use_pre_filter {
            self.search_with_pre_filter(query_vector, filters).await?
        } else {
            self.search_with_post_filter(query_vector, filters).await?
        };

        Ok(results)
    }

    /// HNSW search with pre-filtering
    async fn search_with_pre_filter(
        &self,
        query_vector: Vec<f32>,
        filters: Option<SearchFilters>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        // Build Qdrant filter
        let filter = filters.as_ref().map(|f| self.build_qdrant_filter(f));

        // Execute search
        let search_result = self
            .client
            .search_points(&SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_vector,
                filter,
                limit: self.top_k as u64,
                with_payload: Some(true.into()),
                params: Some(SearchParams {
                    hnsw_ef: Some(self.ef_search as u64),
                    exact: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?;

        self.convert_qdrant_results(search_result)
    }

    /// HNSW search with post-filtering
    async fn search_with_post_filter(
        &self,
        query_vector: Vec<f32>,
        filters: Option<SearchFilters>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        // Overquery to account for filtering
        let overquery_k = self.top_k * 3;

        // Execute search without filters
        let search_result = self
            .client
            .search_points(&SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_vector,
                filter: None,
                limit: overquery_k as u64,
                with_payload: Some(true.into()),
                params: Some(SearchParams {
                    hnsw_ef: Some(self.ef_search as u64),
                    exact: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?;

        // Convert and filter
        let mut results = self.convert_qdrant_results(search_result)?;

        // Apply filters in memory
        if let Some(filters) = filters {
            results.retain(|result| self.matches_filters(&result.content, &filters));
        }

        // Limit to top_k
        results.truncate(self.top_k);

        Ok(results)
    }

    /// Generate embedding for query
    async fn generate_embedding(&self, query: &str) -> anyhow::Result<Vec<f32>> {
        // TODO: Implement actual embedding API call
        // For now, return mock embedding
        Ok(vec![0.0; self.dimension])
    }

    /// Build Qdrant filter from search filters
    fn build_qdrant_filter(&self, filters: &SearchFilters) -> Filter {
        let mut conditions = Vec::new();

        // Genre filter
        if !filters.genres.is_empty() {
            conditions.push(Condition::matches(
                "genres",
                filters.genres.clone(),
            ));
        }

        // Platform filter
        if !filters.platforms.is_empty() {
            conditions.push(Condition::matches(
                "platforms",
                filters.platforms.clone(),
            ));
        }

        // Year range filter
        if let Some((min_year, max_year)) = filters.year_range {
            conditions.push(Condition::range(
                "release_year",
                min_year as f64..max_year as f64,
            ));
        }

        Filter::must(conditions)
    }

    /// Convert Qdrant results to SearchResult
    fn convert_qdrant_results(
        &self,
        response: QdrantSearchResponse,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        for scored_point in response.result {
            // Extract payload
            let payload = scored_point.payload;

            let content = ContentSummary {
                id: Uuid::parse_str(
                    payload
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                )?,
                title: payload
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                overview: payload
                    .get("overview")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                release_year: payload
                    .get("release_year")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(0) as i32,
                genres: payload
                    .get("genres")
                    .and_then(|v| v.as_list())
                    .map(|list| {
                        list.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                platforms: payload
                    .get("platforms")
                    .and_then(|v| v.as_list())
                    .map(|list| {
                        list.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                popularity_score: payload
                    .get("popularity_score")
                    .and_then(|v| v.as_double())
                    .unwrap_or(0.0) as f32,
            };

            results.push(SearchResult {
                content,
                relevance_score: scored_point.score,
                match_reasons: vec!["vector_similarity".to_string()],
                vector_similarity: Some(scored_point.score),
                graph_score: None,
                keyword_score: None,
            });
        }

        Ok(results)
    }

    /// Check if content matches filters
    fn matches_filters(&self, content: &ContentSummary, filters: &SearchFilters) -> bool {
        // Genre filter
        if !filters.genres.is_empty() {
            let has_genre = content
                .genres
                .iter()
                .any(|g| filters.genres.contains(g));
            if !has_genre {
                return false;
            }
        }

        // Platform filter
        if !filters.platforms.is_empty() {
            let has_platform = content
                .platforms
                .iter()
                .any(|p| filters.platforms.contains(p));
            if !has_platform {
                return false;
            }
        }

        // Year range filter
        if let Some((min_year, max_year)) = filters.year_range {
            if content.release_year < min_year || content.release_year > max_year {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matching() {
        let vector_search = VectorSearch::new(
            "http://localhost:6333".to_string(),
            "test".to_string(),
            768,
        );

        let content = ContentSummary {
            id: Uuid::new_v4(),
            title: "Test Movie".to_string(),
            overview: "Description".to_string(),
            release_year: 2020,
            genres: vec!["action".to_string()],
            platforms: vec!["netflix".to_string()],
            popularity_score: 0.8,
        };

        let filters = SearchFilters {
            genres: vec!["action".to_string()],
            platforms: vec!["netflix".to_string()],
            year_range: Some((2018, 2022)),
            rating_range: None,
        };

        assert!(vector_search.matches_filters(&content, &filters));
    }

    #[test]
    fn test_filter_mismatch() {
        let vector_search = VectorSearch::new(
            "http://localhost:6333".to_string(),
            "test".to_string(),
            768,
        );

        let content = ContentSummary {
            id: Uuid::new_v4(),
            title: "Test Movie".to_string(),
            overview: "Description".to_string(),
            release_year: 2020,
            genres: vec!["drama".to_string()],
            platforms: vec!["netflix".to_string()],
            popularity_score: 0.8,
        };

        let filters = SearchFilters {
            genres: vec!["action".to_string()],
            platforms: vec![],
            year_range: None,
            rating_range: None,
        };

        assert!(!vector_search.matches_filters(&content, &filters));
    }
}
