//! Database error types.

use thiserror::Error;

/// Database error type.
#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not determine data directory")]
    NoDataDir,

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Record not found: {0}")]
    NotFound(String),
}

/// Result type for database operations.
pub type DbResult<T> = Result<T, DbError>;
