//! Filesystem watching so the library stays in sync with disk changes.
//!
//! This wraps [`notify`] and translates its low-level events into a small,
//! debounce-friendly [`WatchEvent`] enum that the application layer can act on
//! (rescan a path, remove a track, etc.).

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::{Error, Result};

/// A meaningful change to a watched music folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    /// A file was created or modified and should be (re)scanned.
    Upserted(PathBuf),
    /// A path was removed and any matching tracks should be pruned.
    Removed(PathBuf),
}

/// Watches one or more directories recursively and forwards [`WatchEvent`]s.
///
/// The watcher owns a background thread inside `notify`; dropping it stops
/// watching. Consumers read events from [`LibraryWatcher::events`].
pub struct LibraryWatcher {
    watcher: RecommendedWatcher,
    events: Receiver<WatchEvent>,
}

impl std::fmt::Debug for LibraryWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibraryWatcher").finish_non_exhaustive()
    }
}

impl LibraryWatcher {
    /// Create a watcher that is not yet watching anything.
    pub fn new() -> Result<Self> {
        let (tx, rx) = channel();

        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            let Ok(event) = res else { return };
            for translated in translate(&event) {
                // Ignore send errors: they only happen once the receiver is
                // dropped, at which point the watcher is being torn down.
                let _ = tx.send(translated);
            }
        })
        .map_err(|e| Error::Other(e.into()))?;

        Ok(Self {
            watcher,
            events: rx,
        })
    }

    /// Start watching `path` and everything beneath it.
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| Error::Other(e.into()))
    }

    /// Stop watching a previously watched `path`.
    pub fn unwatch(&mut self, path: &Path) -> Result<()> {
        self.watcher
            .unwatch(path)
            .map_err(|e| Error::Other(e.into()))
    }

    /// The channel of translated events. Poll this from the app's async loop.
    #[must_use]
    pub fn events(&self) -> &Receiver<WatchEvent> {
        &self.events
    }
}

/// Convert a raw `notify` event into zero or more [`WatchEvent`]s.
fn translate(event: &Event) -> Vec<WatchEvent> {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => event
            .paths
            .iter()
            .cloned()
            .map(WatchEvent::Upserted)
            .collect(),
        EventKind::Remove(_) => event
            .paths
            .iter()
            .cloned()
            .map(WatchEvent::Removed)
            .collect(),
        _ => Vec::new(),
    }
}
