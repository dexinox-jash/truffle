//! Comments and Annotations
//!
//! Threaded comments on entities with support for inline annotations,
//! reactions, and resolution tracking. Comments can be attached to
//! specific text ranges within entity fields.
//!
//! # Features
//!
//! - Threaded discussions on entities
//! - Inline text annotations with range selection
//! - Emoji reactions on comments
//! - Thread resolution
//! - Participant notifications

use crate::collaboration::{CollaborationError, CollaborationResult};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, instrument, trace, warn};
use uuid::Uuid;

/// Text range for inline comments
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRange {
    /// Start position (inclusive)
    pub start: u32,
    /// End position (exclusive)
    pub end: u32,
}

impl TextRange {
    /// Create new text range
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Check if position is within range
    pub fn contains(&self, pos: u32) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Get length of range
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Check if range is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Emoji reaction on a comment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reaction {
    /// Emoji character(s)
    pub emoji: String,
    /// User IDs who reacted
    pub user_ids: Vec<Uuid>,
}

impl Reaction {
    /// Create new reaction
    pub fn new(emoji: String) -> Self {
        Self {
            emoji,
            user_ids: Vec::new(),
        }
    }

    /// Add user reaction
    pub fn add_user(&mut self, user_id: Uuid) {
        if !self.user_ids.contains(&user_id) {
            self.user_ids.push(user_id);
        }
    }

    /// Remove user reaction
    pub fn remove_user(&mut self, user_id: Uuid) {
        self.user_ids.retain(|&id| id != user_id);
    }

    /// Get reaction count
    pub fn count(&self) -> usize {
        self.user_ids.len()
    }
}

/// Individual comment in a thread
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    /// Unique comment ID
    pub comment_id: Uuid,
    /// Author user ID
    pub author_id: Uuid,
    /// Author display name
    pub author_name: String,
    /// Comment content (markdown supported)
    pub content: String,
    /// When the comment was created
    pub created_at: DateTime<Utc>,
    /// When the comment was last edited
    pub edited_at: Option<DateTime<Utc>>,
    /// Reactions on this comment
    pub reactions: Vec<Reaction>,
}

impl Comment {
    /// Create a new comment
    pub fn new(author_id: Uuid, author_name: String, content: String) -> Self {
        Self {
            comment_id: Uuid::new_v4(),
            author_id,
            author_name,
            content,
            created_at: Utc::now(),
            edited_at: None,
            reactions: Vec::new(),
        }
    }

    /// Edit the comment
    pub fn edit(&mut self, new_content: String) {
        self.content = new_content;
        self.edited_at = Some(Utc::now());
    }

    /// Add a reaction
    pub fn add_reaction(&mut self, emoji: String, user_id: Uuid) {
        if let Some(reaction) = self.reactions.iter_mut().find(|r| r.emoji == emoji) {
            reaction.add_user(user_id);
        } else {
            let mut reaction = Reaction::new(emoji);
            reaction.add_user(user_id);
            self.reactions.push(reaction);
        }
    }

    /// Remove a reaction
    pub fn remove_reaction(&mut self, emoji: &str, user_id: Uuid) {
        if let Some(reaction) = self.reactions.iter_mut().find(|r| r.emoji == emoji) {
            reaction.remove_user(user_id);
        }
        // Clean up empty reactions
        self.reactions.retain(|r| !r.user_ids.is_empty());
    }
}

/// Thread of comments on an entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommentThread {
    /// Unique thread ID
    pub thread_id: Uuid,
    /// Entity being commented on
    pub entity_id: Uuid,
    /// Specific field (None = general entity comment)
    pub field: Option<String>,
    /// Text selection range (for inline comments)
    pub range: Option<TextRange>,
    /// Comments in this thread
    pub comments: Vec<Comment>,
    /// Whether thread is resolved
    pub resolved: bool,
    /// Who resolved the thread
    pub resolved_by: Option<Uuid>,
    /// When the thread was resolved
    pub resolved_at: Option<DateTime<Utc>>,
    /// When the thread was created
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub updated_at: DateTime<Utc>,
}

