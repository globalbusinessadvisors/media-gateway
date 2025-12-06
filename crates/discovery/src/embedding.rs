use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Embedding service for generating query embeddings
pub struct EmbeddingService {
    client: reqwest::Client,
    api_url: String,
    api_key: String,
    model: String,
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
}

/// Embedding API request
#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: String,
    encoding_format: String,
}

/// Embedding API response
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    model: String,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: usize,
    total_tokens: usize,
}

impl EmbeddingService {
    /// Create new embedding service
    pub fn new(api_url: String, api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url,
            api_key,
            model,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate embedding for text
    pub async fn generate(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(text) {
                tracing::debug!("Cache hit for embedding: {}", text);
                return Ok(cached.clone());
            }
        }

        // Call API
        let embedding = self.call_api(text).await?;

        // L2 normalize
        let normalized = self.l2_normalize(&embedding);

        // Store in cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(text.to_string(), normalized.clone());
        }

        Ok(normalized)
    }

    /// Generate embeddings for multiple texts (batched)
    pub async fn generate_batch(&self, texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();

        // TODO: Implement actual batch API call
        // For now, call individually
        for text in texts {
            let embedding = self.generate(&text).await?;
            results.push(embedding);
        }

        Ok(results)
    }

    /// Call embedding API
    async fn call_api(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let request = EmbeddingRequest {
            model: self.model.clone(),
            input: text.to_string(),
            encoding_format: "float".to_string(),
        };

        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Embedding API error {}: {}", status, body);
        }

        let response_body: EmbeddingResponse = response.json().await?;

        if response_body.data.is_empty() {
            anyhow::bail!("No embeddings returned");
        }

        Ok(response_body.data[0].embedding.clone())
    }

    /// L2 normalize vector
    fn l2_normalize(&self, vector: &[f32]) -> Vec<f32> {
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude == 0.0 {
            return vector.to_vec();
        }

        vector.iter().map(|x| x / magnitude).collect()
    }

    /// Calculate cosine similarity between two vectors
    pub fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache size
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l2_normalize() {
        let service = EmbeddingService::new(
            "http://test".to_string(),
            "test".to_string(),
            "test".to_string(),
        );

        let vector = vec![3.0, 4.0];
        let normalized = service.l2_normalize(&vector);

        assert_eq!(normalized, vec![0.6, 0.8]);
    }

    #[test]
    fn test_cosine_similarity() {
        let service = EmbeddingService::new(
            "http://test".to_string(),
            "test".to_string(),
            "test".to_string(),
        );

        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        let c = vec![0.0, 1.0];

        assert_eq!(service.cosine_similarity(&a, &b), 1.0);
        assert_eq!(service.cosine_similarity(&a, &c), 0.0);
    }

    #[tokio::test]
    async fn test_cache() {
        let service = EmbeddingService::new(
            "http://test".to_string(),
            "test".to_string(),
            "test".to_string(),
        );

        // Manually add to cache
        {
            let mut cache = service.cache.write().await;
            cache.insert("test".to_string(), vec![1.0, 2.0, 3.0]);
        }

        assert_eq!(service.cache_size().await, 1);

        service.clear_cache().await;
        assert_eq!(service.cache_size().await, 0);
    }
}
