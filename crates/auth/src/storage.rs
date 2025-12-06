//! Redis-backed storage for OAuth authentication state

use redis::{AsyncCommands, Client, aio::MultiplexedConnection};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use crate::error::{AuthError, Result};
use crate::oauth::{
    device::DeviceCode,
    pkce::{AuthorizationCode, PkceChallenge},
};

/// TTL constants for different token types
const PKCE_TTL_SECS: u64 = 600;      // 10 minutes
const AUTH_CODE_TTL_SECS: u64 = 300; // 5 minutes
const DEVICE_CODE_TTL_SECS: u64 = 900; // 15 minutes

/// Redis storage manager for auth state
#[derive(Clone)]
pub struct AuthStorage {
    client: Client,
}

impl AuthStorage {
    /// Create new Redis storage connection
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| AuthError::Internal(format!("Redis connection failed: {}", e)))?;
        Ok(Self { client })
    }

    /// Create from REDIS_URL environment variable
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        Self::new(&url)
    }

    /// Get async connection
    async fn get_conn(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AuthError::Internal(format!("Redis connection error: {}", e)))
    }

    // ========== PKCE Sessions ==========

    /// Store PKCE session with 10 minute TTL
    pub async fn store_pkce(&self, state: &str, pkce: &PkceChallenge) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("pkce:{}", state);
        let value = serde_json::to_string(pkce)
            .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;

        conn.set_ex(&key, value, PKCE_TTL_SECS)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;

        Ok(())
    }

    /// Get PKCE session
    pub async fn get_pkce(&self, state: &str) -> Result<Option<PkceChallenge>> {
        let mut conn = self.get_conn().await?;
        let key = format!("pkce:{}", state);

        let value: Option<String> = conn.get(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

        match value {
            Some(v) => {
                let pkce: PkceChallenge = serde_json::from_str(&v)
                    .map_err(|e| AuthError::Internal(format!("Deserialization error: {}", e)))?;
                Ok(Some(pkce))
            }
            None => Ok(None),
        }
    }

    /// Delete PKCE session
    pub async fn delete_pkce(&self, state: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("pkce:{}", state);
        conn.del(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis DEL error: {}", e)))?;
        Ok(())
    }

    // ========== Authorization Codes ==========

    /// Store authorization code with 5 minute TTL
    pub async fn store_auth_code(&self, code: &str, auth_code: &AuthorizationCode) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("authcode:{}", code);
        let value = serde_json::to_string(auth_code)
            .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;

        conn.set_ex(&key, value, AUTH_CODE_TTL_SECS)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;

        Ok(())
    }

    /// Get authorization code
    pub async fn get_auth_code(&self, code: &str) -> Result<Option<AuthorizationCode>> {
        let mut conn = self.get_conn().await?;
        let key = format!("authcode:{}", code);

        let value: Option<String> = conn.get(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

        match value {
            Some(v) => {
                let auth_code: AuthorizationCode = serde_json::from_str(&v)
                    .map_err(|e| AuthError::Internal(format!("Deserialization error: {}", e)))?;
                Ok(Some(auth_code))
            }
            None => Ok(None),
        }
    }

    /// Update authorization code (for marking as used)
    pub async fn update_auth_code(&self, code: &str, auth_code: &AuthorizationCode) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("authcode:{}", code);

        // Get remaining TTL
        let ttl: i64 = conn.ttl(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis TTL error: {}", e)))?;

        if ttl > 0 {
            let value = serde_json::to_string(auth_code)
                .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;
            conn.set_ex(&key, value, ttl as u64)
                .await
                .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;
        }

        Ok(())
    }

    /// Delete authorization code
    pub async fn delete_auth_code(&self, code: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("authcode:{}", code);
        conn.del(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis DEL error: {}", e)))?;
        Ok(())
    }

    // ========== Device Codes ==========

    /// Store device code with 15 minute TTL
    pub async fn store_device_code(&self, device_code: &str, device: &DeviceCode) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("devicecode:{}", device_code);
        let value = serde_json::to_string(device)
            .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;

        conn.set_ex(&key, value, DEVICE_CODE_TTL_SECS)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;

        Ok(())
    }

    /// Get device code
    pub async fn get_device_code(&self, device_code: &str) -> Result<Option<DeviceCode>> {
        let mut conn = self.get_conn().await?;
        let key = format!("devicecode:{}", device_code);

        let value: Option<String> = conn.get(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

        match value {
            Some(v) => {
                let device: DeviceCode = serde_json::from_str(&v)
                    .map_err(|e| AuthError::Internal(format!("Deserialization error: {}", e)))?;
                Ok(Some(device))
            }
            None => Ok(None),
        }
    }

    /// Update device code (for setting user_id after approval)
    pub async fn update_device_code(&self, device_code: &str, device: &DeviceCode) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("devicecode:{}", device_code);

        let ttl: i64 = conn.ttl(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis TTL error: {}", e)))?;

        if ttl > 0 {
            let value = serde_json::to_string(device)
                .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;
            conn.set_ex(&key, value, ttl as u64)
                .await
                .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;
        }

        Ok(())
    }

    /// Delete device code
    pub async fn delete_device_code(&self, device_code: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("devicecode:{}", device_code);
        conn.del(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis DEL error: {}", e)))?;
        Ok(())
    }

    /// Check Redis health
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl_constants() {
        assert_eq!(PKCE_TTL_SECS, 600);
        assert_eq!(AUTH_CODE_TTL_SECS, 300);
        assert_eq!(DEVICE_CODE_TTL_SECS, 900);
    }
}
