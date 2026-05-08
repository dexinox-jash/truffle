//! Ruflo Realtime Service
//!
//! WebSocket server for real-time meeting features:
//! - Live audio streaming
//! - Chat messages
//! - Participant presence
//! - Signaling for WebRTC

use axum::{
    extract::{State, WebSocketUpgrade, Path, ConnectInfo},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod config;
mod connection;
mod handlers;
mod hub;
mod message;
mod nats;
mod signaling;

use config::AppConfig;
use hub::MeetingHub;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruflo_realtime=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Starting Ruflo Realtime Service");

    // Load configuration
    let config = Arc::new(AppConfig::from_env()?);
    
    // Initialize meeting hub
    let hub = Arc::new(MeetingHub::new(config.clone()).await?);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws/meetings/:meeting_id", get(websocket_handler))
        .route("/ws/signaling/:meeting_id", get(signaling_handler))
        .with_state(hub);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("WebSocket server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "ruflo-realtime",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn websocket_handler(
    State(hub): State<Arc<MeetingHub>>,
    Path(meeting_id): Path<uuid::Uuid>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    tracing::info!("WebSocket connection from {} for meeting {}", addr, meeting_id);
    
    ws.on_upgrade(move |socket| {
        handlers::handle_meeting_socket(hub, meeting_id, socket, addr)
    })
}

async fn signaling_handler(
    State(hub): State<Arc<MeetingHub>>,
    Path(meeting_id): Path<uuid::Uuid>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    tracing::info!("Signaling connection from {} for meeting {}", addr, meeting_id);
    
    ws.on_upgrade(move |socket| {
        handlers::handle_signaling_socket(hub, meeting_id, socket, addr)
    })
}
