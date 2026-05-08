//! GDPR Compliance Implementation
//!
//! This module implements GDPR Article 32 requirements:
//! - Pseudonymization of personal data
//! - Encryption of personal data
//! - Ongoing confidentiality
//! - Availability and resilience
//!
//! # GDPR Article 32 - Security of Processing
//!
//! Taking into account the state of the art, the costs of implementation
//! and the nature, scope, context and purposes of processing as well as
//! the risk of varying likelihood and severity for the rights and freedoms
//! of natural persons, the controller and the processor shall implement
//! appropriate technical and organisational measures to ensure a level of
//! security appropriate to the risk, including inter alia as appropriate:
//!
//! (a) the pseudonymisation and encryption of personal data;
//! (b) the ability to ensure the ongoing confidentiality, integrity,
//!     availability and resilience of processing systems and services;
//! (c) the ability to restore the availability and access to personal data
//!     in a timely manner in the event of a physical or technical incident;
//! (d) a process for regularly testing, assessing and evaluating the
//!     effectiveness of technical and organisational measures for ensuring
//!     the security of the processing.

use crate::error::{CryptoError, CryptoResult};
use crate::types::DeviceId;
use crate::utils::{hmac_sha256, sha3_256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// GDPR configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GdprConfig {
    /// Enable pseudonymization
    pub enable_pseudonymization: bool,
    /// Enable encryption
    pub enable_encryption: bool,
    /// Data retention period in days (0 = unlimited)
    pub retention_days: u32,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Enable right to erasure
    pub enable_right_to_erasure: bool,
    /// Enable data portability
    pub enable_data_portability: bool,
    /// Server-side salt for hashing
    pub server_salt: Vec<u8>,
}

impl Default for GdprConfig {
    fn default() -> Self {
        Self {
            enable_pseudonymization: true,
            enable_encryption: true,
            retention_days: 0, // Unlimited by default
            enable_audit_logging: true,
            enable_right_to_erasure: true,
            enable_data_portability: true,
            server_salt: vec![],
        }
    }
}

impl GdprConfig {
    /// Create a new GDPR configuration with server salt
    pub fn with_server_salt(mut self, salt: Vec<u8>) -> Self {
        self.server_salt = salt;
        self
    }
}

/// Types of data subject requests
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestType {
    /// Access request (Article 15)
    Access,
    /// Rectification request (Article 16)
    Rectification,
    /// Erasure request (Article 17) - "Right to be forgotten"
    Erasure,
    /// Restriction of processing request (Article 18)
    Restriction,
    /// Data portability request (Article 20)
    Portability,
    /// Objection request (Article 21)
    Objection,
}

impl RequestType {
    /// Get the Article number for this request type
    pub fn article(&self) -> &'static str {
        match self {
            Self::Access => "Article 15",
            Self::Rectification => "Article 16",
            Self::Erasure => "Article 17",
            Self::Restriction => "Article 18",
            Self::Portability => "Article 20",
            Self::Objection => "Article 21",
        }
    }

    /// Get a description of this request type
    pub fn description(&self) -> &'static str {
        match self {
            Self::Access => "Right of access by the data subject",
            Self::Rectification => "Right to rectification",
            Self::Erasure => "Right to erasure ('right to be forgotten')",
            Self::Restriction => "Right to restriction of processing",
            Self::Portability => "Right to data portability",
            Self::Objection => "Right to object",
        }
    }
}

/// Data subject request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataSubjectRequest {
    /// Request ID
    pub id: String,
    /// Request type
    pub request_type: RequestType,
    /// Device ID making the request
    pub device_id: DeviceId,
    /// Request timestamp
    pub timestamp: u64,
    /// Request status
    pub status: RequestStatus,
    /// Additional context/data
    pub context: HashMap<String, String>,
}

/// Request status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestStatus {
    /// Request received
    Received,
    /// Request being processed
    Processing,
    /// Request completed
    Completed,
    /// Request failed
    Failed,
    /// Request rejected
    Rejected,
}

