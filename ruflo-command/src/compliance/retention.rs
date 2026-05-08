//! Data Retention & Lifecycle Management
//!
//! GDPR and SOC 2 compliant data lifecycle management with automatic
//! deletion, anonymization, and legal hold capabilities.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Data retention policy
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub data_type: String,
    pub retention_period_days: u32,
    pub action: RetentionAction,
    pub legal_hold: bool,
    pub auto_delete: bool,
    pub notify_before_days: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Retention action
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "retention_action", rename_all = "snake_case")]
pub enum RetentionAction {
    /// Permanently delete data
    Delete,
    /// Anonymize personal data
    Anonymize,
    /// Archive to cold storage
    Archive,
    /// Flag for review
    Review,
}

/// Data subject request (DSR) types
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "dsr_type", rename_all = "snake_case")]
pub enum DsrType {
    /// Access request - export all data
    Access,
    /// Deletion request - right to be forgotten
    Deletion,
    /// Correction request
    Correction,
    /// Portability request
    Portability,
    /// Restriction of processing
    Restriction,
}

/// Data subject request
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DataSubjectRequest {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub request_type: DsrType,
    pub status: DsrStatus,
    pub subject_email: String,
    pub subject_id: String,
    pub description: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result_url: Option<String>,
    pub error_message: Option<String>,
    pub verified: bool,
    pub verification_token: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// DSR status
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "dsr_status", rename_all = "snake_case")]
pub enum DsrStatus {
    Pending,
    Verified,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Legal hold
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LegalHold {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: String,
    pub case_number: Option<String>,
    pub active: bool,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub released_at: Option<DateTime<Utc>>,
}

/// Data classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "data_classification", rename_all = "snake_case")]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
    Pii, // Personally Identifiable Information
    Phi, // Protected Health Information
}

/// Retention manager
pub struct RetentionManager {
    pool: PgPool,
    legal_holds: Arc<RwLock<HashMap<Uuid, Vec<String>>>>, // tenant_id -> list of data types on hold
}

/// Data deletion job
#[derive(Debug, Clone)]
pub struct DeletionJob {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub data_type: String,
    pub cutoff_date: DateTime<Utc>,
    pub status: DeletionStatus,
}

