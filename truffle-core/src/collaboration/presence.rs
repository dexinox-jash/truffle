//! User Presence Tracking
//!
//! Tracks users viewing or editing entities in real-time, including
//! their cursor positions, selection states, and activity status.
//! Uses heartbeats to detect inactive users and automatic cleanup.
//!
//! # Features
//!
//! - Real-time cursor position sharing
//! - User status tracking (Viewing, Editing, Idle)
//! - Automatic heartbeat-based cleanup
//! - Color assignment for user identification
//! - Presence broadcasting to all entity viewers

use crate::collaboration::{CollaborationError, CollaborationResult, DEFAULT_PRESENCE_TIMEOUT_SECS};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, instrument, trace, warn};
use uuid::Uuid;

/// User's presence status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresenceStatus {
    /// User is viewing but not actively editing
    Viewing,
    /// User is actively editing
    Editing,
    /// User is idle (no recent activity)
    Idle,
    /// User has gone offline
    Offline,
}

impl Default for PresenceStatus {
    fn default() -> Self {
        Self::Viewing
    }
}

impl std::fmt::Display for PresenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresenceStatus::Viewing => write!(f, "viewing"),
            PresenceStatus::Editing => write!(f, "editing"),
            PresenceStatus::Idle => write!(f, "idle"),
            PresenceStatus::Offline => write!(f, "offline"),
        }
    }
}

/// Cursor position with field information
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorPosition {
    /// Field name (e.g., "description", "name")
    pub field: String,
    /// Character index in the field
    pub index: u32,
}

/// Selection range in a text field
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionRange {
    /// Start position of selection
    pub start: CursorPosition,
    /// End position of selection
    pub end: CursorPosition,
}

/// User presence information for an entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPresence {
    /// User ID
    pub user_id: Uuid,
    /// Display name
    pub user_name: String,
    /// Color for UI highlighting (hex)
    pub user_color: String,
    /// Entity being viewed/edited
    pub entity_id: Uuid,
    /// Current cursor position (if applicable)
    pub cursor: Option<CursorPosition>,
    /// Current text selection (if applicable)
    pub selection: Option<SelectionRange>,
    /// Current presence status
    pub status: PresenceStatus,
    /// When user joined
    pub joined_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_seen: DateTime<Utc>,
}

impl UserPresence {
    /// Create new presence entry
    pub fn new(user_id: Uuid, entity_id: Uuid, user_name: String) -> Self {
        let user_color = generate_user_color(user_id);

        Self {
            user_id,
            user_name,
            user_color,
            entity_id,
            cursor: None,
            selection: None,
            status: PresenceStatus::Viewing,
            joined_at: Utc::now(),
            last_seen: Utc::now(),
        }
    }

    /// Check if presence has timed out
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        Utc::now().signed_duration_since(self.last_seen) > timeout
    }

    /// Update activity timestamp
    pub fn touch(&mut self) {
        self.last_seen = Utc::now();
    }
}

/// Presence change event
#[derive(Clone, Debug)]
pub enum PresenceEvent {
    /// User joined entity
    Joined(UserPresence),
    /// User left entity
    Left { user_id: Uuid, entity_id: Uuid },
    /// User status changed
    StatusChanged {
        user_id: Uuid,
        entity_id: Uuid,
        old_status: PresenceStatus,
        new_status: PresenceStatus,
    },
    /// Cursor position updated
    CursorUpdated {
        user_id: Uuid,
        entity_id: Uuid,
        cursor: CursorPosition,
        selection: Option<SelectionRange>,
    },
}

/// Presence manager tracks users across all entities
pub struct PresenceManager {
    /// Entity ID -> Map of user IDs to presence
    entity_presence: DashMap<Uuid, DashMap<Uuid, UserPresence>>,
    /// Global heartbeat tracking
    heartbeats: DashMap<Uuid, DateTime<Utc>>,
    /// Timeout for inactive users
    timeout: Duration,
    /// Event subscribers
    subscribers: RwLock<Vec<Box<dyn Fn(&PresenceEvent) + Send + Sync>>>,
}

impl PresenceManager {
    /// Create new presence manager with default timeout
    pub fn new() -> Self {
        Self {
            entity_presence: DashMap::new(),
            heartbeats: DashMap::new(),
            timeout: Duration::seconds(DEFAULT_PRESENCE_TIMEOUT_SECS),
            subscribers: RwLock::new(Vec::new()),
        }
    }

