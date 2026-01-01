//! Undertone Database - SQLite persistence layer.
//!
//! This crate provides persistent storage for channels, routing rules,
//! profiles, and device settings.

pub mod error;
pub mod migrations;
pub mod queries;
pub mod schema;

pub use error::{DbError, DbResult};

use directories::ProjectDirs;
use rusqlite::Connection;
use std::path::PathBuf;
use tracing::{debug, info};

/// Database handle for Undertone.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create the database at the default location.
    ///
    /// # Errors
    /// Returns an error if the database cannot be opened or initialized.
    pub fn open() -> DbResult<Self> {
        let path = Self::default_path()?;
        Self::open_at(path)
    }

    /// Open or create the database at a specific path.
    ///
    /// # Errors
    /// Returns an error if the database cannot be opened or initialized.
    pub fn open_at(path: PathBuf) -> DbResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!(?path, "Opening database");
        let conn = Connection::open(&path)?;

        // Enable foreign keys and WAL mode
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;

        let mut db = Self { conn };
        db.run_migrations()?;

        Ok(db)
    }

    /// Open an in-memory database (for testing).
    ///
    /// # Errors
    /// Returns an error if the database cannot be initialized.
    pub fn open_in_memory() -> DbResult<Self> {
        debug!("Opening in-memory database");
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        let mut db = Self { conn };
        db.run_migrations()?;

        Ok(db)
    }

    /// Get the default database path.
    fn default_path() -> DbResult<PathBuf> {
        let dirs = ProjectDirs::from("com", "undertone", "Undertone").ok_or(DbError::NoDataDir)?;
        Ok(dirs.data_dir().join("undertone.db"))
    }

    /// Run database migrations.
    fn run_migrations(&mut self) -> DbResult<()> {
        migrations::run(&mut self.conn)
    }

    /// Get a reference to the underlying connection.
    #[must_use]
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Get a mutable reference to the underlying connection.
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let db = Database::open_in_memory().expect("Failed to open in-memory database");
        assert!(db.conn().is_autocommit());
    }
}