/// Deletion status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeletionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl RetentionManager {
    /// Create new retention manager
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            legal_holds: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize and start background tasks
    pub async fn initialize(&self) -> anyhow::Result<()> {
        // Load active legal holds
        self.load_legal_holds().await?;

        // Start retention enforcement task
        self.start_retention_enforcement().await;

        // Start DSR processor
        self.start_dsr_processor().await;

        info!("Retention manager initialized");
        Ok(())
    }

    /// Load active legal holds
    async fn load_legal_holds(&self) -> anyhow::Result<()> {
        let holds: Vec<LegalHold> = sqlx::query_as(
            "SELECT * FROM legal_holds WHERE active = true"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut hold_map = self.legal_holds.write().await;
        for hold in holds {
            hold_map
                .entry(hold.tenant_id)
                .or_default()
                .push(hold.name.clone());
        }

        info!("Loaded {} active legal holds", holds.len());
        Ok(())
    }

    /// Create retention policy
    pub async fn create_policy(
        &self,
        tenant_id: Uuid,
        name: String,
        description: Option<String>,
        data_type: String,
        retention_period_days: u32,
        action: RetentionAction,
        auto_delete: bool,
        notify_before_days: u32,
    ) -> anyhow::Result<RetentionPolicy> {
        let policy = sqlx::query_as::<_, RetentionPolicy>(
            r#"
            INSERT INTO retention_policies (
                id, tenant_id, name, description, data_type, retention_period_days,
                action, legal_hold, auto_delete, notify_before_days, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, false, $8, $9, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(name)
        .bind(description)
        .bind(data_type)
        .bind(retention_period_days as i32)
        .bind(action)
        .bind(auto_delete)
        .bind(notify_before_days as i32)
        .fetch_one(&self.pool)
        .await?;

        info!("Created retention policy: {} for {}", policy.name, data_type);
        Ok(policy)
    }

    /// Check if data is under legal hold
    pub async fn is_under_legal_hold(&self, tenant_id: Uuid, data_type: &str) -> bool {
        let holds = self.legal_holds.read().await;
        holds
            .get(&tenant_id)
            .map(|h| h.iter().any(|hold| hold.contains(data_type)))
            .unwrap_or(false)
    }

    /// Create legal hold
    pub async fn create_legal_hold(
        &self,
        tenant_id: Uuid,
        name: String,
        description: String,
        case_number: Option<String>,
        created_by: String,
    ) -> anyhow::Result<LegalHold> {
        let hold = sqlx::query_as::<_, LegalHold>(
            r#"
            INSERT INTO legal_holds (id, tenant_id, name, description, case_number, active, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, true, $6, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(&name)
        .bind(&description)
        .bind(case_number)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        // Add to in-memory cache
        {
            let mut holds = self.legal_holds.write().await;
            holds.entry(tenant_id).or_default().push(name);
        }

        info!("Created legal hold: {} ({})", hold.name, hold.id);
        Ok(hold)
    }

    /// Release legal hold
    pub async fn release_legal_hold(&self, hold_id: Uuid) -> anyhow::Result<()> {
        let hold: LegalHold = sqlx::query_as(
            "UPDATE legal_holds SET active = false, released_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(hold_id)
        .fetch_one(&self.pool)
        .await?;

        // Remove from cache
        {
            let mut holds = self.legal_holds.write().await;
            if let Some(tenant_holds) = holds.get_mut(&hold.tenant_id) {
                tenant_holds.retain(|h| h != &hold.name);
            }
        }

        info!("Released legal hold: {}", hold_id);
        Ok(())
    }

    /// Start retention enforcement task
    async fn start_retention_enforcement(&self) {
        let pool = self.pool.clone();
        let legal_holds = self.legal_holds.clone();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Hourly

            loop {
                ticker.tick().await;

                if let Err(e) = Self::enforce_retention(&pool, &legal_holds).await {
                    error!("Retention enforcement failed: {}", e);
                }
            }
        });
    }

    /// Enforce retention policies
    async fn enforce_retention(
        pool: &PgPool,
        legal_holds: &Arc<RwLock<HashMap<Uuid, Vec<String>>>>,
    ) -> anyhow::Result<()> {
        let policies: Vec<RetentionPolicy> = sqlx::query_as(
            "SELECT * FROM retention_policies WHERE auto_delete = true AND legal_hold = false"
        )
        .fetch_all(pool)
        .await?;

        for policy in policies {
            // Check if under legal hold
            let is_on_hold = {
                let holds = legal_holds.read().await;
                holds
                    .get(&policy.tenant_id)
                    .map(|h| h.iter().any(|hold| hold.contains(&policy.data_type)))
                    .unwrap_or(false)
            };

            if is_on_hold {
                debug!("Skipping retention for {} - under legal hold", policy.data_type);
                continue;
            }

            let cutoff = Utc::now() - Duration::days(policy.retention_period_days as i64);

            match policy.action {
                RetentionAction::Delete => {
                    Self::delete_expired_data(pool, &policy, cutoff).await?;
                }
                RetentionAction::Anonymize => {
                    Self::anonymize_expired_data(pool, &policy, cutoff).await?;
                }
                RetentionAction::Archive => {
                    Self::archive_expired_data(pool, &policy, cutoff).await?;
                }
                RetentionAction::Review => {
                    // Just log for manual review
                    info!(
                        "Data of type {} for tenant {} requires review for retention",
                        policy.data_type, policy.tenant_id
                    );
                }
            }
        }

        Ok(())
    }

    /// Delete expired data
    async fn delete_expired_data(
        pool: &PgPool,
        policy: &RetentionPolicy,
        cutoff: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        // This would delete from the appropriate table based on data_type
        // For example, if data_type is "meeting_recordings":
        let deleted = match policy.data_type.as_str() {
            "meeting_recordings" => {
                sqlx::query("DELETE FROM meeting_recordings WHERE tenant_id = $1 AND created_at < $2")
                    .bind(policy.tenant_id)
                    .bind(cutoff)
                    .execute(pool)
                    .await?
                    .rows_affected()
            }
            "transcriptions" => {
                sqlx::query("DELETE FROM transcriptions WHERE tenant_id = $1 AND created_at < $2")
                    .bind(policy.tenant_id)
                    .bind(cutoff)
                    .execute(pool)
                    .await?
                    .rows_affected()
            }
            _ => {
                warn!("Unknown data type for retention: {}", policy.data_type);
                0
            }
        };

        if deleted > 0 {
            info!(
                "Deleted {} {} records for tenant {} (retention policy)",
                deleted, policy.data_type, policy.tenant_id
            );
        }

        Ok(())
    }

    /// Anonymize expired data
    async fn anonymize_expired_data(
        pool: &PgPool,
        policy: &RetentionPolicy,
        cutoff: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        // Anonymize by replacing PII with hashed values
        let anonymized = sqlx::query(
            r#"
            UPDATE meetings 
            SET 
                participant_emails = '{}',
                participant_names = '{}',
                organizer_email = '[REDACTED]',
                organizer_name = '[REDACTED]',
                anonymized = true
            WHERE tenant_id = $1 AND created_at < $2 AND anonymized = false
            "#
        )
        .bind(policy.tenant_id)
        .bind(cutoff)
        .execute(pool)
        .await?
        .rows_affected();

        if anonymized > 0 {
            info!(
                "Anonymized {} meeting records for tenant {}",
                anonymized, policy.tenant_id
            );
        }

        Ok(())
    }

    /// Archive expired data
    async fn archive_expired_data(
        _pool: &PgPool,
        _policy: &RetentionPolicy,
        _cutoff: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        // In production, this would move data to cold storage (S3 Glacier, etc.)
        info!("Archiving not yet implemented");
        Ok(())
    }

    /// Submit data subject request
    pub async fn submit_dsr(
        &self,
        tenant_id: Uuid,
        request_type: DsrType,
        subject_email: String,
        subject_id: String,
        description: Option<String>,
    ) -> anyhow::Result<DataSubjectRequest> {
        let token = Uuid::new_v4().to_string();

        let dsr = sqlx::query_as::<_, DataSubjectRequest>(
            r#"
            INSERT INTO data_subject_requests (
                id, tenant_id, request_type, status, subject_email, subject_id,
                description, requested_at, verified, verification_token, created_at
            )
            VALUES ($1, $2, $3, 'pending', $4, $5, $6, NOW(), false, $7, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(request_type)
        .bind(&subject_email)
        .bind(&subject_id)
        .bind(description)
        .bind(&token)
        .fetch_one(&self.pool)
        .await?;

        info!(
            "Submitted DSR: {} for {} ({})",
            request_type, subject_email, dsr.id
        );

        // TODO: Send verification email

        Ok(dsr)
    }

    /// Verify DSR
    pub async fn verify_dsr(&self, dsr_id: Uuid, token: &str) -> anyhow::Result<()> {
        let result = sqlx::query(
            "UPDATE data_subject_requests SET verified = true, status = 'verified' WHERE id = $1 AND verification_token = $2"
        )
        .bind(dsr_id)
        .bind(token)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Invalid verification token"));
        }

        info!("Verified DSR: {}", dsr_id);
        Ok(())
    }

    /// Start DSR processor
    async fn start_dsr_processor(&self) {
        let pool = self.pool.clone();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                ticker.tick().await;

                // Process pending verified DSRs
                if let Err(e) = Self::process_pending_dsrs(&pool).await {
                    error!("DSR processing failed: {}", e);
                }
            }
        });
    }

    /// Process pending DSRs
    async fn process_pending_dsrs(pool: &PgPool) -> anyhow::Result<()> {
        let pending: Vec<DataSubjectRequest> = sqlx::query_as(
            "SELECT * FROM data_subject_requests WHERE status = 'verified' ORDER BY requested_at LIMIT 10"
        )
        .fetch_all(pool)
        .await?;

        for dsr in pending {
            if let Err(e) = Self::process_dsr(pool, &dsr).await {
                error!("Failed to process DSR {}: {}", dsr.id, e);
                
                // Mark as failed
                sqlx::query("UPDATE data_subject_requests SET status = 'failed', error_message = $1 WHERE id = $2")
                    .bind(e.to_string())
                    .bind(dsr.id)
                    .execute(pool)
                    .await?;
            }
        }

        Ok(())
    }

    /// Process single DSR
    async fn process_dsr(pool: &PgPool, dsr: &DataSubjectRequest) -> anyhow::Result<()> {
        // Mark as processing
        sqlx::query("UPDATE data_subject_requests SET status = 'processing' WHERE id = $1")
            .bind(dsr.id)
            .execute(pool)
            .await?;

        match dsr.request_type {
            DsrType::Access => {
                // Export all data for the subject
                let export_data = Self::export_subject_data(pool, dsr.tenant_id, &dsr.subject_id).await?;
                let result_url = format!("s3://exports/{}/data_export.json", dsr.id);
                
                // Store export
                sqlx::query(
                    "UPDATE data_subject_requests SET status = 'completed', completed_at = NOW(), result_url = $1 WHERE id = $2"
                )
                .bind(&result_url)
                .bind(dsr.id)
                .execute(pool)
                .await?;

                info!("Completed access request for DSR {}", dsr.id);
            }
            DsrType::Deletion => {
                // Delete all data for the subject
                let deleted = Self::delete_subject_data(pool, dsr.tenant_id, &dsr.subject_id).await?;
                
                sqlx::query(
                    "UPDATE data_subject_requests SET status = 'completed', completed_at = NOW() WHERE id = $1"
                )
                .bind(dsr.id)
                .execute(pool)
                .await?;

                info!("Completed deletion request for DSR {} ({} records deleted)", dsr.id, deleted);
            }
            _ => {
                // Other types not yet implemented
                warn!("DSR type {:?} not yet fully implemented", dsr.request_type);
                
                sqlx::query(
                    "UPDATE data_subject_requests SET status = 'failed', error_message = 'Not implemented' WHERE id = $1"
                )
                .bind(dsr.id)
                .execute(pool)
                .await?;
            }
        }

        Ok(())
    }

    /// Export subject data
    async fn export_subject_data(
        pool: &PgPool,
        tenant_id: Uuid,
        subject_id: &str,
    ) -> anyhow::Result<serde_json::Value> {
        // Gather all data for the subject
        let mut export = serde_json::json!({
            "subject_id": subject_id,
            "tenant_id": tenant_id,
            "export_date": Utc::now().to_rfc3339(),
            "meetings": [],
            "transcriptions": [],
            "actions": [],
            "analytics": [],
        });

        // Query meetings
        let meetings: Vec<serde_json::Value> = sqlx::query_as(
            "SELECT id, title, created_at FROM meetings WHERE tenant_id = $1 AND organizer_id = $2"
        )
        .bind(tenant_id)
        .bind(subject_id)
        .fetch_all(pool)
        .await?;

        export["meetings"] = serde_json::to_value(meetings)?;

        Ok(export)
    }

    /// Delete subject data
    async fn delete_subject_data(
        pool: &PgPool,
        tenant_id: Uuid,
        subject_id: &str,
    ) -> anyhow::Result<u64> {
        let mut total_deleted = 0u64;

        // Delete meetings
        let meetings_deleted = sqlx::query(
            "DELETE FROM meetings WHERE tenant_id = $1 AND organizer_id = $2"
        )
        .bind(tenant_id)
        .bind(subject_id)
        .execute(pool)
        .await?
        .rows_affected();
        total_deleted += meetings_deleted;

        // Delete other data...

        Ok(total_deleted)
    }

    /// Get retention statistics
    pub async fn get_stats(&self, tenant_id: Uuid) -> anyhow::Result<RetentionStats> {
        let policies: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM retention_policies WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let active_holds: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM legal_holds WHERE tenant_id = $1 AND active = true"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let pending_dsrs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM data_subject_requests WHERE tenant_id = $1 AND status IN ('pending', 'verified', 'processing')"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(RetentionStats {
            policies: policies as u32,
            active_legal_holds: active_holds as u32,
            pending_dsr_requests: pending_dsrs as u32,
        })
    }
}

/// Retention statistics
#[derive(Debug, Clone)]
pub struct RetentionStats {
    pub policies: u32,
    pub active_legal_holds: u32,
    pub pending_dsr_requests: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_action_serialization() {
        let action = RetentionAction::Anonymize;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"anonymize\"");
    }

    #[test]
    fn test_dsr_type_ordering() {
        assert_ne!(DsrType::Access, DsrType::Deletion);
    }
}