    /// Create with custom timeout
    pub fn with_timeout(timeout_seconds: i64) -> Self {
        Self {
            entity_presence: DashMap::new(),
            heartbeats: DashMap::new(),
            timeout: Duration::seconds(timeout_seconds),
            subscribers: RwLock::new(Vec::new()),
        }
    }

    /// Subscribe to presence events
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&PresenceEvent) + Send + Sync + 'static,
    {
        self.subscribers.write().push(Box::new(callback));
    }

    /// Notify all subscribers of an event
    fn notify(&self, event: &PresenceEvent) {
        for subscriber in self.subscribers.read().iter() {
            subscriber(event);
        }
    }

    /// User joins an entity
    #[instrument(skip(self, user_id, entity_id, user_name))]
    pub fn join(
        &self,
        user_id: Uuid,
        entity_id: Uuid,
        user_name: String,
        status: PresenceStatus,
    ) -> UserPresence {
        let mut presence = UserPresence::new(user_id, entity_id, user_name);
        presence.status = status;

        // Get or create entity presence map
        let entity_users = self
            .entity_presence
            .entry(entity_id)
            .or_insert_with(DashMap::new);

        // Insert user's presence
        entity_users.insert(user_id, presence.clone());

        // Update heartbeat
        self.heartbeats.insert(user_id, Utc::now());

        debug!(user_id = %user_id, entity_id = %entity_id, "User joined entity");

        // Notify subscribers
        self.notify(&PresenceEvent::Joined(presence.clone()));

        presence
    }

    /// User leaves an entity
    #[instrument(skip(self, user_id, entity_id))]
    pub fn leave(&self, user_id: Uuid, entity_id: Uuid) -> CollaborationResult<()> {
        if let Some(entity_users) = self.entity_presence.get(&entity_id) {
            entity_users.remove(&user_id);

            // Clean up empty entity entries
            if entity_users.is_empty() {
                drop(entity_users);
                self.entity_presence.remove(&entity_id);
            }
        }

        // Remove heartbeat
        self.heartbeats.remove(&user_id);

        debug!(user_id = %user_id, entity_id = %entity_id, "User left entity");

        // Notify subscribers
        self.notify(&PresenceEvent::Left { user_id, entity_id });

        Ok(())
    }

    /// User leaves all entities
    #[instrument(skip(self, user_id))]
    pub fn leave_all(&self, user_id: Uuid) {
        let entities_to_leave: Vec<Uuid> = self
            .entity_presence
            .iter()
            .filter(|entry| entry.value().contains_key(&user_id))
            .map(|entry| *entry.key())
            .collect();

        for entity_id in entities_to_leave {
            let _ = self.leave(user_id, entity_id);
        }

        self.heartbeats.remove(&user_id);
    }

    /// Update user status
    #[instrument(skip(self, user_id, entity_id))]
    pub fn update_status(
        &self,
        user_id: Uuid,
        entity_id: Uuid,
        new_status: PresenceStatus,
    ) -> CollaborationResult<()> {
        if let Some(entity_users) = self.entity_presence.get(&entity_id) {
            if let Some(mut presence) = entity_users.get_mut(&user_id) {
                let old_status = presence.status;
                presence.status = new_status;
                presence.touch();

                self.heartbeats.insert(user_id, Utc::now());

                trace!(user_id = %user_id, ?old_status, ?new_status, "Status updated");

                // Notify subscribers
                self.notify(&PresenceEvent::StatusChanged {
                    user_id,
                    entity_id,
                    old_status,
                    new_status,
                });

                Ok(())
            } else {
                Err(CollaborationError::PresenceError(format!(
                    "User {} not found in entity {}",
                    user_id, entity_id
                )))
            }
        } else {
            Err(CollaborationError::PresenceError(format!(
                "Entity {} not found",
                entity_id
            )))
        }
    }

    /// Update cursor position
    #[instrument(skip(self, user_id, entity_id, cursor))]
    pub fn update_cursor(
        &self,
        user_id: Uuid,
        entity_id: Uuid,
        cursor: CursorPosition,
        selection: Option<SelectionRange>,
    ) -> CollaborationResult<()> {
        if let Some(entity_users) = self.entity_presence.get(&entity_id) {
            if let Some(mut presence) = entity_users.get_mut(&user_id) {
                presence.cursor = Some(cursor.clone());
                presence.selection = selection.clone();
                presence.touch();

                self.heartbeats.insert(user_id, Utc::now());

                trace!(user_id = %user_id, field = %cursor.field, index = cursor.index, "Cursor updated");

                // Notify subscribers
                self.notify(&PresenceEvent::CursorUpdated {
                    user_id,
                    entity_id,
                    cursor,
                    selection,
                });

                Ok(())
            } else {
                Err(CollaborationError::PresenceError(format!(
                    "User {} not found in entity {}",
                    user_id, entity_id
                )))
            }
        } else {
            Err(CollaborationError::PresenceError(format!(
                "Entity {} not found",
                entity_id
            )))
        }
    }

    /// Record heartbeat from user
    pub fn heartbeat(&self, user_id: Uuid) {
        self.heartbeats.insert(user_id, Utc::now());

        // Update last_seen for all presences of this user
        for entry in self.entity_presence.iter() {
            if let Some(mut presence) = entry.value().get_mut(&user_id) {
                presence.touch();
            }
        }
    }

    /// Get presence for all users on an entity
    pub fn get_presence(&self, entity_id: Uuid) -> Vec<UserPresence> {
        self.entity_presence
            .get(&entity_id)
            .map(|entity_users| {
                entity_users
                    .iter()
                    .map(|entry| entry.value().clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get single user presence on entity
    pub fn get_user_presence(&self, user_id: Uuid, entity_id: Uuid) -> Option<UserPresence> {
        self.entity_presence
            .get(&entity_id)
            .and_then(|entity_users| entity_users.get(&user_id).map(|p| p.clone()))
    }

    /// Get all entities a user is present on
    pub fn get_user_entities(&self, user_id: Uuid) -> Vec<Uuid> {
        self.entity_presence
            .iter()
            .filter(|entry| entry.value().contains_key(&user_id))
            .map(|entry| *entry.key())
            .collect()
    }

    /// Check if user is present on entity
    pub fn is_present(&self, user_id: Uuid, entity_id: Uuid) -> bool {
        self.entity_presence
            .get(&entity_id)
            .map(|entity_users| entity_users.contains_key(&user_id))
            .unwrap_or(false)
    }

    /// Get count of users on entity
    pub fn get_user_count(&self, entity_id: Uuid) -> usize {
        self.entity_presence
            .get(&entity_id)
            .map(|entity_users| entity_users.len())
            .unwrap_or(0)
    }

    /// Get total active users across all entities
    pub fn get_total_active_users(&self) -> usize {
        let mut unique_users = std::collections::HashSet::new();
        for entry in self.entity_presence.iter() {
            for user_entry in entry.value().iter() {
                unique_users.insert(*user_entry.key());
            }
        }
        unique_users.len()
    }

    /// Clean up inactive users and return removed user IDs
    #[instrument(skip(self))]
    pub fn cleanup_inactive(&self) -> Vec<(Uuid, Uuid)> {
        let now = Utc::now();
        let mut removed = Vec::new();

        for entry in self.entity_presence.iter() {
            let entity_id = *entry.key();
            let entity_users = entry.value();

            let to_remove: Vec<Uuid> = entity_users
                .iter()
                .filter(|user_entry| {
                    let presence = user_entry.value();
                    presence.is_timed_out(self.timeout)
                })
                .map(|user_entry| *user_entry.key())
                .collect();

            for user_id in to_remove {
                entity_users.remove(&user_id);
                self.heartbeats.remove(&user_id);
                removed.push((user_id, entity_id));

                debug!(user_id = %user_id, entity_id = %entity_id, "Removed inactive user");

                // Notify subscribers
                self.notify(&PresenceEvent::Left { user_id, entity_id });
            }
        }

        // Clean up empty entity entries
        let empty_entities: Vec<Uuid> = self
            .entity_presence
            .iter()
            .filter(|entry| entry.value().is_empty())
            .map(|entry| *entry.key())
            .collect();

        for entity_id in empty_entities {
            self.entity_presence.remove(&entity_id);
        }

        removed
    }

    /// Get statistics
    pub fn stats(&self) -> PresenceStats {
        let entity_count = self.entity_presence.len();
        let total_users: usize = self
            .entity_presence
            .iter()
            .map(|entry| entry.value().len())
            .sum();

        PresenceStats {
            entity_count,
            total_presences: total_users,
            unique_users: self.get_total_active_users(),
            timeout_seconds: self.timeout.num_seconds(),
        }
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for presence manager
#[derive(Debug, Clone)]
pub struct PresenceStats {
    /// Number of entities with active users
    pub entity_count: usize,
    /// Total presence entries (users * entities)
    pub total_presences: usize,
    /// Unique active users
    pub unique_users: usize,
    /// Timeout in seconds
    pub timeout_seconds: i64,
}

/// Generate a consistent color for a user based on their ID
fn generate_user_color(user_id: Uuid) -> String {
    // Use the first 3 bytes of UUID to generate HSL color
    let bytes = user_id.as_bytes();
    let hue = ((bytes[0] as u16 * 256 + bytes[1] as u16) % 360) as u16;
    let saturation = 60 + (bytes[2] % 30);
    let lightness = 50;

    hsl_to_hex(hue, saturation, lightness)
}

/// Convert HSL to hex color
fn hsl_to_hex(h: u16, s: u8, l: u8) -> String {
    let s = s as f64 / 100.0;
    let l = l as f64 / 100.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h as f64 / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match h / 60 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let r = ((r + m) * 255.0).round() as u8;
    let g = ((g + m) * 255.0).round() as u8;
    let b = ((b + m) * 255.0).round() as u8;

    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_creation() {
        let user_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();
        let presence = UserPresence::new(user_id, entity_id, "Test User".into());

        assert_eq!(presence.user_id, user_id);
        assert_eq!(presence.entity_id, entity_id);
        assert_eq!(presence.user_name, "Test User");
        assert_eq!(presence.status, PresenceStatus::Viewing);
        assert!(presence.cursor.is_none());
    }

    #[test]
    fn test_join_leave() {
        let manager = PresenceManager::new();
        let user_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();

        // Join
        let presence = manager.join(user_id, entity_id, "Test User".into(), PresenceStatus::Viewing);
        assert_eq!(presence.user_id, user_id);
        assert!(manager.is_present(user_id, entity_id));
        assert_eq!(manager.get_user_count(entity_id), 1);

        // Leave
        manager.leave(user_id, entity_id).expect("Failed to leave");
        assert!(!manager.is_present(user_id, entity_id));
        assert_eq!(manager.get_user_count(entity_id), 0);
    }

    #[test]
    fn test_status_update() {
        let manager = PresenceManager::new();
        let user_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();

        manager.join(user_id, entity_id, "Test User".into(), PresenceStatus::Viewing);

        manager.update_status(user_id, entity_id, PresenceStatus::Editing).expect("Failed to update status");

        let presence = manager.get_user_presence(user_id, entity_id).expect("Presence not found");
        assert_eq!(presence.status, PresenceStatus::Editing);
    }

    #[test]
    fn test_cursor_update() {
        let manager = PresenceManager::new();
        let user_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();

        manager.join(user_id, entity_id, "Test User".into(), PresenceStatus::Editing);

        let cursor = CursorPosition {
            field: "description".into(),
            index: 42,
        };

        manager.update_cursor(user_id, entity_id, cursor.clone(), None).expect("Failed to update cursor");

        let presence = manager.get_user_presence(user_id, entity_id).expect("Presence not found");
        assert_eq!(presence.cursor, Some(cursor));
    }

    #[test]
    fn test_heartbeat() {
        let manager = PresenceManager::new();
        let user_id = Uuid::new_v4();

        manager.heartbeat(user_id);

        let last_heartbeat = manager.heartbeats.get(&user_id).expect("Heartbeat not found");
        let now = Utc::now();
        assert!(now.signed_duration_since(*last_heartbeat).num_seconds() < 1);
    }

    #[test]
    fn test_cleanup_inactive() {
        let manager = PresenceManager::with_timeout(0); // Immediate timeout
        let user_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();

        manager.join(user_id, entity_id, "Test User".into(), PresenceStatus::Viewing);

        // Cleanup should remove the user due to 0-second timeout
        let removed = manager.cleanup_inactive();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], (user_id, entity_id));
    }

    #[test]
    fn test_user_color_generation() {
        let user_id = Uuid::new_v4();
        let color = generate_user_color(user_id);

        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7); // #RRGGBB

        // Same user should get same color
        let color2 = generate_user_color(user_id);
        assert_eq!(color, color2);
    }

    #[test]
    fn test_hsl_to_hex() {
        assert_eq!(hsl_to_hex(0, 100, 50), "#ff0000"); // Red
        assert_eq!(hsl_to_hex(120, 100, 50), "#00ff00"); // Green
        assert_eq!(hsl_to_hex(240, 100, 50), "#0000ff"); // Blue
    }
}
