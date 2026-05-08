//! Ruflo Command Service
//!
//! Core command and control service for the Ruflo AI meeting platform.
//! Provides compliance, security, audit, and enterprise integration features.

pub mod audit;
pub mod compliance;
pub mod encryption;
pub mod geo;
pub mod sso;

// Re-export main types
pub use audit::{AuditLogger, AuditEvent, AuditLevel, IntegrityReport};
pub use compliance::{
    RetentionManager, RetentionPolicy, DataType,
    GdprController, ErasureRequest, ErasureReport,
    Soc2Controls, TrustServiceCriteria, ControlStatus, AuditFinding,
};
pub use encryption::{FieldEncryption, EncryptedField, KeyManager};
pub use geo::{DataRegion, ResidencyManager, GeoAccessResult, ComplianceStatus};
pub use sso::{SamlManager, SamlIdentityProvider, SsoUser, ScimProvisioner};

/// Service version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize all compliance modules
pub async fn initialize(pool: sqlx::PgPool, vault_url: &str) -> anyhow::Result<ServiceContext> {
    // Initialize field-level encryption with Vault
    let encryption = encryption::FieldEncryption::new(vault_url).await?;
    
    // Initialize audit logger
    let audit = audit::AuditLogger::new(pool.clone()).await?;
    
    // Initialize retention manager
    let retention = compliance::RetentionManager::new(pool.clone());
    
    tracing::info!("Ruflo Command Service v{} initialized", VERSION);
    
    Ok(ServiceContext {
        pool,
        encryption,
        audit,
        retention,
    })
}

/// Service context with all initialized components
pub struct ServiceContext {
    pub pool: sqlx::PgPool,
    pub encryption: encryption::FieldEncryption,
    pub audit: audit::AuditLogger,
    pub retention: compliance::RetentionManager,
}

impl ServiceContext {
    /// Create GDPR controller
    pub fn gdpr_controller(&self) -> compliance::GdprController {
        compliance::GdprController::new(
            self.pool.clone(),
            self.audit.clone(),
            self.retention.clone(),
        )
    }
    
    /// Create SOC 2 controls
    pub fn soc2_controls(&self) -> compliance::Soc2Controls {
        compliance::Soc2Controls::new(self.pool.clone(), self.audit.clone())
    }
    
    /// Create residency manager
    pub fn residency_manager(
        &self,
        geoip_db: std::sync::Arc<dyn geo::residency::GeoIpDatabase>,
    ) -> geo::ResidencyManager {
        geo::ResidencyManager::new(self.pool.clone(), geoip_db)
    }
    
    /// Create SAML manager
    pub fn saml_manager(&self) -> sso::SamlManager {
        sso::SamlManager::new(self.pool.clone())
    }
}
