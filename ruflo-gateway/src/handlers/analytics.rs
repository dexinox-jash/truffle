//! Analytics handlers

use axum::{
    extract::Path,
    response::Json,
};
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::user::CurrentUser,
};

pub async fn meeting_analytics(_user: CurrentUser) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({
        "total_meetings": 0,
        "total_duration_minutes": 0,
    })))
}

pub async fn usage_analytics(_user: CurrentUser) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({
        "storage_used_bytes": 0,
        "transcription_minutes": 0,
    })))
}

pub async fn team_analytics(
    _user: CurrentUser,
    Path(_team_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"team_stats": {}})))
}
