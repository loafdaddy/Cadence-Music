//! Dedicated worker thread that owns the SQLite database.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use cadence_core::artwork;
use cadence_core::db::{Database, SongSort};
use cadence_core::metadata::{self, write_metadata};
use cadence_core::models::{
    Album, AlbumId, Artist, ArtistId, Playlist, Track, TrackDisplay, TrackId, TrackMetadata,
};
use cadence_core::organization::{OrganizationPlan, Template, UndoLog};
use cadence_core::paths::{artwork_cache_dir, library_db_path};
use cadence_core::scanner::{self, DiscoveredFile, LibraryWatcher, WatchEvent};
use cadence_core::Result;
use glib::ControlFlow;

/// Events pushed from the worker to the UI thread.
#[derive(Debug, Clone)]
pub enum LibraryEvent {
    ScanProgress { done: usize, total: usize },
    ScanFinished { imported: usize },
    /// Progress while filling missing metadata / artist photos.
    LookupProgress {
        phase: String,
        done: usize,
        total: usize,
    },
    LibraryChanged,
    Error(String),
}

/// Summary from a library-wide “Find Missing Metadata” pass.
#[derive(Debug, Clone, Default)]
pub struct LookupSummary {
    pub albums_scanned: u32,
    pub artwork_updated: u32,
    pub genres_fixed: u32,
    pub metadata_updated: u32,
    pub artist_photos: u32,
    pub needs_review: u32,
    pub network_lookups: u32,
}

enum Command {
    AddFolder {
        path: PathBuf,
        reply: SyncSender<Result<()>>,
    },
    RemoveFolder {
        path: PathBuf,
        reply: SyncSender<Result<()>>,
    },
    ListFolders {
        reply: SyncSender<Result<Vec<PathBuf>>>,
    },
    ScanAll {
        reply: SyncSender<Result<usize>>,
    },
    ListArtists {
        reply: SyncSender<Result<Vec<Artist>>>,
    },
    ListAlbums {
        reply: SyncSender<Result<Vec<Album>>>,
    },
    AlbumsByArtist {
        artist: ArtistId,
        reply: SyncSender<Result<Vec<Album>>>,
    },
    TracksByAlbum {
        album: AlbumId,
        reply: SyncSender<Result<Vec<Track>>>,
    },
    TracksByArtist {
        artist: ArtistId,
        reply: SyncSender<Result<Vec<Track>>>,
    },
    SinglesByArtistDisplay {
        artist: ArtistId,
        reply: SyncSender<Result<Vec<TrackDisplay>>>,
    },
    ListSongsPage {
        sort: SongSort,
        offset: usize,
        limit: usize,
        reply: SyncSender<Result<Vec<Track>>>,
    },
    TrackCount {
        reply: SyncSender<Result<u64>>,
    },
    Search {
        query: String,
        reply: SyncSender<Result<Vec<Track>>>,
    },
    SearchDisplay {
        query: String,
        reply: SyncSender<Result<Vec<TrackDisplay>>>,
    },
    ListSongsDisplay {
        sort: SongSort,
        offset: usize,
        limit: usize,
        reply: SyncSender<Result<Vec<TrackDisplay>>>,
    },
    TracksByAlbumDisplay {
        album: AlbumId,
        reply: SyncSender<Result<Vec<TrackDisplay>>>,
    },
    ArtistDuration {
        artist: ArtistId,
        reply: SyncSender<Result<u64>>,
    },
    GetArtist {
        id: ArtistId,
        reply: SyncSender<Result<Artist>>,
    },
    GetTrack {
        id: TrackId,
        reply: SyncSender<Result<Track>>,
    },
    ArtistName {
        id: ArtistId,
        reply: SyncSender<Result<Option<String>>>,
    },
    Album {
        id: AlbumId,
        reply: SyncSender<Result<Album>>,
    },
    SetFavorite {
        id: TrackId,
        favorite: bool,
        reply: SyncSender<Result<()>>,
    },
    RecordPlay {
        id: TrackId,
        reply: SyncSender<Result<()>>,
    },
    Favorites {
        reply: SyncSender<Result<Vec<Track>>>,
    },
    FavoritesDisplay {
        reply: SyncSender<Result<Vec<TrackDisplay>>>,
    },
    RecentlyAdded {
        limit: usize,
        reply: SyncSender<Result<Vec<Track>>>,
    },
    RecentlyAddedDisplay {
        limit: usize,
        reply: SyncSender<Result<Vec<TrackDisplay>>>,
    },
    RecentlyPlayed {
        limit: usize,
        reply: SyncSender<Result<Vec<Track>>>,
    },
    RecentlyPlayedDisplay {
        limit: usize,
        reply: SyncSender<Result<Vec<TrackDisplay>>>,
    },
    Playlists {
        reply: SyncSender<Result<Vec<Playlist>>>,
    },
    CreatePlaylist {
        name: String,
        reply: SyncSender<Result<i64>>,
    },
    AddToPlaylist {
        playlist: i64,
        tracks: Vec<TrackId>,
        reply: SyncSender<Result<()>>,
    },
    DeletePlaylist {
        id: i64,
        reply: SyncSender<Result<()>>,
    },
    RemoveTracks {
        tracks: Vec<TrackId>,
        reply: SyncSender<Result<u32>>,
    },
    RemoveAlbum {
        album: AlbumId,
        reply: SyncSender<Result<u32>>,
    },
    WriteMetadata {
        path: PathBuf,
        meta: TrackMetadata,
        reply: SyncSender<Result<()>>,
    },
    RescanPath {
        path: PathBuf,
        reply: SyncSender<Result<()>>,
    },
    BuildOrganizationPlan {
        root: PathBuf,
        template: Template,
        reply: SyncSender<Result<OrganizationPlan>>,
    },
    ExecuteOrganization {
        plan: OrganizationPlan,
        reply: SyncSender<Result<UndoLog>>,
    },
    UndoOrganization {
        log: UndoLog,
        reply: SyncSender<Result<()>>,
    },
    LookupAndFill {
        track: TrackId,
        reply: SyncSender<Result<TrackMetadata>>,
    },
    FillMissingMetadata {
        reply: SyncSender<Result<LookupSummary>>,
    },
    ListGenres {
        reply: SyncSender<Result<Vec<String>>>,
    },
    ListYears {
        reply: SyncSender<Result<Vec<i32>>>,
    },
}

