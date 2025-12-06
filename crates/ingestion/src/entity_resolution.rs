//! Entity resolution for matching content across platforms
//!
//! Implements the ResolveContentEntity algorithm from SPARC specification:
//! 1. EIDR exact matching (100% confidence)
//! 2. External ID matching (IMDb, TMDb) (99% confidence)
//! 3. Fuzzy title + year matching (90-98% confidence, threshold 0.85)
//! 4. Embedding similarity matching (85-95% confidence, threshold 0.92)

use crate::{normalizer::CanonicalContent, Result, IngestionError};
use serde::{Deserialize, Serialize};
use strsim::normalized_levenshtein;
use std::collections::HashMap;

/// Entity match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMatch {
    /// Matched entity ID (if found)
    pub entity_id: Option<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Matching method used
    pub method: MatchMethod,
}

/// Method used for entity matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MatchMethod {
    /// Exact EIDR match
    EidrExact,
    /// External ID match (IMDb, TMDb)
    ExternalId { source: String },
    /// Fuzzy title and year matching
    FuzzyTitleYear,
    /// Embedding similarity
    EmbeddingSimilarity,
    /// No match found
    None,
}

/// Entity resolver for matching content across platforms
pub struct EntityResolver {
    // In production, this would connect to the database
    // For now, we'll use in-memory storage
    entity_index: HashMap<String, EntityRecord>,
    eidr_index: HashMap<String, String>,
    imdb_index: HashMap<String, String>,
    tmdb_index: HashMap<String, String>,
}

/// Internal entity record for matching
#[derive(Debug, Clone)]
struct EntityRecord {
    entity_id: String,
    title: String,
    normalized_title: String,
    release_year: Option<i32>,
    eidr: Option<String>,
    imdb_id: Option<String>,
    tmdb_id: Option<String>,
    embedding: Option<Vec<f32>>,
}

impl EntityResolver {
    /// Create a new entity resolver
    pub fn new() -> Self {
        Self {
            entity_index: HashMap::new(),
            eidr_index: HashMap::new(),
            imdb_index: HashMap::new(),
            tmdb_index: HashMap::new(),
        }
    }

    /// Resolve entity for given content
    ///
    /// Implements the ResolveContentEntity algorithm with multiple matching strategies.
    /// Complexity: O(log n) with indices
    pub async fn resolve(&self, content: &CanonicalContent) -> Result<EntityMatch> {
        // Strategy 1: EIDR exact match (100% confidence)
        if let Some(eidr) = content.external_ids.get("eidr") {
            if let Some(entity_id) = self.eidr_index.get(eidr) {
                return Ok(EntityMatch {
                    entity_id: Some(entity_id.clone()),
                    confidence: 1.0,
                    method: MatchMethod::EidrExact,
                });
            }
        }

        // Strategy 2: External ID matching (99% confidence)
        // Check IMDb
        if let Some(imdb_id) = content.external_ids.get("imdb") {
            if let Some(entity_id) = self.imdb_index.get(imdb_id) {
                return Ok(EntityMatch {
                    entity_id: Some(entity_id.clone()),
                    confidence: 0.99,
                    method: MatchMethod::ExternalId {
                        source: "imdb".to_string(),
                    },
                });
            }
        }

        // Check TMDb
        if let Some(tmdb_id) = content.external_ids.get("tmdb") {
            if let Some(entity_id) = self.tmdb_index.get(tmdb_id) {
                return Ok(EntityMatch {
                    entity_id: Some(entity_id.clone()),
                    confidence: 0.99,
                    method: MatchMethod::ExternalId {
                        source: "tmdb".to_string(),
                    },
                });
            }
        }

        // Strategy 3: Fuzzy title + year matching (threshold: 0.85)
        let fuzzy_match = self.fuzzy_title_year_match(content);
        if fuzzy_match.confidence >= 0.85 {
            return Ok(fuzzy_match);
        }

        // Strategy 4: Embedding similarity (threshold: 0.92)
        if let Some(embedding) = &content.embedding {
            let embedding_match = self.embedding_similarity_match(embedding);
            if embedding_match.confidence >= 0.92 {
                return Ok(embedding_match);
            }
        }

        // No match found
        Ok(EntityMatch {
            entity_id: None,
            confidence: 0.0,
            method: MatchMethod::None,
        })
    }

    /// Fuzzy title and year matching
    ///
    /// Returns 90-98% confidence based on similarity score
    fn fuzzy_title_year_match(&self, content: &CanonicalContent) -> EntityMatch {
        let normalized_title = Self::normalize_title(&content.title);
        let mut best_match: Option<EntityMatch> = None;
        let mut best_score = 0.0;

        for record in self.entity_index.values() {
            // Year must match (if available)
            if let (Some(content_year), Some(record_year)) = (content.release_year, record.release_year) {
                if (content_year - record_year).abs() > 1 {
                    continue; // Allow 1 year tolerance
                }
            }

            // Calculate title similarity
            let similarity = normalized_levenshtein(&normalized_title, &record.normalized_title);

            if similarity > best_score {
                best_score = similarity;
                best_match = Some(EntityMatch {
                    entity_id: Some(record.entity_id.clone()),
                    confidence: Self::calculate_fuzzy_confidence(similarity),
                    method: MatchMethod::FuzzyTitleYear,
                });
            }
        }

        best_match.unwrap_or(EntityMatch {
            entity_id: None,
            confidence: 0.0,
            method: MatchMethod::None,
        })
    }

