//! Ruflo Intelligence Service
//!
//! AI/ML inference service providing:
//! - Speech-to-text (Whisper)
//! - Embeddings & semantic search
//! - LLM-powered insights
//! - Sentiment analysis

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod handlers;
mod models;
mod services;

use config::AppConfig;
use services::{EmbeddingService, InsightService, TranscriptionService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruflo_intelligence=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Starting Ruflo Intelligence Service");

    // Load configuration
    let config = Arc::new(AppConfig::from_env()?);

    // Initialize AI services
    let embedding_service = Arc::new(EmbeddingService::new().await?);
    let transcription_service = Arc::new(TranscriptionService::new(config.clone()).await?);
    let insight_service = Arc::new(InsightService::new(config.clone()).await?);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        // Transcription
        .route("/api/v1/transcribe", post(handlers::transcribe_audio))
        .route("/api/v1/transcribe/stream", post(handlers::transcribe_stream))
        // Embeddings
        .route("/api/v1/embed", post(handlers::create_embedding))
        .route("/api/v1/embed/batch", post(handlers::create_embeddings_batch))
        .route("/api/v1/search", post(handlers::semantic_search))
        // Insights
        .route("/api/v1/analyze", post(handlers::analyze_text))
        .route("/api/v1/summarize", post(handlers::summarize))
        .route("/api/v1/extract-action-items", post(handlers::extract_action_items))
        .route("/api/v1/sentiment", post(handlers::analyze_sentiment))
        // LLM
        .route("/api/v1/chat", post(handlers::chat_completion))
        .route("/api/v1/chat/stream", post(handlers::chat_completion_stream))
        .with_state(AppState {
            config: config.clone(),
            embedding: embedding_service,
            transcription: transcription_service,
            insight: insight_service,
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
    pub config: Arc<AppConfig>,
    pub embedding: Arc<EmbeddingService>,
    pub transcription: Arc<TranscriptionService>,
    pub insight: Arc<InsightService>,
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "ruflo-intelligence",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn readiness_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ready",
        "models": {
            "embeddings": true,
            "transcription": true,
            "llm": true,
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
