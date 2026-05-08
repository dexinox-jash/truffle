//! AI insight service using LLMs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::config::AppConfig;

pub struct InsightService {
    config: Arc<AppConfig>,
    http_client: Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryResult {
    pub summary: String,
    pub key_points: Vec<String>,
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub text: String,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    pub overall: String,
    pub score: f32,
    pub breakdown: std::collections::HashMap<String, f32>,
}

impl InsightService {
    pub async fn new(config: Arc<AppConfig>) -> anyhow::Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Generate summary of transcript
    pub async fn summarize(&self, transcript: &str) -> anyhow::Result<SummaryResult> {
        let prompt = format!(
            r#"Analyze this meeting transcript and provide:
1. A concise summary (2-3 sentences)
2. Key points discussed (bullet points)
3. Main topics covered

Transcript:
{}

Respond in JSON format with fields: summary, key_points (array), topics (array)"#,
            transcript
        );

        let response = self.call_llm(&prompt).await?;
        let result: SummaryResult = serde_json::from_str(&response)?;
        Ok(result)
    }

    /// Extract action items from transcript
    pub async fn extract_action_items(&self, transcript: &str) -> anyhow::Result<Vec<ActionItem>> {
        let prompt = format!(
            r#"Extract all action items from this meeting transcript. For each action item, identify:
- The task description
- Who is responsible (if mentioned)
- Due date (if mentioned)
- Priority (high/medium/low)

Transcript:
{}

Respond in JSON format as an array of action items with fields: id, text, assignee, due_date, priority"#,
            transcript
        );

        let response = self.call_llm(&prompt).await?;
        let items: Vec<ActionItem> = serde_json::from_str(&response)?;
        Ok(items)
    }

    /// Analyze sentiment
    pub async fn analyze_sentiment(&self, transcript: &str) -> anyhow::Result<SentimentResult> {
        let prompt = format!(
            r#"Analyze the sentiment of this meeting transcript. Provide:
1. Overall sentiment (positive, neutral, or negative)
2. Sentiment score (-1.0 to 1.0)
3. Breakdown by participant (if identifiable)

Transcript:
{}

Respond in JSON format with fields: overall, score, breakdown (object with participant -> score)"#,
            transcript
        );

        let response = self.call_llm(&prompt).await?;
        let result: SentimentResult = serde_json::from_str(&response)?;
        Ok(result)
    }

    /// General analysis
    pub async fn analyze(&self, text: &str, analysis_type: &str) -> anyhow::Result<serde_json::Value> {
        let prompt = match analysis_type {
            "decisions" => format!(
                "Extract all decisions made in this meeting transcript. Return as JSON array: {}",
                text
            ),
            "questions" => format!(
                "Extract all open questions from this meeting. Return as JSON array: {}",
                text
            ),
            "risks" => format!(
                "Identify potential risks or blockers mentioned. Return as JSON array: {}",
                text
            ),
            _ => format!("Analyze this text: {}", text),
        };

        let response = self.call_llm(&prompt).await?;
        let result: serde_json::Value = serde_json::from_str(&response)?;
        Ok(result)
    }

    /// Chat completion
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        stream: bool,
    ) -> anyhow::Result<String> {
        let request = json!({
            "model": "gpt-4",
            "messages": messages,
            "stream": stream,
            "temperature": 0.7,
            "max_tokens": 2000,
        });

        let response = self.http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.config.openai_api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error));
        }

        let chat_response: ChatCompletionResponse = response.json().await?;
        Ok(chat_response.choices[0].message.content.clone())
    }

    /// Internal method to call LLM
    async fn call_llm(&self, prompt: &str) -> anyhow::Result<String> {
        let request = json!({
            "model": "gpt-4",
            "messages": [
                {
                    "role": "system",
                    "content": "You are an AI assistant analyzing meeting transcripts. Always respond with valid JSON."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 2000,
        });

        let response = self.http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.config.openai_api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error));
        }

        let chat_response: ChatCompletionResponse = response.json().await?;
        
        // Extract JSON from markdown code block if present
        let content = &chat_response.choices[0].message.content;
        let json_str = if content.starts_with("```json") {
            content.trim_start_matches("```json").trim_end_matches("```").trim()
        } else if content.starts_with("```") {
            content.trim_start_matches("```").trim_end_matches("```").trim()
        } else {
            content
        };
        
        Ok(json_str.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}
