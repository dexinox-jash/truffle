//! Data Residency and Geo-Fencing
//!
//! Controls data storage location for GDPR, CCPA, and other privacy regulations.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::net::IpAddr;
use uuid::Uuid;

/// Supported data regions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DataRegion {
    /// European Union (GDPR)
    EuWest,
    /// Germany (strict data protection)
    De,
    /// United States
    UsEast,
    UsWest,
    /// United Kingdom
    Uk,
    /// Canada
    Ca,
    /// Australia
    Au,
    /// Singapore
    Sg,
    /// Japan
    Jp,
    /// India
    In,
    /// Brazil (LGPD)
    Br,
    /// Custom/special region
    Custom(String),
}

impl DataRegion {
    /// Get region from country code
    pub fn from_country_code(code: &str) -> Self {
        match code.to_uppercase().as_str() {
            "AT" | "BE" | "BG" | "HR" | "CY" | "CZ" | "DK" | "EE" | "FI" | "FR" |
            "DE" | "GR" | "HU" | "IE" | "IT" | "LV" | "LT" | "LU" | "MT" | "NL" |
            "PL" | "PT" | "RO" | "SK" | "SI" | "ES" | "SE" => Self::EuWest,
            "DE" => Self::De,
            "US" => Self::UsEast,
            "GB" | "UK" => Self::Uk,
            "CA" => Self::Ca,
            "AU" => Self::Au,
            "SG" => Self::Sg,
            "JP" => Self::Jp,
            "IN" => Self::In,
            "BR" => Self::Br,
            other => Self::Custom(other.to_string()),
        }
    }

    /// Check if region is GDPR-covered
    pub fn is_gdpr(&self) -> bool {
        matches!(self, Self::EuWest | Self::De | Self::Uk)
    }

    /// Get compliance requirements for region
    pub fn compliance_requirements(&self) -> Vec<ComplianceRequirement> {
        let mut requirements = vec![];

        if self.is_gdpr() {
            requirements.extend(vec![
                ComplianceRequirement::GdprArticle17,  // Right to erasure
                ComplianceRequirement::GdprArticle18,  // Right to restriction
                ComplianceRequirement::GdprArticle20,  // Data portability
                ComplianceRequirement::DpoRequired,
                ComplianceRequirement::DpaRequired,
            ]);
        }

        match self {
            Self::De => {
                requirements.push(ComplianceRequirement::BsiC5);
            }
            Self::UsEast | Self::UsWest => {
                requirements.push(ComplianceRequirement::Ccpa);
                requirements.push(ComplianceRequirement::Hipaa);
            }
            Self::Ca => {
                requirements.push(ComplianceRequirement::Pipeda);
            }
            Self::Br => {
                requirements.push(ComplianceRequirement::Lgpd);
            }
            _ => {}
        }

        requirements
    }
}

/// Compliance requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplianceRequirement {
    GdprArticle17,
    GdprArticle18,
    GdprArticle20,
    DpoRequired,
    DpaRequired,
    BsiC5,
    Ccpa,
    Hipaa,
    Pipeda,
    Lgpd,
}

/// Data residency policy
#[derive(Debug, Clone, FromRow)]
pub struct ResidencyPolicy {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub primary_region: DataRegion,
    pub backup_regions: Vec<String>,
    pub allow_cross_region_transfer: bool,
    pub require_encryption_at_rest: bool,
    pub require_encryption_in_transit: bool,
    pub data_retention_days: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Geo-fencing rule
#[derive(Debug, Clone, FromRow)]
pub struct GeoFence {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub allowed_countries: Vec<String>,
    pub allowed_regions: Vec<String>,
    pub blocked_countries: Vec<String>,
    pub require_vpn: bool,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

/// Location from IP geolocation
#[derive(Debug, Clone)]
pub struct GeoLocation {
    pub country_code: String,
    pub country_name: String,
    pub region: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub eu_member: bool,
}

/// Data residency manager
pub struct ResidencyManager {
    pool: PgPool,
    geoip_db: Arc<dyn GeoIpDatabase>,
}

use std::sync::Arc;

impl ResidencyManager {
    /// Create new residency manager
    pub fn new(pool: PgPool, geoip_db: Arc<dyn GeoIpDatabase>) -> Self {
        Self { pool, geoip_db }
    }

