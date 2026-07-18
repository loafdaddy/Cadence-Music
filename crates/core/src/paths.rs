//! XDG base-directory helpers for Cadence data and cache locations.

use std::path::PathBuf;

use crate::APP_NAME;

/// `$XDG_DATA_HOME/cadence` (or `~/.local/share/cadence`).
#[must_use]
pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_NAME.to_ascii_lowercase())
}

/// `$XDG_CACHE_HOME/cadence` (or `~/.cache/cadence`).
#[must_use]
pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_NAME.to_ascii_lowercase())
}

/// Path to the SQLite library database.
#[must_use]
pub fn library_db_path() -> PathBuf {
    data_dir().join("library.db")
}

/// Directory for cached album artwork.
#[must_use]
pub fn artwork_cache_dir() -> PathBuf {
    cache_dir().join("artwork")
}

/// Directory for cached artist portraits.
#[must_use]
pub fn artist_image_cache_dir() -> PathBuf {
    cache_dir().join("artists")
}

/// Directory for MusicBrainz / CAA response caches (optional).
#[must_use]
pub fn lookup_cache_dir() -> PathBuf {
    cache_dir().join("lookup")
}
