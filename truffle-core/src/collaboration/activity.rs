//! Activity Feed
//!
//! Real-time activity feed for tracking all collaborative actions
//! across entities. Supports event subscription for live updates.
//!
//! # Features
//!
//! - Chronological event feed
//! - Event type filtering
//! - Subscriber notifications
//! - Automatic size limiting with LRU eviction
//! - Entity-specific and global feeds

use crate::collaboration::{CollaborationResult, MAX_ACTIVITY_FEED_SIZE};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::{debug, instrument, trace};
use uuid::Uuid;

/// Types of activity events
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    /// Entity was created
    EntityCreated,
    /// Entity was updated
    EntityUpdated,
    /// Entity was deleted
    EntityDeleted,
    /// Entity was merged with another
    EntityMerged,
    /// Entity was split
    EntitySplit,

    /// Comment was added
    CommentAdded,
    /// Comment was edited
    CommentEdited,
    /// Comment was deleted
    CommentDeleted,
    /// Thread was resolved
    CommentResolved,
    /// Thread was unresolved
    CommentUnresolved,
    /// Reaction was added
    ReactionAdded,

    /// User joined session
    UserJoined,
    /// User left session
    UserLeft,
    /// User started editing
    UserStartedEditing,
    /// User stopped editing
    UserStoppedEditing,

    /// Export completed
    ExportCompleted,
    /// Import completed
    ImportCompleted,
    /// Backup created
    BackupCreated,
    /// Restore completed
    RestoreCompleted,

    /// Relationship created
    RelationshipCreated,
    /// Relationship updated
    RelationshipUpdated,
    /// Relationship deleted
    RelationshipDeleted,

    /// Schema applied
    SchemaApplied,
    /// Validation completed
    ValidationCompleted,

    /// Custom activity type
    Custom(String),
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ActivityType::EntityCreated => "entity_created",
            ActivityType::EntityUpdated => "entity_updated",
            ActivityType::EntityDeleted => "entity_deleted",
            ActivityType::EntityMerged => "entity_merged",
            ActivityType::EntitySplit => "entity_split",
            ActivityType::CommentAdded => "comment_added",
            ActivityType::CommentEdited => "comment_edited",
            ActivityType::CommentDeleted => "comment_deleted",
            ActivityType::CommentResolved => "comment_resolved",
            ActivityType::CommentUnresolved => "comment_unresolved",
            ActivityType::ReactionAdded => "reaction_added",
            ActivityType::UserJoined => "user_joined",
            ActivityType::UserLeft => "user_left",
            ActivityType::UserStartedEditing => "user_started_editing",
            ActivityType::UserStoppedEditing => "user_stopped_editing",
            ActivityType::ExportCompleted => "export_completed",
            ActivityType::ImportCompleted => "import_completed",
            ActivityType::BackupCreated => "backup_created",
            ActivityType::RestoreCompleted => "restore_completed",
            ActivityType::RelationshipCreated => "relationship_created",
            ActivityType::RelationshipUpdated => "relationship_updated",
            ActivityType::RelationshipDeleted => "relationship_deleted",
            ActivityType::SchemaApplied => "schema_applied",
            ActivityType::ValidationCompleted => "validation_completed",
            ActivityType::Custom(s) => s.as_str(),
        };
        write!(f, "{}", s)
    }
}

/// An activity event in the feed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityEvent {
    /// Unique event ID
    pub event_id: Uuid,
    /// Type of activity
    pub event_type: ActivityType,
    /// User who performed the action
    pub actor_id: Uuid,
    /// Display name of actor
    pub actor_name: String,
    /// Affected entity ID (if applicable)
    pub entity_id: Option<Uuid>,
    /// Entity name/title (if applicable)
    pub entity_name: Option<String>,
    /// Human-readable description
    pub description: String,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
}

impl ActivityEvent {
    /// Create a new activity event
    pub fn new(
        event_type: ActivityType,
        actor_id: Uuid,
        actor_name: String,
        description: String,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type,
            actor_id,
            actor_name,
            entity_id: None,
            entity_name: None,
            description,
            metadata: None,
            timestamp: Utc::now(),
        }
    }

    /// Set the entity for this event
    pub fn with_entity(mut self, entity_id: Uuid, entity_name: impl Into<String>) -> Self {
        self.entity_id = Some(entity_id);
        self.entity_name = Some(entity_name.into());
        self
    }

    /// Set metadata for this event
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Create a formatted description for display
    pub fn format_description(&self) -> String {
        match (&self.entity_name, &self.entity_id) {
            (Some(name), _) => format!("{} on {}", self.description, name),
            (None, Some(_)) => self.description.clone(),
            (None, None) => self.description.clone(),
        }
    }
}

