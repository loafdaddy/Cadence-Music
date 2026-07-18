//! Error types shared across the core library.

use std::path::PathBuf;

/// The result type used throughout `cadence-core`.
pub type Result<T> = std::result::Result<T, Error>;

/// All errors that the core library can produce.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An underlying SQLite error.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// An I/O error, usually while touching the filesystem.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// A tag reading/writing error from `lofty`.
    #[error("metadata error: {0}")]
    Metadata(#[from] lofty::error::LoftyError),

    /// The file exists but its extension/container is not supported.
    #[error("unsupported audio format: {0}")]
    UnsupportedFormat(PathBuf),

    /// A requested entity was not found in the database.
    #[error("not found: {0}")]
    NotFound(String),

    /// An organization operation could not be planned or executed safely.
    #[error("organization error: {0}")]
    Organization(String),

    /// Any other error, carrying context.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
