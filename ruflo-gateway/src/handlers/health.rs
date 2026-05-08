//! Health check handlers

use axum::{
    extract::State,
    response::Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::services::ServiceRegistry;

/// Basic health check
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "ruflo-gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Readiness check - verifies dependencies
pub async fn readiness_check(
    State(services): State<Arc<ServiceRegistry>>,
) -> Json<serde_json::Value> {
    let mut checks = json!({});
    let mut overall_ready = true;

    // Check database
    let db_status = services.db_pool.acquire().await.is_ok();
    checks["database"] = json!({"status": if db_status { "ok" } else { "error" } });
    overall_ready = overall_ready && db_status;

    // Check cache
    let cache_status = services.cache.ping().await.is_ok();
    checks["cache"] = json!({"status": if cache_status { "ok" } else { "error" } });
    overall_ready = overall_ready && cache_status;

    Json(json!({
        "status": if overall_ready { "ready" } else { "not_ready" },
        "checks": checks,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
