//! Recursive folder scanning and audio-file discovery.
//!
//! The scanner is split into two phases:
//!
//! 1. [`discover`] walks a directory tree and returns every candidate audio
//!    file, cheaply (a `stat` per file, no tag parsing).
//! 2. [`scan_files`] parses metadata for a batch of discovered files in
//!    parallel via `rayon`.
//!
//! Keeping discovery and parsing separate lets the UI show accurate progress
//! and lets the database layer skip files whose size/mtime are unchanged.

mod watcher;

pub use watcher::{LibraryWatcher, WatchEvent};

use std::path::{Path, PathBuf};

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::metadata::read_metadata;
use crate::models::{AudioFormat, TrackMetadata};

/// A file discovered on disk, before metadata parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredFile {
    pub path: PathBuf,
    pub format: AudioFormat,
    pub file_size: u64,
    /// Modification time as Unix seconds.
    pub modified_at: i64,
}

/// A file that has been fully parsed and is ready to be persisted.
#[derive(Debug, Clone, PartialEq)]
pub struct ScannedTrack {
    pub file: DiscoveredFile,
    pub metadata: TrackMetadata,
}

/// Walk `root` recursively and return every supported audio file.
///
/// Symlinks are not followed to avoid cycles and escaping the sandboxed music
/// folders. Hidden files and directories (dot-prefixed) are skipped.
#[must_use]
pub fn discover(root: &Path) -> Vec<DiscoveredFile> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            // Skip hidden entries but always allow the root itself.
            entry.depth() == 0
                || entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| !name.starts_with('.'))
        })
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| discovered_from_path(entry.path()))
        .collect()
}

/// Build a [`DiscoveredFile`] from a path, returning `None` for unsupported
/// extensions or files we cannot `stat`.
fn discovered_from_path(path: &Path) -> Option<DiscoveredFile> {
    let format = path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(AudioFormat::from_extension)?;

    let meta = std::fs::metadata(path).ok()?;
    let modified_at = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs() as i64);

    Some(DiscoveredFile {
        path: path.to_path_buf(),
        format,
        file_size: meta.len(),
        modified_at,
    })
}

/// Parse metadata for a batch of discovered files in parallel.
///
/// Files that fail to parse are logged and skipped rather than aborting the
/// whole scan, because a single corrupt file should never block a library
/// import of thousands of tracks.
#[must_use]
pub fn scan_files(files: Vec<DiscoveredFile>) -> Vec<ScannedTrack> {
    files
        .into_par_iter()
        .filter_map(|file| match read_metadata(&file.path) {
            Ok(metadata) => Some(ScannedTrack { file, metadata }),
            Err(err) => {
                tracing::warn!(path = %file.path.display(), %err, "failed to read metadata");
                None
            }
        })
        .collect()
}

/// Convenience: discover and fully scan a directory in one call.
///
/// Prefer the two-phase API in production code so progress can be reported,
/// but this is handy for tests and simple tools.
#[must_use]
pub fn scan_directory(root: &Path) -> Vec<ScannedTrack> {
    scan_files(discover(root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_skips_unsupported_and_hidden() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("song.mp3"), b"not really audio").unwrap();
        std::fs::write(dir.path().join("cover.jpg"), b"image").unwrap();
        std::fs::write(dir.path().join(".hidden.flac"), b"hidden").unwrap();

        let found = discover(dir.path());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].format, AudioFormat::Mp3);
    }
}
