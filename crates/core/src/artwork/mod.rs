//! Embedded cover-art extraction and on-disk caching.
//!
//! Artwork is stored under an application-provided cache directory as
//! `{sha256(album_artist\\0album)}.ext`. Paths are written back onto album rows
//! via [`crate::db::Database::set_album_artwork`].

use std::fs;
use std::path::{Path, PathBuf};

use lofty::file::TaggedFileExt;
use lofty::picture::MimeType;
use lofty::probe::Probe;
use sha2::{Digest, Sha256};

use crate::error::Result;

/// Stable cache key for an album's artwork.
#[must_use]
pub fn artwork_key(album_artist: &str, album: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(album_artist.as_bytes());
    hasher.update([0]);
    hasher.update(album.as_bytes());
    let digest = hasher.finalize();
    hex_encode(&digest)
}

/// Stable cache key for an artist portrait.
#[must_use]
pub fn artist_image_key(artist: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(artist.as_bytes());
    let digest = hasher.finalize();
    hex_encode(&digest)
}

/// Extract the primary embedded picture from `path` and write it into
/// `cache_dir` under a deterministic filename.
///
/// Returns `Ok(None)` when the file has no usable artwork. On success the
/// returned path points at the cached image file.
pub fn extract_and_cache(
    path: &Path,
    cache_dir: &Path,
    album_artist: &str,
    album: &str,
) -> Result<Option<PathBuf>> {
    let tagged = Probe::open(path)?.read()?;
    let tag = match tagged.primary_tag().or_else(|| tagged.first_tag()) {
        Some(tag) => tag,
        None => return Ok(None),
    };

    let picture = match tag.pictures().first() {
        Some(picture) => picture,
        None => return Ok(None),
    };

    let data = picture.data();
    if data.is_empty() {
        return Ok(None);
    }

    let ext = match picture.mime_type() {
        Some(MimeType::Jpeg) => "jpg",
        Some(MimeType::Png) => "png",
        Some(MimeType::Gif) => "gif",
        Some(MimeType::Bmp) => "bmp",
        Some(MimeType::Tiff) => "tiff",
        _ => "img",
    };

    fs::create_dir_all(cache_dir)?;
    let dest = cache_dir.join(format!("{}.{}", artwork_key(album_artist, album), ext));
    if !dest.exists() {
        fs::write(&dest, data)?;
    }
    Ok(Some(dest))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artwork_key_is_stable_and_distinct() {
        let a = artwork_key("Miles Davis", "Kind of Blue");
        let b = artwork_key("Miles Davis", "Kind of Blue");
        let c = artwork_key("Miles Davis", "Bitches Brew");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64);
    }
}
