//! Zoom integration for Ruflo
//!
//! Features:
//! - Create Zoom meetings
//! - OAuth authentication
//! - Webhook handling
//! - Recording download

use serde::{Deserialize, Serialize};

pub struct ZoomIntegration {
    http_client: reqwest::Client,
    client_id: String,
    client_secret: String,
    account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomMeeting {
    pub topic: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub duration_minutes: i32,
    pub timezone: String,
    pub agenda: Option<String>,
    pub settings: ZoomMeetingSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomMeetingSettings {
    pub host_video: bool,
    pub participant_video: bool,
    pub join_before_host: bool,
    pub mute_upon_entry: bool,
    pub watermark: bool,
    pub use_pmi: bool,
    pub approval_type: i32,
    pub audio: String,
    pub auto_recording: String,
}

impl Default for ZoomMeetingSettings {
    fn default() -> Self {
        Self {
            host_video: true,
            participant_video: true,
            join_before_host: false,
            mute_upon_entry: true,
            watermark: false,
            use_pmi: false,
            approval_type: 2,
            audio: "both".to_string(),
            auto_recording: "cloud".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomMeetingResponse {
    pub id: String,
    pub join_url: String,
    pub start_url: String,
    pub topic: String,
    pub start_time: String,
    pub duration: i32,
    pub password: Option<String>,
    pub h323_password: Option<String>,
    pub pstn_password: Option<String>,
    pub encrypted_password: Option<String>,
    pub settings: ZoomMeetingSettings,
}

impl ZoomIntegration {
    pub fn new(client_id: String, client_secret: String, account_id: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            client_id,
            client_secret,
            account_id,
        }
    }

    /// Get Server-to-Server OAuth token
    async fn get_access_token(&self) -> anyhow::Result<String> {
        let url = format!(
            "https://zoom.us/oauth/token?grant_type=account_credentials&account_id={}",
            self.account_id
        );

        let credentials = base64::encode(format!("{}:{}", self.client_id, self.client_secret));

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Basic {}", credentials))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        
        let token = result["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No access token"))?;

        Ok(token.to_string())
    }

    /// Create a Zoom meeting
    pub async fn create_meeting(&self, meeting: ZoomMeeting) -> anyhow::Result<ZoomMeetingResponse> {
        let token = self.get_access_token().await?;
        let url = "https://api.zoom.us/v2/users/me/meetings";

        let body = serde_json::json!({
            "topic": meeting.topic,
            "type": 2, // Scheduled meeting
            "start_time": meeting.start_time.to_rfc3339(),
            "duration": meeting.duration_minutes,
            "timezone": meeting.timezone,
            "agenda": meeting.agenda,
            "settings": meeting.settings,
        });

        let response = self.http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let meeting_response: ZoomMeetingResponse = response.json().await?;
        Ok(meeting_response)
    }

    /// Delete a Zoom meeting
    pub async fn delete_meeting(&self, meeting_id: &str) -> anyhow::Result<()> {
        let token = self.get_access_token().await?;
        let url = format!("https://api.zoom.us/v2/meetings/{}", meeting_id);

        self.http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        Ok(())
    }

    /// Get meeting recordings
    pub async fn get_recordings(&self, meeting_id: &str) -> anyhow::Result<Vec<ZoomRecording>> {
        let token = self.get_access_token().await?;
        let url = format!("https://api.zoom.us/v2/meetings/{}/recordings", meeting_id);

        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        
        let recordings: Vec<ZoomRecording> = result["recording_files"]
            .as_array()
            .map(|files| {
                files
                    .iter()
                    .filter_map(|f| serde_json::from_value(f.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(recordings)
    }

    /// Download recording
    pub async fn download_recording(&self, download_url: &str) -> anyhow::Result<Vec<u8>> {
        let token = self.get_access_token().await?;
        
        let response = self.http_client
            .get(download_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Handle Zoom webhook
    pub fn handle_webhook(&self, payload: &str, signature: &str, webhook_secret: &str) -> anyhow::Result<ZoomWebhookEvent> {
        // Verify webhook signature
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let message = format!("{}{}", webhook_secret, payload);
        let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes())?;
        mac.update(message.as_bytes());
        let expected_signature = hex::encode(mac.finalize().into_bytes());

        if signature != expected_signature {
            return Err(anyhow::anyhow!("Invalid webhook signature"));
        }

        let event: ZoomWebhookEvent = serde_json::from_str(payload)?;
        Ok(event)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomRecording {
    pub id: String,
    pub meeting_id: String,
    pub recording_start: String,
    pub recording_end: String,
    pub file_type: String,
    pub file_size: i64,
    pub play_url: String,
    pub download_url: String,
    pub recording_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomWebhookEvent {
    pub event: String,
    pub payload: serde_json::Value,
    pub event_ts: i64,
}

pub mod events {
    pub const RECORDING_COMPLETED: &str = "recording.completed";
    pub const MEETING_STARTED: &str = "meeting.started";
    pub const MEETING_ENDED: &str = "meeting.ended";
    pub const PARTICIPANT_JOINED: &str = "meeting.participant_joined";
    pub const PARTICIPANT_LEFT: &str = "meeting.participant_left";
}