/// Handle used by the UI to talk to the library worker.
#[derive(Clone, Debug)]
pub struct LibraryService {
    cmd_tx: Sender<Command>,
}

impl LibraryService {
    /// Start the worker and return a service handle plus an event receiver
    /// attached to the glib main context.
    pub fn start() -> (Self, Receiver<LibraryEvent>) {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();

        thread::Builder::new()
            .name("cadence-library".into())
            .spawn(move || worker_main(cmd_rx, event_tx))
            .expect("failed to spawn library worker");

        (Self { cmd_tx }, event_rx)
    }

    /// Poll events from the worker on the GTK main loop.
    pub fn attach_events<F>(rx: Receiver<LibraryEvent>, mut handler: F)
    where
        F: FnMut(LibraryEvent) + 'static,
    {
        glib::timeout_add_local(Duration::from_millis(50), move || {
            while let Ok(event) = rx.try_recv() {
                handler(event);
            }
            ControlFlow::Continue
        });
    }

    fn call<T, F>(&self, make: F, cont: impl FnOnce(Result<T>) + 'static)
    where
        T: Send + 'static,
        F: FnOnce(SyncSender<Result<T>>) -> Command,
    {
        let (tx, rx) = mpsc::sync_channel(1);
        if self.cmd_tx.send(make(tx)).is_err() {
            cont(Err(cadence_core::Error::Other(anyhow::anyhow!(
                "library worker is not running"
            ))));
            return;
        }
        let cont = std::cell::RefCell::new(Some(cont));
        glib::timeout_add_local(Duration::from_millis(10), move || match rx.try_recv() {
            Ok(result) => {
                if let Some(cb) = cont.borrow_mut().take() {
                    cb(result);
                }
                ControlFlow::Break
            }
            Err(mpsc::TryRecvError::Empty) => ControlFlow::Continue,
            Err(mpsc::TryRecvError::Disconnected) => {
                if let Some(cb) = cont.borrow_mut().take() {
                    cb(Err(cadence_core::Error::Other(anyhow::anyhow!(
                        "library worker disconnected"
                    ))));
                }
                ControlFlow::Break
            }
        });
    }

    pub fn add_folder(&self, path: PathBuf, cont: impl FnOnce(Result<()>) + 'static) {
        self.call(|reply| Command::AddFolder { path, reply }, cont);
    }

    pub fn remove_folder(&self, path: PathBuf, cont: impl FnOnce(Result<()>) + 'static) {
        self.call(|reply| Command::RemoveFolder { path, reply }, cont);
    }

    pub fn list_folders(&self, cont: impl FnOnce(Result<Vec<PathBuf>>) + 'static) {
        self.call(|reply| Command::ListFolders { reply }, cont);
    }

    pub fn scan_all(&self, cont: impl FnOnce(Result<usize>) + 'static) {
        self.call(|reply| Command::ScanAll { reply }, cont);
    }

    pub fn list_artists(&self, cont: impl FnOnce(Result<Vec<Artist>>) + 'static) {
        self.call(|reply| Command::ListArtists { reply }, cont);
    }

    pub fn list_albums(&self, cont: impl FnOnce(Result<Vec<Album>>) + 'static) {
        self.call(|reply| Command::ListAlbums { reply }, cont);
    }

    pub fn albums_by_artist(
        &self,
        artist: ArtistId,
        cont: impl FnOnce(Result<Vec<Album>>) + 'static,
    ) {
        self.call(|reply| Command::AlbumsByArtist { artist, reply }, cont);
    }

    pub fn tracks_by_album(&self, album: AlbumId, cont: impl FnOnce(Result<Vec<Track>>) + 'static) {
        self.call(|reply| Command::TracksByAlbum { album, reply }, cont);
    }

    pub fn tracks_by_artist(
        &self,
        artist: ArtistId,
        cont: impl FnOnce(Result<Vec<Track>>) + 'static,
    ) {
        self.call(|reply| Command::TracksByArtist { artist, reply }, cont);
    }

    pub fn singles_by_artist_display(
        &self,
        artist: ArtistId,
        cont: impl FnOnce(Result<Vec<TrackDisplay>>) + 'static,
    ) {
        self.call(
            |reply| Command::SinglesByArtistDisplay { artist, reply },
            cont,
        );
    }

    pub fn list_songs_page(
        &self,
        sort: SongSort,
        offset: usize,
        limit: usize,
        cont: impl FnOnce(Result<Vec<Track>>) + 'static,
    ) {
        self.call(
            |reply| Command::ListSongsPage {
                sort,
                offset,
                limit,
                reply,
            },
            cont,
        );
    }

    pub fn track_count(&self, cont: impl FnOnce(Result<u64>) + 'static) {
        self.call(|reply| Command::TrackCount { reply }, cont);
    }

    pub fn search(&self, query: String, cont: impl FnOnce(Result<Vec<Track>>) + 'static) {
        self.call(|reply| Command::Search { query, reply }, cont);
    }

    pub fn search_display(
        &self,
        query: String,
        cont: impl FnOnce(Result<Vec<TrackDisplay>>) + 'static,
    ) {
        self.call(|reply| Command::SearchDisplay { query, reply }, cont);
    }

    pub fn list_songs_display(
        &self,
        sort: SongSort,
        offset: usize,
        limit: usize,
        cont: impl FnOnce(Result<Vec<TrackDisplay>>) + 'static,
    ) {
        self.call(
            |reply| Command::ListSongsDisplay {
                sort,
                offset,
                limit,
                reply,
            },
            cont,
        );
    }

    pub fn tracks_by_album_display(
        &self,
        album: AlbumId,
        cont: impl FnOnce(Result<Vec<TrackDisplay>>) + 'static,
    ) {
        self.call(
            |reply| Command::TracksByAlbumDisplay { album, reply },
            cont,
        );
    }

    pub fn artist_duration_ms(
        &self,
        artist: ArtistId,
        cont: impl FnOnce(Result<u64>) + 'static,
    ) {
        self.call(|reply| Command::ArtistDuration { artist, reply }, cont);
    }

    pub fn get_artist(&self, id: ArtistId, cont: impl FnOnce(Result<Artist>) + 'static) {
        self.call(|reply| Command::GetArtist { id, reply }, cont);
    }

    pub fn get_track(&self, id: TrackId, cont: impl FnOnce(Result<Track>) + 'static) {
        self.call(|reply| Command::GetTrack { id, reply }, cont);
    }

    pub fn artist_name(&self, id: ArtistId, cont: impl FnOnce(Result<Option<String>>) + 'static) {
        self.call(|reply| Command::ArtistName { id, reply }, cont);
    }

    pub fn album(&self, id: AlbumId, cont: impl FnOnce(Result<Album>) + 'static) {
        self.call(|reply| Command::Album { id, reply }, cont);
    }

    pub fn set_favorite(
        &self,
        id: TrackId,
        favorite: bool,
        cont: impl FnOnce(Result<()>) + 'static,
    ) {
        self.call(
            |reply| Command::SetFavorite {
                id,
                favorite,
                reply,
            },
            cont,
        );
    }

    pub fn record_play(&self, id: TrackId, cont: impl FnOnce(Result<()>) + 'static) {
        self.call(|reply| Command::RecordPlay { id, reply }, cont);
    }

    pub fn favorites(&self, cont: impl FnOnce(Result<Vec<Track>>) + 'static) {
        self.call(|reply| Command::Favorites { reply }, cont);
    }

    pub fn favorites_display(
        &self,
        cont: impl FnOnce(Result<Vec<TrackDisplay>>) + 'static,
    ) {
        self.call(|reply| Command::FavoritesDisplay { reply }, cont);
    }

    pub fn recently_added(&self, limit: usize, cont: impl FnOnce(Result<Vec<Track>>) + 'static) {
        self.call(|reply| Command::RecentlyAdded { limit, reply }, cont);
    }

    pub fn recently_added_display(
        &self,
        limit: usize,
        cont: impl FnOnce(Result<Vec<TrackDisplay>>) + 'static,
    ) {
        self.call(
            |reply| Command::RecentlyAddedDisplay { limit, reply },
            cont,
        );
    }

    pub fn recently_played(&self, limit: usize, cont: impl FnOnce(Result<Vec<Track>>) + 'static) {
        self.call(|reply| Command::RecentlyPlayed { limit, reply }, cont);
    }

    pub fn recently_played_display(
        &self,
        limit: usize,
        cont: impl FnOnce(Result<Vec<TrackDisplay>>) + 'static,
    ) {
        self.call(
            |reply| Command::RecentlyPlayedDisplay { limit, reply },
            cont,
        );
    }

    pub fn playlists(&self, cont: impl FnOnce(Result<Vec<Playlist>>) + 'static) {
        self.call(|reply| Command::Playlists { reply }, cont);
    }

    pub fn create_playlist(&self, name: String, cont: impl FnOnce(Result<i64>) + 'static) {
        self.call(|reply| Command::CreatePlaylist { name, reply }, cont);
    }

    pub fn add_to_playlist(
        &self,
        playlist: i64,
        tracks: Vec<TrackId>,
        cont: impl FnOnce(Result<()>) + 'static,
    ) {
        self.call(
            |reply| Command::AddToPlaylist {
                playlist,
                tracks,
                reply,
            },
            cont,
        );
    }

    pub fn delete_playlist(&self, id: i64, cont: impl FnOnce(Result<()>) + 'static) {
        self.call(|reply| Command::DeletePlaylist { id, reply }, cont);
    }

    /// Remove tracks from the library and delete their files on disk.
    pub fn remove_tracks(
        &self,
        tracks: Vec<TrackId>,
        cont: impl FnOnce(Result<u32>) + 'static,
    ) {
        self.call(|reply| Command::RemoveTracks { tracks, reply }, cont);
    }

    /// Remove an album, its tracks, and the audio files on disk.
    pub fn remove_album(
        &self,
        album: AlbumId,
        cont: impl FnOnce(Result<u32>) + 'static,
    ) {
        self.call(|reply| Command::RemoveAlbum { album, reply }, cont);
    }

    pub fn write_metadata(
        &self,
        path: PathBuf,
        meta: TrackMetadata,
        cont: impl FnOnce(Result<()>) + 'static,
    ) {
        self.call(|reply| Command::WriteMetadata { path, meta, reply }, cont);
    }

    pub fn rescan_path(&self, path: PathBuf, cont: impl FnOnce(Result<()>) + 'static) {
        self.call(|reply| Command::RescanPath { path, reply }, cont);
    }

    pub fn build_organization_plan(
        &self,
        root: PathBuf,
        template: Template,
        cont: impl FnOnce(Result<OrganizationPlan>) + 'static,
    ) {
        self.call(
            |reply| Command::BuildOrganizationPlan {
                root,
                template,
                reply,
            },
            cont,
        );
    }

    pub fn execute_organization(
        &self,
        plan: OrganizationPlan,
        cont: impl FnOnce(Result<UndoLog>) + 'static,
    ) {
        self.call(|reply| Command::ExecuteOrganization { plan, reply }, cont);
    }

    pub fn undo_organization(&self, log: UndoLog, cont: impl FnOnce(Result<()>) + 'static) {
        self.call(|reply| Command::UndoOrganization { log, reply }, cont);
    }

    pub fn lookup_and_fill(
        &self,
        track: TrackId,
        cont: impl FnOnce(Result<TrackMetadata>) + 'static,
    ) {
        self.call(|reply| Command::LookupAndFill { track, reply }, cont);
    }

    /// Queue a full-library metadata fill on a background thread (artwork, genres,
    /// years, artist portraits). Progress arrives via [`LibraryEvent::LookupProgress`].
    pub fn fill_missing_metadata(
        &self,
        cont: impl FnOnce(Result<LookupSummary>) + 'static,
    ) {
        self.call(|reply| Command::FillMissingMetadata { reply }, cont);
    }

    pub fn list_genres(&self, cont: impl FnOnce(Result<Vec<String>>) + 'static) {
        self.call(|reply| Command::ListGenres { reply }, cont);
    }

    pub fn list_years(&self, cont: impl FnOnce(Result<Vec<i32>>) + 'static) {
        self.call(|reply| Command::ListYears { reply }, cont);
    }
}

fn worker_main(cmd_rx: Receiver<Command>, event_tx: Sender<LibraryEvent>) {
    let db_path = library_db_path();
    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(err) => {
            let _ = event_tx.send(LibraryEvent::Error(format!(
                "failed to open database: {err}"
            )));
            // Keep the thread alive so callers get disconnect errors instead of hanging.
            while cmd_rx.recv().is_ok() {}
            return;
        }
    };
    let fill_running = Arc::new(AtomicBool::new(false));

