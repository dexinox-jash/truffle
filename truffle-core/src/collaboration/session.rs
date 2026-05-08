//! Collaboration Session Management
//!
//! Manages active collaboration sessions that combine live editing,
//! presence tracking, and commenting for a specific entity.
//!
//! # Features
//!
//! - Session lifecycle management (create, join, leave, end)
//! - Participant permissions
//! - Session persistence
//! - Automatic cleanup of inactive sessions

use crate::collaboration::{
    CollaborationError, CollaborationResult, LiveEditSession, PresenceManager, UserPresence,
};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, trace, warn};
use uuid::Uuid;

/// Permissions for a session participant
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    /// Can edit the entity
    pub can_edit: bool,
    /// Can add comments
    pub can_comment: bool,
    /// Can invite others
    pub can_invite: bool,
    /// Can moderate (resolve threads, kick users)
    pub can_moderate: bool,
}

impl Permissions {
    /// Full permissions (host)
    pub fn full() -> Self {
        Self {
            can_edit: true,
            can_comment: true,
            can_invite: true,
            can_moderate: true,
        }
    }

    /// Read-only permissions
    pub fn read_only() -> Self {
        Self {
            can_edit: false,
            can_comment: true,
            can_invite: false,
            can_moderate: false,
        }
    }

    /// Comment-only permissions
    pub fn comment_only() -> Self {
        Self {
            can_edit: false,
            can_comment: true,
            can_invite: false,
            can_moderate: false,
        }
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::read_only()
    }
}

/// A participant in a collaboration session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Participant {
    /// User ID
    pub user_id: Uuid,
    /// When user joined
    pub joined_at: DateTime<Utc>,
    /// Last activity
    pub last_activity: DateTime<Utc>,
    /// User permissions
    pub permissions: Permissions,
}

impl Participant {
    /// Create new participant
    pub fn new(user_id: Uuid, permissions: Permissions) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            joined_at: now,
            last_activity: now,
            permissions,
        }
    }

    /// Update activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Utc::now();
    }
}

/// Active collaboration session
pub struct CollaborationSession {
    /// Unique session ID
    pub session_id: Uuid,
    /// Entity being collaborated on
    pub entity_id: Uuid,
    /// Host user ID
    pub host_id: Uuid,
    /// Session participants
    participants: RwLock<HashMap<Uuid, Participant>>,
    /// When session started
    pub started_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: RwLock<DateTime<Utc>>,
    /// Live editing session
    pub live_edit: Arc<LiveEditSession>,
    /// Whether session is active
    pub is_active: RwLock<bool>,
    /// Session metadata
    pub metadata: RwLock<SessionMetadata>,
}

/// Session metadata
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session title/description
    pub title: Option<String>,
    /// Custom data
    pub custom: Option<serde_json::Value>,
}

impl CollaborationSession {
    /// Create new collaboration session
    #[instrument(skip(entity_id, host_id))]
    pub fn new(entity_id: Uuid, host_id: Uuid, live_edit: Arc<LiveEditSession>) -> Self {
        let now = Utc::now();
        let session = Self {
            session_id: Uuid::new_v4(),
            entity_id,
            host_id,
            participants: RwLock::new(HashMap::new()),
            started_at: now,
            last_activity: RwLock::new(now),
            live_edit,
            is_active: RwLock::new(true),
            metadata: RwLock::new(SessionMetadata::default()),
        };

        // Add host as first participant
        session.participants.write().insert(
            host_id,
            Participant::new(host_id, Permissions::full()),
        );

        info!(session_id = %session.session_id, entity_id = %entity_id, host_id = %host_id, "Created collaboration session");

        session
    }

    /// Add participant to session
    #[instrument(skip(self, user_id))]
    pub fn add_participant(&self, user_id: Uuid, permissions: Permissions) -> CollaborationResult<()> {
        if !*self.is_active.read() {
            return Err(CollaborationError::SessionNotFound(
                "Session is not active".into(),
            ));
        }

        if self.participants.read().contains_key(&user_id) {
            return Err(CollaborationError::InvalidOperation(
                "User already in session".into(),
            ));
        }

        self.participants
            .write()
            .insert(user_id, Participant::new(user_id, permissions));

        *self.last_activity.write() = Utc::now();

        debug!(session_id = %self.session_id, user_id = %user_id, "Added participant");

        Ok(())
    }

