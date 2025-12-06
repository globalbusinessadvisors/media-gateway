//! OpenAI Embedding Service for semantic search

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tracing::{warn};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/embeddings";
const EMBEDDING_MODEL: &str = "text-embedding-3-small";
const EMBEDDING_DIMENSION: usize = 768;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 100;

/// OpenAI embedding request
#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
    dimensions: Option<usize>,
}

/// OpenAI embedding response
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
    prompt_tokens: u32,
    total_tokens: u32,
}

/// OpenAI error response
#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: ApiError,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
}

/// Embedding service using OpenAI API
#[derive(Clone)]
pub struct EmbeddingService {
    client: Client,
    api_key: String,
    dimension: usize,
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
}

impl EmbeddingService {
    /// Create new embedding service
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            dimension: EMBEDDING_DIMENSION,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create from OPENAI_API_KEY environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not set"))?;
        Ok(Self::new(api_key))
    }

    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Generate embedding for text with retries
    pub async fn generate(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(text) {
                tracing::debug!("Cache hit for embedding: {}", text);
                return Ok(cached.clone());
            }
        }

        let mut last_error = None;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        for attempt in 1..=MAX_RETRIES {
            match self.call_api(text).await {
                Ok(embedding) => {
                    // Store in cache
                    {
                        let mut cache = self.cache.write().await;
                        cache.insert(text.to_string(), embedding.clone());
                    }
                    return Ok(embedding);
                }
                Err(e) => {
                    warn!("Embedding attempt {} failed: {}. Retrying in {}ms...",
                        attempt, e, backoff_ms);
                    last_error = Some(e);

                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms *= 2; // Exponential backoff
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("Embedding failed after {} attempts", MAX_RETRIES)))
    }

    /// Generate embeddings for multiple texts (batch)
    pub async fn generate_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());

        // Process sequentially to avoid rate limits
        for text in texts {
            results.push(self.generate(text).await?);
        }

        Ok(results)
    }

    /// Call OpenAI API
    async fn call_api(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            input: text.to_string(),
            model: EMBEDDING_MODEL.to_string(),
            dimensions: Some(self.dimension),
        };

        let response = self.client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(anyhow!("OpenAI API error ({}): {} - {}",
                    status, error_response.error.error_type, error_response.error.message));
            }
            return Err(anyhow!("OpenAI API error ({}): {}", status, error_text));
        }

        let embedding_response: EmbeddingResponse = response.json().await?;

        if embedding_response.data.is_empty() {
            return Err(anyhow!("Empty embedding response from OpenAI"));
        }

        Ok(embedding_response.data[0].embedding.clone())
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
    fn test_embedding_service_creation() {
        let service = EmbeddingService::new("test-key".to_string());
        assert_eq!(service.dimension(), 768);
    }

    #[test]
    fn test_request_serialization() {
        let request = EmbeddingRequest {
            input: "test query".to_string(),
            model: EMBEDDING_MODEL.to_string(),
            dimensions: Some(768),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("text-embedding-3-small"));
        assert!(json.contains("768"));
    }

    #[tokio::test]
    async fn test_cache() {
        let service = EmbeddingService::new("test-key".to_string());

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
