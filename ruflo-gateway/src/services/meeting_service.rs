//! Meeting service

use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::{
    cache::Cache,
    error::ApiError,
    models::meeting::{CreateMeetingRequest, Meeting, MeetingList, Participant, UpdateMeetingRequest},
    handlers::meetings::ListMeetingsQuery,
};

pub struct MeetingService {
    db: PgPool,
    cache: Arc<Cache>,
}

impl MeetingService {
    pub fn new(db: PgPool, cache: Arc<Cache>) -> Self {
        Self { db, cache }
    }

    /// List meetings with filtering
    pub async fn list_meetings(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        query: ListMeetingsQuery,
    ) -> Result<MeetingList, ApiError> {
        let limit = query.limit.unwrap_or(20).min(100);
        let offset = query.offset.unwrap_or(0);

        // Build query dynamically
        let mut sql = r#"
            SELECT m.*, 
                   COALESCE(
                       jsonb_agg(
                           jsonb_build_object(
                               'id', mp.id,
                               'user_id', mp.user_id,
                               'email', mp.email,
                               'name', mp.name,
                               'role', mp.role,
                               'status', mp.status,
                               'is_external', mp.is_external
                           ) ORDER BY mp.created_at
                       ) FILTER (WHERE mp.id IS NOT NULL),
                       '[]'::jsonb
                   ) as participants
            FROM meetings m
            LEFT JOIN meeting_participants mp ON m.id = mp.meeting_id
            WHERE m.tenant_id = $1 AND m.deleted_at IS NULL
        "".to_string();

        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = vec![
            Box::new(tenant_id),
        ];

        if let Some(status) = query.status {
            sql.push_str(&format!(" AND m.status = ${}", params.len() + 1));
            params.push(Box::new(status));
        }

        if let Some(team_id) = query.team_id {
            sql.push_str(&format!(" AND m.team_id = ${}", params.len() + 1));
            params.push(Box::new(team_id));
        }

        if let Some(from_date) = query.from_date {
            sql.push_str(&format!(" AND m.scheduled_start_at >= ${}", params.len() + 1));
            params.push(Box::new(from_date));
        }

        if let Some(to_date) = query.to_date {
            sql.push_str(&format!(" AND m.scheduled_start_at <= ${}", params.len() + 1));
            params.push(Box::new(to_date));
        }

        sql.push_str(
            &format!("
            GROUP BY m.id
            ORDER BY m.scheduled_start_at DESC
            LIMIT ${} OFFSET ${}
        ", params.len() + 1, params.len() + 2));
        params.push(Box::new(limit));
        params.push(Box::new(offset));

        // Execute query using query_as with proper type
        let meetings: Vec<MeetingRow> = sqlx::query_as(&sql)
            .bind(tenant_id)
            .fetch_all(&self.db)
            .await
            .map_err(ApiError::DatabaseError)?;

        // Get total count
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM meetings WHERE tenant_id = $1 AND deleted_at IS NULL"
        )
        .bind(tenant_id)
        .fetch_one(&self.db)
        .await
        .map_err(ApiError::DatabaseError)?;

        let meetings: Vec<Meeting> = meetings.into_iter().map(|r| r.into()).collect();

        Ok(MeetingList {
            data: meetings,
            total,
            limit,
            offset,
        })
    }

    /// Create a new meeting
    pub async fn create_meeting(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        req: CreateMeetingRequest,
    ) -> Result<Meeting, ApiError> {
        // Validate times
        if req.scheduled_end_at <= req.scheduled_start_at {
            return Err(ApiError::ValidationError(
                "End time must be after start time".to_string()
            ));
        }

        let mut tx = self.db.begin().await?;

        // Create meeting
        let meeting: MeetingRow = sqlx::query_as(
            r#"
            INSERT INTO meetings (
                tenant_id, created_by, team_id, title, description, agenda,
                scheduled_start_at, scheduled_end_at, timezone, meeting_type,
                visibility, recording_enabled, transcription_enabled
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *, '[]'::jsonb as participants
            "#
        )
        .bind(tenant_id)
        .bind(user_id)
        .bind(req.team_id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(&req.agenda)
        .bind(req.scheduled_start_at)
        .bind(req.scheduled_end_at)
        .bind(req.timezone.unwrap_or_else(|| "UTC".to_string()))
        .bind(req.meeting_type.unwrap_or_else(|| "video".to_string()))
        .bind(req.visibility.unwrap_or_else(|| "private".to_string()))
        .bind(req.recording_enabled.unwrap_or(true))
        .bind(req.transcription_enabled.unwrap_or(true))
        .fetch_one(&mut *tx)
        .await?;

        // Add creator as participant
        let creator_email: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?;

        sqlx::query(
            r#"
            INSERT INTO meeting_participants (meeting_id, user_id, email, role, status)
            VALUES ($1, $2, $3, 'organizer', 'accepted')
            "#
        )
        .bind(meeting.id)
        .bind(user_id)
        .bind(&creator_email)
        .execute(&mut *tx)
        .await?;

        // Add other participants
        for participant in req.participants {
            sqlx::query(
                r#"
                INSERT INTO meeting_participants (meeting_id, email, name, role, is_external)
                VALUES ($1, $2, $3, $4, true)
                "#
            )
            .bind(meeting.id)
            .bind(&participant.email)
            .bind(&participant.name)
            .bind(participant.role.unwrap_or_else(|| "attendee".to_string()))
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // Fetch complete meeting with participants
        let meeting = self.get_meeting(tenant_id, meeting.id).await?;

        Ok(meeting)
    }

    /// Get meeting by ID
    pub async fn get_meeting(&self, tenant_id: Uuid, meeting_id: Uuid) -> Result<Meeting, ApiError> {
        let meeting: MeetingRow = sqlx::query_as(
            r#"
            SELECT m.*, 
                   COALESCE(
                       jsonb_agg(
                           jsonb_build_object(
                               'id', mp.id,
                               'user_id', mp.user_id,
                               'email', mp.email,
                               'name', mp.name,
                               'role', mp.role,
                               'status', mp.status,
                               'is_external', mp.is_external
                           ) ORDER BY mp.created_at
                       ) FILTER (WHERE mp.id IS NOT NULL),
                       '[]'::jsonb
                   ) as participants
            FROM meetings m
            LEFT JOIN meeting_participants mp ON m.id = mp.meeting_id
            WHERE m.id = $1 AND m.tenant_id = $2 AND m.deleted_at IS NULL
            GROUP BY m.id
            "#
        )
        .bind(meeting_id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Meeting not found".to_string()))?;

        Ok(meeting.into())
    }

    /// Update meeting
    pub async fn update_meeting(
        &self,
        tenant_id: Uuid,
        meeting_id: Uuid,
        user_id: Uuid,
        req: UpdateMeetingRequest,
    ) -> Result<Meeting, ApiError> {
        // Check if user can update (owner or admin)
        let can_update: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM meetings m
                LEFT JOIN meeting_participants mp ON m.id = mp.meeting_id AND mp.user_id = $2
                WHERE m.id = $1 AND m.tenant_id = $3
                AND (m.created_by = $2 OR mp.role = 'organizer')
            )
            "#
        )
        .bind(meeting_id)
        .bind(user_id)
        .bind(tenant_id)
        .fetch_one(&self.db)
        .await?;

        if !can_update {
            return Err(ApiError::Forbidden);
        }

        // Build dynamic update
        let mut updates = vec![];
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = vec![];

        if let Some(title) = req.title {
            updates.push(format!("title = ${}", params.len() + 1));
            params.push(Box::new(title));
        }
        if let Some(description) = req.description {
            updates.push(format!("description = ${}", params.len() + 1));
            params.push(Box::new(description));
        }
        if let Some(agenda) = req.agenda {
            updates.push(format!("agenda = ${}", params.len() + 1));
            params.push(Box::new(agenda));
        }
        if let Some(start) = req.scheduled_start_at {
            updates.push(format!("scheduled_start_at = ${}", params.len() + 1));
            params.push(Box::new(start));
        }
        if let Some(end) = req.scheduled_end_at {
            updates.push(format!("scheduled_end_at = ${}", params.len() + 1));
            params.push(Box::new(end));
        }
        if let Some(timezone) = req.timezone {
            updates.push(format!("timezone = ${}", params.len() + 1));
            params.push(Box::new(timezone));
        }
        if let Some(status) = req.status {
            updates.push(format!("status = ${}", params.len() + 1));
            params.push(Box::new(status));
        }

        if updates.is_empty() {
            return self.get_meeting(tenant_id, meeting_id).await;
        }

        let sql = format!(
            "UPDATE meetings SET {}, updated_at = NOW() WHERE id = ${} AND tenant_id = ${} RETURNING id",
            updates.join(", "),
            params.len() + 1,
            params.len() + 2
        );

        params.push(Box::new(meeting_id));
        params.push(Box::new(tenant_id));

        sqlx::query(&sql)
            .bind(meeting_id)
            .bind(tenant_id)
            .execute(&self.db)
            .await?;

        self.get_meeting(tenant_id, meeting_id).await
    }

    /// Delete meeting
    pub async fn delete_meeting(
        &self,
        tenant_id: Uuid,
        meeting_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ApiError> {
        let result = sqlx::query(
            r#"
            UPDATE meetings 
            SET deleted_at = NOW() 
            WHERE id = $1 AND tenant_id = $2 AND created_by = $3
            "#
        )
        .bind(meeting_id)
        .bind(tenant_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound("Meeting not found".to_string()));
        }

        Ok(())
    }

    /// Join meeting
    pub async fn join_meeting(
        &self,
        _tenant_id: Uuid,
        meeting_id: Uuid,
        user_id: Uuid,
    ) -> Result<serde_json::Value, ApiError> {
        // Update participant status
        sqlx::query(
            r#"
            UPDATE meeting_participants 
            SET status = 'accepted', joined_at = NOW()
            WHERE meeting_id = $1 AND user_id = $2
            "#
        )
        .bind(meeting_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Generate join token
        let join_token = Uuid::new_v4().to_string();

        Ok(serde_json::json!({
            "join_token": join_token,
            "websocket_url": format!("wss://api.ruflo.ai/ws/meetings/{}", meeting_id),
        }))
    }

    /// Leave meeting
    pub async fn leave_meeting(
        &self,
        _tenant_id: Uuid,
        meeting_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ApiError> {
        sqlx::query(
            "UPDATE meeting_participants SET left_at = NOW() WHERE meeting_id = $1 AND user_id = $2"
        )
        .bind(meeting_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Start meeting
    pub async fn start_meeting(
        &self,
        tenant_id: Uuid,
        meeting_id: Uuid,
        user_id: Uuid,
    ) -> Result<Meeting, ApiError> {
        sqlx::query(
            r#"
            UPDATE meetings 
            SET status = 'in_progress', actual_start_at = NOW() 
            WHERE id = $1 AND tenant_id = $2 AND created_by = $3
            "#
        )
        .bind(meeting_id)
        .bind(tenant_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        self.get_meeting(tenant_id, meeting_id).await
    }

    /// End meeting
    pub async fn end_meeting(
        &self,
        tenant_id: Uuid,
        meeting_id: Uuid,
        user_id: Uuid,
    ) -> Result<Meeting, ApiError> {
        sqlx::query(
            r#"
            UPDATE meetings 
            SET status = 'completed', actual_end_at = NOW() 
            WHERE id = $1 AND tenant_id = $2 AND created_by = $3
            "#
        )
        .bind(meeting_id)
        .bind(tenant_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        self.get_meeting(tenant_id, meeting_id).await
    }
}

// Helper struct for database queries
#[derive(sqlx::FromRow)]
struct MeetingRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    agenda: Option<String>,
    scheduled_start_at: chrono::DateTime<Utc>,
    scheduled_end_at: chrono::DateTime<Utc>,
    actual_start_at: Option<chrono::DateTime<Utc>>,
    actual_end_at: Option<chrono::DateTime<Utc>>,
    timezone: String,
    status: String,
    meeting_type: String,
    visibility: String,
    recording_enabled: bool,
    transcription_enabled: bool,
    join_url: Option<String>,
    created_by: Uuid,
    team_id: Option<Uuid>,
    participants: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<MeetingRow> for Meeting {
    fn from(row: MeetingRow) -> Self {
        let participants: Vec<Participant> = serde_json::from_value(row.participants)
            .unwrap_or_default();

        Self {
            id: row.id,
            title: row.title,
            description: row.description,
            agenda: row.agenda,
            scheduled_start_at: row.scheduled_start_at,
            scheduled_end_at: row.scheduled_end_at,
            actual_start_at: row.actual_start_at,
            actual_end_at: row.actual_end_at,
            timezone: row.timezone,
            status: row.status,
            meeting_type: row.meeting_type,
            visibility: row.visibility,
            recording_enabled: row.recording_enabled,
            transcription_enabled: row.transcription_enabled,
            join_url: row.join_url,
            created_by: row.created_by,
            team_id: row.team_id,
            participants,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
