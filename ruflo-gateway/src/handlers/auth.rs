//! Authentication handlers

use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    error::ApiError,
    models::auth::{AuthResponse, LoginRequest, RegisterRequest},
    services::ServiceRegistry,
};

/// User registration
pub async fn register(
    State(services): State<Arc<ServiceRegistry>>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), ApiError> {
    let response = services.auth_service.register(req).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// User login
pub async fn login(
    State(services): State<Arc<ServiceRegistry>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let response = services.auth_service.login(req).await?;
    Ok(Json(response))
}

/// Refresh access token
#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh_token(
    State(services): State<Arc<ServiceRegistry>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let response = services.auth_service.refresh_token(&req.refresh_token).await?;
    Ok(Json(response))
}
