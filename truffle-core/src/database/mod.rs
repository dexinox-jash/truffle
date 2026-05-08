//! Database Layer for Truffle Core
//!
//! This module provides SQLite database access with:
//! - WAL mode for concurrent reads/writes
//! - ACID transactions
//! - Content-addressable storage
//! - Vector search via sqlite-vec

pub mod connection;
pub mod migrations;
pub mod queries;

pub use connection::{Database, DatabaseConfig, DatabaseError};
pub use migrations::{MigrationError, run_migrations, MIGRATIONS};
pub use queries::{RawArtifactRepository, WikiNodeRepository, IngestionQueueRepository};

use rusqlite::Connection;
use std::path::Path;

/// Initialize a new database connection with all migrations
pub fn init_database<P: AsRef<Path>>(path: P) -> anyhow::Result<Connection> {
    let mut conn = Connection::open(path)?;
    
    // Enable WAL mode for concurrent reads/writes
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    
    // Run migrations
    run_migrations(&mut conn)?;
    
    Ok(conn)
}

/// Database result type
pub type Result<T> = std::result::Result<T, DatabaseError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_init_database() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let conn = init_database(&db_path).unwrap();
        
        // Verify WAL mode is enabled
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        
        assert_eq!(journal_mode.to_lowercase(), "wal");
    }
}
