//! Transcription worker - processes audio files

use std::sync::Arc;
use sqlx::PgPool;

use crate::{config::AppConfig, queue::{Job, JobQueue}};

#[derive(Clone)]
pub struct TranscriptionWorker {
    db: PgPool,
    queue: Arc<JobQueue>,
    http_client: reqwest::Client,
    config: Arc<AppConfig>,
}

impl TranscriptionWorker {
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
            // Poll for jobs from database or message queue
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            // Look for pending transcription jobs
            let pending: Option<(uuid::Uuid, String)> = sqlx::query_as(
                "SELECT id, storage_path FROM recordings WHERE status = 'pending' LIMIT 1"
            )
            .fetch_optional(&self.db)
            .await?;

            if let Some((recording_id, _path)) = pending {
                tracing::info!("Processing transcription for recording {}", recording_id);
                
                // Mark as processing
                sqlx::query("UPDATE recordings SET status = 'processing' WHERE id = $1")
                    .bind(recording_id)
                    .execute(&self.db)
                    .await?;

                // Simulate processing time
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

                // Mark as ready
                sqlx::query("UPDATE recordings SET status = 'ready' WHERE id = $1")
                    .bind(recording_id)
                    .execute(&self.db)
                    .await?;

                tracing::info!("Transcription completed for recording {}", recording_id);
            }
        }
    }
}
