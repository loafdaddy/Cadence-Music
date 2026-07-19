//! Online metadata lookup via MusicBrainz and the Cover Art Archive.
//!
//! Lookups are intentionally conservative: callers decide whether to apply
//! results, and [`LookupResult::apply_missing_only`] never overwrites fields
//! that are already populated.

use std::time::Duration;

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::models::TrackMetadata;

const USER_AGENT: &str = "Cadence/0.1.0 (https://github.com/loafdaddy/Cadence-Music)";
const MB_BASE: &str = "https://musicbrainz.org/ws/2";
const CAA_BASE: &str = "https://coverartarchive.org";

/// A proposed fill-in for missing metadata fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LookupResult {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub musicbrainz_track_id: Option<String>,
    pub musicbrainz_album_id: Option<String>,
    /// Direct URL to a front cover image, if Cover Art Archive has one.
    pub cover_art_url: Option<String>,
}

impl LookupResult {
    /// Merge into `meta`, only filling fields that are currently blank.
    pub fn apply_missing_only(&self, meta: &mut TrackMetadata) {
        let fill = |dst: &mut Option<String>, src: &Option<String>| {
            if dst.as_deref().map_or(true, str::is_empty) {
                if let Some(v) = src {
                    *dst = Some(v.clone());
                }
            }
        };
        fill(&mut meta.title, &self.title);
        fill(&mut meta.artist, &self.artist);
        fill(&mut meta.album, &self.album);
        fill(&mut meta.album_artist, &self.album_artist);
        fill(&mut meta.genre, &self.genre);
        if meta.year.is_none() {
            meta.year = self.year;
        }
        fill(&mut meta.musicbrainz_track_id, &self.musicbrainz_track_id);
        fill(&mut meta.musicbrainz_album_id, &self.musicbrainz_album_id);
    }
}

/// Look up a recording by artist + title (and optional album).
///
/// Respects MusicBrainz rate limiting with a short client-side delay. Network
/// failures are returned as [`Error::Other`].
pub fn lookup_recording(
    artist: &str,
    title: &str,
    album: Option<&str>,
) -> Result<Option<LookupResult>> {
    // Be a polite MusicBrainz citizen.
    std::thread::sleep(Duration::from_millis(1100));

    let mut query = format!(
        "recording:\"{}\" AND artist:\"{}\"",
        escape_lucene(title),
        escape_lucene(artist)
    );
    if let Some(album) = album.filter(|a| !a.is_empty()) {
        query.push_str(&format!(" AND release:\"{}\"", escape_lucene(album)));
    }

    let url = format!(
        "{MB_BASE}/recording?query={}&fmt=json&limit=1",
        urlencoding::encode(&query)
    );

    let body = http_get(&url)?;
    let parsed: MbSearch = serde_json::from_str(&body)
        .map_err(|e| Error::Other(anyhow::anyhow!("musicbrainz parse error: {e}")))?;

    let Some(rec) = parsed.recordings.into_iter().next() else {
        return Ok(None);
    };

    let release = rec.releases.as_ref().and_then(|r| r.first());
    let album_id = release.map(|r| r.id.clone());
    let year = release
        .and_then(|r| r.date.as_deref())
        .and_then(|d| d.get(0..4))
        .and_then(|y| y.parse().ok());

    let cover_art_url = match &album_id {
        Some(id) => fetch_cover_art_url(id).ok().flatten(),
        None => None,
    };

    let artist_name = rec
        .artist_credit
        .as_ref()
        .and_then(|c| c.first())
        .map(|c| c.name.clone());

    Ok(Some(LookupResult {
        title: Some(rec.title),
        artist: artist_name.clone(),
        album: release.and_then(|r| r.title.clone()),
        album_artist: artist_name,
        genre: None,
        year,
        musicbrainz_track_id: Some(rec.id),
        musicbrainz_album_id: album_id,
        cover_art_url,
    }))
}

/// Ask Cover Art Archive for the front image URL of a release.
pub fn fetch_cover_art_url(release_mbid: &str) -> Result<Option<String>> {
    std::thread::sleep(Duration::from_millis(200));
    let url = format!("{CAA_BASE}/release/{release_mbid}");
    let body = match http_get(&url) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    let parsed: CaaResponse = serde_json::from_str(&body)
        .map_err(|e| Error::Other(anyhow::anyhow!("cover art parse error: {e}")))?;
    let url = parsed
        .images
        .into_iter()
        .find(|img| img.front)
        .map(|img| img.image);
    Ok(url)
}

