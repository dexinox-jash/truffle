//! Ruflo Notification Service
//!
//! Handles multi-channel notifications:
//! - Email (SES, SendGrid, SMTP)
//! - Slack
//! - Webhooks
//! - Push notifications

use axum::{
    routing::{get, post},
    Router,
    Json,
};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod email;
mod handlers;
mod slack;
mod templates;
mod webhook;

use config::AppConfig;
use email::EmailService;
use slack::SlackService;
use webhook::WebhookService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruflo_notify=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Starting Ruflo Notification Service");

    // Load configuration
    let config = Arc::new(AppConfig::from_env()?);

    // Initialize services
    let email_service = Arc::new(EmailService::new(config.clone()).await?);
    let slack_service = Arc::new(SlackService::new(config.clone()));
    let webhook_service = Arc::new(WebhookService::new(config.clone()));

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/send/email", post(handlers::send_email))
        .route("/api/v1/send/slack", post(handlers::send_slack))
        .route("/api/v1/send/webhook", post(handlers::send_webhook))
        .route("/api/v1/templates/render", post(handlers::render_template))
        .with_state(AppState {
            email: email_service,
            slack: slack_service,
            webhook: webhook_service,
        });

    // Start server
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub email: Arc<EmailService>,
    pub slack: Arc<SlackService>,
    pub webhook: Arc<WebhookService>,
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "ruflo-notify",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