    let mut watcher = LibraryWatcher::new().ok();
    if let Ok(folders) = db.library_folders() {
        if let Some(w) = watcher.as_mut() {
            for folder in &folders {
                let _ = w.watch(folder);
            }
        }
    }

    let mut pending_upserts: HashMap<PathBuf, Instant> = HashMap::new();
    let mut pending_removes: HashMap<PathBuf, Instant> = HashMap::new();
    let debounce = Duration::from_millis(500);

    loop {
        // Drain filesystem events.
        if let Some(w) = watcher.as_ref() {
            while let Ok(ev) = w.events().try_recv() {
                match ev {
                    WatchEvent::Upserted(path) => {
                        pending_upserts.insert(path, Instant::now());
                    }
                    WatchEvent::Removed(path) => {
                        pending_removes.insert(path, Instant::now());
                    }
                }
            }
        }

        // Apply debounced watcher updates.
        let now = Instant::now();
        let upserts: Vec<_> = pending_upserts
            .iter()
            .filter(|(_, t)| now.duration_since(**t) >= debounce)
            .map(|(p, _)| p.clone())
            .collect();
        for path in upserts {
            pending_upserts.remove(&path);
            if let Err(err) = ingest_path(&db, &path) {
                tracing::warn!(%err, path = %path.display(), "watch upsert failed");
            } else {
                let _ = event_tx.send(LibraryEvent::LibraryChanged);
            }
        }
        let removes: Vec<_> = pending_removes
            .iter()
            .filter(|(_, t)| now.duration_since(**t) >= debounce)
            .map(|(p, _)| p.clone())
            .collect();
        for path in removes {
            pending_removes.remove(&path);
            if let Err(err) = db.remove_track_by_path(&path) {
                tracing::warn!(%err, path = %path.display(), "watch remove failed");
            } else {
                let _ = event_tx.send(LibraryEvent::LibraryChanged);
            }
        }

        let cmd = match cmd_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(cmd) => cmd,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };

