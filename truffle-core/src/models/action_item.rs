//! Action Item Model
//!
//! Tracks commitments and tasks extracted from meetings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An action item (commitment) from a meeting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionItem {
    /// Unique identifier
    pub id: Uuid,
    
    /// Description
    pub description: String,
    
    /// Assignee entity ID
    pub assignee_id: Option<Uuid>,
    
    /// Creator entity ID
    pub creator_id: Option<Uuid>,
    
    /// Source meeting ID
    pub source_meeting_id: Option<Uuid>,
    
    /// Source decision ID (if from a decision)
    pub source_decision_id: Option<Uuid>,
    
    /// Status
    pub status: String, // open, in_progress, blocked, completed, cancelled, overdue
    
    /// Deadline
    pub deadline: Option<DateTime<Utc>>,
    
    /// When completed
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Priority
    pub priority: String, // low, medium, high, urgent
    
    /// Who verified completion
    pub completion_verified_by_id: Option<Uuid>,
    
    /// Verification notes
    pub verification_notes: Option<String>,
    
    /// Reminder sent
    pub reminder_sent: bool,
    
    /// When reminder sent
    pub reminder_sent_at: Option<DateTime<Utc>>,
    
    /// Created at
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl ActionItem {
    /// Create a new action item
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            description: description.into(),
            assignee_id: None,
            creator_id: None,
            source_meeting_id: None,
            source_decision_id: None,
            status: "open".to_string(),
            deadline: None,
            completed_at: None,
            priority: "medium".to_string(),
            completion_verified_by_id: None,
            verification_notes: None,
            reminder_sent: false,
            reminder_sent_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Set assignee
    pub fn with_assignee(mut self, assignee_id: Uuid) -> Self {
        self.assignee_id = Some(assignee_id);
        self
    }
    
    /// Set deadline
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority.as_str().to_string();
        self
    }
    
    /// Set source meeting
    pub fn with_meeting(mut self, meeting_id: Uuid) -> Self {
        self.source_meeting_id = Some(meeting_id);
        self
    }
    
    /// Mark as in progress
    pub fn start(&mut self) {
        self.status = "in_progress".to_string();
        self.updated_at = Utc::now();
    }
    
    /// Mark as blocked
    pub fn block(&mut self) {
        self.status = "blocked".to_string();
        self.updated_at = Utc::now();
    }
    
    /// Mark as completed
    pub fn complete(&mut self) {
        self.status = "completed".to_string();
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// Mark as verified
    pub fn verify(&mut self, verifier_id: Uuid, notes: Option<String>) {
        self.completion_verified_by_id = Some(verifier_id);
        self.verification_notes = notes;
        self.updated_at = Utc::now();
    }
    
    /// Check if overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(deadline) = self.deadline {
            Utc::now() > deadline && self.status != "completed" && self.status != "cancelled"
        } else {
            false
        }
    }
    
    /// Update status to overdue if past deadline
    pub fn update_overdue_status(&mut self) {
        if self.is_overdue() && self.status == "open" {
            self.status = "overdue".to_string();
            self.updated_at = Utc::now();
        }
    }
}

/// Priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Low => "🔵",
            Self::Medium => "🟡",
            Self::High => "🟠",
            Self::Urgent => "🔴",
        }
    }
}

/// Action item status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionItemStatus {
    Open,
    InProgress,
    Blocked,
    Completed,
    Cancelled,
    Overdue,
}

impl ActionItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Overdue => "overdue",
        }
    }
    
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    
    #[test]
    fn test_action_item_creation() {
        let item = ActionItem::new("Review the PR");
        
        assert_eq!(item.description, "Review the PR");
        assert_eq!(item.status, "open");
        assert_eq!(item.priority, "medium");
    }
    
    #[test]
    fn test_action_item_builder() {
        let assignee_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let deadline = Utc::now() + Duration::days(7);
        
        let item = ActionItem::new("Review the PR")
            .with_assignee(assignee_id)
            .with_meeting(meeting_id)
            .with_deadline(deadline)
            .with_priority(Priority::High);
        
        assert_eq!(item.assignee_id, Some(assignee_id));
        assert_eq!(item.source_meeting_id, Some(meeting_id));
        assert_eq!(item.priority, "high");
    }
    
    #[test]
    fn test_action_item_completion() {
        let mut item = ActionItem::new("Review the PR");
        
        item.start();
        assert_eq!(item.status, "in_progress");
        
        item.complete();
        assert_eq!(item.status, "completed");
        assert!(item.completed_at.is_some());
    }
    
    #[test]
    fn test_overdue_detection() {
        let mut item = ActionItem::new("Review the PR")
            .with_deadline(Utc::now() - Duration::days(1)); // Yesterday
        
        assert!(item.is_overdue());
        
        item.update_overdue_status();
        assert_eq!(item.status, "overdue");
    }
    
    #[test]
    fn test_not_overdue_if_completed() {
        let mut item = ActionItem::new("Review the PR")
            .with_deadline(Utc::now() - Duration::days(1));
        
        item.complete();
        assert!(!item.is_overdue());
    }
}
