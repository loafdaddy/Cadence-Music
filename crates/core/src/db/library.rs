//! Query and mutation methods for the music library.
//!
//! Everything here is a method on [`Database`]. Methods that change data take
//! `&self` because `rusqlite::Connection` supports interior mutability; the
//! type is still single-threaded by design (see [`super`]).

use std::path::{Path, PathBuf};

use rusqlite::{params, OptionalExtension, Row};

use crate::error::{Error, Result};
use crate::models::{
    Album, AlbumId, Artist, ArtistId, AudioFormat, Playlist, Track, TrackDisplay, TrackId,
    TrackMetadata,
};
use crate::scanner::DiscoveredFile;

use super::Database;

/// Sort orders for the songs view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SongSort {
    TitleAsc,
    ArtistAsc,
    AlbumAsc,
    RecentlyAdded,
    PlayCountDesc,
}

impl SongSort {
    fn order_clause(self) -> &'static str {
        match self {
            Self::TitleAsc => "t.title COLLATE NOCASE ASC",
            Self::ArtistAsc => "ar.name COLLATE NOCASE ASC, t.title COLLATE NOCASE ASC",
            Self::AlbumAsc => "al.name COLLATE NOCASE ASC, t.disc_number, t.track_number",
            Self::RecentlyAdded => "t.added_at DESC, t.id DESC",
            Self::PlayCountDesc => "t.play_count DESC, t.title COLLATE NOCASE ASC",
        }
    }
}

/// The full column list for `tracks`, aliased `t`, in a stable order matched by
/// [`map_track`].
const TRACK_COLUMNS: &str = "\
    t.id, t.path, t.title, t.album_id, t.artist_id, t.album_artist_id, \
    t.composer, t.genre, t.year, t.track_number, t.disc_number, t.duration_ms, \
    t.format, t.file_size, t.modified_at, t.added_at, t.play_count, t.favorite";

impl Database {
    // -- Library folders ---------------------------------------------------

    /// Register a music folder. Returns `true` if it was newly added.
    pub fn add_library_folder(&self, path: &Path) -> Result<bool> {
        let changed = self.conn.execute(
            "INSERT OR IGNORE INTO library_folders (path, added_at) VALUES (?1, ?2)",
            params![path.to_string_lossy(), now()],
        )?;
        Ok(changed > 0)
    }

    /// Remove a music folder registration (does not delete tracks).
    pub fn remove_library_folder(&self, path: &Path) -> Result<()> {
        self.conn.execute(
            "DELETE FROM library_folders WHERE path = ?1",
            params![path.to_string_lossy()],
        )?;
        Ok(())
    }