/// Callback type for activity subscribers
pub type ActivityCallback = Box<dyn Fn(&ActivityEvent) + Send + Sync>;

/// Activity feed for tracking collaborative actions
pub struct ActivityFeed {
    /// Event storage (circular buffer using VecDeque)
    events: RwLock<VecDeque<ActivityEvent>>,
    /// Maximum number of events to store
    max_size: usize,
    /// Event subscribers
    subscribers: RwLock<Vec<ActivityCallback>>,
    /// Total events published (including those evicted)
    total_published: RwLock<u64>,
}

impl ActivityFeed {
    /// Create a new activity feed with specified maximum size
    pub fn new(max_size: usize) -> Self {
        let size = max_size.min(MAX_ACTIVITY_FEED_SIZE);
        Self {
            events: RwLock::new(VecDeque::with_capacity(size)),
            max_size: size,
            subscribers: RwLock::new(Vec::new()),
            total_published: RwLock::new(0),
        }
    }

    /// Create with default maximum size
    pub fn default_size() -> Self {
        Self::new(MAX_ACTIVITY_FEED_SIZE)
    }

    /// Subscribe to activity events
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&ActivityEvent) + Send + Sync + 'static,
    {
        self.subscribers.write().push(Box::new(callback));
    }

    /// Unsubscribe all handlers (useful for testing)
    pub fn clear_subscribers(&self) {
        self.subscribers.write().clear();
    }

    /// Publish an activity event
    #[instrument(skip(self, event))]
    pub fn publish(&self, event: ActivityEvent) {
        // Add to feed with size limit
        {
            let mut events = self.events.write();
            if events.len() >= self.max_size {
                events.pop_front();
            }
            events.push_back(event.clone());
        }

        // Increment total count
        *self.total_published.write() += 1;

        trace!(event_id = %event.event_id, event_type = %event.event_type, "Published activity event");

        // Notify subscribers
        for subscriber in self.subscribers.read().iter() {
            subscriber(&event);
        }
    }

    /// Publish a simple event with minimal parameters
    #[instrument(skip(self))]
    pub fn publish_simple(
        &self,
        event_type: ActivityType,
        actor_id: Uuid,
        actor_name: impl Into<String>,
        description: impl Into<String>,
    ) {
        let event = ActivityEvent::new(
            event_type,
            actor_id,
            actor_name.into(),
            description.into(),
        );
        self.publish(event);
    }

    /// Get recent events (most recent first)
    pub fn get_recent(&self, count: usize) -> Vec<ActivityEvent> {
        let events = self.events.read();
        events.iter().rev().take(count).cloned().collect()
    }

    /// Get events since a specific timestamp
    pub fn get_since(&self, since: DateTime<Utc>) -> Vec<ActivityEvent> {
        let events = self.events.read();
        events
            .iter()
            .filter(|e| e.timestamp > since)
            .cloned()
            .collect()
    }

    /// Get events by type
    pub fn get_by_type(&self, event_type: ActivityType) -> Vec<ActivityEvent> {
        let events = self.events.read();
        events
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get events for a specific entity
    pub fn get_for_entity(&self, entity_id: Uuid) -> Vec<ActivityEvent> {
        let events = self.events.read();
        events
            .iter()
            .filter(|e| e.entity_id == Some(entity_id))
            .cloned()
            .collect()
    }

    /// Get events by actor
    pub fn get_by_actor(&self, actor_id: Uuid) -> Vec<ActivityEvent> {
        let events = self.events.read();
        events
            .iter()
            .filter(|e| e.actor_id == actor_id)
            .cloned()
            .collect()
    }

    /// Get all events
    pub fn get_all(&self) -> Vec<ActivityEvent> {
        self.events.read().iter().cloned().collect()
    }

    /// Get event by ID
    pub fn get_event(&self, event_id: Uuid) -> Option<ActivityEvent> {
        self.events.read().iter().find(|e| e.event_id == event_id).cloned()
    }

    /// Get current event count
    pub fn len(&self) -> usize {
        self.events.read().len()
    }

    /// Check if feed is empty
    pub fn is_empty(&self) -> bool {
        self.events.read().is_empty()
    }

    /// Get total published events (including evicted)
    pub fn total_published(&self) -> u64 {
        *self.total_published.read()
    }

    /// Clear all events
    pub fn clear(&self) {
        self.events.write().clear();
        *self.total_published.write() = 0;
    }

    /// Get maximum size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Get feed statistics
    pub fn stats(&self) -> ActivityFeedStats {
        let events = self.events.read();
        let total_published = *self.total_published.read();

        ActivityFeedStats {
            current_size: events.len(),
            max_size: self.max_size,
            total_published,
            evicted_count: total_published.saturating_sub(events.len() as u64),
            subscriber_count: self.subscribers.read().len(),
        }
    }
}

