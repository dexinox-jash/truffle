//! Ruflo Background Worker
//!
//! Processes background jobs:
//! - Transcription processing
//! - AI insight generation
//! - Email notifications
//! - Data retention policies
//! - Report generation

use std::sync::Arc;
use tokio::task::JoinSet;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod jobs;
mod queue;
mod scheduler;
mod workers;

use config::AppConfig;
use queue::JobQueue;
use workers::{TranscriptionWorker, NotificationWorker, InsightWorker, MaintenanceWorker};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruflo_worker=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Starting Ruflo Background Worker");

    // Load configuration
    let config = Arc::new(AppConfig::from_env()?);

    // Initialize job queue (NATS JetStream)
    let queue = Arc::new(JobQueue::new(config.clone()).await?);

    // Initialize database pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // Create workers
    let transcription_worker = TranscriptionWorker::new(db_pool.clone(), queue.clone());
    let notification_worker = NotificationWorker::new(db_pool.clone(), queue.clone());
    let insight_worker = InsightWorker::new(db_pool.clone(), queue.clone());
    let maintenance_worker = MaintenanceWorker::new(db_pool.clone(), queue.clone());

    // Start scheduler for cron jobs
    let scheduler = scheduler::JobScheduler::new(db_pool.clone(), queue.clone())?;
    tokio::spawn(async move {
        if let Err(e) = scheduler.run().await {
            tracing::error!("Scheduler error: {}", e);
        }
    });

    // Start worker pool
    let mut join_set = JoinSet::new();

    // Transcription workers
    for i in 0..3 {
        let worker = transcription_worker.clone();
        join_set.spawn(async move {
            tracing::info!("Starting transcription worker {}", i);
            worker.run().await
        });
    }

    // Notification workers
    for i in 0..2 {
        let worker = notification_worker.clone();
        join_set.spawn(async move {
            tracing::info!("Starting notification worker {}", i);
            worker.run().await
        });
    }

    // Insight workers
    for i in 0..2 {
        let worker = insight_worker.clone();
        join_set.spawn(async move {
            tracing::info!("Starting insight worker {}", i);
            worker.run().await
        });
    }

    // Maintenance worker (singleton)
    let worker = maintenance_worker.clone();
    join_set.spawn(async move {
        tracing::info!("Starting maintenance worker");
        worker.run().await
    });

    // Wait for all workers
    while let Some(result) = join_set.join_next().await {
        if let Err(e) = result {
            tracing::error!("Worker error: {}", e);
        }
    }

    Ok(())
}
