/// Watchlist synchronization using OR-Set CRDT
///
/// Supports add/remove operations with add-wins conflict resolution

use crate::crdt::{HLCTimestamp, HybridLogicalClock, ORSet, ORSetDelta, ORSetOperation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Watchlist sync manager
pub struct WatchlistSync {
    /// User identifier
    user_id: String,

    /// Device identifier
    device_id: String,

    /// OR-Set for watchlist
    or_set: Arc<RwLock<ORSet>>,

    /// HLC for timestamp generation
    hlc: Arc<HybridLogicalClock>,
}

impl WatchlistSync {
    /// Create new watchlist sync manager
    pub fn new(user_id: String, device_id: String) -> Self {
        Self {
            user_id,
            device_id,
            or_set: Arc::new(RwLock::new(ORSet::new())),
            hlc: Arc::new(HybridLogicalClock::new()),
        }
    }

    /// Add content to watchlist
    pub fn add_to_watchlist(&self, content_id: String) -> WatchlistUpdate {
        let timestamp = self.hlc.now();
        let mut set = self.or_set.write();
        let unique_tag = set.add(content_id.clone(), timestamp, self.device_id.clone());

        WatchlistUpdate {
            operation: WatchlistOperation::Add,
            content_id,
            unique_tag,
            timestamp,
            device_id: self.device_id.clone(),
        }
    }

    /// Remove content from watchlist
    pub fn remove_from_watchlist(&self, content_id: &str) -> Vec<WatchlistUpdate> {
        let timestamp = self.hlc.now();
        let mut set = self.or_set.write();

        // Get all tags for this content
        let tags: Vec<String> = set
            .effective_entries()
            .iter()
            .filter(|e| e.content_id == content_id)
            .map(|e| e.unique_tag.clone())
            .collect();

        // Mark all tags as removed
        set.remove(content_id);

        // Return removal updates for each tag
        tags.into_iter()
            .map(|tag| WatchlistUpdate {
                operation: WatchlistOperation::Remove,
                content_id: content_id.to_string(),
                unique_tag: tag,
                timestamp,
                device_id: self.device_id.clone(),
            })
            .collect()
    }

    /// Get all items in watchlist
    pub fn get_watchlist(&self) -> Vec<String> {
        let set = self.or_set.read();
        set.effective_items().into_iter().collect()
    }

    /// Check if content is in watchlist
    pub fn contains(&self, content_id: &str) -> bool {
        let set = self.or_set.read();
        set.contains(content_id)
    }

    /// Apply remote update from another device
    pub fn apply_remote_update(&self, update: WatchlistUpdate) {
        // Update HLC with received timestamp
        self.hlc.update(update.timestamp);

        // Apply delta to OR-Set
        let mut set = self.or_set.write();
        set.apply_delta(ORSetDelta {
            operation: match update.operation {
                WatchlistOperation::Add => ORSetOperation::Add,
                WatchlistOperation::Remove => ORSetOperation::Remove,
            },
            content_id: update.content_id,
            unique_tag: update.unique_tag,
            timestamp: update.timestamp,
            device_id: update.device_id,
        });
    }

    /// Merge with another OR-Set (for state reconciliation)
    pub fn merge(&self, other_set: &ORSet) {
        let mut set = self.or_set.write();
        set.merge(other_set);
    }

    /// Get watchlist size
    pub fn size(&self) -> usize {
        let set = self.or_set.read();
        set.len()
    }
}

/// Watchlist update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistUpdate {
    pub operation: WatchlistOperation,
    pub content_id: String,
    pub unique_tag: String,
    pub timestamp: HLCTimestamp,
    pub device_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchlistOperation {
    Add,
    Remove,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchlist_add_remove() {
        let sync = WatchlistSync::new("user-1".to_string(), "device-a".to_string());

        let update = sync.add_to_watchlist("content-1".to_string());
        assert_eq!(update.operation, WatchlistOperation::Add);
        assert!(sync.contains("content-1"));

        let removals = sync.remove_from_watchlist("content-1");
        assert!(!removals.is_empty());
        assert!(!sync.contains("content-1"));
    }

    #[test]
    fn test_watchlist_remote_update() {
        let sync1 = WatchlistSync::new("user-1".to_string(), "device-a".to_string());
        let sync2 = WatchlistSync::new("user-1".to_string(), "device-b".to_string());

        let update = sync1.add_to_watchlist("content-1".to_string());
        sync2.apply_remote_update(update);

        assert!(sync2.contains("content-1"));
    }

    #[test]
    fn test_watchlist_concurrent_add_remove() {
        let sync1 = WatchlistSync::new("user-1".to_string(), "device-a".to_string());
        let sync2 = WatchlistSync::new("user-1".to_string(), "device-b".to_string());

        // Device A adds
        let add_update = sync1.add_to_watchlist("content-1".to_string());

        // Device B receives add and then removes
        sync2.apply_remote_update(add_update.clone());
        let remove_updates = sync2.remove_from_watchlist("content-1");

        // Device A applies remove
        for update in remove_updates {
            sync1.apply_remote_update(update);
        }

        // Both should not have content-1
        assert!(!sync1.contains("content-1"));
        assert!(!sync2.contains("content-1"));
    }
}
