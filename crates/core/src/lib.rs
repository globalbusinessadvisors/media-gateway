//! # Media Gateway Core
//!
//! Core data structures and types for the Media Gateway platform.
//!
//! This crate provides the fundamental building blocks for content management,
//! user profiles, search functionality, and error handling across the Media Gateway ecosystem.
//!
//! ## Modules
//!
//! - `types`: Core type definitions and enums
//! - `models`: Domain models for content, users, and search
//! - `error`: Error types and handling
//! - `validation`: Validation utilities and functions

pub mod error;
pub mod models;
pub mod types;
pub mod validation;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use error::MediaGatewayError;
pub use models::{content, search, user};
pub use types::*;

/// Result type alias for Media Gateway operations
pub type Result<T> = std::result::Result<T, MediaGatewayError>;
