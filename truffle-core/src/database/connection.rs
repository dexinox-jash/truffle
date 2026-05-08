//! Database Connection Management
//!
//! Provides SQLite connection with WAL mode, connection pooling,
//! and proper configuration for local-first operation.

use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::Mutex;
use thiserror::Error;
use tracing::{info, debug, error};

/// Database configuration options
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to database file
    pub db_path: PathBuf,
    /// Enable WAL mode (recommended for local-first)
    pub wal_mode: bool,
    /// Synchronous mode (NORMAL recommended for WAL)
    pub synchronous: SynchronousMode,
    /// Cache size in pages (default: -2000 = 2MB)
    pub cache_size: i32,
    /// Enable foreign keys
    pub foreign_keys: bool,
    /// Busy timeout in milliseconds
    pub busy_timeout_ms: u32,
    /// Temporary storage mode
    pub temp_store: TempStore,
    /// Memory-mapped I/O size in bytes (0 = disabled)
    pub mmap_size: i64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("truffle.db"),
            wal_mode: true,
            synchronous: SynchronousMode::Normal,
            cache_size: -2000, // 2MB
            foreign_keys: true,
            busy_timeout_ms: 5000,
            temp_store: TempStore::Memory,
            mmap_size: 0,
        }
    }
}

impl DatabaseConfig {
    /// Create config with default data directory
    pub fn with_data_dir(data_dir: impl AsRef<Path>) -> Self {
        let mut config = Self::default();
        config.db_path = data_dir.as_ref().join("truffle.db");
        config
    }
    
    /// Set WAL mode
    pub fn with_wal(mut self, enabled: bool) -> Self {
        self.wal_mode = enabled;
        self
    }
    
    /// Set synchronous mode
    pub fn with_synchronous(mut self, mode: SynchronousMode) -> Self {
        self.synchronous = mode;
        self
    }
    
    /// Set cache size in megabytes
    pub fn with_cache_size_mb(mut self, mb: i32) -> Self {
        self.cache_size = -mb * 1000;
        self
    }
}

/// SQLite synchronous mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynchronousMode {
    /// OFF: No syncing (fastest, least safe)
    Off = 0,
    /// NORMAL: Sync at critical moments (recommended for WAL)
    Normal = 1,
    /// FULL: Sync at every commit (safest, slowest)
    Full = 2,
    /// EXTRA: Extra durability (WAL mode only)
    Extra = 3,
}

impl SynchronousMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SynchronousMode::Off => "OFF",
            SynchronousMode::Normal => "NORMAL",
            SynchronousMode::Full => "FULL",
            SynchronousMode::Extra => "EXTRA",
        }
    }
}

/// Temporary storage mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempStore {
    /// Default (compile-time setting)
    Default = 0,
    /// Store temporary tables in files
    File = 1,
    /// Store temporary tables in memory
    Memory = 2,
}

impl TempStore {
    pub fn as_str(&self) -> &'static str {
        match self {
            TempStore::Default => "DEFAULT",
            TempStore::File => "FILE",
            TempStore::Memory => "MEMORY",
        }
    }
}

/// Database error types
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Migration error: {0}")]
    Migration(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    #[error("Pool exhausted")]
    PoolExhausted,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Database handle with connection management
pub struct Database {
    config: DatabaseConfig,
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open a new database connection
    pub fn open(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        info!("Opening database at {:?}", config.db_path);
        
        // Ensure parent directory exists
        if let Some(parent) = config.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Open connection with flags for thread safety
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX;
        
        let conn = Connection::open_with_flags(&config.db_path, flags)
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;
        
        let db = Self {
            config: config.clone(),
            connection: Arc::new(Mutex::new(conn)),
        };
        
        // Apply configuration
        db.apply_config(&config)?;
        
        info!("Database opened successfully");
        Ok(db)
    }
    
    /// Open an in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self, DatabaseError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;
        
        let config = DatabaseConfig::default();
        
        let db = Self {
            config: config.clone(),
            connection: Arc::new(Mutex::new(conn)),
        };
        
        db.apply_config(&config)?;
        
        Ok(db)
    }
    
    /// Apply configuration settings
    fn apply_config(&self, config: &DatabaseConfig) -> Result<(), DatabaseError> {
        let conn = self.connection.lock();
        
        // Set WAL mode
        if config.wal_mode {
            debug!("Enabling WAL mode");
            conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        }
        
        // Set synchronous mode
        conn.execute_batch(&format!(
            "PRAGMA synchronous = {};",
            config.synchronous.as_str()
        ))?;
        
        // Set cache size
        conn.execute_batch(&format!("PRAGMA cache_size = {};", config.cache_size))?;
        
        // Enable foreign keys
        if config.foreign_keys {
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        }
        
        // Set busy timeout
        conn.execute_batch(&format!(
            "PRAGMA busy_timeout = {};",
            config.busy_timeout_ms
        ))?;
        
        // Set temp store
        conn.execute_batch(&format!(
            "PRAGMA temp_store = {};",
            config.temp_store.as_str()
        ))?;
        
        // Set mmap size if enabled
        if config.mmap_size > 0 {
            conn.execute_batch(&format!("PRAGMA mmap_size = {};", config.mmap_size))?;
        }
        
        Ok(())
    }
    
    /// Get a reference to the connection
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        self.connection.clone()
    }
    
    /// Execute a function within a transaction
    pub fn with_transaction<F, R>(&self, f: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&mut Connection) -> Result<R, DatabaseError>,
    {
        let mut conn = self.connection.lock();
        let tx = conn.transaction()
            .map_err(|e| DatabaseError::Transaction(e.to_string()))?;
        
        match f(tx.as_mut().map_err(|e| DatabaseError::Transaction(e.to_string()))?) {
            Ok(result) => {
                tx.commit()
                    .map_err(|e| DatabaseError::Transaction(e.to_string()))?;
                Ok(result)
            }
            Err(e) => {
                // Transaction will rollback on drop
                error!("Transaction failed, rolling back: {}", e);
                Err(e)
            }
        }
    }
    
    /// Execute a function with the connection
    pub fn with_connection<F, R>(&self, f: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&Connection) -> Result<R, DatabaseError>,
    {
        let conn = self.connection.lock();
        f(&conn)
    }
    
    /// Get database statistics
    pub fn stats(&self) -> Result<DatabaseStats, DatabaseError> {
        let conn = self.connection.lock();
        
        let page_size: i64 = conn.query_row(
            "PRAGMA page_size",
            [],
            |row| row.get(0)
        )?;
        
        let page_count: i64 = conn.query_row(
            "PRAGMA page_count",
            [],
            |row| row.get(0)
        )?;
        
        let freelist_count: i64 = conn.query_row(
            "PRAGMA freelist_count",
            [],
            |row| row.get(0)
        )?;
        
        let journal_mode: String = conn.query_row(
            "PRAGMA journal_mode",
            [],
            |row| row.get(0)
        )?;
        
        Ok(DatabaseStats {
            page_size: page_size as u64,
            page_count: page_count as u64,
            freelist_count: freelist_count as u64,
            database_size_bytes: (page_size * page_count) as u64,
            journal_mode,
        })
    }
    
    /// Vacuum the database
    pub fn vacuum(&self) -> Result<(), DatabaseError> {
        let conn = self.connection.lock();
        conn.execute_batch("VACUUM;")?;
        info!("Database vacuumed");
        Ok(())
    }
    
    /// Checkpoint WAL (write changes to main database)
    pub fn checkpoint(&self) -> Result<(), DatabaseError> {
        let conn = self.connection.lock();
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        debug!("WAL checkpoint completed");
        Ok(())
    }
    
    /// Close the database connection
    pub fn close(self) -> Result<(), DatabaseError> {
        // The connection will be closed when Arc is dropped
        info!("Database connection closed");
        Ok(())
    }
    
    /// Get the database path
    pub fn path(&self) -> &Path {
        &self.config.db_path
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// Page size in bytes
    pub page_size: u64,
    /// Total number of pages
    pub page_count: u64,
    /// Number of free pages
    pub freelist_count: u64,
    /// Total database size in bytes
    pub database_size_bytes: u64,
    /// Current journal mode
    pub journal_mode: String,
}