        match cmd {
            Command::AddFolder { path, reply } => {
                let result = (|| {
                    db.add_library_folder(&path)?;
                    if let Some(w) = watcher.as_mut() {
                        let _ = w.watch(&path);
                    }
                    Ok(())
                })();
                let _ = reply.send(result);
                if let Ok(count) = scan_folder(&db, &path, &event_tx) {
                    let _ = event_tx.send(LibraryEvent::ScanFinished { imported: count });
                    let _ = event_tx.send(LibraryEvent::LibraryChanged);
                }
            }
            Command::RemoveFolder { path, reply } => {
                if let Some(w) = watcher.as_mut() {
                    let _ = w.unwatch(&path);
                }
                let _ = reply.send(db.remove_library_folder(&path));
            }
            Command::ListFolders { reply } => {
                let _ = reply.send(db.library_folders());
            }
            Command::ScanAll { reply } => {
                let result = (|| {
                    let mut total = 0usize;
                    for folder in db.library_folders()? {
                        total += scan_folder(&db, &folder, &event_tx)?;
                    }
                    Ok(total)
                })();
                if let Ok(imported) = &result {
                    let _ = event_tx.send(LibraryEvent::ScanFinished {
                        imported: *imported,
                    });
                    let _ = event_tx.send(LibraryEvent::LibraryChanged);
                }
                let _ = reply.send(result);
            }
            Command::ListArtists { reply } => {
                let _ = reply.send(db.list_artists());
            }
            Command::ListAlbums { reply } => {
                let _ = reply.send(db.list_albums());
            }
            Command::AlbumsByArtist { artist, reply } => {
                let _ = reply.send(db.albums_by_artist(artist));
            }
            Command::TracksByAlbum { album, reply } => {
                let _ = reply.send(db.tracks_by_album(album));
            }
            Command::TracksByArtist { artist, reply } => {
                let _ = reply.send(db.tracks_by_artist(artist));
            }
            Command::SinglesByArtistDisplay { artist, reply } => {
                let _ = reply.send(db.singles_by_artist_display(artist));
            }
            Command::ListSongsPage {
                sort,
                offset,
                limit,
                reply,
            } => {
                let _ = reply.send(db.list_songs_page(sort, offset, limit));
            }
            Command::TrackCount { reply } => {
                let _ = reply.send(db.track_count());
            }
            Command::Search { query, reply } => {
                let _ = reply.send(db.search(&query, 200));
            }
            Command::SearchDisplay { query, reply } => {
                let _ = reply.send(db.search_display(&query, 200));
            }
            Command::ListSongsDisplay {
                sort,
                offset,
                limit,
                reply,
            } => {
                let _ = reply.send(db.list_songs_display(sort, offset, limit));
            }
            Command::TracksByAlbumDisplay { album, reply } => {
                let _ = reply.send(db.tracks_by_album_display(album));
            }
            Command::ArtistDuration { artist, reply } => {
                let _ = reply.send(db.artist_duration_ms(artist));
            }
            Command::GetArtist { id, reply } => {
                let _ = reply.send(db.artist(id));
            }
            Command::GetTrack { id, reply } => {
                let _ = reply.send(db.track(id));
            }
            Command::ArtistName { id, reply } => {
                let _ = reply.send(db.artist_name(id));
            }
            Command::Album { id, reply } => {
                let _ = reply.send(db.album(id));
            }
            Command::SetFavorite {
                id,
                favorite,
                reply,
            } => {
                let _ = reply.send(db.set_favorite(id, favorite));
            }
            Command::RecordPlay { id, reply } => {
                let _ = reply.send(db.record_play(id));
            }
            Command::Favorites { reply } => {
                let _ = reply.send(db.favorites());
            }
            Command::FavoritesDisplay { reply } => {
                let _ = reply.send(db.favorites_display());
            }
            Command::RecentlyAdded { limit, reply } => {
                let _ = reply.send(db.recently_added(limit));
            }
            Command::RecentlyAddedDisplay { limit, reply } => {
                let _ = reply.send(db.recently_added_display(limit));
            }
            Command::RecentlyPlayed { limit, reply } => {
                let _ = reply.send(db.recently_played(limit));
            }
            Command::RecentlyPlayedDisplay { limit, reply } => {
                let _ = reply.send(db.recently_played_display(limit));
            }
            Command::Playlists { reply } => {
                let _ = reply.send(db.playlists());
            }
            Command::CreatePlaylist { name, reply } => {
                let _ = reply.send(db.create_playlist(&name));
            }
            Command::AddToPlaylist {
                playlist,
                tracks,
                reply,
            } => {
                let _ = reply.send(db.add_to_playlist(playlist, &tracks));
            }
            Command::DeletePlaylist { id, reply } => {
                let _ = reply.send(db.delete_playlist(id));
            }
            Command::RemoveTracks { tracks, reply } => {
                let result = remove_tracks_and_files(&db, &tracks);
                if matches!(&result, Ok(n) if *n > 0) {
                    let _ = event_tx.send(LibraryEvent::LibraryChanged);
                }
                let _ = reply.send(result);
            }
            Command::RemoveAlbum { album, reply } => {
                let result = (|| {
                    let tracks = db.tracks_by_album(album)?;
                    let paths: Vec<_> = tracks.into_iter().map(|t| t.path).collect();
                    let n = db.remove_album(album)?;
                    for path in paths {
                        delete_audio_file(&path);
                    }
                    Ok(n)
                })();
                if matches!(&result, Ok(n) if *n > 0) {
                    let _ = event_tx.send(LibraryEvent::LibraryChanged);
                }
                let _ = reply.send(result);
            }
            Command::WriteMetadata { path, meta, reply } => {
                let result = (|| {
                    write_metadata(&path, &meta)?;
                    ingest_path(&db, &path)?;
                    Ok(())
                })();
                if result.is_ok() {
                    let _ = event_tx.send(LibraryEvent::LibraryChanged);
                }
                let _ = reply.send(result);
            }
            Command::RescanPath { path, reply } => {
                let result = ingest_path(&db, &path);
                if result.is_ok() {
                    let _ = event_tx.send(LibraryEvent::LibraryChanged);
                }
                let _ = reply.send(result);
            }
            Command::BuildOrganizationPlan {
                root,
                template,
                reply,
            } => {
                let result = (|| {
                    // Prefer DB metadata so preview never goes blank when a file
                    // can't be re-read. Fall back to on-disk tags when present.
                    let displays = db.list_songs_display(SongSort::AlbumAsc, 0, 1_000_000)?;
                    let folders = db.library_folders()?;
                    let mut plan = OrganizationPlan::default();
                    for item in displays {
                        let track_root = folders
                            .iter()
                            .filter(|f| item.track.path.starts_with(f))
                            .max_by_key(|f| f.as_os_str().len())
                            .cloned()
                            .unwrap_or_else(|| root.clone());
                        let mut meta = metadata::read_metadata(&item.track.path)
                            .unwrap_or_else(|_| track_display_meta(&item));
                        if meta.title.as_deref().unwrap_or("").is_empty() {
                            meta.title = Some(item.track.title.clone());
                        }
                        if meta.artist.as_deref().unwrap_or("").is_empty()
                            && !item.artist_name.is_empty()
                        {
                            meta.artist = Some(item.artist_name.clone());
                        }
                        if meta.album.as_deref().unwrap_or("").is_empty()
                            && !item.album_name.is_empty()
                        {
                            meta.album = Some(item.album_name.clone());
                        }
                        if meta.album_artist.as_deref().unwrap_or("").is_empty() {
                            meta.album_artist = meta.artist.clone();
                        }
                        if meta.track_number.is_none() {
                            meta.track_number = item.track.track_number;
                        }
                        if meta.disc_number.is_none() {
                            meta.disc_number = item.track.disc_number;
                        }
                        if meta.genre.is_none() {
                            meta.genre = item.track.genre.clone();
                        }
                        if meta.year.is_none() {
                            meta.year = item.track.year;
                        }
                        let partial = OrganizationPlan::build(
                            &track_root,
                            &template,
                            [(item.track.path, meta)],
                        );
                        plan.entries.extend(partial.entries);
                    }
                    Ok(plan)
                })();
                let _ = reply.send(result);
            }
            Command::ExecuteOrganization { plan, reply } => {
                let result = (|| {
                    let moves: Vec<_> = plan
                        .pending_moves()
                        .into_iter()
                        .map(|m| (m.from.clone(), m.to.clone()))
                        .collect();
                    let log = plan.execute()?;
                    for (from, to) in moves {
                        if let Ok(Some(track)) = db.track_id_by_path(&from) {
                            let _ = db.update_track_path(track, &to);
                        }
                    }
                    Ok(log)
                })();
                if result.is_ok() {
                    let _ = event_tx.send(LibraryEvent::LibraryChanged);
                }
                let _ = reply.send(result);
            }
            Command::UndoOrganization { log, reply } => {
                let result = (|| {
                    let moves = log.moves.clone();
                    log.undo()?;
                    for m in moves {
                        if let Ok(Some(track)) = db.track_id_by_path(&m.to) {
                            let _ = db.update_track_path(track, &m.from);
                        }
                    }
                    Ok(())
                })();
                if result.is_ok() {
                    let _ = event_tx.send(LibraryEvent::LibraryChanged);
                }
                let _ = reply.send(result);
            }
            Command::LookupAndFill { track, reply } => {
                let result = (|| {
                    let t = db.track(track)?;
                    let mut meta = metadata::read_metadata(&t.path)?;
                    let artist = meta
                        .artist
                        .clone()
                        .or_else(|| meta.album_artist.clone())
                        .unwrap_or_default();
                    let title = meta.title.clone().unwrap_or_else(|| t.title.clone());
                    if artist.is_empty() || title.is_empty() {
                        return Err(cadence_core::Error::Other(anyhow::anyhow!(
                            "Track needs an artist and title before lookup"
                        )));
                    }
                    let Some(lookup) = cadence_core::lookup::lookup_recording(
                        &artist,
                        &title,
                        meta.album.as_deref(),
                    )?
                    else {
                        return Err(cadence_core::Error::NotFound(
                            "No MusicBrainz match for this track".into(),
                        ));
                    };
                    lookup.apply_missing_only(&mut meta);
                    if let Some(url) = &lookup.cover_art_url {
                        if let (Some(album), Some(album_id)) = (&meta.album, t.album_id) {
                            if let Ok(bytes) = cadence_core::lookup::download_cover_art(url) {
                                let cache = artwork_cache_dir();
                                let artist_name = meta
                                    .album_artist
                                    .as_deref()
                                    .or(meta.artist.as_deref())
                                    .unwrap_or("Unknown Artist");
                                let key = artwork::artwork_key(artist_name, album);
                                let dest = cache.join(format!("{key}.jpg"));
                                let _ = std::fs::create_dir_all(&cache);
                                if std::fs::write(&dest, bytes).is_ok() {
                                    let _ = db.set_album_artwork(album_id, &dest);
                                }
                            }
                        }
                    }
                    Ok(meta)
                })();
                let _ = reply.send(result);
            }
            Command::FillMissingMetadata { reply } => {
                if fill_running
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .is_err()
                {
                    let _ = reply.send(Err(cadence_core::Error::Other(anyhow::anyhow!(
                        "A metadata lookup is already running"
                    ))));
                    continue;
                }
                // Dedicated thread + DB connection so browsing stays responsive
                // while the full queue runs (MusicBrainz-polite, can take a while).
                let event_tx = event_tx.clone();
                let fill_running = Arc::clone(&fill_running);
                let db_path = library_db_path();
                thread::Builder::new()
                    .name("cadence-metadata-fill".into())
                    .spawn(move || {
                        let result = (|| {
                            let db = Database::open(&db_path)?;
                            fill_missing_metadata_run(&db, &event_tx)
                        })();
                        if matches!(
                            &result,
                            Ok(s) if s.metadata_updated + s.artwork_updated + s.artist_photos > 0
                        ) {
                            let _ = event_tx.send(LibraryEvent::LibraryChanged);
                        }
                        let _ = reply.send(result);
                        fill_running.store(false, Ordering::SeqCst);
                    })
                    .expect("failed to spawn metadata fill thread");
            }
            Command::ListGenres { reply } => {
                let _ = reply.send(db.genres());
            }
            Command::ListYears { reply } => {
                let _ = reply.send(db.years());
            }
        }
    }
}