/// Download cover art bytes from a previously resolved URL.
pub fn download_cover_art(url: &str) -> Result<Vec<u8>> {
    http_get_bytes(url)
}

/// Resolved artist portrait from MusicBrainz → Wikidata → Wikimedia Commons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtistImage {
    pub mbid: String,
    pub image_url: String,
}

/// Look up a downloadable portrait for an artist by name.
///
/// Uses MusicBrainz artist search + URL relationships (preferring Wikidata),
/// then Wikidata property P18 (image) to build a Commons thumbnail URL.
pub fn lookup_artist_image(artist_name: &str) -> Result<Option<ArtistImage>> {
    std::thread::sleep(Duration::from_millis(1100));

    // Encode the full Lucene clause — bare `"` in the URL breaks ureq.
    let query = format!("artist:\"{}\"", escape_lucene(artist_name));
    let url = format!(
        "{MB_BASE}/artist?query={}&fmt=json&limit=1",
        urlencoding::encode(&query)
    );
    let body = http_get(&url)?;
    let parsed: MbArtistSearch = serde_json::from_str(&body)
        .map_err(|e| Error::Other(anyhow::anyhow!("musicbrainz artist parse error: {e}")))?;

    let Some(artist) = parsed.artists.into_iter().next() else {
        return Ok(None);
    };
    let mbid = artist.id;

    std::thread::sleep(Duration::from_millis(1100));
    let rel_url = format!("{MB_BASE}/artist/{mbid}?inc=url-rels&fmt=json");
    let rel_body = http_get(&rel_url)?;
    let rels: MbArtistRels = serde_json::from_str(&rel_body)
        .map_err(|e| Error::Other(anyhow::anyhow!("musicbrainz rels parse error: {e}")))?;

    // Prefer a direct image relation; otherwise Wikidata for P18.
    let mut wikidata_id = None;
    let mut direct_image = None;
    for rel in rels.relations.unwrap_or_default() {
        let Some(resource) = rel.url.as_ref().and_then(|u| u.resource.as_deref()) else {
            continue;
        };
        let type_ = rel.r#type.as_deref().unwrap_or("");
        if type_.eq_ignore_ascii_case("image") {
            direct_image = Some(resource.to_string());
        } else if type_.eq_ignore_ascii_case("wikidata") {
            if let Some(qid) = resource.rsplit('/').next() {
                if qid.starts_with('Q') {
                    wikidata_id = Some(qid.to_string());
                }
            }
        }
    }

    if let Some(image_url) = direct_image {
        return Ok(Some(ArtistImage { mbid, image_url }));
    }

    let Some(qid) = wikidata_id else {
        return Ok(None);
    };

    let wd_url = format!(
        "https://www.wikidata.org/wiki/Special:EntityData/{qid}.json"
    );
    let wd_body = http_get(&wd_url)?;
    let wd: WikidataEntity = serde_json::from_str(&wd_body)
        .map_err(|e| Error::Other(anyhow::anyhow!("wikidata parse error: {e}")))?;

    let Some(entities) = wd.entities else {
        return Ok(None);
    };
    let Some(entity) = entities.get(&qid) else {
        return Ok(None);
    };
    let Some(claims) = &entity.claims else {
        return Ok(None);
    };
    let Some(p18) = claims.get("P18").and_then(|v| v.first()) else {
        return Ok(None);
    };
    let Some(filename) = p18
        .mainsnak
        .as_ref()
        .and_then(|s| s.datavalue.as_ref())
        .and_then(|d| d.as_str())
    else {
        return Ok(None);
    };

    // Commons Special:FilePath serves a convenient redirectable image URL.
    let image_url = format!(
        "https://commons.wikimedia.org/wiki/Special:FilePath/{}?width=512",
        urlencoding::encode(filename)
    );
    Ok(Some(ArtistImage { mbid, image_url }))
}

