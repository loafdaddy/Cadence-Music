//! SQLite-backed persistence.
//!
//! [`Database`] owns a single [`rusqlite::Connection`]. It is intentionally
//! **not** `Sync`: the application keeps it on a dedicated worker so the UI
//! thread is never blocked on disk I/O. All query methods live in the
//! [`library`] submodule via `impl Database`.

mod library;
mod schema;

pub use library::SongSort;

use std::path::Path;

use rusqlite::Connection;

use crate::error::Result;

/// A handle to the on-disk music library database.
pub struct Database {
    conn: Connection,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database").finish_non_exhaustive()
    }
}

impl Database {
    /// Open (creating if necessary) the database at `path` and run migrations.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Self::from_connection(conn)
    }

    /// Open an in-memory database. Primarily useful for tests.
    pub fn open_in_memory() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> Result<Self> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let mut db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Apply any outstanding migrations to reach [`schema::LATEST_VERSION`].
    fn migrate(&mut self) -> Result<()> {
        let mut version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;

        while version < schema::LATEST_VERSION {
            let tx = self.conn.transaction()?;
            match version {
                0 => tx.execute_batch(schema::V1)?,
                1 => tx.execute_batch(schema::V2)?,
                other => {
                    return Err(crate::error::Error::Other(anyhow::anyhow!(
                        "no migration path from schema version {other}"
                    )))
                }
            }
            version += 1;
            // `user_version` cannot be parameterised, so format it in; `version`
            // is an internally-controlled integer, never user input.
            tx.pragma_update(None, "user_version", version)?;
            tx.commit()?;
            tracing::info!(version, "applied database migration");
        }
        Ok(())
    }

    /// Borrow the underlying connection (read-only access for advanced callers
    /// and integration tests).
    #[must_use]
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_and_migrates_in_memory() {
        let db = Database::open_in_memory().unwrap();
        let version: i64 = db
            .connection()
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, schema::LATEST_VERSION);
    }
}
