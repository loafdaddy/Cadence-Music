//! Ordered play queue with shuffle and repeat.

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use cadence_core::models::{Track, TrackId};

/// Repeat behaviour for the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

/// In-memory playback queue.
#[derive(Debug, Default)]
pub struct Queue {
    tracks: Vec<Track>,
    current: Option<usize>,
    shuffle: bool,
    repeat: RepeatMode,
    /// Original order used to restore after shuffle is disabled.
    order_backup: Vec<Track>,
}

impl Queue {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    #[must_use]
    pub fn current(&self) -> Option<&Track> {
        self.current.and_then(|i| self.tracks.get(i))
    }

    #[must_use]
    pub fn current_index(&self) -> Option<usize> {
        self.current
    }

    #[must_use]
    pub fn shuffle_enabled(&self) -> bool {
        self.shuffle
    }

    #[must_use]
    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat = mode;
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.order_backup.clear();
        self.current = None;
    }

    /// Replace the queue and start at `start`.
    pub fn replace(&mut self, tracks: Vec<Track>, start: usize) {
        self.order_backup = tracks.clone();
        self.tracks = tracks;
        self.current = if self.tracks.is_empty() {
            None
        } else {
            Some(start.min(self.tracks.len() - 1))
        };
        if self.shuffle {
            self.apply_shuffle();
        }
    }

    pub fn append(&mut self, tracks: Vec<Track>) {
        if self.tracks.is_empty() && !tracks.is_empty() {
            self.replace(tracks, 0);
            return;
        }
        self.order_backup.extend(tracks.iter().cloned());
        self.tracks.extend(tracks);
    }

    pub fn set_shuffle(&mut self, enabled: bool) {
        if self.shuffle == enabled {
            return;
        }
        self.shuffle = enabled;
        if enabled {
            self.apply_shuffle();
        } else {
            let current_id = self.current().map(|t| t.id);
            self.tracks = self.order_backup.clone();
            self.current = current_id.and_then(|id| self.tracks.iter().position(|t| t.id == id));
        }
    }

    fn apply_shuffle(&mut self) {
        let current_id = self.current().map(|t| t.id);
        let mut rest: Vec<Track> = self
            .tracks
            .iter()
            .filter(|t| Some(t.id) != current_id)
            .cloned()
            .collect();
        fisher_yates_shuffle(&mut rest);
        let mut new_tracks = Vec::with_capacity(self.tracks.len());
        if let Some(id) = current_id {
            if let Some(cur) = self.tracks.iter().find(|t| t.id == id).cloned() {
                new_tracks.push(cur);
            }
        }
        new_tracks.extend(rest);
        self.tracks = new_tracks;
        self.current = if self.tracks.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    pub fn jump_to(&mut self, index: usize) -> Option<&Track> {
        if index < self.tracks.len() {
            self.current = Some(index);
            self.current()
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<&Track> {
        let len = self.tracks.len();
        if len == 0 {
            return None;
        }
        match self.repeat {
            RepeatMode::One => self.current(),
            RepeatMode::All => {
                let next = self.current.map(|i| (i + 1) % len).unwrap_or(0);
                self.current = Some(next);
                self.current()
            }
            RepeatMode::Off => {
                let next = self.current.map(|i| i + 1).unwrap_or(0);
                if next >= len {
                    self.current = None;
                    None
                } else {
                    self.current = Some(next);
                    self.current()
                }
            }
        }
    }

    pub fn previous(&mut self) -> Option<&Track> {
        let len = self.tracks.len();
        if len == 0 {
            return None;
        }
        let prev = match self.current {
            Some(0) if self.repeat == RepeatMode::All => len - 1,
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.current = Some(prev);
        self.current()
    }

    pub fn remove(&mut self, id: TrackId) {
        if let Some(idx) = self.tracks.iter().position(|t| t.id == id) {
            self.tracks.remove(idx);
            self.order_backup.retain(|t| t.id != id);
            self.current = match self.current {
                Some(c) if c == idx => {
                    if self.tracks.is_empty() {
                        None
                    } else {
                        Some(c.min(self.tracks.len() - 1))
                    }
                }
                Some(c) if c > idx => Some(c - 1),
                other => other,
            };
        }
    }

    /// Upcoming tracks after the current one.
    #[must_use]
    pub fn upcoming(&self) -> VecDeque<&Track> {
        let start = self.current.map(|i| i + 1).unwrap_or(0);
        self.tracks.iter().skip(start).collect()
    }
}

fn fisher_yates_shuffle<T>(items: &mut [T]) {
    if items.len() < 2 {
        return;
    }
    let mut state = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0xC0FFEE);
    for i in (1..items.len()).rev() {
        // xorshift64*
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let j = ((state.wrapping_mul(0x2545_F491_4F6C_DD1D)) as usize) % (i + 1);
        items.swap(i, j);
    }
}