/// Download an artist portrait and write it into `cache_dir`.
pub fn download_artist_image(
    artist_name: &str,
    cache_dir: &std::path::Path,
) -> Result<Option<(std::path::PathBuf, String)>> {
    let Some(found) = lookup_artist_image(artist_name)? else {
        return Ok(None);
    };
    let bytes = download_cover_art(&found.image_url)?;
    if bytes.is_empty() {
        return Ok(None);
    }
    std::fs::create_dir_all(cache_dir)?;
    let key = crate::artwork::artist_image_key(artist_name);
    let dest = cache_dir.join(format!("{key}.jpg"));
    std::fs::write(&dest, bytes)?;
    Ok(Some((dest, found.mbid)))
}

fn http_get(url: &str) -> Result<String> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .build()
        .new_agent();
    let response = agent
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .call()
        .map_err(|e| Error::Other(anyhow::anyhow!("http error: {e}")))?;
    response
        .into_body()
        .read_to_string()
        .map_err(|e| Error::Other(anyhow::anyhow!("http body error: {e}")))
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .new_agent();
    let response = agent
        .get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| Error::Other(anyhow::anyhow!("http error: {e}")))?;
    response
        .into_body()
        .read_to_vec()
        .map_err(|e| Error::Other(anyhow::anyhow!("http body error: {e}")))
}

fn escape_lucene(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' | '+' | '-' | '!' | '(' | ')' | ':' | '^' | '[' | ']' | '"' | '{' | '}' | '~'
            | '*' | '?' | '|' | '&' | '/' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

#[derive(Debug, Deserialize)]
struct MbSearch {
    #[serde(default)]
    recordings: Vec<MbRecording>,
}

#[derive(Debug, Deserialize)]
struct MbRecording {
    id: String,
    title: String,
    #[serde(rename = "artist-credit")]
    artist_credit: Option<Vec<MbArtistCredit>>,
    releases: Option<Vec<MbRelease>>,
}

#[derive(Debug, Deserialize)]
struct MbArtistCredit {
    name: String,
}

#[derive(Debug, Deserialize)]
struct MbRelease {
    id: String,
    title: Option<String>,
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CaaResponse {
    #[serde(default)]
    images: Vec<CaaImage>,
}

#[derive(Debug, Deserialize)]
struct CaaImage {
    image: String,
    #[serde(default)]
    front: bool,
}

#[derive(Debug, Deserialize)]
struct MbArtistSearch {
    #[serde(default)]
    artists: Vec<MbArtistHit>,
}

#[derive(Debug, Deserialize)]
struct MbArtistHit {
    id: String,
}

#[derive(Debug, Deserialize)]
struct MbArtistRels {
    #[serde(default)]
    relations: Option<Vec<MbRelation>>,
}

#[derive(Debug, Deserialize)]
struct MbRelation {
    #[serde(rename = "type")]
    r#type: Option<String>,
    url: Option<MbUrl>,
}

#[derive(Debug, Deserialize)]
struct MbUrl {
    resource: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WikidataEntity {
    entities: Option<std::collections::HashMap<String, WikidataItem>>,
}

#[derive(Debug, Deserialize)]
struct WikidataItem {
    claims: Option<std::collections::HashMap<String, Vec<WikidataClaim>>>,
}

#[derive(Debug, Deserialize)]
struct WikidataClaim {
    mainsnak: Option<WikidataSnak>,
}

#[derive(Debug, Deserialize)]
struct WikidataSnak {
    datavalue: Option<WikidataValue>,
}

#[derive(Debug, Deserialize)]
struct WikidataValue {
    /// P18 stores the Commons filename as a plain string.
    value: serde_json::Value,
}

impl WikidataValue {
    fn as_str(&self) -> Option<&str> {
        self.value.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_missing_only_preserves_existing() {
        let lookup = LookupResult {
            title: Some("New".into()),
            artist: Some("Fetched".into()),
            year: Some(2000),
            ..Default::default()
        };
        let mut meta = TrackMetadata {
            title: Some("Keep".into()),
            artist: None,
            year: Some(1999),
            ..Default::default()
        };
        lookup.apply_missing_only(&mut meta);
        assert_eq!(meta.title.as_deref(), Some("Keep"));
        assert_eq!(meta.artist.as_deref(), Some("Fetched"));
        assert_eq!(meta.year, Some(1999));
    }

    #[test]
    fn lucene_escape_quotes() {
        assert_eq!(escape_lucene(r#"AC/DC "Live""#), r#"AC\/DC \"Live\""#);
    }
}
