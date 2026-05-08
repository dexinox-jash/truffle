//! Ruflo Billing Service

use axum::{
    routing::{get, post},
    Router,
    Json,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/subscriptions", post(create_subscription))
        .route("/webhooks/stripe", post(stripe_webhook));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "healthy"}))
}

async fn create_subscription(Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"id": "sub_123", "status": "active"}))
}

async fn stripe_webhook(Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"received": true}))
}
