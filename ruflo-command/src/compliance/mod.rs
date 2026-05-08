//! Compliance Module
//!
//! GDPR, SOC 2, and data privacy compliance features.

pub mod retention;

pub use retention::{
    RetentionManager, RetentionPolicy, RetentionAction,
    DataSubjectRequest, DsrType, LegalHold, RetentionStats,
};
