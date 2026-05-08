//! Audit Logging System
//!
//! Provides tamper-evident audit logging for security events.
//! Compliant with SOC 2 CC6.8 and GDPR Article 32 requirements.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self, Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{info, warn, error};
use uuid::Uuid;

/// Maximum number of logs to keep in memory buffer
const MEMORY_BUFFER_SIZE: usize = 1000;

/// Maximum log file size before rotation (10 MB)
const MAX_LOG_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Audit event severity levels
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    /// Debug/trace information
    Debug,
    /// General information
    Info,
    /// Warning - unusual but not critical
    Warning,
    /// Error - operation failed
    Error,
    /// Critical - security incident
    Critical,
}

impl std::fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditSeverity::Debug => write!(f, "debug"),
            AuditSeverity::Info => write!(f, "info"),
            AuditSeverity::Warning => write!(f, "warning"),
            AuditSeverity::Error => write!(f, "error"),
            AuditSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Types of audit events
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// Authentication attempt
    Authentication,
    /// Authorization check
    Authorization,
    /// Data access (read/write)
    DataAccess,
    /// Encryption operation
    Encryption,
    /// Decryption operation
    Decryption,
    /// Key generation
    KeyGeneration,
    /// Key derivation
    KeyDerivation,
    /// Sync operation
    SyncOperation,
    /// Permission change
    PermissionChange,
    /// Configuration change
    ConfigChange,
    /// Export operation
    Export,
    /// Import operation
    Import,
    /// Deletion operation
    Deletion,
    /// System event
    System,
}

/// Audit log entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLog {
    /// Unique log entry ID
    pub log_id: Uuid,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: AuditEventType,
    /// Severity level
    pub severity: AuditSeverity,
    /// User ID (if applicable)
    pub user_id: Option<Uuid>,
    /// Device ID
    pub device_id: Option<String>,
    /// Resource being accessed
    pub resource: String,
    /// Action performed
    pub action: String,
    /// Whether the operation succeeded
    pub success: bool,
    /// Additional context/details
    pub details: Option<String>,
    /// Client IP address (if applicable)
    pub client_ip: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Hash of previous log entry (for tamper detection)
    pub previous_hash: Option<String>,
    /// Hash of this log entry
    pub entry_hash: String,
}

impl AuditLog {
    /// Create a new audit log entry
    pub fn new(
        event_type: AuditEventType,
        severity: AuditSeverity,
        resource: impl Into<String>,
        action: impl Into<String>,
        success: bool,
    ) -> Self {
        let log_id = Uuid::new_v4();
        let timestamp = Utc::now();
        
        Self {
            log_id,
            timestamp,
            event_type,
            severity,
            user_id: None,
            device_id: None,
            resource: resource.into(),
            action: action.into(),
            success,
            details: None,
            client_ip: None,
            session_id: None,
            previous_hash: None,
            entry_hash: String::new(),
        }
    }
    
    /// Add user context
    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    /// Add device context
    pub fn with_device(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }
    
    /// Add details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
    
    /// Add client IP
    pub fn with_client_ip(mut self, ip: impl Into<String>) -> Self {
        self.client_ip = Some(ip.into());
        self
    }
    
    /// Add session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
    
    /// Compute hash of this entry for tamper detection
    fn compute_hash(&self) -> String {
        let data = format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.log_id,
            self.timestamp.to_rfc3339(),
            serde_json::to_string(&self.event_type).unwrap_or_else(|_| "null".to_string()),
            self.severity,
            self.user_id.map(|u| u.to_string()).unwrap_or_default(),
            self.resource,
            self.action,
            self.success,
            self.details.as_deref().unwrap_or(""),
            self.client_ip.as_deref().unwrap_or(""),
            self.previous_hash.as_deref().unwrap_or(""),
        );
        
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Finalize the entry by computing its hash
    fn finalize(&mut self, previous_hash: Option<String>) {
        self.previous_hash = previous_hash;
        self.entry_hash = self.compute_hash();
    }
}

