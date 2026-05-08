//! WebSocket message types

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Messages sent from client to server
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Authentication
    Authenticate {
        token: String,
    },
    
    /// Join a meeting
    Join {
        meeting_id: Uuid,
        display_name: String,
    },
    
    /// Chat message
    Chat {
        content: String,
        reply_to: Option<Uuid>,
    },
    
    /// Audio data (base64 encoded)
    Audio {
        data: String, // base64 encoded audio chunk
        timestamp_ms: u64,
        sequence: u64,
    },
    
    /// Screen share control
    ScreenShare {
        action: ScreenShareAction,
    },
    
    /// WebRTC signaling
    Signal {
        target_user_id: Uuid,
        signal: SignalData,
    },
    
    /// Raise/lower hand
    HandRaise {
        raised: bool,
    },
    
    /// Reaction (emoji)
    Reaction {
        emoji: String,
    },
    
    /// Ping for keepalive
    Ping,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Authentication result
    Authenticated {
        user_id: Uuid,
        success: bool,
        error: Option<String>,
    },
    
    /// Joined meeting confirmation
    Joined {
        meeting_id: Uuid,
        participant_id: Uuid,
        participants: Vec<ParticipantInfo>,
    },
    
    /// Participant joined
    ParticipantJoined {
        participant_id: Uuid,
        participant_name: String,
        joined_at: DateTime<Utc>,
    },
    
    /// Participant left
    ParticipantLeft {
        participant_id: Uuid,
        left_at: DateTime<Utc>,
    },
    
    /// Chat message
    ChatMessage {
        id: Uuid,
        participant_id: Uuid,
        participant_name: String,
        content: String,
        reply_to: Option<Uuid>,
        timestamp: DateTime<Utc>,
    },
    
    /// Audio data (for broadcast)
    AudioData {
        participant_id: Uuid,
        data: String, // base64 encoded
        timestamp_ms: u64,
        sequence: u64,
    },
    
    /// Transcript update (live)
    TranscriptUpdate {
        segment: TranscriptSegment,
        is_final: bool,
    },
    
    /// AI insights real-time
    InsightUpdate {
        insight_type: String,
        data: serde_json::Value,
    },
    
    /// Screen share started/stopped
    ScreenShareUpdate {
        participant_id: Uuid,
        action: ScreenShareAction,
    },
    
    /// WebRTC signaling relay
    SignalRelay {
        from_user_id: Uuid,
        signal: SignalData,
    },
    
    /// Hand raised/lowered
    HandRaised {
        participant_id: Uuid,
        raised: bool,
    },
    
    /// Reaction received
    ReactionReceived {
        participant_id: Uuid,
        emoji: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Error message
    Error {
        code: String,
        message: String,
    },
    
    /// Pong for keepalive
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    pub id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
    pub is_speaking: bool,
    pub is_screen_sharing: bool,
    pub hand_raised: bool,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub id: Uuid,
    pub speaker_id: Uuid,
    pub speaker_name: String,
    pub text: String,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenShareAction {
    Start,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalData {
    Offer { sdp: String },
    Answer { sdp: String },
    IceCandidate { candidate: String, sdp_mid: String, sdp_mline_index: u32 },
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

impl ServerMessage {
    /// Serialize to JSON string
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

impl ClientMessage {
    /// Parse from JSON string
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}
