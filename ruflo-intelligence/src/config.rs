//! Configuration

use std::env;

pub struct AppConfig {
    pub openai_api_key: String,
    pub database_url: String,
    pub redis_url: String,
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            openai_api_key: env::var("OPENAI_API_KEY")
                .unwrap_or_else(|_| "sk-test".to_string()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/ruflo".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
        })
    }
}
