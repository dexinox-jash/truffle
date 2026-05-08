//! Service layer for business logic

use std::sync::Arc;
use crate::config::AppConfig;

pub mod auth_service;
pub mod meeting_service;

pub use auth_service::AuthService;
pub use meeting_service::MeetingService;

/// Registry of all services
pub struct ServiceRegistry {
    pub config: Arc<AppConfig>,
    pub db_pool: sqlx::PgPool,
    pub cache: Arc<crate::cache::Cache>,
    pub auth_service: AuthService,
    pub meeting_service: MeetingService,
}

impl ServiceRegistry {
    pub async fn new(config: Arc<AppConfig>) -> anyhow::Result<Self> {
        // Initialize database pool
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(&config.database_url)
            .await?;

        // Initialize cache
        let cache = Arc::new(crate::cache::Cache::new(&config.redis_url).await?);

        // Initialize services
        let auth_service = AuthService::new(db_pool.clone(), config.clone());
        let meeting_service = MeetingService::new(db_pool.clone(), cache.clone());

        Ok(Self {
            config,
            db_pool,
            cache,
            auth_service,
            meeting_service,
        })
    }
}
