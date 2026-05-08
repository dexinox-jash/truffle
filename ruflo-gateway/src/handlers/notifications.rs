//! Notification handlers

use axum::{
    extract::{Path, Query},
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::user::CurrentUser,
};

#[derive(Deserialize)]
pub struct ListNotificationsQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list_notifications(
    _user: CurrentUser,
    Query(_query): Query<ListNotificationsQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"notifications": []})))
}

pub async fn mark_read(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"status": "read"})))
}

pub async fn mark_all_read(_user: CurrentUser) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"status": "all_read"})))
}