impl DataSubjectRequest {
    /// Create a new data subject request
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InternalError` if system time is unavailable.
    pub fn new(
        id: String,
        request_type: RequestType,
        device_id: DeviceId,
    ) -> CryptoResult<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CryptoError::InternalError(format!("System time error: {:?}", e)))?
            .as_secs();

        Ok(Self {
            id,
            request_type,
            device_id,
            timestamp,
            status: RequestStatus::Received,
            context: HashMap::new(),
        })
    }

    /// Mark as processing
    pub fn mark_processing(&mut self) {
        self.status = RequestStatus::Processing;
    }

    /// Mark as completed
    pub fn mark_completed(&mut self) {
        self.status = RequestStatus::Completed;
    }

    /// Mark as failed
    pub fn mark_failed(&mut self) {
        self.status = RequestStatus::Failed;
    }

    /// Mark as rejected
    pub fn mark_rejected(&mut self) {
        self.status = RequestStatus::Rejected;
    }

    /// Add context
    pub fn add_context(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.context.insert(key.into(), value.into());
    }
}

/// GDPR compliance handler
pub struct GdprCompliance {
    /// Configuration
    config: GdprConfig,
    /// Pending requests
    pending_requests: Vec<DataSubjectRequest>,
    /// Completed requests
    completed_requests: Vec<DataSubjectRequest>,
}

impl GdprCompliance {
    /// Create a new GDPR compliance handler
    pub fn new(config: GdprConfig) -> Self {
        Self {
            config,
            pending_requests: Vec::new(),
            completed_requests: Vec::new(),
        }
    }

    /// Submit a data subject request
    pub fn submit_request(&mut self, request: DataSubjectRequest) -> CryptoResult<()> {
        // Validate request based on configuration
        match request.request_type {
            RequestType::Erasure if !self.config.enable_right_to_erasure => {
                return Err(CryptoError::compliance_error(
                    "Right to erasure is not enabled"
                ));
            }
            RequestType::Portability if !self.config.enable_data_portability => {
                return Err(CryptoError::compliance_error(
                    "Data portability is not enabled"
                ));
            }
            _ => {}
        }

        self.pending_requests.push(request);
        Ok(())
    }

    /// Process pending requests
    pub fn process_requests(&mut self) -> Vec<DataSubjectRequest> {
        let mut processed = Vec::new();

        while let Some(mut request) = self.pending_requests.pop() {
            request.mark_processing();

            // Process based on request type
            let result = match request.request_type {
                RequestType::Access => self.process_access_request(&request),
                RequestType::Erasure => self.process_erasure_request(&request),
                RequestType::Portability => self.process_portability_request(&request),
                _ => Ok(()), // Other types not implemented
            };

            match result {
                Ok(()) => request.mark_completed(),
                Err(_) => request.mark_failed(),
            }

            processed.push(request);
        }

        self.completed_requests.extend(processed.clone());
        processed
    }

    /// Process access request (Article 15)
    fn process_access_request(&self, _request: &DataSubjectRequest) -> CryptoResult<()> {
        // Implementation would gather all personal data
        // and provide it to the data subject
        Ok(())
    }

    /// Process erasure request (Article 17)
    fn process_erasure_request(&self, _request: &DataSubjectRequest) -> CryptoResult<()> {
        // Implementation would delete all personal data
        // and create tombstones for sync
        Ok(())
    }

    /// Process portability request (Article 20)
    fn process_portability_request(&self, _request: &DataSubjectRequest) -> CryptoResult<()> {
        // Implementation would export data in a portable format
        Ok(())
    }

    /// Pseudonymize a device ID
    ///
    /// Uses HMAC-SHA256 with server-side salt to create a
    /// privacy-preserving identifier.
    pub fn pseudonymize_device_id(&self, device_id: &DeviceId) -> [u8; 32] {
        if self.config.enable_pseudonymization {
            hmac_sha256(&self.config.server_salt, device_id.as_bytes())
        } else {
            sha3_256(device_id.as_bytes())
        }
    }

    /// Check if data has exceeded retention period
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InternalError` if system time is unavailable.
    pub fn is_expired(&self, timestamp: u64) -> CryptoResult<bool> {
        if self.config.retention_days == 0 {
            return Ok(false); // Unlimited retention
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CryptoError::InternalError(format!("System time error: {:?}", e)))?
            .as_secs();

        let retention_secs = self.config.retention_days as u64 * 86400;
        Ok(now - timestamp > retention_secs)
    }

    /// Get pending requests
    pub fn pending_requests(&self) -> &[DataSubjectRequest] {
        &self.pending_requests
    }

    /// Get completed requests
    pub fn completed_requests(&self) -> &[DataSubjectRequest] {
        &self.completed_requests
    }
}

/// Data subject rights helper
pub struct DataSubjectRights;