    /// Remove participant from session
    #[instrument(skip(self, user_id))]
    pub fn remove_participant(&self, user_id: Uuid) -> CollaborationResult<()> {
        if self.participants.write().remove(&user_id).is_none() {
            return Err(CollaborationError::NotAuthorized(
                "User not in session".into(),
            ));
        }

        *self.last_activity.write() = Utc::now();

        debug!(session_id = %self.session_id, user_id = %user_id, "Removed participant");

        // Update user state in live edit session
        self.live_edit.remove_user(user_id);

        Ok(())
    }

    /// Get participant info
    pub fn get_participant(&self, user_id: Uuid) -> Option<Participant> {
        self.participants.read().get(&user_id).cloned()
    }

    /// Get all participants
    pub fn get_participants(&self) -> Vec<Participant> {
        self.participants.read().values().cloned().collect()
    }

    /// Get participant count
    pub fn participant_count(&self) -> usize {
        self.participants.read().len()
    }

    /// Check if user is in session
    pub fn has_participant(&self, user_id: Uuid) -> bool {
        self.participants.read().contains_key(&user_id)
    }

    /// Update participant permissions
    pub fn update_permissions(
        &self,
        user_id: Uuid,
        permissions: Permissions,
    ) -> CollaborationResult<()> {
        match self.participants.write().get_mut(&user_id) {
            Some(participant) => {
                participant.permissions = permissions;
                participant.touch();
                *self.last_activity.write() = Utc::now();
                trace!(session_id = %self.session_id, user_id = %user_id, "Updated permissions");
                Ok(())
            }
            None => Err(CollaborationError::NotAuthorized(
                "User not in session".into(),
            )),
        }
    }

    /// Update participant activity
    pub fn touch(&self, user_id: Uuid) {
        if let Some(participant) = self.participants.write().get_mut(&user_id) {
            participant.touch();
            *self.last_activity.write() = Utc::now();
        }
    }

    /// Check if user can perform action
    pub fn can(&self, user_id: Uuid, action: fn(&Permissions) -> bool) -> bool {
        self.participants
            .read()
            .get(&user_id)
            .map(|p| action(&p.permissions))
            .unwrap_or(false)
    }

    /// Check if user can edit
    pub fn can_edit(&self, user_id: Uuid) -> bool {
        self.can(user_id, |p| p.can_edit)
    }

    /// Check if user can comment
    pub fn can_comment(&self, user_id: Uuid) -> bool {
        self.can(user_id, |p| p.can_comment)
    }

    /// Check if user can invite
    pub fn can_invite(&self, user_id: Uuid) -> bool {
        self.can(user_id, |p| p.can_invite)
    }

    /// Check if user can moderate
    pub fn can_moderate(&self, user_id: Uuid) -> bool {
        self.can(user_id, |p| p.can_moderate)
    }

    /// End the session
    #[instrument(skip(self))]
    pub fn end(&self) {
        *self.is_active.write() = false;
        info!(session_id = %self.session_id, "Ended collaboration session");
    }

    /// Get session duration
    pub fn duration(&self) -> Duration {
        Utc::now().signed_duration_since(self.started_at)
    }

    /// Get session statistics
    pub fn stats(&self) -> SessionStats {
        let participants = self.participants.read();
        let active_count = participants
            .values()
            .filter(|p| Utc::now().signed_duration_since(p.last_activity).num_minutes() < 5)
            .count();

        SessionStats {
            session_id: self.session_id,
            entity_id: self.entity_id,
            participant_count: participants.len(),
            active_participant_count: active_count,
            started_at: self.started_at,
            duration_seconds: self.duration().num_seconds(),
            is_active: *self.is_active.read(),
            last_activity: *self.last_activity.read(),
        }
    }

    /// Get session snapshot for serialization
    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            session_id: self.session_id,
            entity_id: self.entity_id,
            host_id: self.host_id,
            participants: self.get_participants(),
            started_at: self.started_at,
            is_active: *self.is_active.read(),
            metadata: self.metadata.read().clone(),
        }
    }
}

