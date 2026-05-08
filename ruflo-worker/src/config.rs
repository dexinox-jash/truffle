//! Configuration

use std::env;

pub struct AppConfig {
    pub database_url: String,
    pub nats_url: String,
    pub redis_url: String,
    pub intelligence_service_url: String,
    pub notification_service_url: String,
    pub storage_bucket: String,
    pub aws_region: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/ruflo".to_string()),
            nats_url: env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            intelligence_service_url: env::var("INTELLIGENCE_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8082".to_string()),
            notification_service_url: env::var("NOTIFICATION_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8083".to_string()),
            storage_bucket: env::var("STORAGE_BUCKET")
                .unwrap_or_else(|_| "ruflo-recordings".to_string()),
            aws_region: env::var("AWS_REGION")
                .unwrap_or_else(|_| "us-west-2".to_string()),
        })
    }
}
