//! Main application window — three-pane master-detail shell.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use cadence_core::metadata;
use cadence_core::models::{Album, AlbumId, ArtistId, Track, TrackDisplay, TrackId};
use cadence_core::organization::UndoLog;
use gtk::gio;
use gtk::glib;
use gtk::glib::ControlFlow;

use crate::mpris::{self, MprisCommand, MprisService};
use crate::playback::{PlaybackState, Player, PlayerEvent, Queue, RepeatMode};
use crate::services::{LibraryEvent, LibraryService};
use crate::ui::{
    AlbumsView, ArtistDetail, ArtistsView, ContextAction, EmptyState, LibraryHome, MetadataDialog,
    NowPlaying, OrganizeDialog, PlayerBar, PlaylistsView, PreferencesWindow, QueueView,
    SearchResults, SongsView,
};

const PAGE_SIZE: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Nav {
    Library,
    Artists,
    Albums,
    Songs,
    Playlists,
    Favourites,
    Recent,
}

struct AppState {
    library: LibraryService,
    player: Rc<Player>,
    queue: RefCell<Queue>,
    player_bar: PlayerBar,
    now_playing: NowPlaying,
    queue_view: QueueView,
    home: LibraryHome,
    artists: ArtistsView,
    artist_detail: ArtistDetail,
    albums: AlbumsView,
    songs: SongsView,
    playlists: PlaylistsView,
    search: SearchResults,
    empty: EmptyState,
    master_pane: gtk::Box,
    master_sep: gtk::Separator,
    detail_stack: gtk::Stack,
    now_playing_revealer: gtk::Revealer,
    context_tracks: RefCell<Vec<Track>>,
    toast: adw::ToastOverlay,
    scan_banner: adw::Banner,
    lookup_spinner: gtk::Spinner,
    mpris: Rc<MprisService>,
    last_undo: RefCell<Option<UndoLog>>,
    nav: Cell<Nav>,
    has_library: Cell<bool>,
    previous_detail: RefCell<String>,
    current_track: RefCell<Option<Track>>,
}

pub struct CadenceWindow {
    pub window: adw::ApplicationWindow,
    _state: Rc<AppState>,
}

impl CadenceWindow {
    pub fn new(app: &adw::Application) -> Self {
        let (library, event_rx) = LibraryService::start();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Cadence")
            .default_width(1280)
            .default_height(800)
            .build();

        let toast = adw::ToastOverlay::new();
        let toolbar = adw::ToolbarView::new();
        let header = adw::HeaderBar::new();
        let search_entry = gtk::SearchEntry::builder()
            .placeholder_text("Search artists, albums, songs…")
            .width_request(280)
            .build();
        header.set_title_widget(Some(&search_entry));

        let brand = brand_widget();
        let add_btn = gtk::Button::from_icon_name("folder-new-symbolic");
        add_btn.set_tooltip_text(Some("Add Music Folder"));
        let lookup_spinner = gtk::Spinner::builder()
            .tooltip_text("Metadata lookup idle")
            .visible(false)
            .build();
        let menu_btn = gtk::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&build_menu())
            .build();
        header.pack_start(&brand);
        header.pack_start(&add_btn);
        header.pack_end(&menu_btn);
        header.pack_end(&lookup_spinner);

        let scan_banner = adw::Banner::builder()
            .title("Scanning library…")
            .revealed(false)
            .build();

        let sidebar = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .css_classes(["navigation-sidebar", "cadence-sidebar"])
            .build();
        for (label, icon) in [
            ("Library", "user-home-symbolic"),
            ("Artists", "avatar-default-symbolic"),
            ("Albums", "media-optical-symbolic"),
            ("Songs", "audio-x-generic-symbolic"),
            ("Playlists", "view-list-symbolic"),
            ("Favourites", "starred-symbolic"),
            ("Recently Added", "document-open-recent-symbolic"),
        ] {
            sidebar.append(&nav_row(label, icon));
        }
        if let Some(row) = sidebar.row_at_index(0) {
            sidebar.select_row(Some(&row));
        }
        let nav_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .width_request(180)
            .child(&sidebar)
            .build();

        let home = LibraryHome::new();
        let artists = ArtistsView::new();
        let artist_detail = ArtistDetail::new();
        let albums = AlbumsView::new();
        let songs = SongsView::new();
        let playlists = PlaylistsView::new();
        let search = SearchResults::new();
        let empty = EmptyState::new();
        let queue_view = QueueView::new();
        let player_bar = PlayerBar::new();
        let now_playing = NowPlaying::new();

        artist_detail.show_placeholder("Choose an artist from the list.");

        // Master list only when Artists (or similar) needs it — never leave a dead column.
        let master_pane = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        master_pane.set_visible(false);
        master_pane.append(&artists.widget);
        let master_sep = gtk::Separator::new(gtk::Orientation::Vertical);
        master_sep.set_visible(false);

        let detail_stack = gtk::Stack::builder()
            .transition_type(gtk::StackTransitionType::Crossfade)
            .transition_duration(180)
            .vexpand(true)
            .hexpand(true)
            .build();
        detail_stack.add_named(&home.widget, Some("home"));
        detail_stack.add_named(&artist_detail.widget, Some("artist"));
        detail_stack.add_named(&albums.widget, Some("albums"));
        detail_stack.add_named(&songs.widget, Some("songs"));
        detail_stack.add_named(&playlists.widget, Some("playlists"));
        detail_stack.add_named(&queue_view.widget, Some("queue"));
        detail_stack.add_named(&search.widget, Some("search"));
        detail_stack.add_named(&empty.widget, Some("empty"));

        let panes = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        panes.set_vexpand(true);
        panes.append(&nav_scroll);
        panes.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        panes.append(&master_pane);
        panes.append(&master_sep);
        panes.append(&detail_stack);

        let library_shell = gtk::Box::new(gtk::Orientation::Vertical, 0);
        library_shell.set_vexpand(true);
        library_shell.append(&scan_banner);
        library_shell.append(&panes);
        library_shell.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        library_shell.append(&player_bar.widget);

