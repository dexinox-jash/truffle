//! Microsoft Teams integration
//!
//! Features:
//! - Teams meeting creation
//! - Tab integration
//! - Bot notifications
//! - Adaptive Cards

use serde::{Deserialize, Serialize};

pub struct TeamsIntegration {
    http_client: reqwest::Client,
    tenant_id: String,
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsMeeting {
    pub subject: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub attendees: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveCard {
    #[serde(rename = "type")]
    pub card_type: String,
    pub version: String,
    pub body: Vec<serde_json::Value>,
    pub actions: Option<Vec<serde_json::Value>>,
}

impl TeamsIntegration {
    pub fn new(tenant_id: String, client_id: String, client_secret: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            tenant_id,
            client_id,
            client_secret,
        }
    }

    /// Get access token for Microsoft Graph API
    async fn get_access_token(&self) -> anyhow::Result<String> {
        let url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.tenant_id
        );

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("scope", "https://graph.microsoft.com/.default"),
        ];

        let response = self.http_client
            .post(&url)
            .form(&params)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        
        let token = result["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No access token in response"))?;

        Ok(token.to_string())
    }

    /// Create a Teams meeting
    pub async fn create_meeting(&self, meeting: TeamsMeeting) -> anyhow::Result<String> {
        let token = self.get_access_token().await?;
        let url = "https://graph.microsoft.com/v1.0/me/events";

        let attendees: Vec<serde_json::Value> = meeting
            .attendees
            .iter()
            .map(|email| {
                serde_json::json!({
                    "emailAddress": {
                        "address": email
                    },
                    "type": "required"
                })
            })
            .collect();

        let body = serde_json::json!({
            "subject": meeting.subject,
            "start": {
                "dateTime": meeting.start_time.to_rfc3339(),
                "timeZone": "UTC"
            },
            "end": {
                "dateTime": meeting.end_time.to_rfc3339(),
                "timeZone": "UTC"
            },
            "attendees": attendees,
            "body": {
                "contentType": "HTML",
                "content": meeting.description.unwrap_or_default()
            },
            "isOnlineMeeting": true,
            "onlineMeetingProvider": "teamsForBusiness"
        });

        let response = self.http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        
        let join_url = result["onlineMeeting"]["joinUrl"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No join URL in response"))?;

        Ok(join_url.to_string())
    }

    /// Send Adaptive Card notification
    pub async fn send_adaptive_card(
        &self,
        webhook_url: &str,
        card: AdaptiveCard,
    ) -> anyhow::Result<()> {
        let body = serde_json::json!({
            "type": "message",
            "attachments": [
                {
                    "contentType": "application/vnd.microsoft.card.adaptive",
                    "contentUrl": null,
                    "content": card
                }
            ]
        });

        let response = self.http_client
            .post(webhook_url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Teams webhook error: {}",
                response.text().await?
            ));
        }

        Ok(())
    }

    /// Create meeting summary card
    pub fn create_summary_card(
        meeting_title: &str,
        summary: &str,
        action_items: &[String],
    ) -> AdaptiveCard {
        let mut body = vec![
            serde_json::json!({
                "type": "TextBlock",
                "text": format!("📝 {}", meeting_title),
                "weight": "Bolder",
                "size": "Large"
            }),
            serde_json::json!({
                "type": "TextBlock",
                "text": summary,
                "wrap": true
            }),
        ];

        if !action_items.is_empty() {
            body.push(serde_json::json!({
                "type": "TextBlock",
                "text": "Action Items",
                "weight": "Bolder",
                "separator": true
            }));

            for item in action_items {
                body.push(serde_json::json!({
                    "type": "TextBlock",
                    "text": format!("• {}", item),
                    "wrap": true
                }));
            }
        }

        AdaptiveCard {
            card_type: "AdaptiveCard".to_string(),
            version: "1.4".to_string(),
            body,
            actions: None,
        }
    }
}
