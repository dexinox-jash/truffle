//! Job queue using NATS JetStream

use async_nats::jetstream::{self, consumer::PullConsumer, stream::Stream};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::AppConfig;

/// Job queue using NATS JetStream
pub struct JobQueue {
    jetstream: jetstream::Context,
    stream: Stream,
}

/// Job types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Job {
    /// Transcribe audio file
    Transcribe {
        recording_id: Uuid,
        meeting_id: Uuid,
        tenant_id: Uuid,
        storage_path: String,
        language: String,
    },

    /// Generate AI insights
    GenerateInsights {
        meeting_id: Uuid,
        tenant_id: Uuid,
        transcript_id: Uuid,
    },

    /// Send email notification
    SendEmail {
        to: Vec<String>,
        template: String,
        data: serde_json::Value,
    },

    /// Send Slack notification
    SendSlack {
        channel: String,
        message: String,
        blocks: Option<serde_json::Value>,
    },

    /// Process data retention policy
    DataRetention {
        tenant_id: Uuid,
        policy: String,
    },

    /// Generate analytics report
    GenerateReport {
        tenant_id: Uuid,
        report_type: String,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    },

    /// Sync integration data
    SyncIntegration {
        integration_id: Uuid,
        tenant_id: Uuid,
    },

    /// Cleanup old recordings
    CleanupRecordings {
        older_than_days: i32,
    },

    /// Refresh materialized views
    RefreshAnalytics,
}

impl JobQueue {
    pub async fn new(config: Arc<AppConfig>) -> anyhow::Result<Self> {
        // Connect to NATS
        let client = async_nats::connect(&config.nats_url).await?;
        let jetstream = async_nats::jetstream::new(client);

        // Create stream if not exists
        let stream = jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: "JOBS".to_string(),
                subjects: vec!["jobs.*".to_string()],
                ..Default::default()
            })
            .await?;

        tracing::info!("Connected to NATS JetStream");

        Ok(Self {
            jetstream,
            stream,
        })
    }

    /// Publish a job to the queue
    pub async fn publish(&self, job: Job) -> anyhow::Result<()> {
        let subject = format!("jobs.{}", job_type(&job));
        let payload = serde_json::to_vec(&job)?;

        self.jetstream
            .publish(async_nats::Publish::build().payload(payload.into()).subject(subject))
            .await?;

        tracing::info!("Published job: {:?}", job);
        Ok(())
    }

    /// Create a consumer for a specific job type
    pub async fn create_consumer(&self, job_type: &str) -> anyhow::Result<PullConsumer> {
        let consumer = self
            .jetstream
            .create_consumer_on_stream(
                format!("{}_consumer", job_type),
                "JOBS",
                async_nats::jetstream::consumer::pull::Config {
                    filter_subject: Some(format!("jobs.{}", job_type)),
                    durable_name: Some(format!("{}_durable", job_type)),
                    ..Default::default()
                },
            )
            .await?;

        Ok(consumer)
    }

    /// Get the underlying JetStream context
    pub fn jetstream(&self) -> &jetstream::Context {
        &self.jetstream
    }
}

fn job_type(job: &Job) -> &'static str {
    match job {
        Job::Transcribe { .. } => "transcribe",
        Job::GenerateInsights { .. } => "insights",
        Job::SendEmail { .. } => "email",
        Job::SendSlack { .. } => "slack",
        Job::DataRetention { .. } => "retention",
        Job::GenerateReport { .. } => "report",
        Job::SyncIntegration { .. } => "sync",
        Job::CleanupRecordings { .. } => "cleanup",
        Job::RefreshAnalytics => "analytics",
    }
}
