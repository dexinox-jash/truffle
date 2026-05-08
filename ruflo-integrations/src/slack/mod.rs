//! Slack integration for Ruflo
//!
//! Features:
//! - Meeting notifications to Slack channels
//! - Transcript sharing
//! - Action item creation
//! - /ruflo slash commands

use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct SlackIntegration {
    http_client: reqwest::Client,
    bot_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    pub channel: String,
    pub text: Option<String>,
    pub blocks: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    pub token: String,
    pub team_id: String,
    pub team_domain: String,
    pub channel_id: String,
    pub channel_name: String,
    pub user_id: String,
    pub user_name: String,
    pub command: String,
    pub text: String,
    pub response_url: String,
    pub trigger_id: String,
}

impl SlackIntegration {
    pub fn new(bot_token: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            bot_token,
        }
    }

    /// Send a message to a Slack channel
    pub async fn send_message(&self, message: SlackMessage) -> anyhow::Result<()> {
        let url = "https://slack.com/api/chat.postMessage";
        
        let response = self.http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .json(&message)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        
        if !result["ok"].as_bool().unwrap_or(false) {
            return Err(anyhow::anyhow!(
                "Slack API error: {}",
                result["error"].as_str().unwrap_or("unknown error")
            ));
        }

        Ok(())
    }

    /// Send meeting started notification
    pub async fn notify_meeting_started(
        &self,
        channel: &str,
        meeting_title: &str,
        join_url: &str,
    ) -> anyhow::Result<()> {
        let message = SlackMessage {
            channel: channel.to_string(),
            text: Some(format!("Meeting started: {}", meeting_title)),
            blocks: Some(vec![
                serde_json::json!({
                    "type": "header",
                    "text": {
                        "type": "plain_text",
                        "text": format!("📹 Meeting Started: {}", meeting_title)
                    }
                }),
                serde_json::json!({
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("<{}|Join Meeting>", join_url)
                    }
                }),
            ]),
        };

        self.send_message(message).await
    }

    /// Send transcript summary
    pub async fn share_transcript(
        &self,
        channel: &str,
        meeting_title: &str,
        summary: &str,
        action_items: &[String],
    ) -> anyhow::Result<()> {
        let mut blocks = vec![
            serde_json::json!({
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": format!("📝 Meeting Summary: {}", meeting_title)
                }
            }),
            serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": summary
                }
            }),
        ];

        if !action_items.is_empty() {
            let action_text = action_items
                .iter()
                .map(|item| format!("• {}", item))
                .collect::<Vec<_>>()
                .join("\n");

            blocks.push(serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!("*Action Items:*\n{}", action_text)
                }
            }));
        }

        let message = SlackMessage {
            channel: channel.to_string(),
            text: Some(format!("Meeting summary: {}", meeting_title)),
            blocks: Some(blocks),
        };

        self.send_message(message).await
    }

    /// Handle slash command
    pub async fn handle_slash_command(&self, command: SlashCommand) -> anyhow::Result<String> {
        match command.command.as_str() {
            "/ruflo" => {
                let args: Vec<&str> = command.text.split_whitespace().collect();
                
                match args.first() {
                    Some(&"join") => {
                        Ok("Click here to join your next meeting: https://ruflo.ai/meetings".to_string())
                    }
                    Some(&"schedule") => {
                        Ok("Schedule a meeting: https://ruflo.ai/meetings/new".to_string())
                    }
                    _ => {
                        Ok("Available commands: /ruflo join, /ruflo schedule".to_string())
                    }
                }
            }
            _ => Ok("Unknown command".to_string()),
        }
    }

    /// Verify Slack request signature
    pub fn verify_signature(
        &self,
        body: &str,
        timestamp: &str,
        signature: &str,
        signing_secret: &str,
    ) -> anyhow::Result<bool> {
        use sha2::{Sha256, Digest};
        use hmac::{Hmac, Mac};

        type HmacSha256 = Hmac<Sha256>;

        let sig_base = format!("v0:{}:{}", timestamp, body);
        
        let mut mac = HmacSha256::new_from_slice(signing_secret.as_bytes())
            .map_err(|e| anyhow::anyhow!("HMAC error: {}", e))?;
        mac.update(sig_base.as_bytes());
        
        let result = mac.finalize();
        let expected_sig = format!("v0={}", hex::encode(result.into_bytes()));
        
        Ok(expected_sig == signature)
    }
}
