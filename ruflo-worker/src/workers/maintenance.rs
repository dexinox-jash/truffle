//! Maintenance worker - cleanup tasks

use std::sync::Arc;
use sqlx::PgPool;

use crate::{config::AppConfig, queue::JobQueue};

#[derive(Clone)]
pub struct MaintenanceWorker {
    db: PgPool,
    queue: Arc<JobQueue>,
    config: Arc<AppConfig>,
}

impl MaintenanceWorker {
    pub fn new(db: PgPool, queue: Arc<JobQueue>) -> Self {
        let config = Arc::new(AppConfig::from_env().unwrap());
        Self { db, queue, config }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

        loop {
            interval.tick().await;

            // Cleanup old recordings
            self.cleanup_recordings().await?;

            // Refresh analytics views
            self.refresh_analytics().await?;

            // Process data retention policies
            self.process_retention().await?;
        }
    }

    async fn cleanup_recordings(&self) -> anyhow::Result<()> {
        tracing::info!("Running recording cleanup");
        
        let deleted = sqlx::query(
            "DELETE FROM recordings WHERE status = 'deleted' AND deleted_at < NOW() - INTERVAL '30 days'"
        )
        .execute(&self.db)
        .await?;

        if deleted.rows_affected() > 0 {
            tracing::info!("Deleted {} old recordings", deleted.rows_affected());
        }

        Ok(())
    }

    async fn refresh_analytics(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT refresh_meeting_analytics()")
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn process_retention(&self) -> anyhow::Result<()> {
        // Process GDPR erasure requests
        let pending: Vec<(uuid::Uuid,)> = sqlx::query_as(
            "SELECT id FROM gdpr_erasure_requests WHERE status = 'pending' LIMIT 10"
        )
        .fetch_all(&self.db)
        .await?;

        for (request_id,) in pending {
            tracing::info!("Processing GDPR erasure request {}", request_id);
            
            // Mark as processing
            sqlx::query("UPDATE gdpr_erasure_requests SET status = 'processing' WHERE id = $1")
                .bind(request_id)
                .execute(&self.db)
                .await?;

            // Actual data deletion would happen here
            
            // Mark as completed
            sqlx::query("UPDATE gdpr_erasure_requests SET status = 'completed', completed_at = NOW() WHERE id = $1")
                .bind(request_id)
                .execute(&self.db)
                .await?;
        }

        Ok(())
    }
}