fn remove_tracks_and_files(db: &Database, tracks: &[TrackId]) -> Result<u32> {
    let mut n = 0u32;
    for id in tracks {
        let track = db.track(*id)?;
        db.remove_track(*id)?;
        delete_audio_file(&track.path);
        n += 1;
    }
    Ok(n)
}

fn delete_audio_file(path: &Path) {
    if !path.exists() {
        return;
    }
    if let Err(err) = std::fs::remove_file(path) {
        tracing::warn!(%err, path = %path.display(), "failed to delete audio file");
    }
}

fn fill_missing_metadata_run(
    db: &Database,
    event_tx: &Sender<LibraryEvent>,
) -> Result<LookupSummary> {
    // Only queue albums that still have gaps. Completed items are skipped so
    // re-running Lookup Metadata is incremental. MusicBrainz ~1.1s/call.
    let albums = db.list_albums()?;
    let album_work: Vec<_> = albums
        .iter()
        .filter(|a| {
            a.artwork_path.is_none()
                || a.genre.as_deref().map_or(true, str::is_empty)
                || a.year.is_none()
        })
        .cloned()
        .collect();
    let total = album_work.len().max(1);
    let mut summary = LookupSummary {
        albums_scanned: album_work.len() as u32,
        ..Default::default()
    };
    let mut done = 0usize;

    let _ = event_tx.send(LibraryEvent::LookupProgress {
        phase: if album_work.is_empty() {
            "Nothing missing — library looks complete".into()
        } else {
            format!("Queued {} albums with missing metadata", album_work.len())
        },
        done: 0,
        total,
    });

    for album in album_work {
        let _ = event_tx.send(LibraryEvent::LookupProgress {
            phase: format!("Album · {}", album.name),
            done,
            total,
        });
        done += 1;

        let missing_art = album.artwork_path.is_none();
        let missing_genre = album.genre.as_deref().map_or(true, str::is_empty);
        let missing_year = album.year.is_none();
        if !missing_art && !missing_genre && !missing_year {
            continue;
        }
        let Ok(tracks) = db.tracks_by_album(album.id) else {
            summary.needs_review += 1;
            continue;
        };
        let Some(track) = tracks.first() else {
            summary.needs_review += 1;
            continue;
        };
        let mut meta = metadata::read_metadata(&track.path).unwrap_or_default();
        if meta.title.as_deref().unwrap_or("").is_empty() {
            meta.title = Some(track.title.clone());
        }
        let artist = meta
            .artist
            .clone()
            .or_else(|| meta.album_artist.clone())
            .filter(|s| !s.is_empty());
        let title = meta.title.clone().filter(|s| !s.is_empty());
        let (Some(artist), Some(title)) = (artist, title) else {
            summary.needs_review += 1;
            continue;
        };
        summary.network_lookups += 1;
        let Ok(Some(lookup)) = cadence_core::lookup::lookup_recording(
            &artist,
            &title,
            meta.album.as_deref().or(Some(album.name.as_str())),
        ) else {
            summary.needs_review += 1;
            continue;
        };
        let before_genre = meta.genre.clone();
        lookup.apply_missing_only(&mut meta);
        if before_genre.as_deref().unwrap_or("").is_empty()
            && meta.genre.as_deref().is_some_and(|g| !g.is_empty())
        {
            summary.genres_fixed += 1;
        }
        if let Some(url) = &lookup.cover_art_url {
            if missing_art {
                if let Ok(bytes) = cadence_core::lookup::download_cover_art(url) {
                    let cache = artwork_cache_dir();
                    let key = artwork::artwork_key(&artist, &album.name);
                    let dest = cache.join(format!("{key}.jpg"));
                    let _ = std::fs::create_dir_all(&cache);
                    if std::fs::write(&dest, &bytes).is_ok()
                        && db.set_album_artwork(album.id, &dest).is_ok()
                    {
                        summary.artwork_updated += 1;
                    }
                }
            }
        }
        if write_metadata(&track.path, &meta).is_ok() {
            let _ = ingest_path(db, &track.path);
            summary.metadata_updated += 1;
        } else {
            summary.needs_review += 1;
        }
    }

    let _ = event_tx.send(LibraryEvent::LookupProgress {
        phase: "Finished".into(),
        done: total,
        total,
    });
    Ok(summary)
}