    /// Set residency policy for tenant
    pub async fn set_policy(
        &self,
        tenant_id: Uuid,
        primary_region: DataRegion,
        backup_regions: Vec<DataRegion>,
        allow_cross_region: bool,
    ) -> anyhow::Result<ResidencyPolicy> {
        let backup_regions_str: Vec<String> = backup_regions
            .iter()
            .map(|r| format!("{:?}", r).to_lowercase())
            .collect();

        let policy = sqlx::query_as::<_, ResidencyPolicy>(
            r#"
            INSERT INTO data_residency_policies (
                id, tenant_id, primary_region, backup_regions, allow_cross_region_transfer,
                require_encryption_at_rest, require_encryption_in_transit, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, true, true, NOW(), NOW())
            ON CONFLICT (tenant_id) DO UPDATE SET
                primary_region = EXCLUDED.primary_region,
                backup_regions = EXCLUDED.backup_regions,
                allow_cross_region_transfer = EXCLUDED.allow_cross_region_transfer,
                updated_at = NOW()
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(primary_region)
        .bind(&backup_regions_str)
        .bind(allow_cross_region)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!(
            "Set data residency policy for tenant {}: primary={:?}",
            tenant_id, primary_region
        );

        Ok(policy)
    }

    /// Get residency policy for tenant
    pub async fn get_policy(&self, tenant_id: Uuid) -> anyhow::Result<Option<ResidencyPolicy>> {
        let policy = sqlx::query_as::<_, ResidencyPolicy>(
            "SELECT * FROM data_residency_policies WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(policy)
    }

    /// Check if data transfer is allowed between regions
    pub async fn is_transfer_allowed(
        &self,
        tenant_id: Uuid,
        from: DataRegion,
        to: DataRegion,
    ) -> anyhow::Result<bool> {
        // Same region always allowed
        if from == to {
            return Ok(true);
        }

        let policy = self.get_policy(tenant_id).await?;
        
        if let Some(policy) = policy {
            if !policy.allow_cross_region_transfer {
                return Ok(false);
            }

            // Check backup regions
            let backup: Vec<String> = serde_json::from_value(
                serde_json::to_value(&policy.backup_regions)?
            )?;
            
            let to_str = format!("{:?}", to).to_lowercase();
            if backup.contains(&to_str) {
                return Ok(true);
            }

            // GDPR to non-GDPR transfer restrictions
            if from.is_gdpr() && !to.is_gdpr() {
                // Would need additional safeguards (SCCs, adequacy decision, etc.)
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get storage endpoint for tenant data
    pub async fn get_storage_endpoint(
        &self,
        tenant_id: Uuid,
        data_type: DataType,
    ) -> anyhow::Result<String> {
        let policy = self.get_policy(tenant_id).await?;
        
        let region = policy
            .as_ref()
            .map(|p| p.primary_region.clone())
            .unwrap_or(DataRegion::UsEast);

        let endpoint = match region {
            DataRegion::EuWest => "https://s3.eu-west-1.amazonaws.com",
            DataRegion::De => "https://s3.eu-central-1.amazonaws.com",
            DataRegion::UsEast => "https://s3.us-east-1.amazonaws.com",
            DataRegion::UsWest => "https://s3.us-west-2.amazonaws.com",
            DataRegion::Uk => "https://s3.eu-west-2.amazonaws.com",
            DataRegion::Ca => "https://s3.ca-central-1.amazonaws.com",
            DataRegion::Au => "https://s3.ap-southeast-2.amazonaws.com",
            DataRegion::Sg => "https://s3.ap-southeast-1.amazonaws.com",
            DataRegion::Jp => "https://s3.ap-northeast-1.amazonaws.com",
            DataRegion::In => "https://s3.ap-south-1.amazonaws.com",
            DataRegion::Br => "https://s3.sa-east-1.amazonaws.com",
            DataRegion::Custom(ref r) => &format!("https://s3.{}.amazonaws.com", r.to_lowercase()),
        };

        Ok(format!("{}/ruflo-{}-{}", endpoint, region_str(&region), data_type_str(&data_type)))
    }

    /// Create geo-fence for tenant
    pub async fn create_geo_fence(
        &self,
        tenant_id: Uuid,
        name: String,
        allowed_countries: Vec<String>,
        blocked_countries: Vec<String>,
        require_vpn: bool,
    ) -> anyhow::Result<GeoFence> {
        let fence = sqlx::query_as::<_, GeoFence>(
            r#"
            INSERT INTO geo_fences (
                id, tenant_id, name, allowed_countries, blocked_countries, require_vpn, active, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, true, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(name)
        .bind(&allowed_countries)
        .bind(&blocked_countries)
        .bind(require_vpn)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Created geo-fence {} for tenant {}", fence.id, tenant_id);
        Ok(fence)
    }

    /// Check if IP address is allowed by geo-fence
    pub async fn check_geo_access(
        &self,
        tenant_id: Uuid,
        ip: IpAddr,
    ) -> anyhow::Result<GeoAccessResult> {
        // Get location from IP
        let location = self.geoip_db.lookup(ip).await?;

        // Get active geo-fences
        let fences: Vec<GeoFence> = sqlx::query_as(
            "SELECT * FROM geo_fences WHERE tenant_id = $1 AND active = true"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        for fence in fences {
            // Check blocked countries
            if fence.blocked_countries.contains(&location.country_code) {
                return Ok(GeoAccessResult::Denied {
                    reason: format!("Country {} is blocked", location.country_code),
                    requires_vpn: fence.require_vpn,
                });
            }

            // Check allowed countries (if specified)
            if !fence.allowed_countries.is_empty() 
                && !fence.allowed_countries.contains(&location.country_code) {
                return Ok(GeoAccessResult::Denied {
                    reason: format!("Country {} not in allowed list", location.country_code),
                    requires_vpn: fence.require_vpn,
                });
            }
        }

        Ok(GeoAccessResult::Allowed {
            country: location.country_code,
            region: DataRegion::from_country_code(&location.country_code),
        })
    }

    /// Get compliance status for tenant
    pub async fn get_compliance_status(
        &self,
        tenant_id: Uuid,
    ) -> anyhow::Result<ComplianceStatus> {
        let policy = self.get_policy(tenant_id).await?;
        
        let mut checks = vec![];

        if let Some(policy) = policy {
            let requirements = policy.primary_region.compliance_requirements();

            for req in requirements {
                let passed = self.check_compliance_requirement(tenant_id, &req).await?;
                checks.push(ComplianceCheck {
                    requirement: req,
                    passed,
                    details: None,
                });
            }
        }

        let all_passed = checks.iter().all(|c| c.passed);

        Ok(ComplianceStatus {
            tenant_id,
            overall_compliant: all_passed,
            checks,
            last_checked: Utc::now(),
        })
    }

    async fn check_compliance_requirement(
        &self,
        tenant_id: Uuid,
        req: &ComplianceRequirement,
    ) -> anyhow::Result<bool> {
        match req {
            ComplianceRequirement::GdprArticle17 => {
                // Check if erasure mechanism is configured
                let count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM retention_policies WHERE tenant_id = $1"
                )
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await?;
                Ok(count > 0)
            }
            ComplianceRequirement::GdprArticle20 => {
                // Check if data export is enabled
                let count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM compliance_settings WHERE tenant_id = $1 AND data_portability = true"
                )
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await?;
                Ok(count > 0)
            }
            ComplianceRequirement::DpaRequired => {
                // Check if DPA is signed
                let signed: Option<bool> = sqlx::query_scalar(
                    "SELECT dpa_signed FROM compliance_settings WHERE tenant_id = $1"
                )
                .bind(tenant_id)
                .fetch_optional(&self.pool)
                .await?;
                Ok(signed.unwrap_or(false))
            }
            _ => Ok(true), // Assume compliant for other requirements
        }
    }

    /// Initiate data migration to new region
    pub async fn initiate_migration(
        &self,
        tenant_id: Uuid,
        target_region: DataRegion,
    ) -> anyhow::Result<MigrationJob> {
        // Check if migration is allowed
        let policy = self.get_policy(tenant_id).await?;
        let current_region = policy.map(|p| p.primary_region).unwrap_or(DataRegion::UsEast);

        if !self.is_transfer_allowed(tenant_id, current_region.clone(), target_region.clone()).await? {
            return Err(anyhow::anyhow!(
                "Data transfer from {:?} to {:?} not allowed by residency policy",
                current_region, target_region
            ));
        }

        // Create migration job
        let job = sqlx::query_as::<_, MigrationJob>(
            r#"
            INSERT INTO data_migration_jobs (id, tenant_id, source_region, target_region, status, started_at)
            VALUES ($1, $2, $3, $4, 'pending', NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(current_region)
        .bind(target_region)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Initiated data migration for tenant {}: {:?}", tenant_id, job);
        Ok(job)
    }
}

/// Data type for storage routing
#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Meetings,
    Recordings,
    Transcripts,
    Analytics,
    UserData,
    AuditLogs,
}

fn data_type_str(dt: &DataType) -> &'static str {
    match dt {
        DataType::Meetings => "meetings",
        DataType::Recordings => "recordings",
        DataType::Transcripts => "transcripts",
        DataType::Analytics => "analytics",
        DataType::UserData => "users",
        DataType::AuditLogs => "audit",
    }
}

fn region_str(r: &DataRegion) -> String {
    format!("{:?}", r).to_lowercase()
}

/// Geo-IP database trait
#[async_trait::async_trait]
pub trait GeoIpDatabase: Send + Sync {
    async fn lookup(&self, ip: IpAddr) -> anyhow::Result<GeoLocation>;
}

/// MaxMind GeoIP implementation
pub struct MaxMindGeoIp {
    reader: maxminddb::Reader<Vec<u8>>,
}

impl MaxMindGeoIp {
    pub fn new(db_path: &str) -> anyhow::Result<Self> {
        let reader = maxminddb::Reader::open_readfile(db_path)?;
        Ok(Self { reader })
    }
}

#[async_trait::async_trait]
impl GeoIpDatabase for MaxMindGeoIp {
    async fn lookup(&self, ip: IpAddr) -> anyhow::Result<GeoLocation> {
        let city: maxminddb::geoip2::City = self.reader.lookup(ip)?;
        
        let country = city.country.as_ref();
        let subdivision = city.subdivisions.as_ref().and_then(|s| s.first());
        let city_name = city.city.as_ref().and_then(|c| c.names.as_ref())
            .and_then(|n| n.get("en").copied());
        let location = city.location.as_ref();

        Ok(GeoLocation {
            country_code: country
                .and_then(|c| c.iso_code)
                .unwrap_or("XX")
                .to_string(),
            country_name: country
                .and_then(|c| c.names.as_ref())
                .and_then(|n| n.get("en").copied())
                .unwrap_or("Unknown")
                .to_string(),
            region: subdivision
                .and_then(|s| s.iso_code)
                .unwrap_or("")
                .to_string(),
            city: city_name.unwrap_or("Unknown").to_string(),
            latitude: location.map(|l| l.latitude.unwrap_or(0.0)).unwrap_or(0.0),
            longitude: location.map(|l| l.longitude.unwrap_or(0.0)).unwrap_or(0.0),
            timezone: location
                .and_then(|l| l.time_zone)
                .unwrap_or("UTC")
                .to_string(),
            eu_member: country
                .map(|c| c.is_in_european_union.unwrap_or(false))
                .unwrap_or(false),
        })
    }
}

/// Geo access check result
#[derive(Debug, Clone)]
pub enum GeoAccessResult {
    Allowed { country: String, region: DataRegion },
    Denied { reason: String, requires_vpn: bool },
}

/// Compliance check
#[derive(Debug, Clone)]
pub struct ComplianceCheck {
    pub requirement: ComplianceRequirement,
    pub passed: bool,
    pub details: Option<String>,
}

/// Compliance status
#[derive(Debug, Clone)]
pub struct ComplianceStatus {
    pub tenant_id: Uuid,
    pub overall_compliant: bool,
    pub checks: Vec<ComplianceCheck>,
    pub last_checked: DateTime<Utc>,
}

/// Migration job
#[derive(Debug, Clone, FromRow)]
pub struct MigrationJob {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub source_region: DataRegion,
    pub target_region: DataRegion,
    pub status: MigrationStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Database migrations for residency
pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS data_residency_policies (
            id UUID PRIMARY KEY,
            tenant_id UUID NOT NULL UNIQUE,
            primary_region VARCHAR(50) NOT NULL,
            backup_regions TEXT[] NOT NULL DEFAULT '{}',
            allow_cross_region_transfer BOOLEAN NOT NULL DEFAULT false,
            require_encryption_at_rest BOOLEAN NOT NULL DEFAULT true,
            require_encryption_in_transit BOOLEAN NOT NULL DEFAULT true,
            data_retention_days INTEGER,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_residency_tenant ON data_residency_policies(tenant_id);

        CREATE TABLE IF NOT EXISTS geo_fences (
            id UUID PRIMARY KEY,
            tenant_id UUID NOT NULL,
            name VARCHAR(255) NOT NULL,
            allowed_countries TEXT[] NOT NULL DEFAULT '{}',
            blocked_countries TEXT[] NOT NULL DEFAULT '{}',
            require_vpn BOOLEAN NOT NULL DEFAULT false,
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_geofence_tenant ON geo_fences(tenant_id);

        CREATE TABLE IF NOT EXISTS data_migration_jobs (
            id UUID PRIMARY KEY,
            tenant_id UUID NOT NULL,
            source_region VARCHAR(50) NOT NULL,
            target_region VARCHAR(50) NOT NULL,
            status VARCHAR(50) NOT NULL,
            started_at TIMESTAMPTZ NOT NULL,
            completed_at TIMESTAMPTZ
        );

        CREATE INDEX IF NOT EXISTS idx_migration_tenant ON data_migration_jobs(tenant_id);
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}
