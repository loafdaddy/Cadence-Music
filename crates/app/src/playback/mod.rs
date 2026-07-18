//! GStreamer-backed playback and in-memory play queue.

mod player;
mod queue;

pub use player::{PlaybackState, Player, PlayerEvent};
pub use queue::{Queue, RepeatMode};
