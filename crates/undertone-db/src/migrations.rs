//! Database migrations.

use rusqlite::Connection;
use tracing::{debug, info};

use crate::error::{DbError, DbResult};
use crate::schema::{DEFAULT_DATA, SCHEMA_V1};

/// Current schema version.
const CURRENT_VERSION: i32 = 1;

/// Run all pending migrations.
pub fn run(conn: &mut Connection) -> DbResult<()> {
    let current = get_version(conn)?;
    info!(current_version = current, target_version = CURRENT_VERSION, "Checking migrations");

    if current < CURRENT_VERSION {
        let tx = conn.transaction()?;

        for version in (current + 1)..=CURRENT_VERSION {
            debug!(version, "Applying migration");
            apply_migration(&tx, version)?;
        }

        tx.commit()?;
        info!("Migrations complete");
    }

    Ok(())
}

/// Get the current schema version.
fn get_version(conn: &Connection) -> DbResult<i32> {
    // Check if schema_version table exists
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_version')",
        [],
        |row| row.get(0),
    )?;

    if !exists {
        return Ok(0);
    }

    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(version)
}

/// Apply a specific migration version.
fn apply_migration(conn: &Connection, version: i32) -> DbResult<()> {
    match version {
        1 => {
            conn.execute_batch(SCHEMA_V1)?;
            conn.execute_batch(DEFAULT_DATA)?;
            conn.execute(
                "INSERT INTO schema_version (version) VALUES (?)",
                [version],
            )?;
        }
        _ => {
            return Err(DbError::MigrationFailed(format!(
                "Unknown migration version: {version}"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        run(&mut conn).expect("Migrations failed");

        let version = get_version(&conn).unwrap();
        assert_eq!(version, CURRENT_VERSION);

        // Verify default channels exist
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM channels", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 5);
    }
}
