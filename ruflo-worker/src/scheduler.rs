//! Job scheduler for cron jobs

use sqlx::PgPool;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler as CronJobScheduler};

use crate::queue::JobQueue;

pub struct JobScheduler {
    db: PgPool,
    queue: Arc<JobQueue>,
    scheduler: CronJobScheduler,
}

impl JobScheduler {
    pub fn new(db: PgPool, queue: Arc<JobQueue>) -> anyhow::Result<Self> {
        let scheduler = CronJobScheduler::new()?;
        
        Ok(Self {
            db,
            queue,
            scheduler,
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        // Schedule daily report generation
        self.scheduler.add(
            Job::new_async("0 0 9 * * *", |_uuid, _l| {
                Box::pin(async move {
                    tracing::info!("Running daily report job");
                })
            })?
        ).await?;

        // Schedule hourly analytics refresh
        self.scheduler.add(
            Job::new_async("0 0 * * * *", |_uuid, _l| {
                Box::pin(async move {
                    tracing::info!("Running hourly analytics refresh");
                })
            })?
        ).await?;

        // Start scheduler
        self.scheduler.start().await?;

        // Keep alive
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}
