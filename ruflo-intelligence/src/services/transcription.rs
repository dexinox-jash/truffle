//! Speech-to-text transcription service using Whisper

use reqwest::Client;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;

pub struct TranscriptionService {
    config: Arc<AppConfig>,
    http_client: Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub id: Uuid,
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub confidence: f32,
    pub speaker_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub segments: Vec<TranscriptionSegment>,
    pub language: String,
    pub duration: f64,
}

impl TranscriptionService {
    pub async fn new(config: Arc<AppConfig>) -> anyhow::Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Transcribe audio file using OpenAI Whisper API
    pub async fn transcribe(&self, audio_data: Vec<u8>, file_name: &str) -> anyhow::Result<TranscriptionResult> {
        // Use OpenAI Whisper API
        let api_key = &self.config.openai_api_key;
        
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(audio_data)
                .file_name(file_name.to_string())
                .mime_str("audio/wav")?)
            .text("model", "whisper-1")
            .text("language", "en")
            .text("response_format", "verbose_json")
            .text("timestamp_granularities[]", "word")
            .text("timestamp_granularities[]", "segment");

        let response = self.http_client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Whisper API error: {}", error_text));
        }

        let whisper_response: WhisperResponse = response.json().await?;
        
        // Convert to our format
        let segments: Vec<TranscriptionSegment> = whisper_response.segments
            .into_iter()
            .map(|s| TranscriptionSegment {
                id: Uuid::new_v4(),
                start: s.start,
                end: s.end,
                text: s.text.trim().to_string(),
                confidence: s.avg_logprob.exp() as f32,
                speaker_id: None,
            })
            .collect();

        Ok(TranscriptionResult {
            text: whisper_response.text,
            segments,
            language: whisper_response.language,
            duration: whisper_response.duration,
        })
    }

    /// Stream transcription for real-time processing
    pub async fn transcribe_stream(
        &self,
        audio_stream: impl futures::Stream<Item = Vec<u8>> + Send + 'static,
    ) -> anyhow::Result<impl futures::Stream<Item = anyhow::Result<TranscriptionSegment>>> {
        // For streaming, we use a chunked approach
        // In production, this would use a streaming API or WebSocket
        
        use async_stream::stream;
        
        let s = stream! {
            // Implementation would buffer audio chunks and send to API
            // For now, yield dummy segment
            yield Ok(TranscriptionSegment {
                id: Uuid::new_v4(),
                start: 0.0,
                end: 1.0,
                text: "Streaming transcription...".to_string(),
                confidence: 0.95,
                speaker_id: None,
            });
        };
        
        Ok(s)
    }

    /// Perform speaker diarization
    pub async fn diarize(&self, _audio_data: Vec<u8>) -> anyhow::Result<Vec<TranscriptionSegment>> {
        // In production, use pyannote.audio or similar
        // For now, return empty
        Ok(vec![])
    }
}

// OpenAI Whisper API response format
#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
    language: String,
    duration: f64,
    segments: Vec<WhisperSegment>,
}

#[derive(Debug, Deserialize)]
struct WhisperSegment {
    id: i32,
    seek: i32,
    start: f64,
    end: f64,
    text: String,
    #[serde(rename = "avg_logprob")]
    avg_logprob: f64,
}
