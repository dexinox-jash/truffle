//! Transcript handlers

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

pub async fn get_transcript(
    _user: CurrentUser,
    Path(_meeting_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement
    Ok(Json(serde_json::json!({"transcript": null})))
}

pub async fn get_segments(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement
    Ok(Json(serde_json::json!({"segments": []})))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub async fn search_transcripts(
    _user: CurrentUser,
    Query(_query): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement
    Ok(Json(serde_json::json!({"results": []})))
}
