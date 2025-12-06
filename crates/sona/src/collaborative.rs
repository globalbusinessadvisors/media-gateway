//! Collaborative Filtering
//!
//! User-based collaborative filtering using similarity computation.

use crate::types::ScoredContent;
use anyhow::Result;
use uuid::Uuid;
use std::collections::HashMap;

/// Collaborative filtering engine
pub struct CollaborativeFilter;

impl CollaborativeFilter {
    /// Generate candidates based on similar users
    pub async fn generate_candidates(
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<ScoredContent>> {
        // Find similar users
        let similar_users = Self::find_similar_users(user_id, 50).await?;

        // Get their preferences
        let mut candidate_scores: HashMap<Uuid, f32> = HashMap::new();

        for (similar_user_id, similarity) in similar_users {
            let preferences = Self::get_user_preferences(similar_user_id).await?;

            for (content_id, score) in preferences {
                *candidate_scores.entry(content_id).or_insert(0.0) += score * similarity;
            }
        }

        // Convert to ScoredContent and sort
        let mut candidates: Vec<ScoredContent> = candidate_scores
            .into_iter()
            .map(|(content_id, score)| ScoredContent {
                content_id,
                score,
                source: crate::types::RecommendationType::Collaborative,
                based_on: vec!["Similar users".to_string()],
            })
            .collect();

        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        candidates.truncate(limit);

        Ok(candidates)
    }

    async fn find_similar_users(user_id: Uuid, k: usize) -> Result<Vec<(Uuid, f32)>> {
        // Simulated - in real implementation:
        // 1. Compute user-user similarity matrix
        // 2. Return top-K similar users
        Ok(Vec::new())
    }

    async fn get_user_preferences(user_id: Uuid) -> Result<HashMap<Uuid, f32>> {
        // Simulated - in real implementation:
        // Query user's viewing history and ratings
        Ok(HashMap::new())
    }

    /// Compute user similarity (cosine similarity of preference vectors)
    pub fn compute_similarity(vector_a: &[f32], vector_b: &[f32]) -> f32 {
        if vector_a.len() != vector_b.len() {
            return 0.0;
        }

        let dot_product: f32 = vector_a.iter().zip(vector_b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = vector_a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = vector_b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_similarity() {
        let user_a = vec![1.0, 2.0, 3.0];
        let user_b = vec![2.0, 4.0, 6.0];
        let similarity = CollaborativeFilter::compute_similarity(&user_a, &user_b);
        assert!((similarity - 1.0).abs() < 0.001); // Parallel vectors
    }
}