impl std::fmt::Display for DatabaseStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size_mb = self.database_size_bytes as f64 / (1024.0 * 1024.0);
        write!(
            f,
            "Database: {} pages ({} MB), journal_mode: {}",
            self.page_count, size_mb, self.journal_mode
        )
    }
}

/// Connection pool for concurrent access
pub struct ConnectionPool {
    config: DatabaseConfig,
    connections: Vec<Arc<Mutex<Connection>>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: DatabaseConfig, size: usize) -> Result<Self, DatabaseError> {
        let mut connections = Vec::with_capacity(size);
        
        for i in 0..size {
            debug!("Creating pool connection {}/{}", i + 1, size);
            let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX;
            
            let conn = Connection::open_with_flags(&config.db_path, flags)
                .map_err(|e| DatabaseError::Connection(e.to_string()))?;
            
            // Apply config to each connection
            if config.wal_mode {
                conn.execute_batch("PRAGMA journal_mode = WAL;")?;
            }
            conn.execute_batch(&format!(
                "PRAGMA synchronous = {};",
                config.synchronous.as_str()
            ))?;
            conn.execute_batch(&format!(
                "PRAGMA busy_timeout = {};",
                config.busy_timeout_ms
            ))?;
            
            connections.push(Arc::new(Mutex::new(conn)));
        }
        
        info!("Connection pool created with {} connections", size);
        
        Ok(Self {
            config,
            connections,
        })
    }
    
    /// Get a connection from the pool
    pub fn acquire(&self) -> Option<Arc<Mutex<Connection>>> {
        // For now, round-robin through connections
        // In production, use a proper pool implementation
        self.connections.first().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_database_open() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config = DatabaseConfig::with_data_dir(temp_dir.path());
        
        let db = Database::open(config).expect("Failed to open database");
        let stats = db.stats().expect("Failed to get stats");
        
        assert_eq!(stats.journal_mode.to_lowercase(), "wal");
    }
    
    #[test]
    fn test_database_in_memory() {
        let db = Database::open_in_memory().expect("Failed to open in-memory database");
        let stats = db.stats().expect("Failed to get stats");
        
        assert!(stats.page_count > 0);
    }
    
    #[test]
    fn test_transaction() {
        let db = Database::open_in_memory().expect("Failed to open in-memory database");
        
        // Create a test table
        db.with_connection(|conn| {
            conn.execute(
                "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)",
                [],
            )?;
            Ok(())
        }).expect("Failed to create table");
        
        // Insert within transaction
        let result = db.with_transaction(|conn| {
            conn.execute(
                "INSERT INTO test (name) VALUES (?1)",
                ["test_value"],
            )?;
            Ok(42)
        });
        
        assert_eq!(result.expect("Transaction failed"), 42);
        
        // Verify insert
        let count: i64 = db.with_connection(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM test",
                [],
                |row| row.get(0)
            ).map_err(DatabaseError::Sqlite)
        }).expect("Failed to query count");
        
        assert_eq!(count, 1);
    }
    
    #[test]
    fn test_synchronous_mode() {
        assert_eq!(SynchronousMode::Normal.as_str(), "NORMAL");
        assert_eq!(SynchronousMode::Full.as_str(), "FULL");
    }
}
