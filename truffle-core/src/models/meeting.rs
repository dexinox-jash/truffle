//! Meeting Model
//!
//! Represents a meeting with transcript and metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A meeting with transcript
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Meeting {
    /// Unique identifier
    pub id: Uuid,
    
    /// Meeting title
    pub title: String,
    
    /// When meeting started
    pub started_at: Option<DateTime<Utc>>,
    
    /// When meeting ended
    pub ended_at: Option<DateTime<Utc>>,
    
    /// Timezone
    pub timezone: String,
    
    /// Participant entity IDs (JSON array)
    pub participant_ids: Option<String>,
    
    /// Full transcript text
    pub transcript: Option<String>,
    
    /// Transcript chunks (JSON array)
    pub transcript_chunks: Option<String>,
    
    /// Meeting type
    pub meeting_type: Option<String>,
    
    /// Platform (zoom, teams, meet, etc.)
    pub platform: Option<String>,
    
    /// Path to recording file
    pub recording_path: Option<String>,
    
    /// Processing status
    pub processing_status: String,
    
    /// Processing error message
    pub processing_error: Option<String>,
    
    /// Created at
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl Meeting {
    /// Create a new meeting
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            started_at: Some(Utc::now()),
            ended_at: None,
            timezone: "UTC".to_string(),
            participant_ids: None,
            transcript: None,
            transcript_chunks: None,
            meeting_type: None,
            platform: None,
            recording_path: None,
            processing_status: "pending".to_string(),
            processing_error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Set transcript
    pub fn with_transcript(mut self, transcript: impl Into<String>) -> Self {
        self.transcript = Some(transcript.into());
        self
    }
    
    /// Set participants
    pub fn with_participants(mut self, participant_ids: Vec<Uuid>) -> Self {
        self.participant_ids = Some(
            serde_json::to_string(&participant_ids).unwrap_or_default()
        );
        self
    }
    
    /// Start processing
    pub fn start_processing(&mut self) {
        self.processing_status = "extracting".to_string();
        self.updated_at = Utc::now();
    }
    
    /// Complete processing
    pub fn complete_processing(&mut self) {
        self.processing_status = "completed".to_string();
        self.updated_at = Utc::now();
    }
    
    /// Fail processing
    pub fn fail_processing(&mut self, error: impl Into<String>) {
        self.processing_status = "failed".to_string();
        self.processing_error = Some(error.into());
        self.updated_at = Utc::now();
    }
}

/// Meeting types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeetingType {
    Standup,
    Planning,
    Review,
    Retrospective,
    OneOnOne,
    Interview,
    Presentation,
    Brainstorm,
    Other,
}

impl MeetingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Standup => "standup",
            Self::Planning => "planning",
            Self::Review => "review",
            Self::Retrospective => "retrospective",
            Self::OneOnOne => "1on1",
            Self::Interview => "interview",
            Self::Presentation => "presentation",
            Self::Brainstorm => "brainstorm",
            Self::Other => "other",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Standup => "Standup",
            Self::Planning => "Planning",
            Self::Review => "Review",
            Self::Retrospective => "Retrospective",
            Self::OneOnOne => "1:1",
            Self::Interview => "Interview",
            Self::Presentation => "Presentation",
            Self::Brainstorm => "Brainstorm",
            Self::Other => "Other",
        }
    }
}

impl Default for Meeting {
    fn default() -> Self {
        Self::new("Untitled Meeting")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_meeting_creation() {
        let meeting = Meeting::new("Q1 Planning");
        
        assert_eq!(meeting.title, "Q1 Planning");
        assert_eq!(meeting.processing_status, "pending");
    }
    
    #[test]
    fn test_meeting_with_transcript() {
        let meeting = Meeting::new("Test Meeting")
            .with_transcript("Alice: Hello everyone.");
        
        assert_eq!(meeting.transcript, Some("Alice: Hello everyone.".to_string()));
    }
    
    #[test]
    fn test_meeting_processing() {
        let mut meeting = Meeting::new("Test Meeting");
        
        meeting.start_processing();
        assert_eq!(meeting.processing_status, "extracting");
        
        meeting.complete_processing();
        assert_eq!(meeting.processing_status, "completed");
    }
}
