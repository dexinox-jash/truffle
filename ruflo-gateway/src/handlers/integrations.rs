//! Integration handlers

use axum::{
    extract::Path,
    response::Json,
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::user::CurrentUser,
};

pub async fn list_integrations(_user: CurrentUser) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"integrations": []})))
}

pub async fn create_integration(
    _user: CurrentUser,
    Json(_req): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": Uuid::new_v4()}))))
}

pub async fn get_integration(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Err(ApiError::NotFound("Integration not found".to_string()))
}

pub async fn delete_integration(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NO_CONTENT)
}

pub async fn sync_integration(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"status": "syncing"})))
}