/// Serializable session snapshot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: Uuid,
    pub entity_id: Uuid,
    pub host_id: Uuid,
    pub participants: Vec<Participant>,
    pub started_at: DateTime<Utc>,
    pub is_active: bool,
    pub metadata: SessionMetadata,
}

/// Session statistics
#[derive(Clone, Debug)]
pub struct SessionStats {
    pub session_id: Uuid,
    pub entity_id: Uuid,
    pub participant_count: usize,
    pub active_participant_count: usize,
    pub started_at: DateTime<Utc>,
    pub duration_seconds: i64,
    pub is_active: bool,
    pub last_activity: DateTime<Utc>,
}

/// Session event for notifications
#[derive(Clone, Debug)]
pub enum SessionEvent {
    SessionCreated { session_id: Uuid, entity_id: Uuid, host_id: Uuid },
    UserJoined { session_id: Uuid, user_id: Uuid },
    UserLeft { session_id: Uuid, user_id: Uuid },
    SessionEnded { session_id: Uuid },
    PermissionsChanged { session_id: Uuid, user_id: Uuid },
}

/// Manages all collaboration sessions
pub struct SessionManager {
    /// Active sessions by session ID
    sessions: DashMap<Uuid, Arc<CollaborationSession>>,
    /// Entity ID -> Session ID mapping
    entity_sessions: DashMap<Uuid, Uuid>,
    /// Presence manager for all sessions
    presence: PresenceManager,
    /// Inactive session timeout
    timeout: Duration,
    /// Event subscribers
    subscribers: RwLock<Vec<Box<dyn Fn(&SessionEvent) + Send + Sync>>>,
}