impl Default for ActivityFeed {
    fn default() -> Self {
        Self::default_size()
    }
}

/// Statistics for activity feed
#[derive(Debug, Clone)]
pub struct ActivityFeedStats {
    /// Current number of events in feed
    pub current_size: usize,
    /// Maximum capacity
    pub max_size: usize,
    /// Total events published (including evicted)
    pub total_published: u64,
    /// Number of events evicted
    pub evicted_count: u64,
    /// Number of active subscribers
    pub subscriber_count: usize,
}

/// Activity feed manager for handling multiple feeds
pub struct ActivityManager {
    /// Global feed for all events
    global_feed: ActivityFeed,
    /// Entity-specific feeds
    entity_feeds: RwLock<std::collections::HashMap<Uuid, ActivityFeed>>,
    /// Default max size for entity feeds
    default_feed_size: usize,
}

impl ActivityManager {
    /// Create new activity manager
    pub fn new(global_max_size: usize) -> Self {
        Self {
            global_feed: ActivityFeed::new(global_max_size),
            entity_feeds: RwLock::new(std::collections::HashMap::new()),
            default_feed_size: 1000,
        }
    }

    /// Publish to both global and entity-specific feeds
    #[instrument(skip(self, event))]
    pub fn publish(&self, event: ActivityEvent) {
        // Publish to global feed
        self.global_feed.publish(event.clone());

        // Publish to entity-specific feed if applicable
        if let Some(entity_id) = event.entity_id {
            let mut feeds = self.entity_feeds.write();
            let feed = feeds
                .entry(entity_id)
                .or_insert_with(|| ActivityFeed::new(self.default_feed_size));
            feed.publish(event);
        }
    }

    /// Get global feed
    pub fn global(&self) -> &ActivityFeed {
        &self.global_feed
    }

    /// Get or create entity feed
    pub fn for_entity(&self, entity_id: Uuid) -> ActivityFeed {
        let mut feeds = self.entity_feeds.write();
        feeds
            .entry(entity_id)
            .or_insert_with(|| ActivityFeed::new(self.default_feed_size))
            .clone()
    }

    /// Get entity feed if exists
    pub fn get_entity_feed(&self, entity_id: Uuid) -> Option<ActivityFeed> {
        self.entity_feeds.read().get(&entity_id).cloned()
    }

    /// Remove entity feed
    pub fn remove_entity_feed(&self, entity_id: Uuid) -> Option<ActivityFeed> {
        self.entity_feeds.write().remove(&entity_id)
    }

    /// Get all entity IDs with feeds
    pub fn get_entity_ids(&self) -> Vec<Uuid> {
        self.entity_feeds.read().keys().cloned().collect()
    }
}

