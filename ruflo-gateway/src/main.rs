//! Ruflo API Gateway
//!
//! Entry point for all API requests. Handles routing, authentication,
//! rate limiting, and request forwarding to microservices.

use axum::{
    routing::{get, post, put, patch, delete},
    Router,
    middleware::{self, from_fn},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
    timeout::TimeoutLayer,
};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod cache;
mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod rate_limit;
mod services;

use config::AppConfig;
use middlewares::auth_middleware;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruflo_gateway=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Starting Ruflo API Gateway");

    // Load configuration
    let config = Arc::new(AppConfig::from_env()?);
    tracing::info!("Configuration loaded successfully");

    // Initialize services
    let services = Arc::new(services::ServiceRegistry::new(config.clone()).await?);
    
    // Build router
    let app = create_router(config, services);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router(config: Arc<AppConfig>, services: Arc<services::ServiceRegistry>) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(["x-request-id"]);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/ready", get(handlers::health::readiness_check))
        .route("/api/v1/auth/register", post(handlers::auth::register))
        .route("/api/v1/auth/login", post(handlers::auth::login))
        .route("/api/v1/auth/refresh", post(handlers::auth::refresh_token));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Users
        .route("/api/v1/users/me", get(handlers::users::get_current_user))
        .route("/api/v1/users/me", patch(handlers::users::update_current_user))
        .route("/api/v1/users", get(handlers::users::list_users))
        .route("/api/v1/users/:id", get(handlers::users::get_user))
        
        // Meetings
        .route("/api/v1/meetings", get(handlers::meetings::list_meetings))
        .route("/api/v1/meetings", post(handlers::meetings::create_meeting))
        .route("/api/v1/meetings/:id", get(handlers::meetings::get_meeting))
        .route("/api/v1/meetings/:id", patch(handlers::meetings::update_meeting))
        .route("/api/v1/meetings/:id", delete(handlers::meetings::delete_meeting))
        .route("/api/v1/meetings/:id/join", post(handlers::meetings::join_meeting))
        .route("/api/v1/meetings/:id/leave", post(handlers::meetings::leave_meeting))
        .route("/api/v1/meetings/:id/start", post(handlers::meetings::start_meeting))
        .route("/api/v1/meetings/:id/end", post(handlers::meetings::end_meeting))
        
        // Recordings
        .route("/api/v1/meetings/:id/recordings", get(handlers::recordings::list_recordings))
        .route("/api/v1/recordings/:id", get(handlers::recordings::get_recording))
        .route("/api/v1/recordings/:id/download", get(handlers::recordings::download_recording))
        
        // Transcripts
        .route("/api/v1/meetings/:id/transcript", get(handlers::transcripts::get_transcript))
        .route("/api/v1/transcripts/:id/segments", get(handlers::transcripts::get_segments))
        .route("/api/v1/transcripts/search", post(handlers::transcripts::search_transcripts))
        
        // Insights
        .route("/api/v1/meetings/:id/insights", get(handlers::insights::get_insights))
        .route("/api/v1/meetings/:id/summary", get(handlers::insights::get_summary))
        .route("/api/v1/meetings/:id/action-items", get(handlers::insights::get_action_items))
        
        // Workflows
        .route("/api/v1/workflows", get(handlers::workflows::list_workflows))
        .route("/api/v1/workflows", post(handlers::workflows::create_workflow))
        .route("/api/v1/workflows/:id", get(handlers::workflows::get_workflow))
        .route("/api/v1/workflows/:id", patch(handlers::workflows::update_workflow))
        .route("/api/v1/workflows/:id", delete(handlers::workflows::delete_workflow))
        .route("/api/v1/workflows/:id/execute", post(handlers::workflows::execute_workflow))
        .route("/api/v1/workflows/:id/executions", get(handlers::workflows::list_executions))
        
        // Teams
        .route("/api/v1/teams", get(handlers::teams::list_teams))
        .route("/api/v1/teams", post(handlers::teams::create_team))
        .route("/api/v1/teams/:id", get(handlers::teams::get_team))
        .route("/api/v1/teams/:id", patch(handlers::teams::update_team))
        .route("/api/v1/teams/:id", delete(handlers::teams::delete_team))
        .route("/api/v1/teams/:id/members", get(handlers::teams::list_team_members))
        .route("/api/v1/teams/:id/members", post(handlers::teams::add_team_member))
        .route("/api/v1/teams/:id/members/:user_id", delete(handlers::teams::remove_team_member))
        
        // Integrations
        .route("/api/v1/integrations", get(handlers::integrations::list_integrations))
        .route("/api/v1/integrations", post(handlers::integrations::create_integration))
        .route("/api/v1/integrations/:id", get(handlers::integrations::get_integration))
        .route("/api/v1/integrations/:id", delete(handlers::integrations::delete_integration))
        .route("/api/v1/integrations/:id/sync", post(handlers::integrations::sync_integration))
        
        // Notifications
        .route("/api/v1/notifications", get(handlers::notifications::list_notifications))
        .route("/api/v1/notifications/:id/read", post(handlers::notifications::mark_read))
        .route("/api/v1/notifications/read-all", post(handlers::notifications::mark_all_read))
        
        // Analytics
        .route("/api/v1/analytics/meetings", get(handlers::analytics::meeting_analytics))
        .route("/api/v1/analytics/usage", get(handlers::analytics::usage_analytics))
        .route("/api/v1/analytics/team/:id", get(handlers::analytics::team_analytics))
        
        // Apply auth middleware
        .route_layer(from_fn(auth_middleware));

    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors)
        .with_state(services)
}
