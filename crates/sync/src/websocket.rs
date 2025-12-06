/// WebSocket handler for real-time synchronization
///
/// Manages WebSocket connections with clients for bidirectional sync

use actix::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// WebSocket connection heartbeat interval (30 seconds)
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Client timeout (60 seconds - 2 missed heartbeats)
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

/// WebSocket session actor
pub struct SyncWebSocket {
    /// User identifier
    user_id: String,

    /// Device identifier
    device_id: String,

    /// Last heartbeat timestamp
    hb: Instant,
}

impl SyncWebSocket {
    /// Create new WebSocket session
    pub fn new(user_id: String, device_id: String) -> Self {
        Self {
            user_id,
            device_id,
            hb: Instant::now(),
        }
    }

    /// Start heartbeat process
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Check if client has sent heartbeat recently
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                tracing::warn!(
                    "WebSocket client {} heartbeat timeout, disconnecting",
                    act.device_id
                );
                ctx.stop();
                return;
            }

            // Send ping to client
            ctx.ping(b"");
        });
    }

    /// Handle incoming sync message
    fn handle_sync_message(&mut self, msg: WebSocketMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            WebSocketMessage::WatchlistUpdate { .. } => {
                tracing::debug!("Received watchlist update from {}", self.device_id);
                // In production: broadcast to other user devices via PubNub
            }
            WebSocketMessage::ProgressUpdate { .. } => {
                tracing::debug!("Received progress update from {}", self.device_id);
                // In production: broadcast to other user devices via PubNub
            }
            WebSocketMessage::DeviceHeartbeat => {
                tracing::trace!("Received heartbeat from {}", self.device_id);
                self.hb = Instant::now();
            }
            WebSocketMessage::DeviceCommand { .. } => {
                tracing::debug!("Received device command from {}", self.device_id);
                // In production: route to target device
            }
            WebSocketMessage::Ping => {
                ctx.pong(b"");
            }
            WebSocketMessage::Pong => {
                self.hb = Instant::now();
            }
        }
    }
}

impl Actor for SyncWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::info!(
            "WebSocket connection established for user {} device {}",
            self.user_id,
            self.device_id
        );
        self.start_heartbeat(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::info!(
            "WebSocket connection closed for user {} device {}",
            self.user_id,
            self.device_id
        );
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SyncWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // Parse JSON message
                match serde_json::from_str::<WebSocketMessage>(&text) {
                    Ok(msg) => self.handle_sync_message(msg, ctx),
                    Err(e) => {
                        tracing::error!("Failed to parse WebSocket message: {}", e);
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                tracing::warn!("Binary WebSocket messages not supported");
            }
            Ok(ws::Message::Close(reason)) => {
                tracing::info!("WebSocket close received: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                tracing::warn!("WebSocket continuation frames not supported");
            }
            Ok(ws::Message::Nop) => {}
            Err(e) => {
                tracing::error!("WebSocket protocol error: {}", e);
                ctx.stop();
            }
        }
    }
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "watchlist_update")]
    WatchlistUpdate {
        operation: String,
        content_id: String,
        timestamp: String,
    },

    #[serde(rename = "progress_update")]
    ProgressUpdate {
        content_id: String,
        position_seconds: u32,
        timestamp: String,
    },

    #[serde(rename = "device_heartbeat")]
    DeviceHeartbeat,

    #[serde(rename = "device_command")]
    DeviceCommand {
        target_device_id: String,
        command: String,
    },

    #[serde(rename = "ping")]
    Ping,

    #[serde(rename = "pong")]
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_message_serialization() {
        let msg = WebSocketMessage::ProgressUpdate {
            content_id: "content-1".to_string(),
            position_seconds: 100,
            timestamp: "1000".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("progress_update"));

        let deserialized: WebSocketMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            WebSocketMessage::ProgressUpdate { position_seconds, .. } => {
                assert_eq!(position_seconds, 100);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