impl Default for ActivityManager {
    fn default() -> Self {
        Self::new(MAX_ACTIVITY_FEED_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_event_creation() {
        let actor_id = Uuid::new_v4();
        let event = ActivityEvent::new(
            ActivityType::EntityCreated,
            actor_id,
            "Test User".into(),
            "Created new entity".into(),
        );

        assert_eq!(event.event_type, ActivityType::EntityCreated);
        assert_eq!(event.actor_id, actor_id);
        assert_eq!(event.actor_name, "Test User");
        assert_eq!(event.description, "Created new entity");
        assert!(event.entity_id.is_none());
    }

    #[test]
    fn test_activity_event_with_entity() {
        let entity_id = Uuid::new_v4();
        let event = ActivityEvent::new(
            ActivityType::EntityUpdated,
            Uuid::new_v4(),
            "Test".into(),
            "Updated".into(),
        )
        .with_entity(entity_id, "My Entity".into());

        assert_eq!(event.entity_id, Some(entity_id));
        assert_eq!(event.entity_name, Some("My Entity".into()));
    }

    #[test]
    fn test_activity_feed_publish() {
        let feed = ActivityFeed::new(10);
        let actor_id = Uuid::new_v4();

        feed.publish_simple(
            ActivityType::EntityCreated,
            actor_id,
            "Test User",
            "Created entity",
        );

        assert_eq!(feed.len(), 1);
        assert_eq!(feed.total_published(), 1);
    }

    #[test]
    fn test_activity_feed_size_limit() {
        let feed = ActivityFeed::new(5);

        // Publish 10 events
        for i in 0..10 {
            feed.publish_simple(
                ActivityType::Custom(format!("event_{}", i)),
                Uuid::new_v4(),
                "User",
                format!("Event {}", i),
            );
        }

        // Should only keep last 5
        assert_eq!(feed.len(), 5);
        assert_eq!(feed.total_published(), 10);

        // Most recent events should be 5-9
        let recent = feed.get_recent(5);
        for (i, event) in recent.iter().rev().enumerate() {
            assert!(matches!(&event.event_type, ActivityType::Custom(s) if s == &format!("event_{}", i + 5)));
        }
    }

    #[test]
    fn test_activity_feed_getters() {
        let feed = ActivityFeed::new(100);
        let actor_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();

        // Create event with entity
        let event = ActivityEvent::new(
            ActivityType::EntityUpdated,
            actor_id,
            "Test".into(),
            "Updated".into(),
        )
        .with_entity(entity_id, "Entity".into());
        feed.publish(event);

        // Create event without entity
        feed.publish_simple(ActivityType::UserJoined, actor_id, "Test", "Joined");

        // Get by entity
        let entity_events = feed.get_for_entity(entity_id);
        assert_eq!(entity_events.len(), 1);

        // Get by actor
        let actor_events = feed.get_by_actor(actor_id);
        assert_eq!(actor_events.len(), 2);

        // Get by type
        let type_events = feed.get_by_type(ActivityType::UserJoined);
        assert_eq!(type_events.len(), 1);
    }

    #[test]
    fn test_activity_feed_subscribers() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let feed = ActivityFeed::new(10);
        let count = Arc::new(AtomicUsize::new(0));

        let count_clone = count.clone();
        feed.subscribe(move |_event| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        feed.publish_simple(ActivityType::EntityCreated, Uuid::new_v4(), "Test", "Created");
        feed.publish_simple(ActivityType::EntityUpdated, Uuid::new_v4(), "Test", "Updated");

        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_activity_type_display() {
        assert_eq!(ActivityType::EntityCreated.to_string(), "entity_created");
        assert_eq!(ActivityType::CommentAdded.to_string(), "comment_added");
        assert_eq!(
            ActivityType::Custom("special".into()).to_string(),
            "special"
        );
    }

    #[test]
    fn test_activity_manager() {
        let manager = ActivityManager::new(100);
        let entity_id = Uuid::new_v4();
        let actor_id = Uuid::new_v4();

        // Publish event
        let event = ActivityEvent::new(
            ActivityType::EntityCreated,
            actor_id,
            "Test".into(),
            "Created".into(),
        )
        .with_entity(entity_id, "Entity".into());
        manager.publish(event);

        // Check global feed
        assert_eq!(manager.global().len(), 1);

        // Check entity feed
        let entity_feed = manager.get_entity_feed(entity_id).unwrap();
        assert_eq!(entity_feed.len(), 1);
    }

    #[test]
    fn test_activity_feed_stats() {
        let feed = ActivityFeed::new(5);

        // Publish more events than capacity
        for i in 0..10 {
            feed.publish_simple(
                ActivityType::Custom(format!("{}", i)),
                Uuid::new_v4(),
                "User",
                "Description",
            );
        }

        let stats = feed.stats();
        assert_eq!(stats.current_size, 5);
        assert_eq!(stats.max_size, 5);
        assert_eq!(stats.total_published, 10);
        assert_eq!(stats.evicted_count, 5);
    }
}
