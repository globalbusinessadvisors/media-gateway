/// PubNub integration for real-time cross-device synchronization
///
/// Channel structure:
/// - user.{userId}.sync - Watchlist, preferences, progress
/// - user.{userId}.devices - Device presence, heartbeat
/// - user.{userId}.notifications - Alerts, recommendations

use crate::crdt::HLCTimestamp;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

/// PubNub client configuration
#[derive(Debug, Clone)]
pub struct PubNubConfig {
    /// PubNub publish key
    pub publish_key: String,

    /// PubNub subscribe key
    pub subscribe_key: String,

    /// API origin
    pub origin: String,
}

impl Default for PubNubConfig {
    fn default() -> Self {
        Self {
            publish_key: std::env::var("PUBNUB_PUBLISH_KEY")
                .unwrap_or_else(|_| "demo".to_string()),
            subscribe_key: std::env::var("PUBNUB_SUBSCRIBE_KEY")
                .unwrap_or_else(|_| "demo".to_string()),
            origin: "ps.pndsn.com".to_string(),
        }
    }
}

/// PubNub client for Media Gateway
pub struct PubNubClient {
    config: PubNubConfig,
    http_client: Client,
    user_id: String,
    device_id: String,
}

impl PubNubClient {
    /// Create new PubNub client
    pub fn new(config: PubNubConfig, user_id: String, device_id: String) -> Self {
        Self {
            config,
            http_client: Client::new(),
            user_id,
            device_id,
        }
    }

    /// Get sync channel name for user
    pub fn sync_channel(&self) -> String {
        format!("user.{}.sync", self.user_id)
    }

    /// Get devices channel name for user
    pub fn devices_channel(&self) -> String {
        format!("user.{}.devices", self.user_id)
    }

    /// Get notifications channel name for user
    pub fn notifications_channel(&self) -> String {
        format!("user.{}.notifications", self.user_id)
    }

    /// Publish message to channel
    pub async fn publish<T: Serialize>(
        &self,
        channel: &str,
        message: &T,
    ) -> Result<PublishResponse, PubNubError> {
        let url = format!(
            "https://{}/publish/{}/{}/0/{}/0",
            self.config.origin, self.config.publish_key, self.config.subscribe_key, channel
        );

        let message_json = serde_json::to_string(message)
            .map_err(|e| PubNubError::SerializationError(e.to_string()))?;

        let response = self
            .http_client
            .post(&url)
            .json(&message_json)
            .send()
            .await
            .map_err(|e| PubNubError::NetworkError(e.to_string()))?;

        let publish_response: PublishResponse = response
            .json()
            .await
            .map_err(|e| PubNubError::DeserializationError(e.to_string()))?;

        Ok(publish_response)
    }

    /// Subscribe to channels (establishes long-poll connection)
    pub async fn subscribe(&self, channels: Vec<String>) -> Result<(), PubNubError> {
        // In production, this would establish a persistent connection
        // For now, this is a placeholder
        tracing::info!("Subscribing to channels: {:?}", channels);
        Ok(())
    }

    /// Publish presence heartbeat
    pub async fn heartbeat(&self) -> Result<(), PubNubError> {
        let url = format!(
            "https://{}/v2/presence/sub-key/{}/channel/{}/heartbeat",
            self.config.origin,
            self.config.subscribe_key,
            self.devices_channel()
        );

        self.http_client
            .get(&url)
            .query(&[("heartbeat", "300"), ("uuid", &self.device_id)])
            .send()
            .await
            .map_err(|e| PubNubError::NetworkError(e.to_string()))?;

        Ok(())
    }

    /// Get presence information for channel
    pub async fn here_now(&self, channel: &str) -> Result<HereNowResponse, PubNubError> {
        let url = format!(
            "https://{}/v2/presence/sub-key/{}/channel/{}",
            self.config.origin, self.config.subscribe_key, channel
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| PubNubError::NetworkError(e.to_string()))?;

        let here_now: HereNowResponse = response
            .json()
            .await
            .map_err(|e| PubNubError::DeserializationError(e.to_string()))?;

        Ok(here_now)
    }

    /// Fetch message history from channel
    pub async fn history(
        &self,
        channel: &str,
        count: usize,
    ) -> Result<HistoryResponse, PubNubError> {
        let url = format!(
            "https://{}/v2/history/sub-key/{}/channel/{}",
            self.config.origin, self.config.subscribe_key, channel
        );

        let response = self
            .http_client
            .get(&url)
            .query(&[("count", count.to_string())])
            .send()
            .await
            .map_err(|e| PubNubError::NetworkError(e.to_string()))?;

        let history: HistoryResponse = response
            .json()
            .await
            .map_err(|e| PubNubError::DeserializationError(e.to_string()))?;

        Ok(history)
    }
}

/// PubNub publish response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub status: i32,
    pub timetoken: String,
}

/// PubNub presence response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereNowResponse {
    pub status: i32,
    pub message: String,
    pub occupancy: usize,
    pub uuids: Vec<String>,
}

/// PubNub history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub status: i32,
    pub messages: Vec<serde_json::Value>,
}

/// PubNub errors
#[derive(Debug, Error)]
pub enum PubNubError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Channel error: {0}")]
    ChannelError(String),
}

/// Message types for PubNub channels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    #[serde(rename = "watchlist_update")]
    WatchlistUpdate {
        operation: String,
        content_id: String,
        unique_tag: String,
        timestamp: HLCTimestamp,
        device_id: String,
    },

    #[serde(rename = "progress_update")]
    ProgressUpdate {
        content_id: String,
        position_seconds: u32,
        duration_seconds: u32,
        timestamp: HLCTimestamp,
        device_id: String,
    },

    #[serde(rename = "device_handoff")]
    DeviceHandoff {
        target_device_id: String,
        content_id: String,
        position_seconds: Option<u32>,
        timestamp: HLCTimestamp,
    },
}

/// Device message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DeviceMessage {
    #[serde(rename = "device_heartbeat")]
    Heartbeat {
        device_id: String,
        capabilities: DeviceCapabilities,
        timestamp: HLCTimestamp,
    },

    #[serde(rename = "device_command")]
    Command {
        target_device_id: String,
        command: RemoteCommand,
        timestamp: HLCTimestamp,
    },
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub max_resolution: String,
    pub hdr_support: Vec<String>,
    pub audio_codecs: Vec<String>,
    pub can_cast: bool,
}

/// Remote control commands
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command_type")]
pub enum RemoteCommand {
    #[serde(rename = "play")]
    Play,

    #[serde(rename = "pause")]
    Pause,

    #[serde(rename = "seek")]
    Seek { position_seconds: u32 },

    #[serde(rename = "cast")]
    Cast { content_id: String },
}