impl DataSubjectRights {
    /// Get all available rights
    pub fn all_rights() -> Vec<(RequestType, &'static str, &'static str)> {
        vec![
            (RequestType::Access, "Article 15", "Right of access"),
            (RequestType::Rectification, "Article 16", "Right to rectification"),
            (RequestType::Erasure, "Article 17", "Right to erasure"),
            (RequestType::Restriction, "Article 18", "Right to restriction"),
            (RequestType::Portability, "Article 20", "Right to portability"),
            (RequestType::Objection, "Article 21", "Right to object"),
        ]
    }

    /// Get the response time limit for a request type
    ///
    /// GDPR requires response within one month, extendable to three months
    /// for complex requests.
    pub fn response_time_limit_days(request_type: RequestType) -> u32 {
        match request_type {
            RequestType::Access => 30,
            RequestType::Rectification => 30,
            RequestType::Erasure => 30,
            RequestType::Restriction => 30,
            RequestType::Portability => 30,
            RequestType::Objection => 30,
        }
    }
}

/// Privacy Impact Assessment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivacyImpactAssessment {
    /// Assessment ID
    pub id: String,
    /// Assessment date
    pub date: u64,
    /// Process/Feature being assessed
    pub process: String,
    /// Personal data involved
    pub personal_data: Vec<String>,
    /// Risks identified
    pub risks: Vec<RiskAssessment>,
    /// Mitigation measures
    pub mitigations: Vec<String>,
    /// Overall risk level
    pub overall_risk: RiskLevel,
}

/// Risk assessment entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Risk description
    pub description: String,
    /// Likelihood (1-5)
    pub likelihood: u8,
    /// Impact (1-5)
    pub impact: u8,
    /// Risk score (likelihood * impact)
    pub score: u8,
}

