//! Ruflo Integration Tests
//!
//! Comprehensive integration testing across all microservices.

pub mod fixtures;
pub mod helpers;

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment
pub fn init() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    });
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub api_endpoint: String,
    pub database_url: String,
    pub redis_url: String,
    pub nats_url: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            api_endpoint: std::env::var("TEST_API_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            database_url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ruflo_test".to_string()),
            redis_url: std::env::var("TEST_REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            nats_url: std::env::var("TEST_NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
        }
    }
}