/// Configuration for audit logger
#[derive(Clone, Debug)]
pub struct AuditConfig {
    /// Directory for log files
    pub log_dir: PathBuf,
    /// Maximum log file size before rotation
    pub max_file_size: usize,
    /// Maximum number of log files to retain
    pub max_files: usize,
    /// Minimum severity to log
    pub min_severity: AuditSeverity,
    /// Whether to log to console
    pub console_output: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("./logs/audit"),
            max_file_size: MAX_LOG_FILE_SIZE,
            max_files: 10,
            min_severity: AuditSeverity::Info,
            console_output: false,
        }
    }
}

/// Audit logger with tamper detection
pub struct AuditLogger {
    config: AuditConfig,
    memory_buffer: VecDeque<AuditLog>,
    current_file: Option<File>,
    current_file_size: usize,
    last_hash: Option<String>,
    file_counter: u32,
}

/// Result type for audit operations
pub type AuditResult<T> = Result<T, AuditError>;

/// Audit error types
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Log directory not accessible: {0}")]
    DirectoryNotAccessible(String),
    
    #[error("Maximum log files exceeded")]
    MaxFilesExceeded,
    
    #[error("Tamper detected - hash chain broken")]
    TamperDetected,
}

impl AuditLogger {
    /// Create a new audit logger with default configuration
    pub fn new() -> Self {
        Self::with_config(AuditConfig::default())
    }
    
    /// Create a new audit logger with custom configuration
    pub fn with_config(config: AuditConfig) -> Self {
        let mut logger = Self {
            config,
            memory_buffer: VecDeque::with_capacity(MEMORY_BUFFER_SIZE),
            current_file: None,
            current_file_size: 0,
            last_hash: None,
            file_counter: 0,
        };
        
        // Initialize log directory
        if let Err(e) = logger.init_directory() {
            error!("Failed to initialize audit log directory: {}", e);
        }
        
        logger
    }
    
    /// Initialize the log directory
    fn init_directory(&mut self) -> AuditResult<()> {
        if !self.config.log_dir.exists() {
            std::fs::create_dir_all(&self.config.log_dir)?;
        }
        
        if !self.config.log_dir.is_dir() {
            return Err(AuditError::DirectoryNotAccessible(
                self.config.log_dir.to_string_lossy().to_string()
            ));
        }
        
        // Open current log file
        self.rotate_file()?;
        
        Ok(())
    }
    
    /// Rotate to a new log file
    fn rotate_file(&mut self) -> AuditResult<()> {
        // Close current file if open
        if let Some(mut file) = self.current_file.take() {
            file.flush()?;
        }
        
        // Check if we need to clean up old files
        self.cleanup_old_files()?;
        
        // Create new file
        self.file_counter += 1;
        let filename = format!("audit_{}.log", self.file_counter);
        let filepath = self.config.log_dir.join(filename);
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filepath)?;
        
        self.current_file = Some(file);
        self.current_file_size = 0;
        
        info!("Audit log rotated to: {:?}", filepath);
        