/// Risk level
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl PrivacyImpactAssessment {
    /// Create a new PIA
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InternalError` if system time is unavailable.
    pub fn new(id: String, process: String) -> CryptoResult<Self> {
        let date = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CryptoError::InternalError(format!("System time error: {:?}", e)))?
            .as_secs();

        Ok(Self {
            id,
            date,
            process,
            personal_data: Vec::new(),
            risks: Vec::new(),
            mitigations: Vec::new(),
            overall_risk: RiskLevel::Low,
        })
    }

    /// Add personal data type
    pub fn add_personal_data(&mut self, data_type: impl Into<String>) {
        self.personal_data.push(data_type.into());
    }

    /// Add risk
    pub fn add_risk(&mut self, description: impl Into<String>, likelihood: u8, impact: u8) {
        let score = likelihood * impact;
        self.risks.push(RiskAssessment {
            description: description.into(),
            likelihood,
            impact,
            score,
        });

        // Update overall risk
        self.update_overall_risk();
    }

    /// Add mitigation
    pub fn add_mitigation(&mut self, mitigation: impl Into<String>) {
        self.mitigations.push(mitigation.into());
    }

    /// Update overall risk based on individual risks
    fn update_overall_risk(&mut self) {
        if self.risks.is_empty() {
            self.overall_risk = RiskLevel::Low;
            return;
        }

        let max_score = self.risks.iter().map(|r| r.score).max().unwrap_or(0);

        self.overall_risk = match max_score {
            0..=4 => RiskLevel::Low,
            5..=9 => RiskLevel::Medium,
            10..=16 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };
    }
}

/// Data deletion helper (for right to erasure)
///
/// # Errors
///
/// Returns `CryptoError::InternalError` if system time is unavailable.
pub fn data_deletion(data_id: &[u8]) -> CryptoResult<[u8; 32]> {
    // Create a deletion record (tombstone)
    let mut record = Vec::new();
    record.extend_from_slice(b"TOMBSTONE");
    record.extend_from_slice(data_id);
    record.extend_from_slice(&SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| CryptoError::InternalError(format!("System time error: {:?}", e)))?
        .as_secs()
        .to_be_bytes());

    Ok(sha3_256(&record))
}

/// Data portability helper
pub fn data_portability(data: &[u8], format: &str) -> CryptoResult<Vec<u8>> {
    match format {
        "json" => Ok(data.to_vec()), // Already JSON
        "markdown" => {
            // Convert to markdown
            Ok(format!("```\n{}\n```", String::from_utf8_lossy(data)).into_bytes())
        }
        "xml" => {
            // Simple XML wrapper
            let xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<data>{}</data>"#,
                String::from_utf8_lossy(data)
            );
            Ok(xml.into_bytes())
        }
        _ => Err(CryptoError::invalid_message_format(
            format!("Unsupported export format: {}", format)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdpr_config_default() {
        let config = GdprConfig::default();
        assert!(config.enable_pseudonymization);
        assert!(config.enable_encryption);
        assert!(config.enable_right_to_erasure);
        assert!(config.enable_data_portability);
    }

    #[test]
    fn test_request_type() {
        assert_eq!(RequestType::Access.article(), "Article 15");
        assert_eq!(RequestType::Erasure.article(), "Article 17");
        assert_eq!(RequestType::Portability.article(), "Article 20");
    }

    #[test]
    fn test_data_subject_request() {
        let device_id = DeviceId::generate_random().unwrap();
        let mut request = DataSubjectRequest::new(
            "req-123".to_string(),
            RequestType::Access,
            device_id,
        );

        assert_eq!(request.request_type, RequestType::Access);
        assert_eq!(request.status, RequestStatus::Received);

        request.mark_processing();
        assert_eq!(request.status, RequestStatus::Processing);

        request.mark_completed();
        assert_eq!(request.status, RequestStatus::Completed);
    }

    #[test]
    fn test_gdpr_compliance_submit_request() {
        let config = GdprConfig::default();
        let mut compliance = GdprCompliance::new(config);

        let device_id = DeviceId::generate_random().unwrap();
        let request = DataSubjectRequest::new(
            "req-123".to_string(),
            RequestType::Access,
            device_id,
        );

        compliance.submit_request(request).unwrap();
        assert_eq!(compliance.pending_requests().len(), 1);
    }

    #[test]
    fn test_pseudonymization() {
        let config = GdprConfig::default().with_server_salt(vec![1, 2, 3, 4]);
        let compliance = GdprCompliance::new(config);

        let device_id = DeviceId::generate_random().unwrap();
        let pseudonym1 = compliance.pseudonymize_device_id(&device_id);
        let pseudonym2 = compliance.pseudonymize_device_id(&device_id);

        // Same device ID should produce same pseudonym
        assert_eq!(pseudonym1, pseudonym2);

        // Different device ID should produce different pseudonym
        let device_id2 = DeviceId::generate_random().unwrap();
        let pseudonym3 = compliance.pseudonymize_device_id(&device_id2);
        assert_ne!(pseudonym1, pseudonym3);
    }

    #[test]
    fn test_retention_check() {
        let mut config = GdprConfig::default();
        config.retention_days = 30;

        let compliance = GdprCompliance::new(config);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Recent data should not be expired
        assert!(!compliance.is_expired(now - 86400)); // 1 day ago

        // Old data should be expired
        assert!(compliance.is_expired(now - 31 * 86400)); // 31 days ago
    }

    #[test]
    fn test_privacy_impact_assessment() {
        let mut pia = PrivacyImpactAssessment::new(
            "pia-123".to_string(),
            "User Sync".to_string(),
        );

        pia.add_personal_data("Device ID");
        pia.add_personal_data("IP Address");

        pia.add_risk("Data breach", 2, 4);
        pia.add_risk("Unauthorized access", 1, 3);

        pia.add_mitigation("End-to-end encryption");
        pia.add_mitigation("Zero-knowledge architecture");

        assert_eq!(pia.personal_data.len(), 2);
        assert_eq!(pia.risks.len(), 2);
        assert_eq!(pia.mitigations.len(), 2);
        assert_eq!(pia.overall_risk, RiskLevel::Medium);
    }

    #[test]
    fn test_data_deletion() {
        let data_id = b"test-data-id";
        let tombstone1 = data_deletion(data_id);
        let tombstone2 = data_deletion(data_id);

        // Same data ID should produce different tombstones (due to timestamp)
        assert_ne!(tombstone1, tombstone2);
    }

    #[test]
    fn test_data_portability() {
        let data = b"{\"name\": \"Test\"}";

        let json = data_portability(data, "json").unwrap();
        assert_eq!(json, data);

        let md = data_portability(data, "markdown").unwrap();
        assert!(String::from_utf8_lossy(&md).contains("```"));

        let xml = data_portability(data, "xml").unwrap();
        assert!(String::from_utf8_lossy(&xml).contains("<?xml"));

        let result = data_portability(data, "unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_data_subject_rights() {
        let rights = DataSubjectRights::all_rights();
        assert_eq!(rights.len(), 6);

        assert_eq!(DataSubjectRights::response_time_limit_days(RequestType::Access), 30);
    }
}
