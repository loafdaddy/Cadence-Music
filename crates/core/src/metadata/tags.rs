//! Concrete `lofty`-backed tag reading and writing.

use std::path::Path;

use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::prelude::{Accessor, ItemKey};
use lofty::probe::Probe;
use lofty::tag::{Tag, TagExt, TagItem};

use crate::error::Result;
use crate::models::TrackMetadata;

/// Read all supported metadata from an audio file.
///
/// Missing tags are represented as `None`. The duration is read from the audio
/// properties, so it is available even for completely untagged files.
pub fn read_metadata(path: &Path) -> Result<TrackMetadata> {
    let tagged = Probe::open(path)?.read()?;

    let duration_ms = Some(tagged.properties().duration().as_millis() as u64);

    let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) else {
        // No tags at all: still return the duration we discovered.
        return Ok(TrackMetadata {
            duration_ms,
            ..Default::default()
        });
    };

    let string = |key: &ItemKey| tag.get_string(key).map(str::to_owned);

    Ok(TrackMetadata {
        title: tag.title().map(|c| c.into_owned()),
        artist: tag.artist().map(|c| c.into_owned()),
        album: tag.album().map(|c| c.into_owned()),
        album_artist: string(&ItemKey::AlbumArtist),
        composer: string(&ItemKey::Composer),
        genre: tag.genre().map(|c| c.into_owned()),
        year: tag.year().map(|y| y as i32),
        track_number: tag.track(),
        track_total: tag.track_total(),
        disc_number: tag.disk(),
        disc_total: tag.disk_total(),
        compilation: tag
            .get_string(&ItemKey::FlagCompilation)
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true")),
        duration_ms,
        musicbrainz_track_id: string(&ItemKey::MusicBrainzRecordingId),
        musicbrainz_album_id: string(&ItemKey::MusicBrainzReleaseId),
    })
}

/// Write metadata back to a file, preserving the file's existing tag type
/// where possible and creating an appropriate one otherwise.
///
/// This is destructive to the on-disk tags and must only be called after the
/// user has explicitly confirmed the edit.
pub fn write_metadata(path: &Path, meta: &TrackMetadata) -> Result<()> {
    let mut tagged = Probe::open(path)?.read()?;

    let tag_type = tagged
        .primary_tag()
        .map(Tag::tag_type)
        .unwrap_or_else(|| tagged.file_type().primary_tag_type());

    if tagged.primary_tag().is_none() {
        tagged.insert_tag(Tag::new(tag_type));
    }
    let tag = tagged
        .primary_tag_mut()
        .expect("tag was just inserted above");

    apply(tag, meta);

    tag.save_to_path(path, WriteOptions::default())?;
    Ok(())
}

/// Copy the fields of [`TrackMetadata`] into a `lofty` [`Tag`].
fn apply(tag: &mut Tag, meta: &TrackMetadata) {
    fn set(tag: &mut Tag, key: ItemKey, value: &Option<String>) {
        match value {
            Some(v) if !v.is_empty() => {
                tag.insert(TagItem::new(key, lofty::tag::ItemValue::Text(v.clone())));
            }
            _ => {
                tag.remove_key(&key);
            }
        }
    }

    set(tag, ItemKey::TrackTitle, &meta.title);
    set(tag, ItemKey::TrackArtist, &meta.artist);
    set(tag, ItemKey::AlbumTitle, &meta.album);
    set(tag, ItemKey::AlbumArtist, &meta.album_artist);
    set(tag, ItemKey::Composer, &meta.composer);
    set(tag, ItemKey::Genre, &meta.genre);

    match meta.year {
        Some(year) => tag.set_year(year as u32),
        None => tag.remove_year(),
    }
    match meta.track_number {
        Some(n) => tag.set_track(n),
        None => tag.remove_track(),
    }
    match meta.disc_number {
        Some(n) => tag.set_disk(n),
        None => tag.remove_disk(),
    }
}
