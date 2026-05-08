//! Compliance Module
//!
//! This module implements compliance features for:
//! - GDPR Article 32 (encryption, pseudonymization)
//! - Immutable audit logs with Merkle tree
//! - 5-minute complete export capability

mod audit;
mod export;
mod gdpr;

pub use audit::{
    AuditEntry, AuditLog, AuditLogConfig, MerkleAuditLog, MerkleNode,
    TamperDetectionResult, create_audit_log, verify_audit_log,
};
pub use export::{
    DataExporter, ExportConfig, ExportFormat, ExportProgress, ExportResult,
    MarkdownExporter, ObsidianExporter,
};
pub use gdpr::{
    DataSubjectRequest, DataSubjectRights, GdprCompliance, GdprConfig,
    PrivacyImpactAssessment, RequestType, data_deletion, data_portability,
};

use crate::error::CryptoResult;

/// Initialize the compliance module
pub fn init() -> CryptoResult<()> {
    Ok(())
}

/// Compliance level
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComplianceLevel {
    /// Basic compliance (minimal requirements)
    Basic,
    /// Standard compliance (GDPR, CCPA)
    Standard,
    /// Enterprise compliance (SOC 2, ISO 27001)
    Enterprise,
}

impl ComplianceLevel {
    /// Check if audit logging is required
    pub fn requires_audit_logging(&self) -> bool {
        matches!(self, Self::Standard | Self::Enterprise)
    }

    /// Check if data export is required
    pub fn requires_data_export(&self) -> bool {
        matches!(self, Self::Standard | Self::Enterprise)
    }

    /// Check if GDPR compliance is required
    pub fn requires_gdpr(&self) -> bool {
        matches!(self, Self::Standard | Self::Enterprise)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_level() {
        assert!(ComplianceLevel::Standard.requires_audit_logging());
        assert!(ComplianceLevel::Enterprise.requires_audit_logging());
        assert!(!ComplianceLevel::Basic.requires_audit_logging());

        assert!(ComplianceLevel::Standard.requires_data_export());
        assert!(ComplianceLevel::Enterprise.requires_gdpr());
    }
}
