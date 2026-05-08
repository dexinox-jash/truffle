//! Audit Logging System
//!
//! SOC 2 and GDPR compliant audit trail with tamper-evident hashing.

pub mod logger;

pub use logger::{
    AuditLogger, AuditConfig, AuditEvent, AuditContext, 
    AuditCategory, AuditSeverity, AuditFilters, ExportFormat,
    IntegrityReport, middleware,
};
