//! User models

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Current authenticated user (extracted from JWT)
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub role: String,
}

/// User response model
#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: Option<String>,
    pub timezone: String,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// User list response
#[derive(Debug, Clone, Serialize)]
pub struct UserList {
    pub data: Vec<User>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Update user request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub timezone: Option<String>,
    pub avatar_url: Option<String>,
}
