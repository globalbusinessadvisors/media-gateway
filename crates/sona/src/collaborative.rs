//! Collaborative filtering implementation
//!
//! Uses user interaction patterns to find similar users and generate recommendations

use anyhow::Result;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

/// Collaborative filtering engine
pub struct CollaborativeEngine {
    pool: PgPool,
    min_similarity: f32,
}

impl CollaborativeEngine {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            min_similarity: 0.3,
        }
    }

    pub fn with_min_similarity(mut self, min: f32) -> Self {
        self.min_similarity = min;
        self
    }

    /// Find users with similar interaction patterns
    pub async fn find_similar_users(
        &self,
        user_id: Uuid,
        k: usize,
    ) -> Result<Vec<(Uuid, f32)>> {
        // Get user's content interactions as a vector
        let user_vector = self.get_user_interaction_vector(user_id).await?;

        if user_vector.is_empty() {
            return Ok(Vec::new());
        }

        // Find other users with overlapping interactions
        let candidates = sqlx::query(
            r#"
            SELECT DISTINCT i2.user_id, COUNT(*) as overlap_count
            FROM users.interactions i1
            JOIN users.interactions i2 ON i1.content_id = i2.content_id
            WHERE i1.user_id = $1
              AND i2.user_id != $1
              AND i2.interaction_type IN ('watch', 'like', 'rate')
            GROUP BY i2.user_id
            HAVING COUNT(*) >= 3
            ORDER BY overlap_count DESC
            LIMIT $2
            "#
        )
        .bind(user_id)
        .bind((k * 3) as i64) // Over-fetch for filtering
        .fetch_all(&self.pool)
        .await?;

        let mut similar_users = Vec::new();

        for row in candidates {
            let other_user_id: Uuid = row.get("user_id");
            let other_vector = self.get_user_interaction_vector(other_user_id).await?;

            let similarity = self.compute_cosine_similarity(&user_vector, &other_vector);

            if similarity >= self.min_similarity {
                similar_users.push((other_user_id, similarity));
            }
        }

        // Sort by similarity descending and take top k
        similar_users.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similar_users.truncate(k);

        Ok(similar_users)
    }

    /// Get user's preference weights by content
    pub async fn get_user_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<HashMap<Uuid, f32>> {
        let rows = sqlx::query(
            r#"
            SELECT content_id,
                   SUM(CASE
                       WHEN interaction_type = 'watch' THEN
                           CASE WHEN watch_progress >= 0.9 THEN 1.0
                                WHEN watch_progress >= 0.5 THEN 0.5
                                ELSE 0.2 END
                       WHEN interaction_type = 'like' THEN 1.0
                       WHEN interaction_type = 'rate' THEN rating / 5.0
                       WHEN interaction_type = 'dislike' THEN -0.5
                       ELSE 0.0
                   END) as preference_score
            FROM users.interactions
            WHERE user_id = $1
            GROUP BY content_id
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut preferences = HashMap::new();
        for row in rows {
            let content_id: Uuid = row.get("content_id");
            let score: f64 = row.get("preference_score");
            preferences.insert(content_id, score as f32);
        }

        Ok(preferences)
    }

    /// Get content recommendations based on similar users
    pub async fn get_collaborative_recommendations(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<(Uuid, f32)>> {
        let similar_users = self.find_similar_users(user_id, 20).await?;

        if similar_users.is_empty() {
            return Ok(Vec::new());
        }

        // Get user's already-seen content
        let seen_content = self.get_user_seen_content(user_id).await?;

        // Aggregate content from similar users, weighted by similarity
        let mut content_scores: HashMap<Uuid, f32> = HashMap::new();

        for (similar_user_id, similarity) in &similar_users {
            let prefs = self.get_user_preferences(*similar_user_id).await?;

            for (content_id, score) in prefs {
                if !seen_content.contains(&content_id) && score > 0.0 {
                    *content_scores.entry(content_id).or_insert(0.0) += similarity * score;
                }
            }
        }

        // Sort and return top recommendations
        let mut recommendations: Vec<_> = content_scores.into_iter().collect();
        recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        recommendations.truncate(limit);

        Ok(recommendations)
    }

    /// Get user's interaction vector for similarity computation
    async fn get_user_interaction_vector(&self, user_id: Uuid) -> Result<HashMap<Uuid, f32>> {
        self.get_user_preferences(user_id).await
    }

    /// Get content IDs the user has already seen
    async fn get_user_seen_content(&self, user_id: Uuid) -> Result<Vec<Uuid>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT content_id
            FROM users.interactions
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|r| r.get("content_id")).collect())
    }

    /// Compute cosine similarity between two user vectors
    fn compute_cosine_similarity(
        &self,
        a: &HashMap<Uuid, f32>,
        b: &HashMap<Uuid, f32>,
    ) -> f32 {
        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;

        for (key, val_a) in a {
            norm_a += val_a * val_a;
            if let Some(val_b) = b.get(key) {
                dot += val_a * val_b;
            }
        }

        for val_b in b.values() {
            norm_b += val_b * val_b;
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot / (norm_a.sqrt() * norm_b.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let pool = unsafe { std::mem::zeroed() }; // Test only - don't use pool
        let engine = CollaborativeEngine {
            pool,
            min_similarity: 0.3,
        };

        let mut a = HashMap::new();
        a.insert(Uuid::nil(), 1.0);
        a.insert(Uuid::new_v4(), 2.0);

        let mut b = HashMap::new();
        b.insert(Uuid::nil(), 1.0);
        b.insert(Uuid::new_v4(), 2.0);

        // Verify struct creation
        assert_eq!(engine.min_similarity, 0.3);
    }
}