    /// Embedding similarity matching
    ///
    /// Returns 85-95% confidence based on cosine similarity
    fn embedding_similarity_match(&self, embedding: &[f32]) -> EntityMatch {
        let mut best_match: Option<EntityMatch> = None;
        let mut best_similarity = 0.0;

        for record in self.entity_index.values() {
            if let Some(record_embedding) = &record.embedding {
                let similarity = Self::cosine_similarity(embedding, record_embedding);

                if similarity > best_similarity {
                    best_similarity = similarity;
                    best_match = Some(EntityMatch {
                        entity_id: Some(record.entity_id.clone()),
                        confidence: Self::calculate_embedding_confidence(similarity),
                        method: MatchMethod::EmbeddingSimilarity,
                    });
                }
            }
        }

        best_match.unwrap_or(EntityMatch {
            entity_id: None,
            confidence: 0.0,
            method: MatchMethod::None,
        })
    }

    /// Normalize title for fuzzy matching
    fn normalize_title(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Calculate confidence for fuzzy matching (90-98%)
    fn calculate_fuzzy_confidence(similarity: f64) -> f32 {
        // Map similarity [0.85, 1.0] to confidence [0.90, 0.98]
        let normalized = ((similarity - 0.85) / 0.15).max(0.0).min(1.0);
        (0.90 + normalized * 0.08) as f32
    }

    /// Calculate confidence for embedding matching (85-95%)
    fn calculate_embedding_confidence(similarity: f64) -> f32 {
        // Map similarity [0.92, 1.0] to confidence [0.85, 0.95]
        let normalized = ((similarity - 0.92) / 0.08).max(0.0).min(1.0);
        (0.85 + normalized * 0.10) as f32
    }

    /// Calculate cosine similarity between two embeddings
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        (dot_product / (norm_a * norm_b)) as f64
    }

    /// Add entity to index (for testing and database sync)
    #[cfg(test)]
    pub fn add_entity(
        &mut self,
        entity_id: String,
        title: String,
        release_year: Option<i32>,
        eidr: Option<String>,
        imdb_id: Option<String>,
        tmdb_id: Option<String>,
        embedding: Option<Vec<f32>>,
    ) {
        let normalized_title = Self::normalize_title(&title);

        let record = EntityRecord {
            entity_id: entity_id.clone(),
            title: title.clone(),
            normalized_title,
            release_year,
            eidr: eidr.clone(),
            imdb_id: imdb_id.clone(),
            tmdb_id: tmdb_id.clone(),
            embedding,
        };

        // Update indices
        if let Some(eidr_val) = eidr {
            self.eidr_index.insert(eidr_val, entity_id.clone());
        }
        if let Some(imdb_val) = imdb_id {
            self.imdb_index.insert(imdb_val, entity_id.clone());
        }
        if let Some(tmdb_val) = tmdb_id {
            self.tmdb_index.insert(tmdb_val, entity_id.clone());
        }

        self.entity_index.insert(entity_id, record);
    }
}

impl Default for EntityResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_normalization() {
        assert_eq!(
            EntityResolver::normalize_title("The Matrix (1999)"),
            "the matrix 1999"
        );
        assert_eq!(
            EntityResolver::normalize_title("Star Wars: A New Hope"),
            "star wars a new hope"
        );
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((EntityResolver::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!(EntityResolver::cosine_similarity(&c, &d).abs() < 0.001);
    }

    #[test]
    fn test_fuzzy_confidence_calculation() {
        assert!((EntityResolver::calculate_fuzzy_confidence(0.85) - 0.90).abs() < 0.01);
        assert!((EntityResolver::calculate_fuzzy_confidence(1.0) - 0.98).abs() < 0.01);
    }

    #[test]
    fn test_embedding_confidence_calculation() {
        assert!((EntityResolver::calculate_embedding_confidence(0.92) - 0.85).abs() < 0.01);
        assert!((EntityResolver::calculate_embedding_confidence(1.0) - 0.95).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_eidr_exact_match() {
        let mut resolver = EntityResolver::new();
        resolver.add_entity(
            "entity_1".to_string(),
            "The Matrix".to_string(),
            Some(1999),
            Some("10.5240/ABCD-1234".to_string()),
            None,
            None,
            None,
        );

        let mut content = CanonicalContent {
            platform_content_id: "test".to_string(),
            platform_id: "netflix".to_string(),
            entity_id: None,
            title: "The Matrix".to_string(),
            overview: None,
            content_type: crate::normalizer::ContentType::Movie,
            release_year: Some(1999),
            runtime_minutes: None,
            genres: vec![],
            external_ids: HashMap::new(),
            availability: crate::normalizer::AvailabilityInfo {
                regions: vec![],
                subscription_required: false,
                purchase_price: None,
                rental_price: None,
                currency: None,
                available_from: None,
                available_until: None,
            },
            images: crate::normalizer::ImageSet::default(),
            rating: None,
            user_rating: None,
            embedding: None,
            updated_at: chrono::Utc::now(),
        };

        content.external_ids.insert("eidr".to_string(), "10.5240/ABCD-1234".to_string());

        let result = resolver.resolve(&content).await.unwrap();
        assert_eq!(result.entity_id, Some("entity_1".to_string()));
        assert_eq!(result.confidence, 1.0);
        assert_eq!(result.method, MatchMethod::EidrExact);
    }
}
