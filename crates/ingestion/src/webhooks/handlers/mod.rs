//! Platform-specific webhook handlers

pub mod netflix;
pub mod generic;

pub use netflix::NetflixWebhookHandler;
pub use generic::GenericWebhookHandler;
