//! Path templates that turn metadata into a target relative path.

use serde::{Deserialize, Serialize};

use crate::models::TrackMetadata;

const UNKNOWN_ARTIST: &str = "Unknown Artist";
const UNKNOWN_ALBUM: &str = "Unknown Album";
const UNKNOWN_TITLE: &str = "Unknown Title";
const UNKNOWN_GENRE: &str = "Unknown Genre";
const COMPILATIONS: &str = "Compilations";

/// Built-in organization layouts described in the design brief.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    /// `Album Artist/Album/01 Title`
    ArtistAlbumTrack,
    /// `Album Artist/2001 - Album/01 Title`
    ArtistYearAlbumTrack,
    /// `Genre/Album Artist/Album/01 Title`
    GenreArtistAlbumTrack,
    /// `Album Artist/Singles/Title` for tracks with no album.
    ArtistSingles,
}

impl Preset {
    /// The token pattern this preset expands to.
    #[must_use]
    pub fn pattern(self) -> &'static str {
        match self {
            Self::ArtistAlbumTrack => "{albumartist}/{album}/{track2} {title}",
            Self::ArtistYearAlbumTrack => "{albumartist}/{year} - {album}/{track2} {title}",
            Self::GenreArtistAlbumTrack => "{genre}/{albumartist}/{album}/{track2} {title}",
            Self::ArtistSingles => "{albumartist}/Singles/{title}",
        }
    }

    /// A short human-readable label for settings UI.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::ArtistAlbumTrack => "Artist / Album / Track",
            Self::ArtistYearAlbumTrack => "Artist / Year - Album / Track",
            Self::GenreArtistAlbumTrack => "Genre / Artist / Album / Track",
            Self::ArtistSingles => "Artist / Singles",
        }
    }

    /// Every preset, for populating a combo box.
    #[must_use]
    pub fn all() -> &'static [Preset] {
        &[
            Self::ArtistAlbumTrack,
            Self::ArtistYearAlbumTrack,
            Self::GenreArtistAlbumTrack,
            Self::ArtistSingles,
        ]
    }
}

/// A resolved template: either a preset or a user-supplied token pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Template {
    Preset(Preset),
    Custom(String),
}

impl Default for Template {
    fn default() -> Self {
        Self::Preset(Preset::ArtistAlbumTrack)
    }
}

impl Template {
    fn pattern(&self) -> &str {
        match self {
            Self::Preset(p) => p.pattern(),
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Render this template into a target relative path (without extension).
    ///
    /// Compilations are grouped under a top-level `Compilations` directory and
    /// multi-disc albums gain a `Disc N` sub-directory automatically, matching
    /// the behaviour described in the brief.
    #[must_use]
    pub fn render(&self, meta: &TrackMetadata) -> String {
        let mut segments: Vec<String> = self
            .pattern()
            .split('/')
            .map(|segment| render_segment(segment, meta))
            .map(|segment| sanitize_component(&segment))
            .filter(|segment| !segment.is_empty())
            .collect();

        // Insert a disc folder before the file name for multi-disc releases.
        if let (Some(disc), Some(total)) = (meta.disc_number, meta.disc_total) {
            if total > 1 && segments.len() >= 2 {
                let file = segments.pop().expect("checked len >= 2");
                segments.push(sanitize_component(&format!("Disc {disc}")));
                segments.push(file);
            }
        }

        if meta.compilation {
            segments.insert(0, COMPILATIONS.to_owned());
        }

        segments.join("/")
    }
}

/// Replace every `{token}` in a single path segment.
fn render_segment(segment: &str, meta: &TrackMetadata) -> String {
    let mut out = String::with_capacity(segment.len());
    let mut chars = segment.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '{' {
            out.push(ch);
            continue;
        }
        let mut token = String::new();
        for c in chars.by_ref() {
            if c == '}' {
                break;
            }
            token.push(c);
        }
        out.push_str(&expand_token(&token, meta));
    }
    out
}

/// Expand a single token name to its metadata value with sensible fallbacks.
fn expand_token(token: &str, meta: &TrackMetadata) -> String {
    let s = |value: &Option<String>, fallback: &str| {
        value
            .as_deref()
            .filter(|v| !v.is_empty())
            .unwrap_or(fallback)
            .to_owned()
    };

    match token {
        "title" => s(&meta.title, UNKNOWN_TITLE),
        "artist" => s(&meta.artist, UNKNOWN_ARTIST),
        "album" => s(&meta.album, UNKNOWN_ALBUM),
        "albumartist" => meta
            .album_artist
            .as_deref()
            .or(meta.artist.as_deref())
            .filter(|v| !v.is_empty())
            .unwrap_or(UNKNOWN_ARTIST)
            .to_owned(),
        "composer" => s(&meta.composer, ""),
        "genre" => s(&meta.genre, UNKNOWN_GENRE),
        "year" => meta.year.map(|y| y.to_string()).unwrap_or_default(),
        "track" => meta.track_number.map(|n| n.to_string()).unwrap_or_default(),
        "track2" => meta
            .track_number
            .map(|n| format!("{n:02}"))
            .unwrap_or_default(),
        "disc" => meta.disc_number.map(|n| n.to_string()).unwrap_or_default(),
        other => format!("{{{other}}}"),
    }
}

/// Characters that are illegal or problematic in filenames across Linux,
/// Windows and macOS, or that cause trouble in shells.
const ILLEGAL: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];

/// Make a single path component safe: strip illegal characters, collapse
/// whitespace and trailing dots, and never return an empty string.
#[must_use]
pub fn sanitize_component(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .map(|c| if ILLEGAL.contains(&c) { ' ' } else { c })
        .collect();

    // Collapse runs of whitespace and trim leading/trailing spaces and dots,
    // which are invalid at the end of a component on some filesystems.
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = collapsed.trim_matches([' ', '.'].as_ref());

    trimmed.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> TrackMetadata {
        TrackMetadata {
            title: Some("Song".to_owned()),
            artist: Some("Band".to_owned()),
            album: Some("Great Album".to_owned()),
            album_artist: Some("Band".to_owned()),
            genre: Some("Rock".to_owned()),
            year: Some(2001),
            track_number: Some(3),
            ..Default::default()
        }
    }

    #[test]
    fn renders_preset() {
        let t = Template::Preset(Preset::ArtistYearAlbumTrack);
        assert_eq!(t.render(&sample()), "Band/2001 - Great Album/03 Song");
    }

    #[test]
    fn sanitizes_illegal_characters() {
        assert_eq!(sanitize_component("AC/DC: Live?"), "AC DC Live");
        assert_eq!(sanitize_component("  trailing.  "), "trailing");
    }

    #[test]
    fn compilations_are_grouped() {
        let mut meta = sample();
        meta.compilation = true;
        let rendered = Template::Preset(Preset::ArtistAlbumTrack).render(&meta);
        assert!(rendered.starts_with("Compilations/"));
    }

    #[test]
    fn multi_disc_gets_disc_folder() {
        let mut meta = sample();
        meta.disc_number = Some(2);
        meta.disc_total = Some(2);
        let rendered = Template::Preset(Preset::ArtistAlbumTrack).render(&meta);
        assert!(rendered.contains("/Disc 2/"), "got: {rendered}");
    }
}
