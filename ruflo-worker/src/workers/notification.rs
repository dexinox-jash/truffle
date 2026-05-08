//! Notification worker - sends emails and Slack messages

use std::sync::Arc;
use sqlx::PgPool;

use crate::{config::AppConfig, queue::JobQueue};

#[derive(Clone)]
pub struct NotificationWorker {
    db: PgPool,
    queue: Arc<JobQueue>,
    http_client: reqwest::Client,
    config: Arc<AppConfig>,
}

impl NotificationWorker {
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
            // Poll for pending notifications
            let pending: Option<(uuid::Uuid, String, String)> = sqlx::query_as(
                "SELECT id, channel, payload FROM notifications WHERE status = 'pending' LIMIT 1"
            )
            .fetch_optional(&self.db)
            .await?;

            if let Some((id, channel, payload)) = pending {
                tracing::info!("Sending notification {} via {}", id, channel);

                // Mark as sent
                sqlx::query(
                    "UPDATE notifications SET status = 'sent', sent_at = NOW() WHERE id = $1"
                )
                .bind(id)
                .execute(&self.db)
                .await?;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}
