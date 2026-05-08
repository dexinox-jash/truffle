//! Meeting models

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Meeting model
#[derive(Debug, Clone, Serialize)]
pub struct Meeting {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub agenda: Option<String>,
    pub scheduled_start_at: DateTime<Utc>,
    pub scheduled_end_at: DateTime<Utc>,
    pub actual_start_at: Option<DateTime<Utc>>,
    pub actual_end_at: Option<DateTime<Utc>>,
    pub timezone: String,
    pub status: String,
    pub meeting_type: String,
    pub visibility: String,
    pub recording_enabled: bool,
    pub transcription_enabled: bool,
    pub join_url: Option<String>,
    pub created_by: Uuid,
    pub team_id: Option<Uuid>,
    pub participants: Vec<Participant>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Meeting participant
#[derive(Debug, Clone, Serialize)]
pub struct Participant {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub status: String,
    pub is_external: bool,
}

/// Meeting list response
#[derive(Debug, Clone, Serialize)]
pub struct MeetingList {
    pub data: Vec<Meeting>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Create meeting request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMeetingRequest {
    pub title: String,
    pub description: Option<String>,
    pub agenda: Option<String>,
    pub scheduled_start_at: DateTime<Utc>,
    pub scheduled_end_at: DateTime<Utc>,
    pub timezone: Option<String>,
    pub meeting_type: Option<String>,
    pub visibility: Option<String>,
    pub recording_enabled: Option<bool>,
    pub transcription_enabled: Option<bool>,
    pub team_id: Option<Uuid>,
    pub participants: Vec<CreateParticipant>,
}

/// Create participant request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateParticipant {
    pub email: String,
    pub name: Option<String>,
    pub role: Option<String>,
}

/// Update meeting request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMeetingRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub agenda: Option<String>,
    pub scheduled_start_at: Option<DateTime<Utc>>,
    pub scheduled_end_at: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub status: Option<String>,
}
