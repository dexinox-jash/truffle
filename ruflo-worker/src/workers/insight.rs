//! AI insight worker - generates meeting insights

use std::sync::Arc;
use sqlx::PgPool;

use crate::{config::AppConfig, queue::JobQueue};

#[derive(Clone)]
pub struct InsightWorker {
    db: PgPool,
    queue: Arc<JobQueue>,
    http_client: reqwest::Client,
    config: Arc<AppConfig>,
}

impl InsightWorker {
    pub fn new(db: PgPool, queue: Arc<JobQueue>) -> Self {
        let config = Arc::new(AppConfig::from_env().unwrap());
        Self {
            db,
            queue,
            http_client: reqwest::Client::new(),
            config,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            // Look for meetings needing insights
            let pending: Option<(uuid::Uuid, uuid::Uuid)> = sqlx::query_as(
                "SELECT m.id, t.id FROM meetings m JOIN transcripts t ON m.id = t.meeting_id 
                 WHERE m.status = 'completed' AND NOT EXISTS (
                     SELECT 1 FROM meeting_insights mi WHERE mi.meeting_id = m.id
                 ) LIMIT 1"
            )
            .fetch_optional(&self.db)
            .await?;

            if let Some((meeting_id, transcript_id)) = pending {
                tracing::info!("Generating insights for meeting {}", meeting_id);

                // Get transcript text
                let transcript: Option<(String,)> = sqlx::query_as(
                    "SELECT full_text FROM transcripts WHERE id = $1"
                )
                .bind(transcript_id)
                .fetch_optional(&self.db)
                .await?;

                if let Some((text,)) = transcript {
                    // Call intelligence service
                    let response = self
                        .http_client
                        .post(format!("{}/api/v1/summarize", self.config.intelligence_service_url))
                        .json(&serde_json::json!({ "text": text }))
                        .send()
                        .await;

                    if let Ok(resp) = response {
                        if let Ok(result) = resp.json::<serde_json::Value>().await {
                            // Store insights
                            sqlx::query(
                                "INSERT INTO meeting_insights (meeting_id, tenant_id, transcript_id, summary, key_points) 
                                 VALUES ($1, $2, $3, $4, $5)"
                            )
                            .bind(meeting_id)
                            .bind(uuid::Uuid::new_v4())
                            .bind(transcript_id)
                            .bind(result["summary"].as_str())
                            .bind(&result["key_points"])
                            .execute(&self.db)
                            .await?;
                        }
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}
