//! Validation & Contradiction Engine
//!
//! Ensures knowledge graph integrity through automated fact checking,
//! confidence scoring, and redundancy detection.
//!
//! ## Components
//!
//! - **Contradiction Checker**: Detects temporal and factual contradictions
//! - **Confidence Scorer**: Bayesian confidence updating with source reliability
//! - **Redundancy Detector**: Finds and resolves duplicate entities
//!
//! ## Example
//!
//! ```rust,no_run
//! use truffle_core::validation::{
//!     ValidationEngine, ValidationConfig, ValidationSummary,
//! };
//!
//! async fn validate_knowledge_graph() {
//!     let config = ValidationConfig::default();
//!     // let engine = ValidationEngine::new(config, repository);
//!     // let summary = engine.validate_all().await.unwrap();
//!     // println!("Validation status: {}", summary.status_text());
//! }
//! ```

pub mod config;
pub mod engine;
pub mod models;

pub mod confidence;
pub mod contradiction_checker;
pub mod redundancy;

// Re-export main types
pub use config::{ConfidenceConfig, ContradictionConfig, RedundancyConfig, ValidationConfig};
pub use engine::{EntityValidationResult, ValidationEngine, ValidationSummary};

pub use contradiction_checker::{ContradictionChecker, ValidationReport};

pub use models::{
    ConfidenceFactor, ConfidenceScore, ConflictResolution, Contradiction, ContradictionSeverity,
    ContradictionType, DuplicateCandidate, Fact, FactSource, FactValue, ResolutionType,
};
