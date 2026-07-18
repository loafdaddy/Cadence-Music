//! Plain data types that model the music library.
//!
//! These types are intentionally free of any GUI, database, or GStreamer
//! dependency so they can be shared between every layer of the application and
//! unit-tested in isolation.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(pub i64);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<i64> for $name {
            fn from(value: i64) -> Self {
                Self(value)
            }
        }

        impl From<$name> for i64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

id_type!(
    /// Primary key of a track row.
    TrackId
);
id_type!(
    /// Primary key of an album row.
    AlbumId
);
id_type!(
    /// Primary key of an artist row.
    ArtistId
);

/// Audio container formats Cadence knows how to index and play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioFormat {
    Flac,
    Mp3,
    Aac,
    M4a,
    Ogg,
    Opus,
    Wav,
    Aiff,
    Alac,
}

impl AudioFormat {
    /// Detect a format from a file extension (case-insensitive, no dot).
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        Some(match ext.to_ascii_lowercase().as_str() {
            "flac" => Self::Flac,
            "mp3" => Self::Mp3,
            "aac" => Self::Aac,
            "m4a" | "m4b" => Self::M4a,
            "ogg" | "oga" => Self::Ogg,
            "opus" => Self::Opus,
            "wav" | "wave" => Self::Wav,
            "aif" | "aiff" | "aifc" => Self::Aiff,
            "alac" => Self::Alac,
            _ => return None,
        })
    }

    /// The canonical lower-case extension for this format.
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Self::Flac => "flac",
            Self::Mp3 => "mp3",
            Self::Aac => "aac",
            Self::M4a => "m4a",
            Self::Ogg => "ogg",
            Self::Opus => "opus",
            Self::Wav => "wav",
            Self::Aiff => "aiff",
            Self::Alac => "alac",
        }
    }

    /// Whether the format supports lossless audio. Used purely for display.
    #[must_use]
    pub fn is_lossless(self) -> bool {
        matches!(self, Self::Flac | Self::Wav | Self::Aiff | Self::Alac)
    }
}

/// Metadata extracted from an audio file, before it is persisted.
///
/// Every field is optional except the file path because real-world files are
/// frequently missing tags. The scanner fills in sensible fallbacks
/// (for example deriving a title from the file stem) at persistence time.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub track_number: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_number: Option<u32>,
    pub disc_total: Option<u32>,
    /// Whether this track belongs to a compilation (various-artists album).
    pub compilation: bool,
    /// Duration in milliseconds.
    pub duration_ms: Option<u64>,
    /// MusicBrainz recording/track/release identifiers, when present.
    pub musicbrainz_track_id: Option<String>,
    pub musicbrainz_album_id: Option<String>,
}

/// A fully indexed track as stored in the database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub path: PathBuf,
    pub title: String,
    pub album_id: Option<AlbumId>,
    pub artist_id: Option<ArtistId>,
    pub album_artist_id: Option<ArtistId>,
    pub composer: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub duration_ms: Option<u64>,
    pub format: Option<AudioFormat>,
    /// File size in bytes; used to cheaply detect changes on rescans.
    pub file_size: u64,
    /// File modification time as a Unix timestamp (seconds).
    pub modified_at: i64,
    /// When the track was first added to the library (Unix seconds).
    pub added_at: i64,
    pub play_count: u32,
    pub favorite: bool,
}

/// An album aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Album {
    pub id: AlbumId,
    pub name: String,
    pub album_artist_id: Option<ArtistId>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub is_compilation: bool,
    pub track_count: u32,
    /// Path to a cached cover-art image, if one has been extracted/fetched.
    pub artwork_path: Option<PathBuf>,
}

/// An artist aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub album_count: u32,
    pub track_count: u32,
    /// Cached portrait on disk, if one has been downloaded.
    pub image_path: Option<PathBuf>,
    /// MusicBrainz artist MBID, when known.
    pub mbid: Option<String>,
}

/// A named group of tracks in a user-defined order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub track_ids: Vec<TrackId>,
}

/// A track plus joined display fields for list UIs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackDisplay {
    pub track: Track,
    pub artist_name: String,
    pub album_name: String,
    pub artwork_path: Option<PathBuf>,
}

impl TrackDisplay {
    /// Human-readable artist, never empty.
    #[must_use]
    pub fn artist_label(&self) -> &str {
        if self.artist_name.is_empty() {
            "Unknown Artist"
        } else {
            &self.artist_name
        }
    }

    /// Human-readable album, never empty.
    #[must_use]
    pub fn album_label(&self) -> &str {
        if self.album_name.is_empty() {
            "Unknown Album"
        } else {
            &self.album_name
        }
    }
}