impl SessionManager {
    /// Create new session manager
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            entity_sessions: DashMap::new(),
            presence: PresenceManager::new(),
            timeout: Duration::hours(1),
            subscribers: RwLock::new(Vec::new()),
        }
    }

    /// Create with custom timeout
    pub fn with_timeout(timeout_minutes: i64) -> Self {
        Self {
            sessions: DashMap::new(),
            entity_sessions: DashMap::new(),
            presence: PresenceManager::new(),
            timeout: Duration::minutes(timeout_minutes),
            subscribers: RwLock::new(Vec::new()),
        }
    }

    /// Subscribe to session events
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&SessionEvent) + Send + Sync + 'static,
    {
        self.subscribers.write().push(Box::new(callback));
    }

    /// Notify subscribers
    fn notify(&self, event: &SessionEvent) {
        for subscriber in self.subscribers.read().iter() {
            subscriber(event);
        }
    }

    /// Create a new collaboration session
    #[instrument(skip(self, entity_id, host_id))]
    pub fn create_session(
        &self,
        entity_id: Uuid,
        host_id: Uuid,
        live_edit: Arc<LiveEditSession>,
    ) -> Uuid {
        // Check if session already exists for this entity
        if let Some(existing) = self.entity_sessions.get(&entity_id) {
            return *existing;
        }

        let session = Arc::new(CollaborationSession::new(entity_id, host_id, live_edit));
        let session_id = session.session_id;

        self.sessions.insert(session_id, session);
        self.entity_sessions.insert(entity_id, session_id);

        info!(session_id = %session_id, entity_id = %entity_id, host_id = %host_id, "Created session");

        // Notify subscribers
        self.notify(&SessionEvent::SessionCreated {
            session_id,
            entity_id,
            host_id,
        });

        session_id
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: Uuid) -> Option<Arc<CollaborationSession>> {
        self.sessions.get(&session_id).map(|s| s.clone())
    }

    /// Get session for entity
    pub fn get_session_for_entity(&self, entity_id: Uuid) -> Option<Arc<CollaborationSession>> {
        self.entity_sessions
            .get(&entity_id)
            .and_then(|id| self.get_session(*id))
    }

    /// Join an existing session
    #[instrument(skip(self, session_id, user_id))]
    pub fn join_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        permissions: Option<Permissions>,
    ) -> CollaborationResult<()> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| CollaborationError::SessionNotFound(format!("Session {} not found", session_id)))?;

        let perms = permissions.unwrap_or_else(Permissions::read_only);
        session.add_participant(user_id, perms)?;

        debug!(session_id = %session_id, user_id = %user_id, "User joined session");

        // Notify subscribers
        self.notify(&SessionEvent::UserJoined { session_id, user_id });

        Ok(())
    }

    /// Leave a session
    #[instrument(skip(self, session_id, user_id))]
    pub fn leave_session(&self, session_id: Uuid, user_id: Uuid) -> CollaborationResult<()> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| CollaborationError::SessionNotFound(format!("Session {} not found", session_id)))?;

        session.remove_participant(user_id)?;

        debug!(session_id = %session_id, user_id = %user_id, "User left session");

        // Notify subscribers
        self.notify(&SessionEvent::UserLeft { session_id, user_id });

        // End session if no participants left (except host)
        if session.participant_count() == 0 {
            self.end_session(session_id);
        }

        Ok(())
    }

    /// End a session
    #[instrument(skip(self, session_id))]
    pub fn end_session(&self, session_id: Uuid) -> Option<Arc<CollaborationSession>> {
        match self.sessions.remove(&session_id) {
            Some((_, session)) => {
                let entity_id = session.entity_id;

                session.end();
                self.entity_sessions.remove(&entity_id);

                info!(session_id = %session_id, "Ended session");

                // Notify subscribers
                self.notify(&SessionEvent::SessionEnded { session_id });

                Some(session)
            }
            None => None,
        }
    }

    /// Get presence manager
    pub fn presence(&self) -> &PresenceManager {
        &self.presence
    }

    /// Check if user is in session
    pub fn is_in_session(&self, session_id: Uuid, user_id: Uuid) -> bool {
        self.sessions
            .get(&session_id)
            .map(|s| s.has_participant(user_id))
            .unwrap_or(false)
    }

    /// Get all active session IDs
    pub fn get_active_sessions(&self) -> Vec<Uuid> {
        self.sessions.iter().map(|entry| *entry.key()).collect()
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get entity IDs with active sessions
    pub fn get_active_entities(&self) -> Vec<Uuid> {
        self.entity_sessions.iter().map(|entry| *entry.key()).collect()
    }

    /// Clean up inactive sessions
    #[instrument(skip(self))]
    pub fn cleanup_inactive(&self) -> Vec<Uuid> {
        let now = Utc::now();
        let mut ended = Vec::new();

        self.sessions.retain(|session_id, session| {
            let last_activity = *session.last_activity.read();
            let is_active = *session.is_active.read();
            let should_retain = is_active && now.signed_duration_since(last_activity) < self.timeout;

            if !should_retain {
                ended.push(*session_id);
                self.entity_sessions.remove(&session.entity_id);
                info!(session_id = %session_id, "Cleaned up inactive session");
            }

            should_retain
        });

        ended
    }

    /// Get statistics for all sessions
    pub fn stats(&self) -> Vec<SessionStats> {
        self.sessions
            .iter()
            .map(|entry| entry.value().stats())
            .collect()
    }

    /// Get manager statistics
    pub fn manager_stats(&self) -> SessionManagerStats {
        SessionManagerStats {
            active_sessions: self.sessions.len(),
            active_entities: self.entity_sessions.len(),
            total_participants: self
                .sessions
                .iter()
                .map(|s| s.participant_count())
                .sum(),
            timeout_minutes: self.timeout.num_minutes(),
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Session manager statistics
#[derive(Debug, Clone)]
pub struct SessionManagerStats {
    pub active_sessions: usize,
    pub active_entities: usize,
    pub total_participants: usize,
    pub timeout_minutes: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collaboration::LiveEditSession;

    #[test]
    fn test_permissions() {
        let full = Permissions::full();
        assert!(full.can_edit);
        assert!(full.can_comment);
        assert!(full.can_invite);
        assert!(full.can_moderate);

        let read_only = Permissions::read_only();
        assert!(!read_only.can_edit);
        assert!(read_only.can_comment);
        assert!(!read_only.can_invite);
        assert!(!read_only.can_moderate);
    }

    #[test]
    fn test_participant() {
        let user_id = Uuid::new_v4();
        let participant = Participant::new(user_id, Permissions::full());

        assert_eq!(participant.user_id, user_id);
        assert!(participant.permissions.can_edit);
    }

    #[test]
    fn test_collaboration_session() {
        let entity_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let live_edit = Arc::new(LiveEditSession::new(entity_id));

        let session = CollaborationSession::new(entity_id, host_id, live_edit);

        assert_eq!(session.entity_id, entity_id);
        assert_eq!(session.host_id, host_id);
        assert_eq!(session.participant_count(), 1);
        assert!(session.has_participant(host_id));
        assert!(session.can_edit(host_id));
    }

    #[test]
    fn test_session_add_remove_participants() {
        let entity_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let live_edit = Arc::new(LiveEditSession::new(entity_id));

        let session = CollaborationSession::new(entity_id, host_id, live_edit);

        let user_id = Uuid::new_v4();
        session
            .add_participant(user_id, Permissions::read_only())
            .expect("Failed to add participant");

        assert_eq!(session.participant_count(), 2);
        assert!(session.has_participant(user_id));
        assert!(!session.can_edit(user_id));

        session.remove_participant(user_id).expect("Failed to remove participant");
        assert_eq!(session.participant_count(), 1);
        assert!(!session.has_participant(user_id));
    }

    #[test]
    fn test_session_permissions() {
        let entity_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let live_edit = Arc::new(LiveEditSession::new(entity_id));

        let session = CollaborationSession::new(entity_id, host_id, live_edit);

        let user_id = Uuid::new_v4();
        session
            .add_participant(user_id, Permissions::read_only())
            .expect("Failed to add participant");

        assert!(!session.can_edit(user_id));
        assert!(session.can_comment(user_id));

        session.update_permissions(user_id, Permissions::full()).expect("Failed to update permissions");

        assert!(session.can_edit(user_id));
        assert!(session.can_invite(user_id));
        assert!(session.can_moderate(user_id));
    }

    #[test]
    fn test_session_manager() {
        let manager = SessionManager::new();
        let entity_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let live_edit = Arc::new(LiveEditSession::new(entity_id));

        // Create session
        let session_id = manager.create_session(entity_id, host_id, live_edit);
        assert_eq!(manager.session_count(), 1);

        // Get session
        let session = manager.get_session(session_id).expect("Session not found");
        assert_eq!(session.entity_id, entity_id);

        // Join session
        let user_id = Uuid::new_v4();
        manager.join_session(session_id, user_id, None).expect("Failed to join session");

        let session = manager.get_session(session_id).expect("Session not found");
        assert_eq!(session.participant_count(), 2);

        // Leave session
        manager.leave_session(session_id, user_id).expect("Failed to leave session");

        // End session
        manager.end_session(session_id);
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_session_duplicate_entity() {
        let manager = SessionManager::new();
        let entity_id = Uuid::new_v4();
        let live_edit = Arc::new(LiveEditSession::new(entity_id));

        let session_id1 = manager.create_session(entity_id, Uuid::new_v4(), live_edit.clone());
        let session_id2 = manager.create_session(entity_id, Uuid::new_v4(), live_edit);

        // Should return existing session for entity
        assert_eq!(session_id1, session_id2);
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_session_stats() {
        let entity_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let live_edit = Arc::new(LiveEditSession::new(entity_id));

        let session = CollaborationSession::new(entity_id, host_id, live_edit);

        let stats = session.stats();
        assert_eq!(stats.session_id, session.session_id);
        assert_eq!(stats.entity_id, entity_id);
        assert_eq!(stats.participant_count, 1);
        assert!(stats.is_active);
        assert!(stats.duration_seconds >= 0);
    }

    #[test]
    fn test_session_manager_stats() {
        let manager = SessionManager::new();
        let entity_id = Uuid::new_v4();
        let live_edit = Arc::new(LiveEditSession::new(entity_id));

        manager.create_session(entity_id, Uuid::new_v4(), live_edit);

        let stats = manager.manager_stats();
        assert_eq!(stats.active_sessions, 1);
        assert_eq!(stats.active_entities, 1);
    }
}
