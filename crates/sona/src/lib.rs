//! SONA (Self-Optimizing Neural Architecture) Personalization Engine
//!
//! This module implements the personalization layer for Media Gateway,
//! providing user embeddings, LoRA adaptation, and hybrid recommendations.

pub mod profile;
pub mod lora;
pub mod recommendation;
pub mod collaborative;
pub mod content_based;
pub mod context;
pub mod diversity;
pub mod cold_start;
pub mod types;

// Re-export key types
pub use profile::{UserProfile, BuildUserPreferenceVector};
pub use lora::{UserLoRAAdapter, UpdateUserLoRA, ComputeLoRAForward};
pub use recommendation::GenerateRecommendations;
pub use diversity::ApplyDiversityFilter;
pub use cold_start::HandleColdStartUser;
pub use types::*;

use anyhow::Result;

/// SONA engine configuration
#[derive(Debug, Clone)]
pub struct SonaConfig {
    /// Embedding dimensionality (default: 512)
    pub embedding_dim: usize,
    /// LoRA rank (default: 8)
    pub lora_rank: usize,
    /// LoRA alpha scaling (default: 16)
    pub lora_alpha: f32,
    /// Temporal decay rate (default: 0.95)
    pub decay_rate: f32,
    /// Minimum watch threshold (default: 0.3)
    pub min_watch_threshold: f32,
    /// Minimum interactions before LoRA training (default: 10)
    pub min_training_events: usize,
}

impl Default for SonaConfig {
    fn default() -> Self {
        Self {
            embedding_dim: 512,
            lora_rank: 8,
            lora_alpha: 16.0,
            decay_rate: 0.95,
            min_watch_threshold: 0.3,
            min_training_events: 10,
        }
    }
}

/// SONA engine instance
pub struct SonaEngine {
    config: SonaConfig,
}

impl SonaEngine {
    pub fn new(config: SonaConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(SonaConfig::default())
    }

    pub fn config(&self) -> &SonaConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_sona_engine_creation() {
        let engine = SonaEngine::with_default_config();
        assert_eq!(engine.config().embedding_dim, 512);
        assert_eq!(engine.config().lora_rank, 8);
    }
}