    /// List all registered music folders.
    pub fn library_folders(&self) -> Result<Vec<PathBuf>> {
        let mut stmt = self
            .conn
            .prepare("SELECT path FROM library_folders ORDER BY path")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows
            .filter_map(std::result::Result::ok)
            .map(PathBuf::from)
            .collect())
    }

    // -- Artists & albums --------------------------------------------------

    /// Look up an artist id by name, creating the row if needed.
    pub fn get_or_create_artist(&self, name: &str) -> Result<ArtistId> {
        self.conn.execute(
            "INSERT OR IGNORE INTO artists (name) VALUES (?1)",
            params![name],
        )?;
        let id: i64 = self.conn.query_row(
            "SELECT id FROM artists WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )?;
        Ok(ArtistId(id))
    }

    /// Look up an album id, creating the row if needed.
    pub fn get_or_create_album(
        &self,
        name: &str,
        album_artist_id: Option<ArtistId>,
        year: Option<i32>,
        genre: Option<&str>,
        is_compilation: bool,
    ) -> Result<AlbumId> {
        let aa = album_artist_id.map(i64::from);
        self.conn.execute(
            "INSERT OR IGNORE INTO albums (name, album_artist_id, year, genre, is_compilation) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, aa, year, genre, is_compilation as i64],
        )?;
        // `album_artist_id` is part of the uniqueness key and may be NULL, so
        // the lookup uses `IS` semantics rather than `=`.
        let id: i64 = self.conn.query_row(
            "SELECT id FROM albums WHERE name = ?1 AND album_artist_id IS ?2",
            params![name, aa],
            |row| row.get(0),
        )?;
        Ok(AlbumId(id))
    }

    /// List every artist with aggregate counts, ordered by name.
    pub fn list_artists(&self) -> Result<Vec<Artist>> {
        let mut stmt = self.conn.prepare(
            "SELECT ar.id, ar.name, \
                    (SELECT COUNT(DISTINCT al.id) FROM albums al WHERE al.album_artist_id = ar.id), \
                    (SELECT COUNT(*) FROM tracks t WHERE t.artist_id = ar.id OR t.album_artist_id = ar.id), \
                    ar.image_path, ar.mbid \
             FROM artists ar \
             ORDER BY ar.name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], map_artist)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// List every album with its track count, ordered by album-artist then year.
    pub fn list_albums(&self) -> Result<Vec<Album>> {
        let mut stmt = self.conn.prepare(
            "SELECT al.id, al.name, al.album_artist_id, al.year, al.genre, al.is_compilation, \
                    al.artwork_path, (SELECT COUNT(*) FROM tracks t WHERE t.album_id = al.id) \
             FROM albums al \
             ORDER BY al.name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], map_album)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Fetch a single artist by id.
    pub fn artist(&self, id: ArtistId) -> Result<Artist> {
        self.conn
            .query_row(
                "SELECT ar.id, ar.name, \
                        (SELECT COUNT(DISTINCT al.id) FROM albums al WHERE al.album_artist_id = ar.id), \
                        (SELECT COUNT(*) FROM tracks t WHERE t.artist_id = ar.id OR t.album_artist_id = ar.id), \
                        ar.image_path, ar.mbid \
                 FROM artists ar WHERE ar.id = ?1",
                params![i64::from(id)],
                map_artist,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Error::NotFound(format!("artist {id}")),
                other => Error::Database(other),
            })
    }

    /// Persist a cached portrait path (and optional MBID) for an artist.
    pub fn set_artist_image(
        &self,
        artist: ArtistId,
        image_path: &Path,
        mbid: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE artists SET image_path = ?2, mbid = COALESCE(?3, mbid) WHERE id = ?1",
            params![
                i64::from(artist),
                image_path.to_string_lossy(),
                mbid
            ],
        )?;
        Ok(())
    }

    /// Artists that still need a portrait download.
    pub fn artists_missing_image(&self) -> Result<Vec<Artist>> {
        let mut stmt = self.conn.prepare(
            "SELECT ar.id, ar.name, \
                    (SELECT COUNT(DISTINCT al.id) FROM albums al WHERE al.album_artist_id = ar.id), \
                    (SELECT COUNT(*) FROM tracks t WHERE t.artist_id = ar.id OR t.album_artist_id = ar.id), \
                    ar.image_path, ar.mbid \
             FROM artists ar \
             WHERE ar.image_path IS NULL OR ar.image_path = '' \
             ORDER BY ar.name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], map_artist)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Fetch a single album by id.
    pub fn album(&self, id: AlbumId) -> Result<Album> {
        self.conn
            .query_row(
                "SELECT al.id, al.name, al.album_artist_id, al.year, al.genre, al.is_compilation, \
                        al.artwork_path, (SELECT COUNT(*) FROM tracks t WHERE t.album_id = al.id) \
                 FROM albums al WHERE al.id = ?1",
                params![i64::from(id)],
                map_album,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Error::NotFound(format!("album {id}")),
                other => Error::Database(other),
            })
    }

    /// Resolve an artist's display name, if present.
    pub fn artist_name(&self, id: ArtistId) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT name FROM artists WHERE id = ?1",
                params![i64::from(id)],
                |row| row.get(0),
            )
            .optional()
            .map_err(Error::from)
    }

    /// List albums belonging to an artist (as album artist).
    pub fn albums_by_artist(&self, artist: ArtistId) -> Result<Vec<Album>> {
        let mut stmt = self.conn.prepare(
            "SELECT al.id, al.name, al.album_artist_id, al.year, al.genre, al.is_compilation, \
                    al.artwork_path, (SELECT COUNT(*) FROM tracks t WHERE t.album_id = al.id) \
             FROM albums al \
             WHERE al.album_artist_id = ?1 \
             ORDER BY al.year, al.name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map(params![i64::from(artist)], map_album)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Persist the album artwork cache path.
    pub fn set_album_artwork(&self, album: AlbumId, artwork_path: &Path) -> Result<()> {
        self.conn.execute(
            "UPDATE albums SET artwork_path = ?2 WHERE id = ?1",
            params![i64::from(album), artwork_path.to_string_lossy()],
        )?;
        Ok(())
    }

    // -- Tracks ------------------------------------------------------------

    /// Whether a file at `path` needs (re)scanning given its current size and
    /// modification time. Returns `true` if the file is unknown or changed.
    pub fn track_needs_rescan(
        &self,
        path: &Path,
        file_size: u64,
        modified_at: i64,
    ) -> Result<bool> {
        let existing: Option<(i64, i64)> = self
            .conn
            .query_row(
                "SELECT file_size, modified_at FROM tracks WHERE path = ?1",
                params![path.to_string_lossy()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        Ok(match existing {
            Some((size, mtime)) => size != file_size as i64 || mtime != modified_at,
            None => true,
        })
    }

    /// Insert or update a track from a scan result, resolving artist/album rows.
    ///
    /// `play_count` and `favorite` are preserved across updates so a rescan
    /// never resets a user's listening history.
    pub fn upsert_track(&self, file: &DiscoveredFile, meta: &TrackMetadata) -> Result<TrackId> {
        let title = meta
            .title
            .clone()
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| title_from_path(&file.path));

        let artist_name = meta.artist.as_deref().filter(|s| !s.is_empty());
        let album_artist_name = meta
            .album_artist
            .as_deref()
            .or(artist_name)
            .filter(|s| !s.is_empty());

        let artist_id = match artist_name {
            Some(name) => Some(self.get_or_create_artist(name)?),
            None => None,
        };
        let album_artist_id = match album_artist_name {
            Some(name) => Some(self.get_or_create_artist(name)?),
            None => None,
        };
        let album_id = match meta.album.as_deref().filter(|s| !s.is_empty()) {
            Some(name) => Some(self.get_or_create_album(
                name,
                album_artist_id,
                meta.year,
                meta.genre.as_deref(),
                meta.compilation,
            )?),
            None => None,
        };

        let track_id: i64 = self.conn.query_row(
            "INSERT INTO tracks (\
                 path, title, album_id, artist_id, album_artist_id, composer, genre, year, \
                 track_number, disc_number, duration_ms, format, file_size, modified_at, added_at\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15) \
             ON CONFLICT(path) DO UPDATE SET \
                 title = excluded.title, album_id = excluded.album_id, \
                 artist_id = excluded.artist_id, album_artist_id = excluded.album_artist_id, \
                 composer = excluded.composer, genre = excluded.genre, year = excluded.year, \
                 track_number = excluded.track_number, disc_number = excluded.disc_number, \
                 duration_ms = excluded.duration_ms, format = excluded.format, \
                 file_size = excluded.file_size, modified_at = excluded.modified_at \
             RETURNING id",
            params![
                file.path.to_string_lossy(),
                title,
                album_id.map(i64::from),
                artist_id.map(i64::from),
                album_artist_id.map(i64::from),
                meta.composer,
                meta.genre,
                meta.year,
                meta.track_number,
                meta.disc_number,
                meta.duration_ms.map(|d| d as i64),
                file.format.extension(),
                file.file_size as i64,
                file.modified_at,
                now(),
            ],
            |row| row.get(0),
        )?;

        self.reindex_search(
            track_id,
            &title,
            artist_name.or(album_artist_name),
            meta.album.as_deref(),
        )?;

        Ok(TrackId(track_id))
    }

    /// Remove a track (and its search/playlist rows) by file path.
    pub fn remove_track_by_path(&self, path: &Path) -> Result<()> {
        if let Some(id) = self.track_id_by_path(path)? {
            self.remove_track(id)?;
        }
        Ok(())
    }

    /// Remove a track from the library database (not from disk).
    pub fn remove_track(&self, id: TrackId) -> Result<()> {
        self.conn
            .execute("DELETE FROM track_search WHERE rowid = ?1", params![i64::from(id)])?;
        self.conn
            .execute("DELETE FROM tracks WHERE id = ?1", params![i64::from(id)])?;
        self.prune_empty_albums()?;
        Ok(())
    }

    /// Remove every track belonging to an album (library only, not disk).
    pub fn remove_album(&self, id: AlbumId) -> Result<u32> {
        let ids: Vec<i64> = {
            let mut stmt = self
                .conn
                .prepare("SELECT id FROM tracks WHERE album_id = ?1")?;
            let rows = stmt.query_map(params![i64::from(id)], |row| row.get(0))?;
            rows.filter_map(std::result::Result::ok).collect()
        };
        let mut removed = 0u32;
        for track_id in ids {
            self.remove_track(TrackId(track_id))?;
            removed += 1;
        }
        // Album row may already be pruned; delete explicitly if still present.
        let _ = self
            .conn
            .execute("DELETE FROM albums WHERE id = ?1", params![i64::from(id)]);
        Ok(removed)
    }

    fn prune_empty_albums(&self) -> Result<()> {
        self.conn.execute(
            "DELETE FROM albums WHERE id NOT IN \
             (SELECT DISTINCT album_id FROM tracks WHERE album_id IS NOT NULL)",
            [],
        )?;
        Ok(())
    }

    /// Look up a track id by absolute path.
    pub fn track_id_by_path(&self, path: &Path) -> Result<Option<TrackId>> {
        let id = self
            .conn
            .query_row(
                "SELECT id FROM tracks WHERE path = ?1",
                params![path.to_string_lossy()],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        Ok(id.map(TrackId))
    }

    /// Fetch a single track by id.
    pub fn track(&self, id: TrackId) -> Result<Track> {
        let sql = format!("SELECT {TRACK_COLUMNS} FROM tracks t WHERE t.id = ?1");
        self.conn
            .query_row(&sql, params![i64::from(id)], map_track)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Error::NotFound(format!("track {id}")),
                other => Error::Database(other),
            })
    }

    /// List songs across the whole library with the given sort order.
    pub fn list_songs(&self, sort: SongSort) -> Result<Vec<Track>> {
        let sql = format!(
            "SELECT {TRACK_COLUMNS} FROM tracks t \
             LEFT JOIN artists ar ON ar.id = t.artist_id \
             LEFT JOIN albums al ON al.id = t.album_id \
             ORDER BY {}",
            sort.order_clause()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], map_track)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// List the tracks of an album in disc/track order.
    pub fn tracks_by_album(&self, album: AlbumId) -> Result<Vec<Track>> {
        Ok(self
            .tracks_by_album_display(album)?
            .into_iter()
            .map(|d| d.track)
            .collect())
    }

    /// Album tracks with joined artist/album display fields.
    pub fn tracks_by_album_display(&self, album: AlbumId) -> Result<Vec<TrackDisplay>> {
        let sql = format!(
            "SELECT {TRACK_COLUMNS}, \
                    COALESCE(ar.name, ''), COALESCE(al.name, ''), al.artwork_path \
             FROM tracks t \
             LEFT JOIN artists ar ON ar.id = t.artist_id \
             LEFT JOIN albums al ON al.id = t.album_id \
             WHERE t.album_id = ?1 \
             ORDER BY t.disc_number, t.track_number, t.title COLLATE NOCASE"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![i64::from(album)], map_track_display)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// List tracks credited to an artist.
    pub fn tracks_by_artist(&self, artist: ArtistId) -> Result<Vec<Track>> {
        let sql = format!(
            "SELECT {TRACK_COLUMNS} FROM tracks t \
             WHERE t.artist_id = ?1 OR t.album_artist_id = ?1 \
             ORDER BY t.title COLLATE NOCASE"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![i64::from(artist)], map_track)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Tracks for an artist that are not attached to any album (singles / loose files).
    pub fn singles_by_artist_display(&self, artist: ArtistId) -> Result<Vec<TrackDisplay>> {
        let sql = format!(
            "SELECT {TRACK_COLUMNS}, \
                    COALESCE(ar.name, ''), COALESCE(al.name, ''), al.artwork_path \
             FROM tracks t \
             LEFT JOIN artists ar ON ar.id = t.artist_id \
             LEFT JOIN albums al ON al.id = t.album_id \
             WHERE (t.artist_id = ?1 OR t.album_artist_id = ?1) \
               AND t.album_id IS NULL \
             ORDER BY t.title COLLATE NOCASE"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![i64::from(artist)], map_track_display)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Update a track's on-disk path after an organization move.
    pub fn update_track_path(&self, id: TrackId, new_path: &Path) -> Result<()> {
        self.conn.execute(
            "UPDATE tracks SET path = ?2 WHERE id = ?1",
            params![i64::from(id), new_path.to_string_lossy()],
        )?;
        Ok(())
    }

    /// Paginated song listing for large libraries.
    pub fn list_songs_page(
        &self,
        sort: SongSort,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Track>> {
        Ok(self
            .list_songs_display(sort, offset, limit)?
            .into_iter()
            .map(|d| d.track)
            .collect())
    }

    /// Paginated songs with artist/album names and artwork for list UIs.
    pub fn list_songs_display(
        &self,
        sort: SongSort,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<TrackDisplay>> {
        let sql = format!(
            "SELECT {TRACK_COLUMNS}, \
                    COALESCE(ar.name, ''), COALESCE(al.name, ''), al.artwork_path \
             FROM tracks t \
             LEFT JOIN artists ar ON ar.id = t.artist_id \
             LEFT JOIN albums al ON al.id = t.album_id \
             ORDER BY {} LIMIT ?1 OFFSET ?2",
            sort.order_clause()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![limit as i64, offset as i64], map_track_display)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Search with display fields for rich results.
    pub fn search_display(&self, query: &str, limit: usize) -> Result<Vec<TrackDisplay>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        let fts_query = build_fts_query(trimmed);
        let sql = format!(
            "SELECT {TRACK_COLUMNS}, \
                    COALESCE(ar.name, ''), COALESCE(al.name, ''), al.artwork_path \
             FROM track_search s \
             JOIN tracks t ON t.id = s.rowid \
             LEFT JOIN artists ar ON ar.id = t.artist_id \
             LEFT JOIN albums al ON al.id = t.album_id \
             WHERE track_search MATCH ?1 \
             ORDER BY rank \
             LIMIT ?2"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![fts_query, limit as i64], map_track_display)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Album display name for an id, if present.
    pub fn album_name(&self, id: AlbumId) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT name FROM albums WHERE id = ?1",
                params![i64::from(id)],
                |row| row.get(0),
            )
            .optional()
            .map_err(Error::from)
    }

    /// Total duration of an artist's tracks in milliseconds.
    pub fn artist_duration_ms(&self, artist: ArtistId) -> Result<u64> {
        let n: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(duration_ms), 0) FROM tracks \
             WHERE artist_id = ?1 OR album_artist_id = ?1",
            params![i64::from(artist)],
            |row| row.get(0),
        )?;
        Ok(n as u64)
    }

    /// Total duration of an album's tracks in milliseconds.
    pub fn album_duration_ms(&self, album: AlbumId) -> Result<u64> {
        let n: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(duration_ms), 0) FROM tracks WHERE album_id = ?1",
            params![i64::from(album)],
            |row| row.get(0),
        )?;
        Ok(n as u64)
    }

    /// Total number of tracks in the library.
    pub fn track_count(&self) -> Result<u64> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))?;
        Ok(n as u64)
    }

    /// Full-text search across title, artist and album with prefix matching.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Track>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        let fts_query = build_fts_query(trimmed);
        let sql = format!(
            "SELECT {TRACK_COLUMNS} FROM track_search s \
             JOIN tracks t ON t.id = s.rowid \
             WHERE track_search MATCH ?1 \
             ORDER BY rank \
             LIMIT ?2"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![fts_query, limit as i64], map_track)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// The most recently added tracks.
    pub fn recently_added(&self, limit: usize) -> Result<Vec<Track>> {
        let sql = format!(
            "SELECT {TRACK_COLUMNS} FROM tracks t ORDER BY t.added_at DESC, t.id DESC LIMIT ?1"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![limit as i64], map_track)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Distinct genres present in the library, alphabetically.
    pub fn genres(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT genre FROM tracks WHERE genre IS NOT NULL AND genre <> '' \
             ORDER BY genre COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Distinct release years present in the library, newest first.
    pub fn years(&self) -> Result<Vec<i32>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT year FROM tracks WHERE year IS NOT NULL ORDER BY year DESC",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, i32>(0))?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    // -- Favorites & play tracking -----------------------------------------

    /// Toggle or set a track's favorite flag.
    pub fn set_favorite(&self, track: TrackId, favorite: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE tracks SET favorite = ?2 WHERE id = ?1",
            params![i64::from(track), favorite as i64],
        )?;
        Ok(())
    }

    /// Favourite tracks, most recently added first.
    pub fn favorites(&self) -> Result<Vec<Track>> {
        Ok(self
            .favorites_display()?
            .into_iter()
            .map(|d| d.track)
            .collect())
    }

    /// Favourite tracks with joined artist/album/artwork for rich rows.
    pub fn favorites_display(&self) -> Result<Vec<TrackDisplay>> {
        let sql = format!(
            "SELECT {TRACK_COLUMNS}, \
                    COALESCE(ar.name, ''), COALESCE(al.name, ''), al.artwork_path \
             FROM tracks t \
             LEFT JOIN artists ar ON ar.id = t.artist_id \
             LEFT JOIN albums al ON al.id = t.album_id \
             WHERE t.favorite = 1 \
             ORDER BY t.added_at DESC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], map_track_display)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Record a play: bump the counter and append to history.
    pub fn record_play(&self, track: TrackId) -> Result<()> {
        self.conn.execute(
            "UPDATE tracks SET play_count = play_count + 1 WHERE id = ?1",
            params![i64::from(track)],
        )?;
        self.conn.execute(
            "INSERT INTO play_history (track_id, played_at) VALUES (?1, ?2)",
            params![i64::from(track), now()],
        )?;
        Ok(())
    }

    /// Recently played distinct tracks, most recent first.
    pub fn recently_played(&self, limit: usize) -> Result<Vec<Track>> {
        Ok(self
            .recently_played_display(limit)?
            .into_iter()
            .map(|d| d.track)
            .collect())
    }

    /// Recently played with display fields.
    pub fn recently_played_display(&self, limit: usize) -> Result<Vec<TrackDisplay>> {
        let sql = format!(
            "SELECT {TRACK_COLUMNS}, \
                    COALESCE(ar.name, ''), COALESCE(al.name, ''), al.artwork_path \
             FROM tracks t \
             LEFT JOIN artists ar ON ar.id = t.artist_id \
             LEFT JOIN albums al ON al.id = t.album_id \
             JOIN (SELECT track_id, MAX(played_at) AS last FROM play_history GROUP BY track_id) h \
               ON h.track_id = t.id \
             ORDER BY h.last DESC LIMIT ?1"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![limit as i64], map_track_display)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Recently added with display fields.
    pub fn recently_added_display(&self, limit: usize) -> Result<Vec<TrackDisplay>> {
        let sql = format!(
            "SELECT {TRACK_COLUMNS}, \
                    COALESCE(ar.name, ''), COALESCE(al.name, ''), al.artwork_path \
             FROM tracks t \
             LEFT JOIN artists ar ON ar.id = t.artist_id \
             LEFT JOIN albums al ON al.id = t.album_id \
             ORDER BY t.added_at DESC, t.id DESC LIMIT ?1"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![limit as i64], map_track_display)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// Most played tracks.
    pub fn most_played(&self, limit: usize) -> Result<Vec<Track>> {
        let sql = format!(
            "SELECT {TRACK_COLUMNS} FROM tracks t WHERE t.play_count > 0 \
             ORDER BY t.play_count DESC LIMIT ?1"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![limit as i64], map_track)?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    // -- Playlists ---------------------------------------------------------

    /// Create a playlist and return its id.
    pub fn create_playlist(&self, name: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO playlists (name, created_at) VALUES (?1, ?2)",
            params![name, now()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Delete a playlist and its membership rows.
    pub fn delete_playlist(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Append tracks to the end of a playlist, preserving order.
    pub fn add_to_playlist(&self, playlist: i64, tracks: &[TrackId]) -> Result<()> {
        let start: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist],
            |row| row.get(0),
        )?;
        let mut stmt = self.conn.prepare(
            "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
        )?;
        for (offset, track) in tracks.iter().enumerate() {
            stmt.execute(params![playlist, i64::from(*track), start + offset as i64])?;
        }
        Ok(())
    }

    /// List all playlists with their ordered track ids.
    pub fn playlists(&self) -> Result<Vec<Playlist>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM playlists ORDER BY name COLLATE NOCASE")?;
        let bare = stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(std::result::Result::ok)
            .collect::<Vec<_>>();

        let mut playlists = Vec::with_capacity(bare.len());
        for (id, name) in bare {
            let mut track_stmt = self.conn.prepare(
                "SELECT track_id FROM playlist_tracks WHERE playlist_id = ?1 ORDER BY position",
            )?;
            let track_ids = track_stmt
                .query_map(params![id], |row| row.get::<_, i64>(0))?
                .filter_map(std::result::Result::ok)
                .map(TrackId)
                .collect();
            playlists.push(Playlist {
                id,
                name,
                track_ids,
            });
        }
        Ok(playlists)
    }

    // -- Internal helpers --------------------------------------------------

    /// Replace a track's row in the FTS index.
    fn reindex_search(
        &self,
        track_id: i64,
        title: &str,
        artist: Option<&str>,
        album: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "DELETE FROM track_search WHERE rowid = ?1",
            params![track_id],
        )?;
        self.conn.execute(
            "INSERT INTO track_search (rowid, title, artist, album) VALUES (?1, ?2, ?3, ?4)",
            params![track_id, title, artist.unwrap_or(""), album.unwrap_or("")],
        )?;
        Ok(())
    }
}

/// Map a `tracks` row (in [`TRACK_COLUMNS`] order) to a [`Track`].
fn map_track(row: &Row<'_>) -> rusqlite::Result<Track> {
    let format: Option<String> = row.get(12)?;
    Ok(Track {
        id: TrackId(row.get(0)?),
        path: PathBuf::from(row.get::<_, String>(1)?),
        title: row.get(2)?,
        album_id: row.get::<_, Option<i64>>(3)?.map(AlbumId),
        artist_id: row.get::<_, Option<i64>>(4)?.map(ArtistId),
        album_artist_id: row.get::<_, Option<i64>>(5)?.map(ArtistId),
        composer: row.get(6)?,
        genre: row.get(7)?,
        year: row.get(8)?,
        track_number: row.get::<_, Option<i64>>(9)?.map(|n| n as u32),
        disc_number: row.get::<_, Option<i64>>(10)?.map(|n| n as u32),
        duration_ms: row.get::<_, Option<i64>>(11)?.map(|d| d as u64),
        format: format.as_deref().and_then(AudioFormat::from_extension),
        file_size: row.get::<_, i64>(13)? as u64,
        modified_at: row.get(14)?,
        added_at: row.get(15)?,
        play_count: row.get::<_, i64>(16)? as u32,
        favorite: row.get::<_, i64>(17)? != 0,
    })
}

/// Map a track row plus joined display columns.
fn map_track_display(row: &Row<'_>) -> rusqlite::Result<TrackDisplay> {
    Ok(TrackDisplay {
        track: map_track(row)?,
        artist_name: row.get(18)?,
        album_name: row.get(19)?,
        artwork_path: row.get::<_, Option<String>>(20)?.map(PathBuf::from),
    })
}

/// Map an `albums` aggregate row to an [`Album`].
fn map_album(row: &Row<'_>) -> rusqlite::Result<Album> {
    Ok(Album {
        id: AlbumId(row.get(0)?),
        name: row.get(1)?,
        album_artist_id: row.get::<_, Option<i64>>(2)?.map(ArtistId),
        year: row.get(3)?,
        genre: row.get(4)?,
        is_compilation: row.get::<_, i64>(5)? != 0,
        artwork_path: row.get::<_, Option<String>>(6)?.map(PathBuf::from),
        track_count: row.get::<_, i64>(7)? as u32,
    })
}

fn map_artist(row: &Row<'_>) -> rusqlite::Result<Artist> {
    Ok(Artist {
        id: ArtistId(row.get(0)?),
        name: row.get(1)?,
        album_count: row.get::<_, i64>(2)? as u32,
        track_count: row.get::<_, i64>(3)? as u32,
        image_path: row.get::<_, Option<String>>(4)?.map(PathBuf::from),
        mbid: row.get(5)?,
    })
}

/// Current Unix time in seconds.
fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Derive a display title from a file path when the file has no title tag.
fn title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Title")
        .to_owned()
}

/// Turn a user's free-text query into a safe FTS5 prefix query.
///
/// Each whitespace-separated word becomes a quoted prefix term so punctuation
/// in the query can never be interpreted as FTS syntax.
fn build_fts_query(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let escaped = word.replace('"', "\"\"");
            format!("\"{escaped}\"*")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AudioFormat;

    fn discovered(path: &str) -> DiscoveredFile {
        DiscoveredFile {
            path: PathBuf::from(path),
            format: AudioFormat::Flac,
            file_size: 123,
            modified_at: 1000,
        }
    }

    fn meta(title: &str, artist: &str, album: &str) -> TrackMetadata {
        TrackMetadata {
            title: Some(title.to_owned()),
            artist: Some(artist.to_owned()),
            album: Some(album.to_owned()),
            album_artist: Some(artist.to_owned()),
            genre: Some("Jazz".to_owned()),
            year: Some(1959),
            track_number: Some(1),
            ..Default::default()
        }
    }

    #[test]
    fn upsert_preserves_id_and_history() {
        let db = Database::open_in_memory().unwrap();
        let f = discovered("/music/a.flac");
        let id1 = db
            .upsert_track(&f, &meta("So What", "Miles Davis", "Kind of Blue"))
            .unwrap();
        db.record_play(id1).unwrap();

        // Re-scan the same path with edited metadata.
        let id2 = db
            .upsert_track(&f, &meta("So What (Take 1)", "Miles Davis", "Kind of Blue"))
            .unwrap();
        assert_eq!(id1, id2, "id must be stable across rescans");

        let track = db.track(id1).unwrap();
        assert_eq!(track.title, "So What (Take 1)");
        assert_eq!(track.play_count, 1, "play history must survive rescans");
    }

    #[test]
    fn search_matches_prefix() {
        let db = Database::open_in_memory().unwrap();
        db.upsert_track(
            &discovered("/m/1.flac"),
            &meta("So What", "Miles Davis", "Kind of Blue"),
        )
        .unwrap();
        db.upsert_track(
            &discovered("/m/2.flac"),
            &meta("Flamenco Sketches", "Miles Davis", "Kind of Blue"),
        )
        .unwrap();

        let hits = db.search("mil", 50).unwrap();
        assert_eq!(hits.len(), 2);

        let hits = db.search("flam", 50).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Flamenco Sketches");
    }

    #[test]
    fn favorites_and_playlists() {
        let db = Database::open_in_memory().unwrap();
        let id = db
            .upsert_track(&discovered("/m/1.flac"), &meta("T", "A", "Al"))
            .unwrap();
        db.set_favorite(id, true).unwrap();
        assert_eq!(db.favorites().unwrap().len(), 1);

        let pl = db.create_playlist("Roadtrip").unwrap();
        db.add_to_playlist(pl, &[id]).unwrap();
        let lists = db.playlists().unwrap();
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].track_ids, vec![id]);
    }
}
