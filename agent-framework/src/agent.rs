//! Agent definition and management

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Unique agent identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub uuid::Uuid);

impl AgentId {
    /// Generate a new agent ID
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::generate()
    }
}

/// Agent role hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// Master level - strategic decisions
    Master(MasterRole),
    /// Domain level - architectural decisions
    Domain(DomainRole),
    /// Feature level - implementation decisions
    Feature(FeatureRole),
    /// Support level - operational tasks
    Support(SupportRole),
}

/// Master agent roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MasterRole {
    /// System Orchestrator - central coordination
    Orchestrator,
    /// Security Guardian - security enforcement
    SecurityGuardian,
    /// Quality Assurance - quality gates
    QualityAssurance,
    /// Learning Master - pattern aggregation
    LearningMaster,
}

/// Domain agent roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainRole {
    /// Backend architecture
    Backend,
    /// Frontend architecture
    Frontend,
    /// AI/ML engineering
    AiMl,
    /// Infrastructure/DevOps
    Infrastructure,
    /// Security engineering
    Security,
    /// Data engineering
    Data,
    /// Testing/QA
    Testing,
    /// Documentation
    Documentation,
    /// Performance optimization
    Performance,
}

/// Feature agent roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureRole {
    /// Audio ingestion
    AudioIngestion,
    /// ASR pipeline
    AsrPipeline,
    /// Speaker diarization
    Diarization,
    /// Meeting analysis
    Analysis,
    /// Sync protocol
    Sync,
    /// Collaboration
    Collaboration,
}

/// Support agent roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportRole {
    /// Testing
    Testing,
    /// Monitoring
    Monitoring,
    /// Learning/optimization
    Learning,
    /// Pattern recognition
    Pattern,
    /// A/B testing
    AbTesting,
    /// Documentation
    Documentation,
}

/// Agent state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    /// Agent is idle
    Idle,
    /// Agent is working on a task
    Working,
    /// Agent is waiting for input
    Waiting,
    /// Agent encountered an error
    Error,
    /// Agent is offline
    Offline,
}

/// Agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Agent ID
    pub id: AgentId,
    /// Agent name
    pub name: String,
    /// Agent role
    pub role: AgentRole,
    /// Current state
    pub state: AgentState,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Current task (if any)
    pub current_task: Option<String>,
    /// Tasks completed
    pub tasks_completed: u64,
    /// Tasks failed
    pub tasks_failed: u64,
    /// Decisions made
    pub decisions_made: u64,
}

impl Agent {
    /// Create a new agent
    pub fn new(name: impl Into<String>, role: AgentRole) -> Self {
        Self {
            id: AgentId::generate(),
            name: name.into(),
            role,
            state: AgentState::Idle,
            capabilities: Vec::new(),
            current_task: None,
            tasks_completed: 0,
            tasks_failed: 0,
            decisions_made: 0,
        }
    }

    /// Add a capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Start a task
    pub fn start_task(&mut self, task: impl Into<String>) {
        self.current_task = Some(task.into());
        self.state = AgentState::Working;
    }

    /// Complete current task
    pub fn complete_task(&mut self) {
        self.current_task = None;
        self.state = AgentState::Idle;
        self.tasks_completed += 1;
    }

    /// Mark task as failed
    pub fn fail_task(&mut self) {
        self.current_task = None;
        self.state = AgentState::Error;
        self.tasks_failed += 1;
    }

    /// Record a decision
    pub fn record_decision(&mut self) {
        self.decisions_made += 1;
    }
}

/// Registry of all agents
pub struct AgentRegistry {
    agents: Vec<Agent>,
}

impl AgentRegistry {
    /// Create a new registry with all Ruflo agents
    pub fn create_ruflo_agents() -> Self {
        let agents = vec![
            // Master Agents (L1)
            Agent::new("MA-001 System Orchestrator", AgentRole::Master(MasterRole::Orchestrator))
                .with_capability("coordination")
                .with_capability("arbitration"),
            Agent::new("MA-002 Security Guardian", AgentRole::Master(MasterRole::SecurityGuardian))
                .with_capability("security_audit")
                .with_capability("threat_detection"),
            Agent::new("MA-003 Quality Assurance", AgentRole::Master(MasterRole::QualityAssurance))
                .with_capability("testing")
                .with_capability("code_review"),
            Agent::new("MA-004 Learning Master", AgentRole::Master(MasterRole::LearningMaster))
                .with_capability("pattern_recognition")
                .with_capability("optimization"),
            
            // Domain Agents (L2)
            Agent::new("DA-001 Backend Architect", AgentRole::Domain(DomainRole::Backend))
                .with_capability("rust")
                .with_capability("api_design"),
            Agent::new("DA-002 Frontend Architect", AgentRole::Domain(DomainRole::Frontend))
                .with_capability("react")
                .with_capability("typescript"),
            Agent::new("DA-003 AI/ML Engineer", AgentRole::Domain(DomainRole::AiMl))
                .with_capability("llm")
                .with_capability("prompts"),
            Agent::new("DA-004 Infrastructure", AgentRole::Domain(DomainRole::Infrastructure))
                .with_capability("kubernetes")
                .with_capability("nats"),
            Agent::new("DA-005 Security Engineer", AgentRole::Domain(DomainRole::Security))
                .with_capability("cryptography")
                .with_capability("audit"),
            Agent::new("DA-006 Data Engineer", AgentRole::Domain(DomainRole::Data))
                .with_capability("postgres")
                .with_capability("crdt"),
            
            // Feature Agents (L3)
            Agent::new("FA-001 Audio Ingestion", AgentRole::Feature(FeatureRole::AudioIngestion))
                .with_capability("audio_processing"),
            Agent::new("FA-002 ASR Pipeline", AgentRole::Feature(FeatureRole::AsrPipeline))
                .with_capability("whisper")
                .with_capability("transcription"),
            Agent::new("FA-003 Analysis", AgentRole::Feature(FeatureRole::Analysis))
                .with_capability("nlp")
                .with_capability("summarization"),
            Agent::new("FA-004 Sync", AgentRole::Feature(FeatureRole::Sync))
                .with_capability("p2p")
                .with_capability("encryption"),
            
            // Support Agents (L4)
            Agent::new("UA-001 Testing", AgentRole::Support(SupportRole::Testing))
                .with_capability("unit_tests")
                .with_capability("integration_tests"),
            Agent::new("UA-002 Monitoring", AgentRole::Support(SupportRole::Monitoring))
                .with_capability("metrics")
                .with_capability("logging"),
        ];
        
        Self { agents }
    }
    
    /// Get all agents
    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }
    
    /// Find agent by name
    pub fn find_by_name(&self, name: &str) -> Option<&Agent> {
        self.agents.iter().find(|a| a.name.contains(name))
    }
    
    /// Get agents by role
    pub fn by_role(&self, role: AgentRole) -> Vec<&Agent> {
        self.agents.iter().filter(|a| a.role == role).collect()
    }
}
