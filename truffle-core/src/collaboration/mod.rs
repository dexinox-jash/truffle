//! Real-Time Collaboration Module
//!
//! Enables multi-user collaboration on knowledge graph entities
//! using CRDT-based conflict resolution.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                 Collaboration System                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Live Edit    │  Presence    │  Comments   │  Activity      │
//! │  ─────────    │  ────────    │  ────────   │  ───────       │
//! │  CRDT (Yrs)   │  Heartbeats  │  Threads    │  Event Feed    │
//! │  Operations   │  Cursors     │  Reactions  │  Subscribers   │
//! │  Conflict     │  Status      │  Resolutions│  Broadcasting  │
//! │  Resolution   │  Broadcasting│             │                │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                    ┌─────────┴─────────┐
//!                    ▼                   ▼
//!              WebSocket Transport   Session Manager
//! ```
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use truffle_core::collaboration::{
//!     LiveEditManager, PresenceManager, CommentManager, ActivityFeed, SessionManager
//! };
//!
//! // Create managers
//! let mut live_edit = LiveEditManager::new();
//! let mut presence = PresenceManager::new();
//! let mut comments = CommentManager::new();
//! let mut activity = ActivityFeed::new(1000);
//! ```

pub mod activity;
pub mod comments;
pub mod live_edit;
pub mod presence;
pub mod session;

// Re-export main types for convenience
pub use activity::{ActivityEvent, ActivityFeed, ActivityType};
pub use comments::{Comment, CommentManager, CommentThread, TextRange};
pub use live_edit::{LiveEditManager, LiveEditSession, Operation, OperationType};
pub use presence::{CursorPosition, PresenceManager, SelectionRange, UserPresence};
pub use session::{CollaborationSession, Participant, Permissions, SessionManager};

use thiserror::Error;

/// Errors that can occur in the collaboration system
#[derive(Error, Debug, Clone)]
pub enum CollaborationError {
    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// User not authorized
    #[error("User not authorized: {0}")]
    NotAuthorized(String),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// CRDT error
    #[error("CRDT error: {0}")]
    CrdtError(String),

    /// Comment error
    #[error("Comment error: {0}")]
    CommentError(String),

    /// Presence error
    #[error("Presence error: {0}")]
    PresenceError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for collaboration operations
pub type CollaborationResult<T> = std::result::Result<T, CollaborationError>;

/// Version of the collaboration protocol
pub const PROTOCOL_VERSION: u32 = 1;

/// Default timeout for inactive users (in seconds)
pub const DEFAULT_PRESENCE_TIMEOUT_SECS: i64 = 60;

/// Maximum number of operations to keep in history
pub const MAX_OPERATION_HISTORY: usize = 10000;

/// Maximum length of activity feed
pub const MAX_ACTIVITY_FEED_SIZE: usize = 10000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }

    #[test]
    fn test_default_timeout() {
        assert_eq!(DEFAULT_PRESENCE_TIMEOUT_SECS, 60);
    }
}
