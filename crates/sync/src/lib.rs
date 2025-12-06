/// Media Gateway Sync Service
///
/// Real-time cross-device synchronization with CRDT support
///
/// Features:
/// - CRDT-based conflict resolution (HLC, LWW-Register, OR-Set)
/// - PubNub integration for real-time messaging
/// - WebSocket support for bidirectional sync
/// - Device management and presence tracking
/// - Watchlist and watch progress synchronization

pub mod crdt;
pub mod device;
pub mod pubnub;
pub mod server;
pub mod sync;
pub mod websocket;

pub use crdt::{HLCTimestamp, HybridLogicalClock, LWWRegister, ORSet, PlaybackPosition, PlaybackState};
pub use device::{
    AudioCodec, CommandType, DeviceCapabilities, DeviceHandoff, DeviceInfo, DevicePlatform,
    DeviceRegistry, DeviceType, HDRFormat, RemoteCommand, VideoResolution,
};
pub use pubnub::{DeviceMessage, PubNubClient, PubNubConfig, PubNubError, SyncMessage};
pub use server::{start_server, ServerState};
pub use sync::{ProgressSync, ProgressUpdate, WatchlistOperation, WatchlistSync, WatchlistUpdate};

/// Initialize tracing for the sync service
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "media_gateway_sync=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all public types are accessible
        let _hlc = HybridLogicalClock::new();
        let _or_set = ORSet::new();

        let user_id = "test-user".to_string();
        let device_id = "test-device".to_string();

        let _watchlist = WatchlistSync::new(user_id.clone(), device_id.clone());
        let _progress = ProgressSync::new(user_id.clone(), device_id.clone());
        let _registry = DeviceRegistry::new(user_id);
    }
}
