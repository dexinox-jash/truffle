//! Team handlers

use axum::{
    extract::{Path, Query},
    response::Json,
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::user::CurrentUser,
};

pub async fn list_teams(_user: CurrentUser) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"teams": []})))
}

pub async fn create_team(
    _user: CurrentUser,
    Json(_req): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": Uuid::new_v4()}))))
}

pub async fn get_team(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Err(ApiError::NotFound("Team not found".to_string()))
}

pub async fn update_team(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"status": "updated"})))
}

pub async fn delete_team(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_team_members(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"members": []})))
}

pub async fn add_team_member(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
    Json(_req): Json<serde_json::Value>,
) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::CREATED)
}

pub async fn remove_team_member(
    _user: CurrentUser,
    Path((_team_id, _user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NO_CONTENT)
}