impl CommentThread {
    /// Create a new comment thread
    pub fn new(
        entity_id: Uuid,
        author_id: Uuid,
        author_name: String,
        content: String,
        field: Option<String>,
        range: Option<TextRange>,
    ) -> Self {
        let now = Utc::now();
        let initial_comment = Comment::new(author_id, author_name, content);

        Self {
            thread_id: Uuid::new_v4(),
            entity_id,
            field,
            range,
            comments: vec![initial_comment],
            resolved: false,
            resolved_by: None,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a comment to the thread
    pub fn add_comment(&mut self, author_id: Uuid, author_name: String, content: String) -> Uuid {
        let comment = Comment::new(author_id, author_name, content);
        let comment_id = comment.comment_id;
        self.comments.push(comment);
        self.updated_at = Utc::now();
        comment_id
    }

    /// Resolve the thread
    pub fn resolve(&mut self, user_id: Uuid) {
        self.resolved = true;
        self.resolved_by = Some(user_id);
        self.resolved_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Unresolve the thread
    pub fn unresolve(&mut self) {
        self.resolved = false;
        self.resolved_by = None;
        self.resolved_at = None;
        self.updated_at = Utc::now();
    }

    /// Get all unique participant IDs
    pub fn get_participants(&self) -> Vec<Uuid> {
        let mut participants: Vec<Uuid> = self
            .comments
            .iter()
            .map(|c| c.author_id)
            .collect();
        participants.dedup();
        participants
    }

    /// Get comment count
    pub fn comment_count(&self) -> usize {
        self.comments.len()
    }

    /// Get a specific comment
    pub fn get_comment(&self, comment_id: Uuid) -> Option<&Comment> {
        self.comments.iter().find(|c| c.comment_id == comment_id)
    }

    /// Get mutable reference to a comment
    pub fn get_comment_mut(&mut self, comment_id: Uuid) -> Option<&mut Comment> {
        self.comments.iter_mut().find(|c| c.comment_id == comment_id)
    }
}

/// Comment change event
#[derive(Clone, Debug)]
pub enum CommentEvent {
    /// New thread created
    ThreadCreated {
        thread_id: Uuid,
        entity_id: Uuid,
        author_id: Uuid,
    },
    /// Comment added to thread
    CommentAdded {
        thread_id: Uuid,
        comment_id: Uuid,
        author_id: Uuid,
    },
    /// Thread resolved
    ThreadResolved {
        thread_id: Uuid,
        resolved_by: Uuid,
    },
    /// Thread unresolved
    ThreadUnresolved {
        thread_id: Uuid,
    },
    /// Reaction added
    ReactionAdded {
        thread_id: Uuid,
        comment_id: Uuid,
        emoji: String,
        user_id: Uuid,
    },
    /// Reaction removed
    ReactionRemoved {
        thread_id: Uuid,
        comment_id: Uuid,
        emoji: String,
        user_id: Uuid,
    },
}

/// Comment manager for handling all comments
pub struct CommentManager {
    /// Thread ID -> CommentThread
    threads: DashMap<Uuid, RwLock<CommentThread>>,
    /// Entity ID -> Vec<Thread IDs>
    entity_threads: DashMap<Uuid, RwLock<Vec<Uuid>>>,
    /// Event subscribers
    subscribers: RwLock<Vec<Box<dyn Fn(&CommentEvent) + Send + Sync>>>,
}

impl CommentManager {
    /// Create new comment manager
    pub fn new() -> Self {
        Self {
            threads: DashMap::new(),
            entity_threads: DashMap::new(),
            subscribers: RwLock::new(Vec::new()),
        }
    }

    /// Subscribe to comment events
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&CommentEvent) + Send + Sync + 'static,
    {
        self.subscribers.write().push(Box::new(callback));
    }

    /// Notify all subscribers
    fn notify(&self, event: &CommentEvent) {
        for subscriber in self.subscribers.read().iter() {
            subscriber(event);
        }
    }

    /// Create a new comment thread
    #[instrument(skip(self, author_id, content))]
    pub fn create_thread(
        &self,
        entity_id: Uuid,
        author_id: Uuid,
        author_name: String,
        content: String,
        field: Option<String>,
        range: Option<TextRange>,
    ) -> Uuid {
        let thread = CommentThread::new(
            entity_id,
            author_id,
            author_name,
            content,
            field,
            range,
        );
        let thread_id = thread.thread_id;

        // Add to threads map
        self.threads.insert(thread_id, RwLock::new(thread));

        // Add to entity threads
        let entity_thread_list = self
            .entity_threads
            .entry(entity_id)
            .or_insert_with(|| RwLock::new(Vec::new()));
        entity_thread_list.write().push(thread_id);

        debug!(thread_id = %thread_id, entity_id = %entity_id, "Created comment thread");

        // Notify subscribers
        self.notify(&CommentEvent::ThreadCreated {
            thread_id,
            entity_id,
            author_id,
        });

        thread_id
    }

    /// Add a comment to a thread
    #[instrument(skip(self, content))]
    pub fn add_comment(
        &self,
        thread_id: Uuid,
        author_id: Uuid,
        author_name: String,
        content: String,
    ) -> CollaborationResult<Uuid> {
        match self.threads.get(&thread_id) {
            Some(thread) => {
                let mut thread = thread.write();
                let comment_id = thread.add_comment(author_id, author_name, content);

                trace!(thread_id = %thread_id, comment_id = %comment_id, "Added comment");

                // Notify subscribers
                self.notify(&CommentEvent::CommentAdded {
                    thread_id,
                    comment_id,
                    author_id,
                });

                Ok(comment_id)
            }
            None => Err(CollaborationError::CommentError(format!(
                "Thread {} not found",
                thread_id
            ))),
        }
    }

    /// Resolve a thread
    #[instrument(skip(self))]
    pub fn resolve_thread(&self, thread_id: Uuid, user_id: Uuid) -> CollaborationResult<()> {
        match self.threads.get(&thread_id) {
            Some(thread) => {
                let mut thread = thread.write();
                thread.resolve(user_id);

                debug!(thread_id = %thread_id, user_id = %user_id, "Resolved thread");

                // Notify subscribers
                self.notify(&CommentEvent::ThreadResolved {
                    thread_id,
                    resolved_by: user_id,
                });

                Ok(())
            }
            None => Err(CollaborationError::CommentError(format!(
                "Thread {} not found",
                thread_id
            ))),
        }
    }

    /// Unresolve a thread
    pub fn unresolve_thread(&self, thread_id: Uuid) -> CollaborationResult<()> {
        match self.threads.get(&thread_id) {
            Some(thread) => {
                let mut thread = thread.write();
                thread.unresolve();

                trace!(thread_id = %thread_id, "Unresolved thread");

                // Notify subscribers
                self.notify(&CommentEvent::ThreadUnresolved { thread_id });

                Ok(())
            }
            None => Err(CollaborationError::CommentError(format!(
                "Thread {} not found",
                thread_id
            ))),
        }
    }

    /// Add reaction to a comment
    pub fn add_reaction(
        &self,
        thread_id: Uuid,
        comment_id: Uuid,
        emoji: String,
        user_id: Uuid,
    ) -> CollaborationResult<()> {
        match self.threads.get(&thread_id) {
            Some(thread) => {
                let mut thread = thread.write();
                if let Some(comment) = thread.get_comment_mut(comment_id) {
                    comment.add_reaction(emoji.clone(), user_id);

                    trace!(thread_id = %thread_id, comment_id = %comment_id, emoji = %emoji, "Added reaction");

                    // Notify subscribers
                    self.notify(&CommentEvent::ReactionAdded {
                        thread_id,
                        comment_id,
                        emoji,
                        user_id,
                    });

                    Ok(())
                } else {
                    Err(CollaborationError::CommentError(format!(
                        "Comment {} not found in thread {}",
                        comment_id, thread_id
                    )))
                }
            }
            None => Err(CollaborationError::CommentError(format!(
                "Thread {} not found",
                thread_id
            ))),
        }
    }

    /// Remove reaction from a comment
    pub fn remove_reaction(
        &self,
        thread_id: Uuid,
        comment_id: Uuid,
        emoji: &str,
        user_id: Uuid,
    ) -> CollaborationResult<()> {
        match self.threads.get(&thread_id) {
            Some(thread) => {
                let mut thread = thread.write();
                if let Some(comment) = thread.get_comment_mut(comment_id) {
                    comment.remove_reaction(emoji, user_id);

                    trace!(thread_id = %thread_id, comment_id = %comment_id, emoji = %emoji, "Removed reaction");

                    // Notify subscribers
                    self.notify(&CommentEvent::ReactionRemoved {
                        thread_id,
                        comment_id,
                        emoji: emoji.to_string(),
                        user_id,
                    });

                    Ok(())
                } else {
                    Err(CollaborationError::CommentError(format!(
                        "Comment {} not found in thread {}",
                        comment_id, thread_id
                    )))
                }
            }
            None => Err(CollaborationError::CommentError(format!(
                "Thread {} not found",
                thread_id
            ))),
        }
    }

    /// Get a thread by ID
    pub fn get_thread(&self, thread_id: Uuid) -> Option<CommentThread> {
        self.threads.get(&thread_id).map(|t| t.read().clone())
    }

    /// Get all threads for an entity
    pub fn get_threads(&self, entity_id: Uuid) -> Vec<CommentThread> {
        self.entity_threads
            .get(&entity_id)
            .map(|thread_ids| {
                let ids = thread_ids.read();
                ids.iter()
                    .filter_map(|id| self.get_thread(*id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get unresolved threads for an entity
    pub fn get_unresolved_threads(&self, entity_id: Uuid) -> Vec<CommentThread> {
        self.get_threads(entity_id)
            .into_iter()
            .filter(|t| !t.resolved)
            .collect()
    }

    /// Get thread count for entity
    pub fn get_thread_count(&self, entity_id: Uuid) -> usize {
        self.entity_threads
            .get(&entity_id)
            .map(|ids| ids.read().len())
            .unwrap_or(0)
    }

    /// Get total comment count for entity
    pub fn get_total_comment_count(&self, entity_id: Uuid) -> usize {
        self.get_threads(entity_id)
            .iter()
            .map(|t| t.comment_count())
            .sum()
    }

    /// Delete a thread
    pub fn delete_thread(&self, thread_id: Uuid) -> CollaborationResult<()> {
        match self.threads.remove(&thread_id) {
            Some((_, thread)) => {
                let thread = thread.read();
                let entity_id = thread.entity_id;

                // Remove from entity threads
                if let Some(entity_threads) = self.entity_threads.get(&entity_id) {
                    let mut ids = entity_threads.write();
                    ids.retain(|&id| id != thread_id);
                }

                trace!(thread_id = %thread_id, "Deleted thread");
                Ok(())
            }
            None => Err(CollaborationError::CommentError(format!(
                "Thread {} not found",
                thread_id
            ))),
        }
    }

    /// Edit a comment
    pub fn edit_comment(
        &self,
        thread_id: Uuid,
        comment_id: Uuid,
        new_content: String,
    ) -> CollaborationResult<()> {
        match self.threads.get(&thread_id) {
            Some(thread) => {
                let mut thread = thread.write();
                if let Some(comment) = thread.get_comment_mut(comment_id) {
                    comment.edit(new_content);
                    thread.updated_at = Utc::now();

                    trace!(thread_id = %thread_id, comment_id = %comment_id, "Edited comment");
                    Ok(())
                } else {
                    Err(CollaborationError::CommentError(format!(
                        "Comment {} not found in thread {}",
                        comment_id, thread_id
                    )))
                }
            }
            None => Err(CollaborationError::CommentError(format!(
                "Thread {} not found",
                thread_id
            ))),
        }
    }

    /// Get all thread IDs
    pub fn get_all_thread_ids(&self) -> Vec<Uuid> {
        self.threads.iter().map(|entry| *entry.key()).collect()
    }

    /// Get statistics
    pub fn stats(&self) -> CommentStats {
        let total_threads = self.threads.len();
        let total_comments: usize = self
            .threads
            .iter()
            .map(|entry| entry.value().read().comment_count())
            .sum();

        CommentStats {
            total_threads,
            total_comments,
            entities_with_threads: self.entity_threads.len(),
        }
    }
}

impl Default for CommentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for comment manager
#[derive(Debug, Clone)]
pub struct CommentStats {
    /// Total number of threads
    pub total_threads: usize,
    /// Total number of comments across all threads
    pub total_comments: usize,
    /// Number of entities with at least one thread
    pub entities_with_threads: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_creation() {
        let author_id = Uuid::new_v4();
        let comment = Comment::new(author_id, "Test User".into(), "Hello, world!".into());

        assert_eq!(comment.author_id, author_id);
        assert_eq!(comment.author_name, "Test User");
        assert_eq!(comment.content, "Hello, world!");
        assert!(comment.reactions.is_empty());
        assert!(comment.edited_at.is_none());
    }

    #[test]
    fn test_comment_edit() {
        let mut comment = Comment::new(Uuid::new_v4(), "Test".into(), "Original".into());

        comment.edit("Updated".into());

        assert_eq!(comment.content, "Updated");
        assert!(comment.edited_at.is_some());
    }

    #[test]
    fn test_reaction() {
        let mut reaction = Reaction::new("👍".into());
        let user_id = Uuid::new_v4();

        reaction.add_user(user_id);
        assert_eq!(reaction.count(), 1);

        reaction.add_user(user_id); // Duplicate should not count
        assert_eq!(reaction.count(), 1);

        reaction.remove_user(user_id);
        assert_eq!(reaction.count(), 0);
    }

    #[test]
    fn test_thread_creation() {
        let entity_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let thread = CommentThread::new(
            entity_id,
            author_id,
            "Test User".into(),
            "First comment".into(),
            Some("description".into()),
            Some(TextRange::new(0, 10)),
        );

        assert_eq!(thread.entity_id, entity_id);
        assert_eq!(thread.comments.len(), 1);
        assert!(!thread.resolved);
        assert_eq!(thread.field, Some("description".into()));
    }

    #[test]
    fn test_thread_resolve() {
        let mut thread = CommentThread::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".into(),
            "Comment".into(),
            None,
            None,
        );

        let resolver_id = Uuid::new_v4();
        thread.resolve(resolver_id);

        assert!(thread.resolved);
        assert_eq!(thread.resolved_by, Some(resolver_id));
        assert!(thread.resolved_at.is_some());
    }

    #[test]
    fn test_text_range() {
        let range = TextRange::new(10, 20);

        assert_eq!(range.start, 10);
        assert_eq!(range.end, 20);
        assert_eq!(range.len(), 10);
        assert!(range.contains(15));
        assert!(!range.contains(25));
        assert!(!range.contains(5));
    }

    #[test]
    fn test_comment_manager() {
        let manager = CommentManager::new();
        let entity_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        // Create thread
        let thread_id = manager.create_thread(
            entity_id,
            author_id,
            "Test User".into(),
            "Initial comment".into(),
            None,
            None,
        );

        // Check thread exists
        assert!(manager.get_thread(thread_id).is_some());
        assert_eq!(manager.get_thread_count(entity_id), 1);

        // Add comment
        let comment_id = manager
            .add_comment(thread_id, author_id, "Test User".into(), "Reply".into())
            .expect("Failed to add comment");

        let thread = manager.get_thread(thread_id).expect("Thread not found");
        assert_eq!(thread.comment_count(), 2);

        // Resolve thread
        manager.resolve_thread(thread_id, author_id).expect("Failed to resolve thread");
        let thread = manager.get_thread(thread_id).expect("Thread not found");
        assert!(thread.resolved);

        // Check stats
        let stats = manager.stats();
        assert_eq!(stats.total_threads, 1);
        assert_eq!(stats.total_comments, 2);
    }

    #[test]
    fn test_reactions() {
        let manager = CommentManager::new();
        let entity_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let reactor_id = Uuid::new_v4();

        let thread_id = manager.create_thread(
            entity_id,
            author_id,
            "Author".into(),
            "Comment".into(),
            None,
            None,
        );

        let thread = manager.get_thread(thread_id).expect("Thread not found");
        let comment_id = thread.comments[0].comment_id;

        // Add reaction
        manager.add_reaction(thread_id, comment_id, "👍".into(), reactor_id).expect("Failed to add reaction");

        let thread = manager.get_thread(thread_id).expect("Thread not found");
        let comment = &thread.comments[0];
        assert_eq!(comment.reactions.len(), 1);
        assert_eq!(comment.reactions[0].emoji, "👍");
        assert_eq!(comment.reactions[0].count(), 1);

        // Remove reaction
        manager.remove_reaction(thread_id, comment_id, "👍", reactor_id).expect("Failed to remove reaction");

        let thread = manager.get_thread(thread_id).expect("Thread not found");
        let comment = &thread.comments[0];
        assert!(comment.reactions.is_empty());
    }
}
