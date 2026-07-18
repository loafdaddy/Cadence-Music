//! Building, previewing, executing and undoing organization plans.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::models::TrackMetadata;

use super::template::Template;

/// A single proposed file move.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMove {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl FileMove {
    /// Whether this move actually changes the file's location.
    #[must_use]
    pub fn is_noop(&self) -> bool {
        self.from == self.to
    }
}

/// The status of a planned entry, surfaced to the user during preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanEntry {
    /// The file is already in the right place.
    AlreadyOrganized(PathBuf),
    /// The file will be moved.
    Move(FileMove),
    /// The move was skipped because the destination already exists and is a
    /// different file (a likely duplicate).
    Conflict { r#move: FileMove },
}

/// A reviewed, ready-to-execute set of moves rooted at a library directory.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OrganizationPlan {
    pub entries: Vec<PlanEntry>,
}

impl OrganizationPlan {
    /// Build a plan for a set of `(current_path, metadata)` pairs.
    ///
    /// `root` is the destination library folder; every rendered template path
    /// is joined onto it. Nothing touches the disk here — this is pure
    /// computation so it is safe (and cheap) to call for previews.
    #[must_use]
    pub fn build(
        root: &Path,
        template: &Template,
        tracks: impl IntoIterator<Item = (PathBuf, TrackMetadata)>,
    ) -> Self {
        // Track destinations we have already assigned within this plan so two
        // tracks never race for the same target path.
        let mut claimed: HashSet<PathBuf> = HashSet::new();
        let mut entries = Vec::new();

        for (from, meta) in tracks {
            let ext = from
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            let relative = template.render(&meta);
            let mut to = root.join(relative);
            if !ext.is_empty() {
                to.set_extension(&ext);
            }

            if to == from {
                entries.push(PlanEntry::AlreadyOrganized(from));
                continue;
            }

            let collides_on_disk = to.exists();
            let collides_in_plan = claimed.contains(&to);
            let r#move = FileMove {
                from,
                to: to.clone(),
            };

            if collides_on_disk || collides_in_plan {
                entries.push(PlanEntry::Conflict { r#move });
            } else {
                claimed.insert(to);
                entries.push(PlanEntry::Move(r#move));
            }
        }

        Self { entries }
    }

    /// The moves that would actually be performed by [`Self::execute`].
    #[must_use]
    pub fn pending_moves(&self) -> Vec<&FileMove> {
        self.entries
            .iter()
            .filter_map(|entry| match entry {
                PlanEntry::Move(m) => Some(m),
                _ => None,
            })
            .collect()
    }

    /// Number of files that will be moved.
    #[must_use]
    pub fn move_count(&self) -> usize {
        self.pending_moves().len()
    }

    /// Number of conflicting entries that will be skipped.
    #[must_use]
    pub fn conflict_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e, PlanEntry::Conflict { .. }))
            .count()
    }

    /// Execute every non-conflicting move, creating parent directories as
    /// needed, and return an [`UndoLog`] recording what happened.
    ///
    /// Executed moves are recorded incrementally, so if an error occurs partway
    /// through, the returned error still leaves a usable log for the moves that
    /// already succeeded (the caller gets it via [`Error`] context only; to
    /// obtain the partial log, prefer [`Self::execute_logged`]).
    pub fn execute(&self) -> Result<UndoLog> {
        let mut log = UndoLog::default();
        self.execute_logged(&mut log)?;
        Ok(log)
    }

    /// Like [`Self::execute`] but writes into a caller-owned log so partial
    /// progress is retained even on failure.
    pub fn execute_logged(&self, log: &mut UndoLog) -> Result<()> {
        for m in self.pending_moves() {
            if let Some(parent) = m.to.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if m.to.exists() {
                return Err(Error::Organization(format!(
                    "destination appeared during execution: {}",
                    m.to.display()
                )));
            }
            std::fs::rename(&m.from, &m.to)?;
            log.moves.push(m.clone());
        }
        Ok(())
    }
}

/// A record of executed moves that can be reversed.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UndoLog {
    pub moves: Vec<FileMove>,
}

impl UndoLog {
    /// Reverse every recorded move, most recent first.
    ///
    /// Empty directories left behind by the original operation are removed on a
    /// best-effort basis.
    pub fn undo(&self) -> Result<()> {
        for m in self.moves.iter().rev() {
            if let Some(parent) = m.from.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::rename(&m.to, &m.from)?;
            if let Some(parent) = m.to.parent() {
                let _ = std::fs::remove_dir(parent); // Only succeeds if empty.
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organization::Preset;

    fn meta(track: u32) -> TrackMetadata {
        TrackMetadata {
            title: Some(format!("Track {track}")),
            album_artist: Some("Artist".to_owned()),
            album: Some("Album".to_owned()),
            track_number: Some(track),
            ..Default::default()
        }
    }

    #[test]
    fn build_and_execute_and_undo() {
        let src = tempfile::tempdir().unwrap();
        let lib = tempfile::tempdir().unwrap();

        let f1 = src.path().join("a.mp3");
        std::fs::write(&f1, b"one").unwrap();

        let plan = OrganizationPlan::build(
            lib.path(),
            &Template::Preset(Preset::ArtistAlbumTrack),
            [(f1.clone(), meta(1))],
        );
        assert_eq!(plan.move_count(), 1);

        let log = plan.execute().unwrap();
        let dest = lib.path().join("Artist/Album/01 Track 1.mp3");
        assert!(dest.exists());
        assert!(!f1.exists());

        log.undo().unwrap();
        assert!(f1.exists());
        assert!(!dest.exists());
    }

    #[test]
    fn detects_in_plan_conflicts() {
        let src = tempfile::tempdir().unwrap();
        let lib = tempfile::tempdir().unwrap();
        let f1 = src.path().join("a.mp3");
        let f2 = src.path().join("b.mp3");
        std::fs::write(&f1, b"one").unwrap();
        std::fs::write(&f2, b"two").unwrap();

        // Both files render to the same destination (identical metadata).
        let plan = OrganizationPlan::build(
            lib.path(),
            &Template::Preset(Preset::ArtistAlbumTrack),
            [(f1, meta(1)), (f2, meta(1))],
        );
        assert_eq!(plan.move_count(), 1);
        assert_eq!(plan.conflict_count(), 1);
    }
}
