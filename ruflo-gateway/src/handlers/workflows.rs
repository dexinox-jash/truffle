//! Workflow handlers

use axum::{
    extract::{State, Path, Query},
    response::Json,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::user::CurrentUser,
    services::ServiceRegistry,
};

pub async fn list_workflows(_user: CurrentUser) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"workflows": []})))
}

pub async fn create_workflow(
    _user: CurrentUser,
    Json(_req): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": Uuid::new_v4()}))))
}

pub async fn get_workflow(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Err(ApiError::NotFound("Workflow not found".to_string()))
}

pub async fn update_workflow(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"status": "updated"})))
}

pub async fn delete_workflow(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NO_CONTENT)
}

pub async fn execute_workflow(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"execution_id": Uuid::new_v4()})))
}

pub async fn list_executions(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"executions": []})))
}
