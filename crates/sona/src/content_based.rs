//! Content-Based Filtering
//!
//! Recommends content similar to user's viewing history.

use crate::types::ScoredContent;
use crate::profile::UserProfile;
use anyhow::Result;
use uuid::Uuid;

/// Content-based filtering engine
pub struct ContentBasedFilter;

impl ContentBasedFilter {
    /// Generate candidates based on content similarity
    pub async fn generate_candidates(
        profile: &UserProfile,
        limit: usize,
    ) -> Result<Vec<ScoredContent>> {
        // Get user's recently watched content
        let recent_content = Self::get_recent_content(&profile.user_id, 20).await?;

        let mut candidates = Vec::new();

        for content_id in recent_content {
            // Find similar content
            let similar = Self::find_similar_content(content_id, 10).await?;
            candidates.extend(similar);
        }

        // Deduplicate and sort by score
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        candidates.dedup_by(|a, b| a.content_id == b.content_id);
        candidates.truncate(limit);

        Ok(candidates)
    }

    async fn get_recent_content(user_id: &Uuid, limit: usize) -> Result<Vec<Uuid>> {
        // Simulated - in real implementation:
        // Query user's recent viewing history
        Ok(Vec::new())
    }

    async fn find_similar_content(content_id: Uuid, k: usize) -> Result<Vec<ScoredContent>> {
        // Simulated - in real implementation:
        // 1. Get content embedding
        // 2. Vector search for similar embeddings
        // 3. Score by genre overlap, credits similarity
        Ok(Vec::new())
    }

    /// Calculate genre overlap score
    pub fn genre_overlap_score(genres_a: &[String], genres_b: &[String]) -> f32 {
        if genres_a.is_empty() || genres_b.is_empty() {
            return 0.0;
        }

        let set_a: std::collections::HashSet<_> = genres_a.iter().collect();
        let set_b: std::collections::HashSet<_> = genres_b.iter().collect();

        let intersection_size = set_a.intersection(&set_b).count() as f32;
        let union_size = set_a.union(&set_b).count() as f32;

        intersection_size / union_size // Jaccard similarity
    }

    /// Calculate credits similarity (actors, directors)
    pub fn credits_similarity(credits_a: &[String], credits_b: &[String]) -> f32 {
        if credits_a.is_empty() || credits_b.is_empty() {
            return 0.0;
        }

        let set_a: std::collections::HashSet<_> = credits_a.iter().collect();
        let set_b: std::collections::HashSet<_> = credits_b.iter().collect();

        let intersection_size = set_a.intersection(&set_b).count() as f32;
        let max_size = set_a.len().max(set_b.len()) as f32;

        intersection_size / max_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genre_overlap() {
        let genres_a = vec!["action".to_string(), "sci-fi".to_string()];
        let genres_b = vec!["sci-fi".to_string(), "thriller".to_string()];
        let score = ContentBasedFilter::genre_overlap_score(&genres_a, &genres_b);
        assert!((score - 0.333).abs() < 0.01); // 1/3 Jaccard similarity
    }

    #[test]
    fn test_credits_similarity() {
        let credits_a = vec!["Actor A".to_string(), "Actor B".to_string()];
        let credits_b = vec!["Actor B".to_string(), "Actor C".to_string()];
        let score = ContentBasedFilter::credits_similarity(&credits_a, &credits_b);
        assert!((score - 0.5).abs() < 0.01); // 1/2 overlap
    }
}
