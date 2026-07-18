//! Database schema and forward-only migrations.
//!
//! Migrations are tracked with SQLite's `user_version` pragma. Each migration
//! bumps the version by one; [`LATEST_VERSION`] is the target for a fresh
//! database. Keeping migrations append-only makes upgrades deterministic.

/// The schema version this build of Cadence expects.
pub const LATEST_VERSION: i64 = 2;

/// The initial schema (version 1).
///
/// An FTS5 virtual table (`track_search`) mirrors the searchable text of each
/// track and is kept in sync explicitly by the query layer, giving instant
/// prefix search across very large libraries.
pub const V1: &str = r#"
CREATE TABLE artists (
    id        INTEGER PRIMARY KEY,
    name      TEXT NOT NULL,
    sort_name TEXT,
    UNIQUE (name)
);

CREATE TABLE albums (
    id              INTEGER PRIMARY KEY,
    name            TEXT NOT NULL,
    album_artist_id INTEGER REFERENCES artists(id) ON DELETE SET NULL,
    year            INTEGER,
    genre           TEXT,
    is_compilation  INTEGER NOT NULL DEFAULT 0,
    artwork_path    TEXT,
    UNIQUE (name, album_artist_id)
);

CREATE TABLE tracks (
    id              INTEGER PRIMARY KEY,
    path            TEXT NOT NULL UNIQUE,
    title           TEXT NOT NULL,
    album_id        INTEGER REFERENCES albums(id)  ON DELETE SET NULL,
    artist_id       INTEGER REFERENCES artists(id) ON DELETE SET NULL,
    album_artist_id INTEGER REFERENCES artists(id) ON DELETE SET NULL,
    composer        TEXT,
    genre           TEXT,
    year            INTEGER,
    track_number    INTEGER,
    disc_number     INTEGER,
    duration_ms     INTEGER,
    format          TEXT,
    file_size       INTEGER NOT NULL DEFAULT 0,
    modified_at     INTEGER NOT NULL DEFAULT 0,
    added_at        INTEGER NOT NULL DEFAULT 0,
    play_count      INTEGER NOT NULL DEFAULT 0,
    favorite        INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_tracks_album  ON tracks(album_id);
CREATE INDEX idx_tracks_artist ON tracks(artist_id);
CREATE INDEX idx_tracks_added  ON tracks(added_at);
CREATE INDEX idx_tracks_genre  ON tracks(genre);

CREATE TABLE library_folders (
    id       INTEGER PRIMARY KEY,
    path     TEXT NOT NULL UNIQUE,
    added_at INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE playlists (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE playlist_tracks (
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id    INTEGER NOT NULL REFERENCES tracks(id)    ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, position)
);

CREATE TABLE play_history (
    id        INTEGER PRIMARY KEY,
    track_id  INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    played_at INTEGER NOT NULL
);

CREATE INDEX idx_history_track ON play_history(track_id);
CREATE INDEX idx_history_time  ON play_history(played_at);

CREATE VIRTUAL TABLE track_search USING fts5(
    title,
    artist,
    album
);
"#;

/// Version 2 — cached artist portraits + MusicBrainz artist id.
pub const V2: &str = r#"
ALTER TABLE artists ADD COLUMN image_path TEXT;
ALTER TABLE artists ADD COLUMN mbid TEXT;
"#;
