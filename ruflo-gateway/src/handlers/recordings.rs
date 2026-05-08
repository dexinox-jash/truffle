//! Recording handlers

use axum::{
    extract::{State, Path, Query},
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::user::CurrentUser,
    services::ServiceRegistry,
};

#[derive(Deserialize)]
pub struct ListRecordingsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_recordings(
    _user: CurrentUser,
    Path(_meeting_id): Path<Uuid>,
    Query(_query): Query<ListRecordingsQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement
    Ok(Json(serde_json::json!({"recordings": []})))
}

pub async fn get_recording(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement
    Err(ApiError::NotFound("Recording not found".to_string()))
}

pub async fn download_recording(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement
    Ok(Json(serde_json::json!({"download_url": ""})))
}
