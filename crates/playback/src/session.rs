//! Playback session management

use redis::{AsyncCommands, Client, aio::MultiplexedConnection};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};

const SESSION_TTL_SECS: u64 = 86400; // 24 hours

/// Playback session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub device_id: String,
    pub position_seconds: u32,
    pub duration_seconds: u32,
    pub playback_state: PlaybackState,
    pub quality: VideoQuality,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackState {
    Playing,
    Paused,
    Buffering,
    Stopped,
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoQuality {
    Auto,
    Low,    // 480p
    Medium, // 720p
    High,   // 1080p
    Ultra,  // 4K
}

impl Default for VideoQuality {
    fn default() -> Self {
        VideoQuality::Auto
    }
}

/// Request to create a new playback session
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub device_id: String,
    pub duration_seconds: u32,
    pub quality: Option<VideoQuality>,
}

/// Request to update playback position
#[derive(Debug, Deserialize)]
pub struct UpdatePositionRequest {
    pub position_seconds: u32,
    pub playback_state: Option<PlaybackState>,
}

/// Session manager using Redis storage
pub struct SessionManager {
    client: Client,
}

impl SessionManager {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub fn from_env() -> Result<Self, redis::RedisError> {
        let url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        Self::new(&url)
    }

    async fn get_conn(&self) -> Result<MultiplexedConnection, redis::RedisError> {
        self.client.get_multiplexed_async_connection().await
    }

    /// Create new playback session
    pub async fn create(&self, request: CreateSessionRequest) -> Result<PlaybackSession, SessionError> {
        let mut conn = self.get_conn().await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let now = Utc::now();
        let session = PlaybackSession {
            id: Uuid::new_v4(),
            user_id: request.user_id,
            content_id: request.content_id,
            device_id: request.device_id,
            position_seconds: 0,
            duration_seconds: request.duration_seconds,
            playback_state: PlaybackState::Playing,
            quality: request.quality.unwrap_or_default(),
            started_at: now,
            updated_at: now,
        };

        let key = format!("session:{}", session.id);
        let value = serde_json::to_string(&session)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        conn.set_ex(&key, value, SESSION_TTL_SECS)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // Also index by user for lookup
        let user_key = format!("user:{}:sessions", session.user_id);
        conn.sadd(&user_key, session.id.to_string())
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;
        conn.expire(&user_key, SESSION_TTL_SECS as i64)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        Ok(session)
    }

    /// Get session by ID
    pub async fn get(&self, session_id: Uuid) -> Result<Option<PlaybackSession>, SessionError> {
        let mut conn = self.get_conn().await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let key = format!("session:{}", session_id);
        let value: Option<String> = conn.get(&key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        match value {
            Some(v) => {
                let session: PlaybackSession = serde_json::from_str(&v)
                    .map_err(|e| SessionError::Serialization(e.to_string()))?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    /// Update playback position
    pub async fn update_position(
        &self,
        session_id: Uuid,
        request: UpdatePositionRequest,
    ) -> Result<PlaybackSession, SessionError> {
        let mut session = self.get(session_id).await?
            .ok_or(SessionError::NotFound)?;

        session.position_seconds = request.position_seconds;
        session.updated_at = Utc::now();

        if let Some(state) = request.playback_state {
            session.playback_state = state;
        }

        // Check if playback completed
        if session.position_seconds >= session.duration_seconds {
            session.playback_state = PlaybackState::Ended;
        }

        let mut conn = self.get_conn().await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let key = format!("session:{}", session.id);
        let value = serde_json::to_string(&session)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        // Get remaining TTL and preserve it
        let ttl: i64 = conn.ttl(&key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let ttl = if ttl > 0 { ttl as u64 } else { SESSION_TTL_SECS };

        conn.set_ex(&key, value, ttl)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        Ok(session)
    }

    /// Delete session
    pub async fn delete(&self, session_id: Uuid) -> Result<(), SessionError> {
        let session = self.get(session_id).await?;

        let mut conn = self.get_conn().await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let key = format!("session:{}", session_id);
        conn.del(&key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // Remove from user index
        if let Some(s) = session {
            let user_key = format!("user:{}:sessions", s.user_id);
            conn.srem(&user_key, session_id.to_string())
                .await
                .map_err(|e| SessionError::Storage(e.to_string()))?;
        }

        Ok(())
    }

    /// Get all sessions for a user
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<PlaybackSession>, SessionError> {
        let mut conn = self.get_conn().await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let user_key = format!("user:{}:sessions", user_id);
        let session_ids: Vec<String> = conn.smembers(&user_key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let mut sessions = Vec::new();
        for id_str in session_ids {
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if let Some(session) = self.get(id).await? {
                    sessions.push(session);
                }
            }
        }

        Ok(sessions)
    }

    /// Check health
    pub async fn is_healthy(&self) -> bool {
        match self.get_conn().await {
            Ok(mut conn) => {
                redis::cmd("PING")
                    .query_async::<_, String>(&mut conn)
                    .await
                    .is_ok()
            }
            Err(_) => false,
        }
    }
}

/// Session errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl actix_web::ResponseError for SessionError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            SessionError::NotFound => {
                actix_web::HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Session not found"
                }))
            }
            _ => {
                actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": self.to_string()
                }))
            }
        }
    }
}