        let now_playing_revealer = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::SlideUp)
            .transition_duration(280)
            .reveal_child(false)
            .hexpand(true)
            .vexpand(true)
            .child(&now_playing.widget)
            .build();
        // Critical: a full-size overlay child still intercepts clicks when
        // collapsed unless targeting is disabled.
        now_playing_revealer.set_can_target(false);
        now_playing_revealer.set_visible(false);

        let overlay = gtk::Overlay::new();
        overlay.set_child(Some(&library_shell));
        overlay.add_overlay(&now_playing_revealer);
        now_playing_revealer.set_halign(gtk::Align::Fill);
        now_playing_revealer.set_valign(gtk::Align::Fill);

        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&overlay));
        toast.set_child(Some(&toolbar));
        window.set_content(Some(&toast));

        let (player_tx, player_rx) = std::sync::mpsc::channel::<PlayerEvent>();
        let player = Player::new(move |event| {
            let _ = player_tx.send(event);
        });

        let state_slot: Rc<RefCell<Option<Rc<AppState>>>> = Rc::new(RefCell::new(None));
        let mpris = {
            let state_slot = Rc::clone(&state_slot);
            mpris::start_mpris(move |cmd| {
                let Some(state) = state_slot.borrow().clone() else {
                    return;
                };
                match cmd {
                    MprisCommand::Play => state.player.play(),
                    MprisCommand::Pause => state.player.pause(),
                    MprisCommand::PlayPause => state.player.toggle(),
                    MprisCommand::Stop => state.player.stop(),
                    MprisCommand::Next => play_next(&state),
                    MprisCommand::Previous => {
                        let track = state.queue.borrow_mut().previous().cloned();
                        if let Some(track) = track {
                            start_track(&state, &track);
                        }
                    }
                }
            })
        };

        let state = Rc::new(AppState {
            library,
            player: Rc::new(player),
            queue: RefCell::new(Queue::new()),
            player_bar,
            now_playing,
            queue_view,
            home,
            artists,
            artist_detail,
            albums,
            songs,
            playlists,
            search,
            empty,
            master_pane: master_pane.clone(),
            master_sep: master_sep.clone(),
            detail_stack: detail_stack.clone(),
            now_playing_revealer: now_playing_revealer.clone(),
            context_tracks: RefCell::new(Vec::new()),
            toast: toast.clone(),
            scan_banner: scan_banner.clone(),
            lookup_spinner: lookup_spinner.clone(),
            mpris,
            last_undo: RefCell::new(None),
            nav: Cell::new(Nav::Library),
            has_library: Cell::new(false),
            previous_detail: RefCell::new("home".into()),
            current_track: RefCell::new(None),
        });
        *state_slot.borrow_mut() = Some(Rc::clone(&state));

        {
            let state = Rc::clone(&state);
            glib::timeout_add_local(Duration::from_millis(50), move || {
                while let Ok(event) = player_rx.try_recv() {
                    match event {
                        PlayerEvent::StateChanged(s) => {
                            let playing = s == PlaybackState::Playing;
                            state.player_bar.set_playing(playing);
                            state.now_playing.set_playing(playing);
                        }
                        PlayerEvent::EndOfStream => play_next(&state),
                        PlayerEvent::Error(err) => {
                            tracing::warn!(%err, "playback error");
                            state.player_bar.set_playing(false);
                            state.now_playing.set_playing(false);
                            let msg = if err.contains("missing a plug-in") {
                                "Can't play this file — a GStreamer decoder plugin is missing. \
                                 If you sourced .envrc.build, restart without it so system plugins load."
                                    .to_string()
                            } else {
                                format!("Playback error: {err}")
                            };
                            state.toast.add_toast(
                                adw::Toast::builder().title(msg).timeout(8).build(),
                            );
                        }
                        PlayerEvent::PositionUpdated { .. } => {}
                    }
                }
                ControlFlow::Continue
            });
        }
        {
            let state = Rc::clone(&state);
            glib::timeout_add_local(Duration::from_millis(250), move || {
                let pos = state.player.position_ns();
                let dur = state.player.duration_ns();
                if dur > 0 {
                    let pos_ms = pos / 1_000_000;
                    let dur_ms = dur / 1_000_000;
                    state.player_bar.update_position(pos_ms, dur_ms);
                    state.now_playing.update_position(pos_ms, dur_ms);
                }
                ControlFlow::Continue
            });
        }

        wire_player_controls(&state);
        wire_library_events(&state, event_rx);
        wire_navigation(&state, &sidebar);
        wire_views(&state, &window);
        wire_search(&state, &search_entry);
        wire_add_folder(&state, &window, &add_btn);
        wire_home_actions(&state, &window);
        wire_actions(&state, &window);
        wire_shortcuts(&state, &window);

        {
            let state = Rc::clone(&state);
            let library = state.library.clone();
            library.list_folders(move |result| {
                let folders = result.unwrap_or_default();
                if folders.is_empty() {
                    set_master_visible(&state, false);
                    state.detail_stack.set_visible_child_name("empty");
                    state.has_library.set(false);
                } else {
                    state.has_library.set(true);
                    show_nav(&state, Nav::Library);
                    state.library.scan_all(|_| {});
                }
            });
        }

        Self {
            window,
            _state: state,
        }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn brand_widget() -> gtk::Box {
    let icon = gtk::Image::from_icon_name(cadence_core::APP_ID);
    icon.set_pixel_size(28);
    icon.add_css_class("cadence-brand-icon");

    let name = gtk::Label::builder()
        .use_markup(true)
        .label("Cadence<span foreground=\"#A882FF\">.</span>")
        .css_classes(["cadence-brand"])
        .valign(gtk::Align::Center)
        .build();

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("cadence-brand-row");
    row.set_margin_start(4);
    row.set_margin_end(8);
    row.append(&icon);
    row.append(&name);
    row
}

fn build_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some("Preferences"), Some("win.preferences"));
    menu.append(Some("Scan Library"), Some("win.scan-library"));
    menu.append(Some("Organise Library"), Some("win.organize"));
    menu.append(Some("Edit Metadata"), Some("win.edit-metadata"));
    menu.append(Some("Lookup Metadata"), Some("win.lookup-metadata"));
    menu.append(Some("Undo Organisation"), Some("win.undo-organize"));
    menu.append(Some("About Cadence"), Some("app.about"));
    menu.append(Some("Quit"), Some("app.quit"));
    menu
}

fn nav_row(label: &str, icon: &str) -> gtk::ListBoxRow {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.set_margin_start(12);
    row.set_margin_end(12);
    row.set_margin_top(8);
    row.set_margin_bottom(8);
    row.append(&gtk::Image::from_icon_name(icon));
    row.append(&gtk::Label::builder().label(label).xalign(0.0).build());
    gtk::ListBoxRow::builder().child(&row).build()
}

fn show_detail(state: &Rc<AppState>, name: &str) {
    if name != "queue" {
        *state.previous_detail.borrow_mut() = name.to_string();
    }
    state.detail_stack.set_visible_child_name(name);
}

fn set_master_visible(state: &Rc<AppState>, visible: bool) {
    state.master_pane.set_visible(visible);
    state.master_sep.set_visible(visible);
}

fn set_now_playing_open(state: &Rc<AppState>, open: bool) {
    state.now_playing_revealer.set_visible(open);
    state.now_playing_revealer.set_can_target(open);
    state.now_playing_revealer.set_reveal_child(open);
}