        Ok(())
    }
    
    /// Clean up old log files
    fn cleanup_old_files(&mut self) -> AuditResult<()> {
        let mut files: Vec<_> = std::fs::read_dir(&self.config.log_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|s| s.starts_with("audit_") && s.ends_with(".log"))
                    .unwrap_or(false)
            })
            .collect();
        
        // Sort by modification time (oldest first)
        files.sort_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        
        // Remove oldest files if over limit
        while files.len() >= self.config.max_files {
            if let Some(oldest) = files.first() {
                let path = oldest.path();
                if let Err(e) = std::fs::remove_file(&path) {
                    warn!("Failed to remove old audit log {:?}: {}", path, e);
                } else {
                    info!("Removed old audit log: {:?}", path);
                }
                files.remove(0);
            }
        }
        
        Ok(())
    }
    
    /// Log an audit event
    pub fn log(&mut self, mut entry: AuditLog) -> AuditResult<()> {
        // Check severity filter
        if !self.meets_severity_threshold(&entry.severity) {
            return Ok(());
        }
        
        // Finalize entry with hash
        entry.finalize(self.last_hash.clone());
        self.last_hash = Some(entry.entry_hash.clone());
        
        // Serialize entry
        let json = serde_json::to_string(&entry)?;
        let line = format!("{}\n", json);
        
        // Write to file
        if let Some(file) = &mut self.current_file {
            // Check if rotation needed
            if self.current_file_size + line.len() > self.config.max_file_size {
                self.rotate_file()?;
            }
            
            file.write_all(line.as_bytes())?;
            file.flush()?;
            self.current_file_size += line.len();
        }
        
        // Add to memory buffer
        if self.memory_buffer.len() >= MEMORY_BUFFER_SIZE {
            self.memory_buffer.pop_front();
        }
        self.memory_buffer.push_back(entry.clone());
        
        // Console output if enabled
        if self.config.console_output {
            println!("[AUDIT] {} - {} - {} - {}",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                entry.severity,
                entry.event_type,
                entry.action
            );
        }
        
        // Log to tracing
        match entry.severity {
            AuditSeverity::Debug => debug!("Audit: {:?}", entry),
            AuditSeverity::Info => info!("Audit: {} - {}", entry.action, entry.resource),
            AuditSeverity::Warning => warn!("Audit: {} - {}", entry.action, entry.resource),
            AuditSeverity::Error | AuditSeverity::Critical => {
                error!("Audit: {} - {} - Success: {}", entry.action, entry.resource, entry.success)
            }
        }
        
        Ok(())
    }
    
    /// Check if severity meets threshold
    fn meets_severity_threshold(&self, severity: &AuditSeverity) -> bool {
        let severity_level = match severity {
            AuditSeverity::Debug => 0,
            AuditSeverity::Info => 1,
            AuditSeverity::Warning => 2,
            AuditSeverity::Error => 3,
            AuditSeverity::Critical => 4,
        };
        
        let threshold_level = match self.config.min_severity {
            AuditSeverity::Debug => 0,
            AuditSeverity::Info => 1,
            AuditSeverity::Warning => 2,
            AuditSeverity::Error => 3,
            AuditSeverity::Critical => 4,
        };
        
        severity_level >= threshold_level
    }
    
    /// Query recent logs from memory buffer
    pub fn query_recent(&self, count: usize) -> Vec<&AuditLog> {
        self.memory_buffer.iter().rev().take(count).collect()
    }
    
    /// Export logs to file
    pub fn export(&self, output_path: &Path) -> AuditResult<()> {
        let mut output = File::create(output_path)?;
        
        // Write memory buffer
        for entry in &self.memory_buffer {
            let json = serde_json::to_string(entry)?;
            writeln!(output, "{}", json)?;
        }
        
        // Write file logs
        for entry in std::fs::read_dir(&self.config.log_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("log") {
                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    let line = line?;
                    writeln!(output, "{}", line)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Verify log integrity (tamper detection)
    pub fn verify_integrity(&self) -> AuditResult<bool> {
        let mut previous_hash: Option<String> = None;
        let mut tampered = false;
        
        for entry_result in std::fs::read_dir(&self.config.log_dir)? {
            let entry = entry_result?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("log") {
                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                
                for line in reader.lines() {
                    let line = line?;
                    let log: AuditLog = match serde_json::from_str(&line) {
                        Ok(l) => l,
                        Err(e) => {
                            error!("Failed to parse audit log entry: {}", e);
                            tampered = true;
                            continue;
                        }
                    };
                    
                    // Verify previous hash chain
                    if log.previous_hash != previous_hash {
                        error!("Tamper detected: hash chain broken at log {}", log.log_id);
                        tampered = true;
                    }
                    
                    // Verify entry hash
                    let computed_hash = log.compute_hash();
                    if computed_hash != log.entry_hash {
                        error!("Tamper detected: entry hash mismatch at log {}", log.log_id);
                        tampered = true;
                    }
                    
                    previous_hash = Some(log.entry_hash);
                }
            }
        }
        
        if tampered {
            Err(AuditError::TamperDetected)
        } else {
            Ok(true)
        }
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe audit logger wrapper
pub struct SharedAuditLogger {
    inner: Arc<Mutex<AuditLogger>>,
}

impl SharedAuditLogger {
    /// Create a new shared audit logger
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AuditLogger::new())),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: AuditConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AuditLogger::with_config(config))),
        }
    }
    
    /// Log an audit event
    pub fn log(&self, entry: AuditLog) -> AuditResult<()> {
        let mut logger = self.inner.lock()
            .map_err(|_| AuditError::TamperDetected)?;
        logger.log(entry)
    }
    
    /// Query recent logs
    pub fn query_recent(&self, count: usize) -> AuditResult<Vec<AuditLog>> {
        let logger = self.inner.lock()
            .map_err(|_| AuditError::TamperDetected)?;
        Ok(logger.query_recent(count).into_iter().cloned().collect())
    }
}

