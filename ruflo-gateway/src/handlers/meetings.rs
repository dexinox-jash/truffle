//! Meeting handlers

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
    models::{
        meeting::{CreateMeetingRequest, Meeting, MeetingList, UpdateMeetingRequest},
        user::CurrentUser,
    },
    services::ServiceRegistry,
};

/// List meetings with filtering
#[derive(Deserialize)]
pub struct ListMeetingsQuery {
    pub status: Option<String>,
    pub team_id: Option<Uuid>,
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_meetings(
    State(services): State<Arc<ServiceRegistry>>,
    user: CurrentUser,
    Query(query): Query<ListMeetingsQuery>,
) -> Result<Json<MeetingList>, ApiError> {
    let meetings = services
        .meeting_service
        .list_meetings(user.tenant_id, user.id, query)
        .await?;
    Ok(Json(meetings))
}

/// Create a new meeting
pub async fn create_meeting(
    State(services): State<Arc<ServiceRegistry>>,
    user: CurrentUser,
    Json(req): Json<CreateMeetingRequest>,
) -> Result<(StatusCode, Json<Meeting>), ApiError> {
    let meeting = services
        .meeting_service
        .create_meeting(user.tenant_id, user.id, req)
        .await?;
    Ok((StatusCode::CREATED, Json(meeting)))
}

/// Get meeting by ID
pub async fn get_meeting(
    State(services): State<Arc<ServiceRegistry>>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Meeting>, ApiError> {
    let meeting = services
        .meeting_service
        .get_meeting(user.tenant_id, id)
        .await?;
    Ok(Json(meeting))
}

/// Update meeting
pub async fn update_meeting(
    State(services): State<Arc<ServiceRegistry>>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateMeetingRequest>,
) -> Result<Json<Meeting>, ApiError> {
    let meeting = services
        .meeting_service
        .update_meeting(user.tenant_id, id, user.id, req)
        .await?;
    Ok(Json(meeting))
}

/// Delete meeting
pub async fn delete_meeting(
    State(services): State<Arc<ServiceRegistry>>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    services
        .meeting_service
        .delete_meeting(user.tenant_id, id, user.id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Join meeting
pub async fn join_meeting(
    State(services): State<Arc<ServiceRegistry>>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let join_info = services
        .meeting_service
        .join_meeting(user.tenant_id, id, user.id)
        .await?;
    Ok(Json(join_info))
}

/// Leave meeting
pub async fn leave_meeting(
    State(services): State<Arc<ServiceRegistry>>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    services
        .meeting_service
        .leave_meeting(user.tenant_id, id, user.id)
        .await?;
    Ok(StatusCode::OK)
}

/// Start meeting
pub async fn start_meeting(
    State(services): State<Arc<ServiceRegistry>>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Meeting>, ApiError> {
    let meeting = services
        .meeting_service
        .start_meeting(user.tenant_id, id, user.id)
        .await?;
    Ok(Json(meeting))
}

/// End meeting
pub async fn end_meeting(
    State(services): State<Arc<ServiceRegistry>>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Meeting>, ApiError> {
    let meeting = services
        .meeting_service
        .end_meeting(user.tenant_id, id, user.id)
        .await?;
    Ok(Json(meeting))
}