fn wire_player_controls(state: &Rc<AppState>) {
    let play = state.player_bar.play_button.clone();
    let prev = state.player_bar.prev_button.clone();
    let next = state.player_bar.next_button.clone();
    let shuffle = state.player_bar.shuffle_button.clone();
    let repeat = state.player_bar.repeat_button.clone();
    let seek = state.player_bar.seek.clone();
    let volume = state.player_bar.volume.clone();
    let queue_btn = state.player_bar.queue_button.clone();
    let fav = state.player_bar.favorite_button.clone();

    {
        let state = Rc::clone(state);
        play.connect_clicked(move |_| state.player.toggle());
    }
    {
        let state_cb = Rc::clone(state);
        state.now_playing.play_button.connect_clicked(move |_| {
            state_cb.player.toggle();
        });
    }
    {
        let state = Rc::clone(state);
        prev.connect_clicked(move |_| {
            let track = state.queue.borrow_mut().previous().cloned();
            if let Some(track) = track {
                start_track(&state, &track);
            }
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.now_playing.prev_button.connect_clicked(move |_| {
            let track = state_cb.queue.borrow_mut().previous().cloned();
            if let Some(track) = track {
                start_track(&state_cb, &track);
            }
        });
    }
    {
        let state = Rc::clone(state);
        next.connect_clicked(move |_| play_next(&state));
    }
    {
        let state_cb = Rc::clone(state);
        state
            .now_playing
            .next_button
            .connect_clicked(move |_| play_next(&state_cb));
    }
    {
        let state = Rc::clone(state);
        shuffle.connect_toggled(move |btn| {
            state.queue.borrow_mut().set_shuffle(btn.is_active());
        });
    }
    {
        let state = Rc::clone(state);
        repeat.connect_clicked(move |btn| {
            let mode = match state.queue.borrow().repeat_mode() {
                RepeatMode::Off => RepeatMode::All,
                RepeatMode::All => RepeatMode::One,
                RepeatMode::One => RepeatMode::Off,
            };
            state.queue.borrow_mut().set_repeat(mode);
            btn.set_tooltip_text(Some(match mode {
                RepeatMode::Off => "Repeat: Off",
                RepeatMode::All => "Repeat: All",
                RepeatMode::One => "Repeat: One",
            }));
        });
    }
    {
        let state = Rc::clone(state);
        seek.connect_change_value(move |_, _, value| {
            state.player.seek_fraction(value);
            state.player_bar.finish_seek();
            glib::Propagation::Proceed
        });
    }
    {
        let state_cb = Rc::clone(state);
        state
            .now_playing
            .seek_widget()
            .connect_change_value(move |_, _, value| {
                state_cb.player.seek_fraction(value);
                state_cb.now_playing.finish_seek();
                glib::Propagation::Proceed
            });
    }
    {
        let state = Rc::clone(state);
        volume.connect_value_changed(move |s| state.player.set_volume(s.value()));
    }
    {
        let state = Rc::clone(state);
        queue_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                set_now_playing_open(&state, false);
                let q = state.queue.borrow();
                state.queue_view.set_tracks(q.tracks(), q.current_index());
                show_detail(&state, "queue");
            } else {
                let prev = state.previous_detail.borrow().clone();
                show_detail(&state, &prev);
            }
        });
    }
    {
        let state = Rc::clone(state);
        fav.connect_toggled(move |btn| {
            let Some(track) = state.current_track.borrow().clone() else {
                return;
            };
            let active = btn.is_active();
            btn.set_icon_name(if active {
                "starred-symbolic"
            } else {
                "non-starred-symbolic"
            });
            state.library.set_favorite(track.id, active, |_| {});
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.player_bar.connect_expand(move || {
            set_now_playing_open(&state_cb, true);
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.now_playing.close_button.connect_clicked(move |_| {
            set_now_playing_open(&state_cb, false);
        });
    }
}

fn wire_library_events(state: &Rc<AppState>, rx: std::sync::mpsc::Receiver<LibraryEvent>) {
    let state = Rc::clone(state);
    LibraryService::attach_events(rx, move |event| match event {
        LibraryEvent::ScanProgress { done, total } => {
            state
                .scan_banner
                .set_title(&format!("Scanning… {done}/{total}"));
            state.scan_banner.set_revealed(true);
        }
        LibraryEvent::LookupProgress { phase, done, total } => {
            set_lookup_busy(&state, true, &format!("{done}/{total} — {phase}"));
        }
        LibraryEvent::ScanFinished { summary } => {
            state.scan_banner.set_revealed(false);
            if let Some(title) = summary.toast_message() {
                state.toast.add_toast(
                    adw::Toast::builder()
                        .title(title)
                        .timeout(4)
                        .build(),
                );
            }
            state.has_library.set(true);
            show_nav(&state, state.nav.get());
        }
        LibraryEvent::LibraryChanged => {
            state.has_library.set(true);
            show_nav(&state, state.nav.get());
        }
        LibraryEvent::Error(err) => state.toast.add_toast(adw::Toast::new(&err)),
    });
}

fn wire_navigation(state: &Rc<AppState>, sidebar: &gtk::ListBox) {
    let state = Rc::clone(state);
    sidebar.connect_row_selected(move |_, row| {
        let Some(row) = row else { return };
        if !state.has_library.get() {
            set_master_visible(&state, false);
            show_detail(&state, "empty");
            return;
        }
        let nav = match row.index() {
            0 => Nav::Library,
            1 => Nav::Artists,
            2 => Nav::Albums,
            3 => Nav::Songs,
            4 => Nav::Playlists,
            5 => Nav::Favourites,
            6 => Nav::Recent,
            _ => Nav::Library,
        };
        show_nav(&state, nav);
    });
}

fn show_nav(state: &Rc<AppState>, nav: Nav) {
    state.nav.set(nav);
    state.player_bar.queue_button.set_active(false);

    match nav {
        Nav::Library => {
            set_master_visible(state, false);
            show_detail(state, "home");
            refresh_home(state);
        }
        Nav::Artists => {
            set_master_visible(state, true);
            show_detail(state, "artist");
            let state2 = Rc::clone(state);
            state.library.list_artists(move |result| {
                if let Ok(artists) = result {
                    state2.artists.set_artists(artists);
                }
            });
        }
        Nav::Albums => {
            set_master_visible(state, false);
            show_detail(state, "albums");
            load_albums_grid(state);
        }
        Nav::Songs => {
            set_master_visible(state, false);
            show_detail(state, "songs");
            load_songs_page(state, 0);
        }
        Nav::Playlists => {
            set_master_visible(state, false);
            show_detail(state, "playlists");
            let state2 = Rc::clone(state);
            state.library.playlists(move |result| {
                if let Ok(list) = result {
                    state2.playlists.set_playlists(list);
                }
            });
        }
        Nav::Favourites => {
            set_master_visible(state, false);
            show_detail(state, "songs");
            let state2 = Rc::clone(state);
            state.library.favorites_display(move |result| {
                if let Ok(tracks) = result {
                    *state2.context_tracks.borrow_mut() =
                        tracks.iter().map(|t| t.track.clone()).collect();
                    state2.songs.replace_display(tracks);
                    state2.songs.set_has_more(false);
                }
            });
        }
        Nav::Recent => {
            set_master_visible(state, false);
            show_detail(state, "songs");
            let state2 = Rc::clone(state);
            state.library.recently_added_display(200, move |result| {
                if let Ok(tracks) = result {
                    *state2.context_tracks.borrow_mut() =
                        tracks.iter().map(|t| t.track.clone()).collect();
                    state2.songs.replace_display(tracks);
                    state2.songs.set_has_more(false);
                }
            });
        }
    }
}

fn refresh_home(state: &Rc<AppState>) {
    let state_c = Rc::clone(state);
    state.library.recently_played_display(12, move |result| {
        if let Ok(tracks) = result {
            state_c.home.set_continue(tracks);
        }
    });
    let state_r = Rc::clone(state);
    state.library.recently_added_display(12, move |result| {
        if let Ok(tracks) = result {
            state_r.home.set_recent(tracks);
        }
    });
    let state_a = Rc::clone(state);
    state.library.list_albums(move |result| {
        let Ok(albums) = result else { return };
        let mut albums = albums;
        albums.sort_by(|a, b| b.id.cmp(&a.id));
        albums.truncate(12);
        resolve_album_artists(&state_a, albums, {
            let state_a = Rc::clone(&state_a);
            move |albums, names| {
                let pairs: Vec<_> = albums.into_iter().zip(names).collect();
                state_a.home.set_recent_albums(pairs);
            }
        });
    });
    let state_s = Rc::clone(state);
    state.library.list_artists(move |artists| {
        let artist_n = artists.ok().map(|a| a.len() as u64).unwrap_or(0);
        let state_s2 = Rc::clone(&state_s);
        state_s.library.list_albums(move |albums| {
            let album_n = albums.ok().map(|a| a.len() as u64).unwrap_or(0);
            let state_s3 = Rc::clone(&state_s2);
            state_s2.library.track_count(move |songs| {
                let song_n = songs.unwrap_or(0);
                state_s3.home.set_stats(artist_n, album_n, song_n);
            });
        });
    });
}

fn load_albums_grid(state: &Rc<AppState>) {
    let state2 = Rc::clone(state);
    state.library.list_albums(move |result| {
        let Ok(albums) = result else { return };
        resolve_album_artists(&state2, albums, {
            let state2 = Rc::clone(&state2);
            move |albums, names| {
                state2.albums.set_albums(albums, &names);
            }
        });
    });
}

fn resolve_album_artists(
    state: &Rc<AppState>,
    albums: Vec<Album>,
    cont: impl FnOnce(Vec<Album>, Vec<String>) + 'static,
) {
    if albums.is_empty() {
        cont(albums, Vec::new());
        return;
    }
    let names = Rc::new(RefCell::new(vec![String::new(); albums.len()]));
    let remaining = Rc::new(Cell::new(albums.len()));
    let albums_rc = Rc::new(albums);
    let cont: Rc<RefCell<Option<Box<dyn FnOnce(Vec<Album>, Vec<String>)>>>> =
        Rc::new(RefCell::new(Some(Box::new(cont))));

    for (i, album) in albums_rc.iter().enumerate() {
        let names = Rc::clone(&names);
        let remaining = Rc::clone(&remaining);
        let albums_rc = Rc::clone(&albums_rc);
        let cont = Rc::clone(&cont);
        match album.album_artist_id {
            Some(id) => {
                state.library.artist_name(id, move |result| {
                    names.borrow_mut()[i] = result
                        .ok()
                        .flatten()
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "Unknown Artist".into());
                    let left = remaining.get().saturating_sub(1);
                    remaining.set(left);
                    if left == 0 {
                        if let Some(cb) = cont.borrow_mut().take() {
                            cb((*albums_rc).clone(), names.borrow().clone());
                        }
                    }
                });
            }
            None => {
                names.borrow_mut()[i] = "Unknown Artist".into();
                let left = remaining.get().saturating_sub(1);
                remaining.set(left);
                if left == 0 {
                    if let Some(cb) = cont.borrow_mut().take() {
                        cb((*albums_rc).clone(), names.borrow().clone());
                    }
                }
            }
        }
    }
}

fn load_songs_page(state: &Rc<AppState>, offset: usize) {
    let sort = state.songs.current_sort();
    let state2 = Rc::clone(state);
    state
        .library
        .list_songs_display(sort, offset, PAGE_SIZE, move |result| {
            if let Ok(tracks) = result {
                let has_more = tracks.len() >= PAGE_SIZE;
                *state2.context_tracks.borrow_mut() = if offset == 0 {
                    tracks.iter().map(|t| t.track.clone()).collect()
                } else {
                    let mut all = state2.context_tracks.borrow().clone();
                    all.extend(tracks.iter().map(|t| t.track.clone()));
                    all
                };
                if offset == 0 {
                    state2.songs.replace_display(tracks);
                } else {
                    state2.songs.append_display(tracks);
                }
                state2.songs.set_has_more(has_more);
            }
        });
}

fn wire_views(state: &Rc<AppState>, window: &adw::ApplicationWindow) {
    {
        let state_cb = Rc::clone(state);
        state.artists.connect_select(move |artist_id| {
            show_artist_detail(&state_cb, artist_id);
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.artist_detail.connect_play(move |tracks, index| {
            *state_cb.context_tracks.borrow_mut() = tracks.clone();
            play_list(&state_cb, tracks, index);
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.albums.connect_activate(move |album_id| {
            show_album_tracks(&state_cb, album_id);
        });
    }
    {
        let state_cb = Rc::clone(state);
        state
            .songs
            .connect_activate(move |tracks, index| play_list(&state_cb, tracks, index));
    }
    {
        let state_cb = Rc::clone(state);
        state.songs.connect_load_page(move |_sort, offset| {
            load_songs_page(&state_cb, offset);
        });
    }
    {
        let state_cb = Rc::clone(state);
        let window = window.clone();
        let new_btn = state.playlists.new_button.clone();
        new_btn.connect_clicked(move |_| {
            let dialog = adw::AlertDialog::builder().heading("New Playlist").build();
            let entry = gtk::Entry::new();
            entry.set_placeholder_text(Some("Playlist name"));
            dialog.set_extra_child(Some(&entry));
            dialog.add_response("cancel", "Cancel");
            dialog.add_response("create", "Create");
            dialog.set_response_appearance("create", adw::ResponseAppearance::Suggested);
            let state = Rc::clone(&state_cb);
            dialog.connect_response(None, move |_, response| {
                if response != "create" {
                    return;
                }
                let name = entry.text().to_string();
                if name.is_empty() {
                    return;
                }
                let state2 = Rc::clone(&state);
                state.library.create_playlist(name, move |_| {
                    show_nav(&state2, Nav::Playlists);
                });
            });
            dialog.present(Some(&window));
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.playlists.connect_activate(move |id| {
            if let Some(pl) = state_cb.playlists.playlist(id) {
                let state2 = Rc::clone(&state_cb);
                load_tracks_by_ids(&state_cb.library, pl.track_ids, move |tracks| {
                    *state2.context_tracks.borrow_mut() = tracks.clone();
                    let display: Vec<TrackDisplay> = tracks
                        .into_iter()
                        .map(|track| TrackDisplay {
                            track,
                            artist_name: String::new(),
                            album_name: String::new(),
                            artwork_path: None,
                        })
                        .collect();
                    state2.songs.replace_display(display);
                    state2.songs.set_has_more(false);
                    show_detail(&state2, "songs");
                });
            }
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.home.connect_play(move |items, index| {
            let tracks: Vec<Track> = items.into_iter().map(|t| t.track).collect();
            play_list(&state_cb, tracks, index);
        });
    }
    {
        let state_home = Rc::clone(state);
        state.home.connect_favorite(move |id, active| {
            state_home.library.set_favorite(id, active, |_| {});
        });
        let state_songs = Rc::clone(state);
        state.songs.connect_favorite(move |id, active| {
            state_songs.library.set_favorite(id, active, |_| {});
        });
        let state_search = Rc::clone(state);
        state.search.connect_favorite(move |id, active| {
            state_search.library.set_favorite(id, active, |_| {});
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.search.connect_artist(move |id| {
            state_cb.nav.set(Nav::Artists);
            set_master_visible(&state_cb, true);
            let state2 = Rc::clone(&state_cb);
            state_cb.library.list_artists(move |result| {
                if let Ok(artists) = result {
                    state2.artists.set_artists(artists);
                }
            });
            show_artist_detail(&state_cb, id);
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.home.connect_album(move |id| {
            show_album_tracks(&state_cb, id);
        });
    }
    wire_context_menus(state, window);
    {
        let state_cb = Rc::clone(state);
        state.search.connect_album(move |id| {
            show_album_tracks(&state_cb, id);
        });
    }
    {
        let state_cb = Rc::clone(state);
        state
            .search
            .connect_play(move |tracks, index| play_list(&state_cb, tracks, index));
    }
    {
        let state_cb = Rc::clone(state);
        state.search.connect_genre(move |genre| {
            let state2 = Rc::clone(&state_cb);
            state_cb
                .library
                .list_songs_display(cadence_core::db::SongSort::TitleAsc, 0, 500, move |result| {
                    if let Ok(tracks) = result {
                        let filtered: Vec<_> = tracks
                            .into_iter()
                            .filter(|t| {
                                t.track
                                    .genre
                                    .as_deref()
                                    .is_some_and(|g| g.eq_ignore_ascii_case(&genre))
                            })
                            .collect();
                        *state2.context_tracks.borrow_mut() =
                            filtered.iter().map(|t| t.track.clone()).collect();
                        state2.songs.replace_display(filtered);
                        state2.songs.set_has_more(false);
                        set_master_visible(&state2, false);
                        show_detail(&state2, "songs");
                        state2.toast.add_toast(adw::Toast::new(&format!("Genre · {genre}")));
                    }
                });
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.search.connect_year(move |year| {
            let state2 = Rc::clone(&state_cb);
            state_cb.library.list_albums(move |result| {
                if let Ok(albums) = result {
                    let filtered: Vec<_> = albums
                        .into_iter()
                        .filter(|a| a.year == Some(year))
                        .collect();
                    resolve_album_artists(&state2, filtered, {
                        let state2 = Rc::clone(&state2);
                        move |albums, names| {
                            state2.albums.set_albums(albums, &names);
                            set_master_visible(&state2, false);
                            show_detail(&state2, "albums");
                            state2
                                .toast
                                .add_toast(adw::Toast::new(&format!("Year · {year}")));
                        }
                    });
                }
            });
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.search.connect_folder(move |path| {
            state_cb.toast.add_toast(adw::Toast::new(&format!(
                "Library folder · {}",
                path.display()
            )));
        });
    }
}

fn show_artist_detail(state: &Rc<AppState>, artist_id: ArtistId) {
    show_detail(state, "artist");
    let state_a = Rc::clone(state);
    state.library.get_artist(artist_id, move |artist_res| {
        let Ok(artist) = artist_res else { return };
        let state_b = Rc::clone(&state_a);
        let artist_clone = artist.clone();
        state_a
            .library
            .artist_duration_ms(artist_id, move |dur_res| {
                let duration_ms = dur_res.unwrap_or(0);
                let state_c = Rc::clone(&state_b);
                state_b.library.albums_by_artist(artist_id, move |albums_res| {
                    let Ok(albums) = albums_res else { return };
                    let state_singles = Rc::clone(&state_c);
                    let artist_for_singles = artist_clone.clone();
                    state_c
                        .library
                        .singles_by_artist_display(artist_id, move |singles_res| {
                            let singles = singles_res.unwrap_or_default();
                            if albums.is_empty() {
                                state_singles.artist_detail.set_artist(
                                    &artist_for_singles,
                                    duration_ms,
                                    Vec::new(),
                                    singles,
                                );
                                return;
                            }
                            let total = albums.len();
                            let collected = Rc::new(RefCell::new(vec![None; total]));
                            let remaining = Rc::new(Cell::new(total));
                            let artist_for_ui = artist_for_singles.clone();
                            let singles = Rc::new(singles);
                            for (i, album) in albums.into_iter().enumerate() {
                                let collected = Rc::clone(&collected);
                                let remaining = Rc::clone(&remaining);
                                let state_d = Rc::clone(&state_singles);
                                let artist_for_ui = artist_for_ui.clone();
                                let singles = Rc::clone(&singles);
                                let album_id = album.id;
                                state_singles.library.tracks_by_album_display(
                                    album_id,
                                    move |tracks_res| {
                                        let tracks = tracks_res.unwrap_or_default();
                                        collected.borrow_mut()[i] = Some((album, tracks));
                                        let left = remaining.get().saturating_sub(1);
                                        remaining.set(left);
                                        if left == 0 {
                                            let pairs: Vec<_> = collected
                                                .borrow_mut()
                                                .drain(..)
                                                .flatten()
                                                .collect();
                                            state_d.artist_detail.set_artist(
                                                &artist_for_ui,
                                                duration_ms,
                                                pairs,
                                                (*singles).clone(),
                                            );
                                        }
                                    },
                                );
                            }
                        });
                });
            });
    });
}

fn show_album_tracks(state: &Rc<AppState>, album_id: AlbumId) {
    let state2 = Rc::clone(state);
    state.library.tracks_by_album_display(album_id, move |result| {
        if let Ok(tracks) = result {
            *state2.context_tracks.borrow_mut() =
                tracks.iter().map(|t| t.track.clone()).collect();
            state2.songs.replace_display(tracks);
            state2.songs.set_has_more(false);
            show_detail(&state2, "songs");
        }
    });
}

fn wire_search(state: &Rc<AppState>, search: &gtk::SearchEntry) {
    let state = Rc::clone(state);
    let token = Rc::new(Cell::new(0u32));
    search.connect_search_changed(move |entry| {
        let query = entry.text().to_string();
        let t = token.get().wrapping_add(1);
        token.set(t);
        let state = Rc::clone(&state);
        let token = Rc::clone(&token);
        if query.trim().is_empty() {
            show_nav(&state, state.nav.get());
            return;
        }
        glib::timeout_add_local(Duration::from_millis(220), move || {
            if token.get() != t {
                return ControlFlow::Break;
            }
            run_global_search(&state, query.clone());
            ControlFlow::Break
        });
    });
}

fn run_global_search(state: &Rc<AppState>, query: String) {
    let q_lower = query.to_lowercase();
    let state_songs = Rc::clone(state);
    let query_songs = query.clone();
    let artists_holder = Rc::new(RefCell::new(None));
    let albums_holder = Rc::new(RefCell::new(None));
    let songs_holder = Rc::new(RefCell::new(None));
    let genres_holder = Rc::new(RefCell::new(None));
    let years_holder = Rc::new(RefCell::new(None));
    let folders_holder = Rc::new(RefCell::new(None));
    let pending = Rc::new(Cell::new(6u8));

    let finish = {
        let state = Rc::clone(state);
        let query = query.clone();
        let artists_holder = Rc::clone(&artists_holder);
        let albums_holder = Rc::clone(&albums_holder);
        let songs_holder = Rc::clone(&songs_holder);
        let genres_holder = Rc::clone(&genres_holder);
        let years_holder = Rc::clone(&years_holder);
        let folders_holder = Rc::clone(&folders_holder);
        let pending = Rc::clone(&pending);
        Rc::new(move || {
            let left = pending.get().saturating_sub(1);
            pending.set(left);
            if left != 0 {
                return;
            }
            let artists = artists_holder.borrow_mut().take().unwrap_or_default();
            let albums = albums_holder.borrow_mut().take().unwrap_or_default();
            let songs = songs_holder.borrow_mut().take().unwrap_or_default();
            let genres = genres_holder.borrow_mut().take().unwrap_or_default();
            let years = years_holder.borrow_mut().take().unwrap_or_default();
            let folders = folders_holder.borrow_mut().take().unwrap_or_default();
            state.search.set_results(
                &query, artists, albums, songs, genres, years, folders,
            );
            set_master_visible(&state, false);
            show_detail(&state, "search");
        })
    };

    {
        let holder = Rc::clone(&artists_holder);
        let finish = Rc::clone(&finish);
        let q = q_lower.clone();
        state.library.list_artists(move |result| {
            let mut artists = result.unwrap_or_default();
            artists.retain(|a| a.name.to_lowercase().contains(&q));
            artists.truncate(20);
            *holder.borrow_mut() = Some(artists);
            finish();
        });
    }
    {
        let holder = Rc::clone(&albums_holder);
        let finish = Rc::clone(&finish);
        let q = q_lower.clone();
        let state2 = Rc::clone(state);
        state.library.list_albums(move |result| {
            let albums = result.unwrap_or_default();
            let filtered: Vec<Album> = albums
                .into_iter()
                .filter(|a| a.name.to_lowercase().contains(&q))
                .take(20)
                .collect();
            resolve_album_artists(&state2, filtered, move |albums, names| {
                let pairs: Vec<_> = albums.into_iter().zip(names).collect();
                *holder.borrow_mut() = Some(pairs);
                finish();
            });
        });
    }
    {
        let holder = Rc::clone(&songs_holder);
        let finish = Rc::clone(&finish);
        state_songs
            .library
            .search_display(query_songs, move |result| {
                *holder.borrow_mut() = Some(result.unwrap_or_default());
                finish();
            });
    }
    {
        let holder = Rc::clone(&genres_holder);
        let finish = Rc::clone(&finish);
        let q = q_lower.clone();
        state.library.list_genres(move |result| {
            let mut genres = result.unwrap_or_default();
            genres.retain(|g| g.to_lowercase().contains(&q));
            genres.truncate(20);
            *holder.borrow_mut() = Some(genres);
            finish();
        });
    }
    {
        let holder = Rc::clone(&years_holder);
        let finish = Rc::clone(&finish);
        let q = q_lower.clone();
        state.library.list_years(move |result| {
            let mut years = result.unwrap_or_default();
            years.retain(|y| y.to_string().contains(&q));
            years.truncate(20);
            *holder.borrow_mut() = Some(years);
            finish();
        });
    }
    {
        let holder = Rc::clone(&folders_holder);
        let finish = Rc::clone(&finish);
        let q = q_lower.clone();
        state.library.list_folders(move |result| {
            let mut folders = result.unwrap_or_default();
            folders.retain(|p| p.display().to_string().to_lowercase().contains(&q));
            folders.truncate(20);
            *holder.borrow_mut() = Some(folders);
            finish();
        });
    }
}

fn wire_add_folder(
    state: &Rc<AppState>,
    window: &adw::ApplicationWindow,
    add_btn: &gtk::Button,
) {
    let open = {
        let state = Rc::clone(state);
        let window = window.clone();
        Rc::new(move || {
            let dialog = gtk::FileDialog::builder()
                .title("Add Music Folder")
                .modal(true)
                .build();
            let state = Rc::clone(&state);
            dialog.select_folder(Some(&window), gio::Cancellable::NONE, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        let state2 = Rc::clone(&state);
                        state.library.add_folder(path, move |result| {
                            if let Err(err) = result {
                                state2.toast.add_toast(adw::Toast::new(&err.to_string()));
                            } else {
                                state2.has_library.set(true);
                                show_nav(&state2, Nav::Library);
                            }
                        });
                    }
                }
            });
        })
    };
    {
        let open = Rc::clone(&open);
        add_btn.connect_clicked(move |_| open());
    }
    {
        let open = Rc::clone(&open);
        state.empty.add_button.connect_clicked(move |_| open());
    }
}

fn wire_home_actions(state: &Rc<AppState>, window: &adw::ApplicationWindow) {
    {
        let state_cb = Rc::clone(state);
        let window = window.clone();
        state.home.organise_button.connect_clicked(move |_| {
            open_organise(&state_cb, &window);
        });
    }
    {
        let state_cb = Rc::clone(state);
        state.home.lookup_button.connect_clicked(move |_| {
            run_library_lookup(&state_cb);
        });
    }
}

fn set_lookup_busy(state: &AppState, busy: bool, detail: &str) {
    if busy {
        state.lookup_spinner.set_visible(true);
        state.lookup_spinner.start();
        state
            .lookup_spinner
            .set_tooltip_text(Some(&format!("Looking up metadata — {detail}")));
    } else {
        state.lookup_spinner.stop();
        state.lookup_spinner.set_visible(false);
        state
            .lookup_spinner
            .set_tooltip_text(Some("Metadata lookup idle"));
    }
}

fn run_library_lookup(state: &Rc<AppState>) {
    set_lookup_busy(state, true, "starting…");
    let toast = state.toast.clone();
    let state_done = Rc::clone(state);
    state.library.fill_missing_metadata(move |result| {
        set_lookup_busy(&state_done, false, "");
        match result {
            Ok(s) => {
                let title = if s.albums_scanned == 0 {
                    "Nothing missing — metadata looks complete".into()
                } else {
                    format!(
                        "Done — {} albums checked · {} artwork · {} genres · {} tags · {} need review",
                        s.albums_scanned,
                        s.artwork_updated,
                        s.genres_fixed,
                        s.metadata_updated,
                        s.needs_review
                    )
                };
                toast.add_toast(
                    adw::Toast::builder().title(title).timeout(8).build(),
                );
            }
            Err(err) => toast.add_toast(adw::Toast::new(&err.to_string())),
        }
    });
}

fn open_organise(state: &Rc<AppState>, window: &adw::ApplicationWindow) {
    let org = Rc::new(OrganizeDialog::new(window));
    {
        let org_ui = Rc::clone(&org);
        let state = Rc::clone(state);
        org.connect_preview(move || {
            org_ui.set_busy(true);
            let template = org_ui.selected_template();
            let org_ui = Rc::clone(&org_ui);
            let library = state.library.clone();
            library.clone().list_folders(move |folders| {
                let roots = folders.unwrap_or_default();
                if roots.is_empty() {
                    org_ui.set_busy(false);
                    org_ui.show_error("Add a music folder first.");
                    return;
                }
                // Preview across the first library root (multi-root plans can follow).
                let root = roots[0].clone();
                let org_ui = Rc::clone(&org_ui);
                library.build_organization_plan(root, template, move |result| {
                    org_ui.set_busy(false);
                    match result {
                        Ok(plan) => org_ui.show_plan(plan),
                        Err(err) => org_ui.show_error(&err.to_string()),
                    }
                });
            });
        });
    }
    {
        let org_ui = Rc::clone(&org);
        let state = Rc::clone(state);
        org.connect_apply(move |plan| {
            org_ui.set_busy(true);
            let org_ui = Rc::clone(&org_ui);
            let state2 = Rc::clone(&state);
            state.library.clone().execute_organization(plan, move |result| {
                org_ui.set_busy(false);
                match result {
                    Ok(log) => {
                        *state2.last_undo.borrow_mut() = Some(log);
                        state2.toast.add_toast(
                            adw::Toast::builder()
                                .title("Organisation applied — undo available from the menu")
                                .timeout(6)
                                .build(),
                        );
                        org_ui.close();
                    }
                    Err(err) => {
                        org_ui.show_error(&err.to_string());
                        state2.toast.add_toast(adw::Toast::new(&err.to_string()));
                    }
                }
            });
        });
    }
    // Auto-preview so the window never looks inert.
    org_preview_kickoff(&org, state);
    org.present();
}

fn org_preview_kickoff(org: &Rc<OrganizeDialog>, state: &Rc<AppState>) {
    org.set_busy(true);
    let template = org.selected_template();
    let org_ui = Rc::clone(org);
    let library = state.library.clone();
    library.clone().list_folders(move |folders| {
        let roots = folders.unwrap_or_default();
        if roots.is_empty() {
            org_ui.set_busy(false);
            org_ui.show_error("Add a music folder first.");
            return;
        }
        let root = roots[0].clone();
        library.build_organization_plan(root, template, move |result| {
            org_ui.set_busy(false);
            match result {
                Ok(plan) => org_ui.show_plan(plan),
                Err(err) => org_ui.show_error(&err.to_string()),
            }
        });
    });
}

fn context_track(state: &Rc<AppState>) -> Option<Track> {
    state
        .context_tracks
        .borrow()
        .first()
        .cloned()
        .or_else(|| state.queue.borrow().current().cloned())
}

fn wire_actions(state: &Rc<AppState>, window: &adw::ApplicationWindow) {
    let prefs = gio::SimpleAction::new("preferences", None);
    {
        let state = Rc::clone(state);
        let window = window.clone();
        prefs.connect_activate(move |_, _| {
            PreferencesWindow::new(&window, state.library.clone()).present();
        });
    }
    window.add_action(&prefs);

    let scan_library = gio::SimpleAction::new("scan-library", None);
    {
        let state = Rc::clone(state);
        scan_library.connect_activate(move |_, _| {
            state.scan_banner.set_title("Scanning library…");
            state.scan_banner.set_revealed(true);
            let state_cb = Rc::clone(&state);
            state.library.scan_all(move |result| {
                if let Err(err) = result {
                    state_cb.scan_banner.set_revealed(false);
                    state_cb
                        .toast
                        .add_toast(adw::Toast::new(&err.to_string()));
                }
            });
        });
    }
    window.add_action(&scan_library);

    let organize = gio::SimpleAction::new("organize", None);
    {
        let state = Rc::clone(state);
        let window = window.clone();
        organize.connect_activate(move |_, _| open_organise(&state, &window));
    }
    window.add_action(&organize);

    let edit = gio::SimpleAction::new("edit-metadata", None);
    {
        let state = Rc::clone(state);
        let window = window.clone();
        edit.connect_activate(move |_, _| {
            let Some(track) = context_track(&state) else {
                state.toast.add_toast(adw::Toast::new(
                    "Play a song or open a track list, then edit metadata",
                ));
                return;
            };
            let meta = metadata::read_metadata(&track.path).unwrap_or_default();
            let editor = MetadataDialog::new(&meta);
            let path = track.path.clone();
            let title = editor.title.clone();
            let artist = editor.artist.clone();
            let album = editor.album.clone();
            let album_artist = editor.album_artist.clone();
            let genre = editor.genre.clone();
            let year = editor.year.clone();
            let track_number = editor.track_number.clone();
            let library = state.library.clone();
            let toast = state.toast.clone();
            editor.dialog.connect_response(None, move |_, response| {
                if response != "save" {
                    return;
                }
                let text = |row: &adw::EntryRow| {
                    let t = row.text().to_string();
                    if t.is_empty() {
                        None
                    } else {
                        Some(t)
                    }
                };
                let new_meta = cadence_core::models::TrackMetadata {
                    title: text(&title),
                    artist: text(&artist),
                    album: text(&album),
                    album_artist: text(&album_artist),
                    genre: text(&genre),
                    year: year.text().parse().ok(),
                    track_number: track_number.text().parse().ok(),
                    ..Default::default()
                };
                let toast = toast.clone();
                library.write_metadata(path.clone(), new_meta, move |result| match result {
                    Ok(()) => toast.add_toast(adw::Toast::new("Metadata saved")),
                    Err(err) => toast.add_toast(adw::Toast::new(&err.to_string())),
                });
            });
            editor.dialog.present(Some(&window));
        });
    }
    window.add_action(&edit);

    let lookup = gio::SimpleAction::new("lookup-metadata", None);
    {
        let state = Rc::clone(state);
        lookup.connect_activate(move |_, _| run_library_lookup(&state));
    }
    window.add_action(&lookup);

    let undo = gio::SimpleAction::new("undo-organize", None);
    {
        let state = Rc::clone(state);
        undo.connect_activate(move |_, _| {
            if let Some(log) = state.last_undo.borrow_mut().take() {
                let toast = state.toast.clone();
                state.library.undo_organization(log, move |result| match result {
                    Ok(()) => toast.add_toast(adw::Toast::new("Organisation undone")),
                    Err(err) => toast.add_toast(adw::Toast::new(&err.to_string())),
                });
            } else {
                state
                    .toast
                    .add_toast(adw::Toast::new("Nothing to undo yet"));
            }
        });
    }
    window.add_action(&undo);
}

fn wire_shortcuts(state: &Rc<AppState>, window: &adw::ApplicationWindow) {
    let state = Rc::clone(state);
    let controller = gtk::EventControllerKey::new();
    controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::space {
            state.player.toggle();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(controller);
}

fn play_list(state: &Rc<AppState>, tracks: Vec<Track>, index: usize) {
    *state.context_tracks.borrow_mut() = tracks.clone();
    state.queue.borrow_mut().replace(tracks, index);
    if let Some(track) = state.queue.borrow().current().cloned() {
        start_track(state, &track);
    }
}

fn play_next(state: &Rc<AppState>) {
    // Drop the RefMut before start_track (which also borrows the queue).
    let track = state.queue.borrow_mut().next().cloned();
    if let Some(track) = track {
        start_track(state, &track);
    }
}

fn start_track(state: &Rc<AppState>, track: &Track) {
    *state.current_track.borrow_mut() = Some(track.clone());
    state.player.set_uri_from_path(&track.path);
    state.player.play();
    state.player_bar.set_playing(true);
    state.now_playing.set_playing(true);
    state
        .player_bar
        .set_track_info(&track.title, "…", "", None, track.favorite);
    state
        .now_playing
        .set_track_info(&track.title, "…", "", None);
    {
        let q = state.queue.borrow();
        state.queue_view.set_tracks(q.tracks(), q.current_index());
    }

    let title = track.title.clone();
    let favorite = track.favorite;
    let album_id = track.album_id;
    let artist_id = track.artist_id;
    let track_id = track.id;
    let duration_us = track.duration_ms.map(|ms| (ms * 1000) as i64).unwrap_or(0);

    if let Some(artist_id) = artist_id {
        let state2 = Rc::clone(state);
        let title2 = title.clone();
        state.library.artist_name(artist_id, move |result| {
            let artist = result
                .ok()
                .flatten()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Unknown Artist".into());
            state2
                .player_bar
                .set_track_info(&title2, &artist, "", None, favorite);
            state2
                .now_playing
                .set_track_info(&title2, &artist, "", None);
            state2.mpris.update_track(
                &title2,
                &artist,
                "",
                None,
                duration_us,
                PlaybackState::Playing,
            );
        });
    } else {
        state
            .player_bar
            .set_track_info(&title, "Unknown Artist", "", None, favorite);
        state
            .now_playing
            .set_track_info(&title, "Unknown Artist", "", None);
    }

    if let Some(album_id) = album_id {
        let state2 = Rc::clone(state);
        let title2 = title.clone();
        state.library.album(album_id, move |result| {
            if let Ok(album) = result {
                let subtitle = state2.player_bar.subtitle.label().to_string();
                let artist = subtitle
                    .split("  ·  ")
                    .next()
                    .unwrap_or("Unknown Artist")
                    .to_string();
                state2.player_bar.set_track_info(
                    &title2,
                    &artist,
                    &album.name,
                    album.artwork_path.as_deref(),
                    favorite,
                );
                state2.now_playing.set_track_info(
                    &title2,
                    &artist,
                    &album.name,
                    album.artwork_path.as_deref(),
                );
                let art_url = album
                    .artwork_path
                    .as_ref()
                    .and_then(|p| glib::filename_to_uri(p, None).ok())
                    .map(|s| s.to_string());
                state2.mpris.update_track(
                    &title2,
                    &artist,
                    &album.name,
                    art_url.as_deref(),
                    duration_us,
                    PlaybackState::Playing,
                );
            }
        });
    }

    state.library.record_play(track_id, |_| {});
}

fn load_tracks_by_ids(
    library: &LibraryService,
    ids: Vec<TrackId>,
    cont: impl FnOnce(Vec<Track>) + 'static,
) {
    if ids.is_empty() {
        cont(Vec::new());
        return;
    }
    let results = Rc::new(RefCell::new(vec![None; ids.len()]));
    let remaining = Rc::new(Cell::new(ids.len()));
    let cont = Rc::new(RefCell::new(Some(cont)));
    for (i, id) in ids.into_iter().enumerate() {
        let results = Rc::clone(&results);
        let remaining = Rc::clone(&remaining);
        let cont = Rc::clone(&cont);
        library.get_track(id, move |result| {
            if let Ok(track) = result {
                results.borrow_mut()[i] = Some(track);
            }
            let left = remaining.get().saturating_sub(1);
            remaining.set(left);
            if left == 0 {
                let tracks = results.borrow_mut().drain(..).flatten().collect();
                if let Some(cb) = cont.borrow_mut().take() {
                    cb(tracks);
                }
            }
        });
    }
}

fn wire_context_menus(state: &Rc<AppState>, window: &adw::ApplicationWindow) {
    {
        let state_cb = Rc::clone(state);
        let window = window.clone();
        state.songs.connect_context(move |track, action| {
            handle_track_context(&state_cb, &window, track, action);
        });
    }
    {
        let state_cb = Rc::clone(state);
        let window = window.clone();
        state.home.connect_context(move |track, action| {
            handle_track_context(&state_cb, &window, track, action);
        });
    }
    {
        let state_cb = Rc::clone(state);
        let window = window.clone();
        state.search.connect_context(move |track, action| {
            handle_track_context(&state_cb, &window, track, action);
        });
    }
    {
        let state_cb = Rc::clone(state);
        let window = window.clone();
        state.artist_detail.connect_context(move |track, action| {
            handle_track_context(&state_cb, &window, track, action);
        });
    }
    {
        let state_cb = Rc::clone(state);
        let window = window.clone();
        state.albums.connect_context(move |album_id, action| {
            handle_album_context(&state_cb, &window, album_id, action);
        });
    }
    {
        let state_cb = Rc::clone(state);
        let window = window.clone();
        state.home.connect_album_context(move |album_id, action| {
            handle_album_context(&state_cb, &window, album_id, action);
        });
    }
}

fn handle_track_context(
    state: &Rc<AppState>,
    window: &adw::ApplicationWindow,
    track: Track,
    action: ContextAction,
) {
    match action {
        ContextAction::AddToQueue => {
            state.queue.borrow_mut().append(vec![track]);
            let q = state.queue.borrow();
            state.queue_view.set_tracks(q.tracks(), q.current_index());
            state.toast.add_toast(adw::Toast::new("Added to queue"));
        }
        ContextAction::AddToPlaylist => {
            show_playlist_picker(state, window, vec![track.id]);
        }
        ContextAction::Delete => {
            confirm_delete_tracks(state, window, vec![track]);
        }
    }
}

fn handle_album_context(
    state: &Rc<AppState>,
    window: &adw::ApplicationWindow,
    album_id: AlbumId,
    action: ContextAction,
) {
    let state_cb = Rc::clone(state);
    let window = window.clone();
    state.library.tracks_by_album(album_id, move |result| {
        let Ok(tracks) = result else {
            state_cb
                .toast
                .add_toast(adw::Toast::new("Could not load album tracks"));
            return;
        };
        if tracks.is_empty() {
            state_cb
                .toast
                .add_toast(adw::Toast::new("Album has no tracks"));
            return;
        }
        match action {
            ContextAction::AddToQueue => {
                state_cb.queue.borrow_mut().append(tracks);
                let q = state_cb.queue.borrow();
                state_cb
                    .queue_view
                    .set_tracks(q.tracks(), q.current_index());
                state_cb
                    .toast
                    .add_toast(adw::Toast::new("Album added to queue"));
            }
            ContextAction::AddToPlaylist => {
                let ids = tracks.into_iter().map(|t| t.id).collect();
                show_playlist_picker(&state_cb, &window, ids);
            }
            ContextAction::Delete => {
                confirm_delete_album(&state_cb, &window, album_id, tracks.len());
            }
        }
    });
}

fn show_playlist_picker(
    state: &Rc<AppState>,
    window: &adw::ApplicationWindow,
    track_ids: Vec<TrackId>,
) {
    let state_cb = Rc::clone(state);
    let window = window.clone();
    state.library.playlists(move |result| {
        let Ok(playlists) = result else {
            state_cb
                .toast
                .add_toast(adw::Toast::new("Could not load playlists"));
            return;
        };
        if playlists.is_empty() {
            state_cb.toast.add_toast(adw::Toast::new(
                "No playlists yet — create one from Playlists",
            ));
            return;
        }

        let dialog = adw::AlertDialog::builder()
            .heading("Add to playlist")
            .body("Choose a playlist")
            .build();
        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();
        for pl in &playlists {
            list.append(
                &gtk::ListBoxRow::builder()
                    .child(
                        &gtk::Label::builder()
                            .label(&pl.name)
                            .xalign(0.0)
                            .margin_start(12)
                            .margin_end(12)
                            .margin_top(10)
                            .margin_bottom(10)
                            .build(),
                    )
                    .build(),
            );
        }
        list.select_row(list.row_at_index(0).as_ref());
        dialog.set_extra_child(Some(&list));
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("add", "Add");
        dialog.set_response_appearance("add", adw::ResponseAppearance::Suggested);

        let playlists = playlists;
        let track_ids = track_ids;
        let state2 = Rc::clone(&state_cb);
        dialog.connect_response(None, move |_, response| {
            if response != "add" {
                return;
            }
            let Some(row) = list.selected_row() else {
                return;
            };
            let idx = row.index() as usize;
            let Some(pl) = playlists.get(idx) else {
                return;
            };
            let name = pl.name.clone();
            let state3 = Rc::clone(&state2);
            state2
                .library
                .add_to_playlist(pl.id, track_ids.clone(), move |result| match result {
                    Ok(()) => state3.toast.add_toast(adw::Toast::new(&format!(
                        "Added to “{name}”"
                    ))),
                    Err(err) => state3.toast.add_toast(adw::Toast::new(&err.to_string())),
                });
        });
        dialog.present(Some(&window));
    });
}

fn confirm_delete_tracks(
    state: &Rc<AppState>,
    window: &adw::ApplicationWindow,
    tracks: Vec<Track>,
) {
    let n = tracks.len();
    let title = tracks
        .first()
        .map(|t| t.title.as_str())
        .unwrap_or("track");
    let dialog = adw::AlertDialog::builder()
        .heading("Delete permanently?")
        .body(format!(
            "Delete “{title}”{} from Cadence and remove the file{} from disk. This cannot be undone.",
            if n > 1 {
                format!(" and {} other tracks", n - 1)
            } else {
                String::new()
            },
            if n > 1 { "s" } else { "" }
        ))
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("delete", "Delete");
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    let state = Rc::clone(state);
    let ids: Vec<TrackId> = tracks.into_iter().map(|t| t.id).collect();
    dialog.connect_response(None, move |_, response| {
        if response != "delete" {
            return;
        }
        let state2 = Rc::clone(&state);
        state.library.remove_tracks(ids.clone(), move |result| match result {
            Ok(n) => state2.toast.add_toast(adw::Toast::new(&format!(
                "Deleted {n} from library and disk"
            ))),
            Err(err) => state2.toast.add_toast(adw::Toast::new(&err.to_string())),
        });
    });
    dialog.present(Some(window));
}

fn confirm_delete_album(
    state: &Rc<AppState>,
    window: &adw::ApplicationWindow,
    album_id: AlbumId,
    track_count: usize,
) {
    let dialog = adw::AlertDialog::builder()
        .heading("Delete album permanently?")
        .body(format!(
            "Delete this album ({track_count} tracks) from Cadence and remove the files from disk. This cannot be undone."
        ))
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("delete", "Delete");
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    let state = Rc::clone(state);
    dialog.connect_response(None, move |_, response| {
        if response != "delete" {
            return;
        }
        let state2 = Rc::clone(&state);
        state
            .library
            .remove_album(album_id, move |result| match result {
                Ok(n) => state2.toast.add_toast(adw::Toast::new(&format!(
                    "Deleted album ({n} tracks) from library and disk"
                ))),
                Err(err) => state2.toast.add_toast(adw::Toast::new(&err.to_string())),
            });
    });
    dialog.present(Some(window));
}
