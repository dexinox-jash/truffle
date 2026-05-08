//! Geographic Data Management
//!
//! Data residency, geo-fencing, and regional compliance.

pub mod residency;

pub use residency::{
    DataRegion, ResidencyPolicy, ResidencyManager, GeoFence,
    DataType, GeoAccessResult, ComplianceStatus, ComplianceRequirement,
};
