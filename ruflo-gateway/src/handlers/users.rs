//! User handlers

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
    models::user::{CurrentUser, UpdateUserRequest, User, UserList},
    services::ServiceRegistry,
};

#[derive(Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_current_user(user: CurrentUser) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "tenant_id": user.tenant_id,
        "role": user.role,
    }))
}

pub async fn update_current_user(
    _user: CurrentUser,
    Json(_req): Json<UpdateUserRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement
    Ok(Json(serde_json::json!({"status": "updated"})))
}

pub async fn list_users(
    _user: CurrentUser,
    Query(_query): Query<ListUsersQuery>,
) -> Result<Json<UserList>, ApiError> {
    // TODO: Implement
    Ok(Json(UserList {
        data: vec![],
        total: 0,
        limit: 20,
        offset: 0,
    }))
}

pub async fn get_user(
    _user: CurrentUser,
    Path(_id): Path<Uuid>,
) -> Result<Json<User>, ApiError> {
    // TODO: Implement
    Err(ApiError::NotFound("User not found".to_string()))
}
