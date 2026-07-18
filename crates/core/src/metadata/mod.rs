//! Reading and writing audio file metadata via [`lofty`].
//!
//! This module is deliberately the *only* place in the core that talks to
//! `lofty`, so the rest of the code depends on our own [`TrackMetadata`] type
//! rather than the tagging library's API surface.

mod tags;

pub use tags::{read_metadata, write_metadata};

use crate::models::TrackMetadata;

/// A single field on [`TrackMetadata`], used to describe batch edits and to
/// report which fields are missing during validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetadataField {
    Title,
    Artist,
    Album,
    AlbumArtist,
    Composer,
    Genre,
    Year,
    TrackNumber,
    DiscNumber,
    Artwork,
}

impl MetadataField {
    /// A short human-readable label for use in the UI.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Artist => "Artist",
            Self::Album => "Album",
            Self::AlbumArtist => "Album Artist",
            Self::Composer => "Composer",
            Self::Genre => "Genre",
            Self::Year => "Year",
            Self::TrackNumber => "Track Number",
            Self::DiscNumber => "Disc Number",
            Self::Artwork => "Artwork",
        }
    }
}

/// Report which important tags are missing from a set of metadata.
///
/// Artwork is not represented in [`TrackMetadata`] and therefore is never
/// reported here; the caller checks artwork separately against the cache.
#[must_use]
pub fn missing_fields(meta: &TrackMetadata) -> Vec<MetadataField> {
    let mut missing = Vec::new();
    let is_blank = |value: &Option<String>| value.as_deref().map_or(true, str::is_empty);

    if is_blank(&meta.title) {
        missing.push(MetadataField::Title);
    }
    if is_blank(&meta.artist) {
        missing.push(MetadataField::Artist);
    }
    if is_blank(&meta.album) {
        missing.push(MetadataField::Album);
    }
    if is_blank(&meta.genre) {
        missing.push(MetadataField::Genre);
    }
    if meta.year.is_none() {
        missing.push(MetadataField::Year);
    }
    if meta.track_number.is_none() {
        missing.push(MetadataField::TrackNumber);
    }
    missing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_flags_blanks() {
        let meta = TrackMetadata {
            title: Some(String::new()),
            artist: Some("Someone".to_owned()),
            ..Default::default()
        };
        let missing = missing_fields(&meta);
        assert!(missing.contains(&MetadataField::Title));
        assert!(!missing.contains(&MetadataField::Artist));
        assert!(missing.contains(&MetadataField::Year));
    }
}
