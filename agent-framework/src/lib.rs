//! # Ruflo Agent Framework
//!
//! The agent orchestration system for the Ruflo AI Meeting Notes Platform.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    AGENT FRAMEWORK                               │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Master Agents      │  Domain Agents      │  Feature Agents      │
//! │  ─────────────      │  ────────────       │  ─────────────       │
//! │  MA-001 Orchestrator│  DA-001 Backend     │  FA-001 Audio        │
//! │  MA-002 Security    │  DA-002 Frontend    │  FA-002 ASR          │
//! │  MA-003 QA          │  DA-003 AI/ML       │  FA-003 Analysis     │
//! │  MA-004 Learning    │  DA-004 Infrastructure│  FA-004 Sync        │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

pub mod agent;
pub mod communication;
pub mod decision;
pub mod monitoring;

pub use agent::{Agent, AgentId, AgentRole, AgentState};
pub use communication::{AgentClient, Message, MessageType};
pub use decision::{DecisionEngine, DecisionRecord, Vote};
pub use monitoring::{AgentMonitor, MetricsCollector};

/// Framework version
pub const FRAMEWORK_VERSION: &str = "0.1.0";

/// Default NATS URL
pub const DEFAULT_NATS_URL: &str = "nats://localhost:4222";

/// Default agent heartbeat interval (seconds)
pub const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Maximum decision time (seconds)
pub const MAX_DECISION_TIME_SECS: u64 = 300; // 5 minutes