impl Default for SharedAuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_audit_log_creation() {
        let log = AuditLog::new(
            AuditEventType::Authentication,
            AuditSeverity::Info,
            "user:123",
            "login",
            true,
        );
        
        assert_eq!(log.event_type, AuditEventType::Authentication);
        assert_eq!(log.severity, AuditSeverity::Info);
        assert!(log.success);
    }
    
    #[test]
    fn test_audit_log_builder() {
        let user_id = Uuid::new_v4();
        let log = AuditLog::new(
            AuditEventType::DataAccess,
            AuditSeverity::Warning,
            "entity:456",
            "read",
            true,
        )
        .with_user(user_id)
        .with_device("device-abc")
        .with_details("sensitive data accessed");
        
        assert_eq!(log.user_id, Some(user_id));
        assert_eq!(log.device_id, Some("device-abc".to_string()));
        assert!(log.details.is_some());
    }
    
    #[test]
    fn test_audit_logger_log_and_query() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = AuditConfig {
            log_dir: temp_dir.path().to_path_buf(),
            max_file_size: 1024 * 1024,
            max_files: 5,
            min_severity: AuditSeverity::Debug,
            console_output: false,
        };
        
        let mut logger = AuditLogger::with_config(config);
        
        // Log some entries
        for i in 0..10 {
            let log = AuditLog::new(
                AuditEventType::System,
                AuditSeverity::Info,
                format!("resource:{}", i),
                "test_action",
                true,
            );
            logger.log(log).expect("Failed to log entry");
        }
        
        // Query recent
        let recent = logger.query_recent(5);
        assert_eq!(recent.len(), 5);
    }
    
    #[test]
    fn test_tamper_detection() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = AuditConfig {
            log_dir: temp_dir.path().to_path_buf(),
            max_file_size: 1024 * 1024,
            max_files: 5,
            min_severity: AuditSeverity::Debug,
            console_output: false,
        };
        
        let mut logger = AuditLogger::with_config(config);
        
        // Log entries
        let log1 = AuditLog::new(
            AuditEventType::System,
            AuditSeverity::Info,
            "resource:1",
            "action1",
            true,
        );
        logger.log(log1).expect("Failed to log entry");
        
        let log2 = AuditLog::new(
            AuditEventType::System,
            AuditSeverity::Info,
            "resource:2",
            "action2",
            true,
        );
        logger.log(log2).expect("Failed to log entry");
        
        // Verify integrity
        assert!(logger.verify_integrity().expect("Failed to verify integrity"));
    }
}