fn scan_folder(db: &Database, root: &Path, event_tx: &Sender<LibraryEvent>) -> Result<usize> {
    let discovered = scanner::discover(root);
    let total = discovered.len();
    let mut imported = 0usize;
    let mut batch = Vec::new();

    for (idx, file) in discovered.into_iter().enumerate() {
        if db.track_needs_rescan(&file.path, file.file_size, file.modified_at)? {
            batch.push(file);
        }
        if batch.len() >= 64 || idx + 1 == total {
            let scanned = scanner::scan_files(std::mem::take(&mut batch));
            for item in scanned {
                upsert_with_artwork(db, &item.file, &item.metadata)?;
                imported += 1;
            }
            let _ = event_tx.send(LibraryEvent::ScanProgress {
                done: idx + 1,
                total,
            });
        }
    }
    Ok(imported)
}

fn ingest_path(db: &Database, path: &Path) -> Result<()> {
    let Some(file) = discovered_one(path) else {
        return Ok(());
    };
    if !db.track_needs_rescan(&file.path, file.file_size, file.modified_at)? {
        return Ok(());
    }
    let meta = metadata::read_metadata(&file.path)?;
    upsert_with_artwork(db, &file, &meta)
}

fn discovered_one(path: &Path) -> Option<DiscoveredFile> {
    scanner::discover(path.parent().unwrap_or(path))
        .into_iter()
        .find(|f| f.path == path)
        .or_else(|| {
            // Direct file discover: reuse scanner internals via a one-file walk.
            let format = path
                .extension()
                .and_then(|e| e.to_str())
                .and_then(cadence_core::models::AudioFormat::from_extension)?;
            let meta = std::fs::metadata(path).ok()?;
            let modified_at = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs() as i64);
            Some(DiscoveredFile {
                path: path.to_path_buf(),
                format,
                file_size: meta.len(),
                modified_at,
            })
        })
}

fn upsert_with_artwork(db: &Database, file: &DiscoveredFile, meta: &TrackMetadata) -> Result<()> {
    let id = db.upsert_track(file, meta)?;
    let track = db.track(id)?;
    if let Some(album_id) = track.album_id {
        let album = db.album(album_id)?;
        if album.artwork_path.is_none() {
            let artist = meta
                .album_artist
                .as_deref()
                .or(meta.artist.as_deref())
                .unwrap_or("Unknown Artist");
            let album_name = meta.album.as_deref().unwrap_or("Unknown Album");
            if let Ok(Some(art)) =
                artwork::extract_and_cache(&file.path, &artwork_cache_dir(), artist, album_name)
            {
                let _ = db.set_album_artwork(album_id, &art);
            }
        }
    }
    Ok(())
}

fn track_display_meta(item: &TrackDisplay) -> TrackMetadata {
    TrackMetadata {
        title: Some(item.track.title.clone()),
        artist: (!item.artist_name.is_empty()).then(|| item.artist_name.clone()),
        album: (!item.album_name.is_empty()).then(|| item.album_name.clone()),
        album_artist: (!item.artist_name.is_empty()).then(|| item.artist_name.clone()),
        genre: item.track.genre.clone(),
        year: item.track.year,
        track_number: item.track.track_number,
        disc_number: item.track.disc_number,
        duration_ms: item.track.duration_ms,
        ..Default::default()
    }
}
