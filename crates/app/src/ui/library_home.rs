//! Welcoming Library home — full-width discovery surface.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use cadence_core::models::{Album, AlbumId, Track, TrackDisplay, TrackId};

use super::{
    artwork_frame, attach_context_menu, format_duration_ms, set_artwork_file, ContextAction,
};

pub type FavoriteHandler = Rc<dyn Fn(TrackId, bool)>;
pub type ContextHandler = Rc<dyn Fn(Track, ContextAction)>;
pub type AlbumContextHandler = Rc<dyn Fn(AlbumId, ContextAction)>;

pub struct LibraryHome {
    pub widget: gtk::ScrolledWindow,
    continue_box: gtk::Box,
    recent_albums_box: gtk::FlowBox,
    recent_tracks_box: gtk::Box,
    stats_label: gtk::Label,
    pub scan_button: gtk::Button,
    pub organise_button: gtk::Button,
    pub lookup_button: gtk::Button,
    on_play: Rc<RefCell<Option<Box<dyn Fn(Vec<TrackDisplay>, usize)>>>>,
    on_album: Rc<RefCell<Option<Box<dyn Fn(AlbumId)>>>>,
    on_favorite: Rc<RefCell<Option<FavoriteHandler>>>,
    on_context: Rc<RefCell<Option<ContextHandler>>>,
    on_album_context: Rc<RefCell<Option<AlbumContextHandler>>>,
    continue_tracks: Rc<RefCell<Vec<TrackDisplay>>>,
    recent_tracks: Rc<RefCell<Vec<TrackDisplay>>>,
}

impl LibraryHome {
    #[must_use]
    pub fn new() -> Self {
        let heading = gtk::Label::builder()
            .label("Library")
            .xalign(0.0)
            .css_classes(["title-1"])
            .build();
        let blurb = gtk::Label::builder()
            .label("Welcome back. Pick up where you left off, or tidy your collection.")
            .xalign(0.0)
            .wrap(true)
            .css_classes(["dim-label"])
            .build();

        let stats_label = gtk::Label::builder()
            .label("")
            .xalign(0.0)
            .css_classes(["caption", "dim-label"])
            .margin_bottom(8)
            .build();

        let scan_button = pill_button("Scan Library", "view-refresh-symbolic");
        let organise_button = pill_button("Organise Files", "folder-symbolic");
        let lookup_button = pill_button("Find Missing Metadata", "edit-find-symbolic");

        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        actions.set_margin_bottom(8);
        actions.append(&scan_button);
        actions.append(&organise_button);
        actions.append(&lookup_button);

        let continue_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let recent_albums_box = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .max_children_per_line(8)
            .min_children_per_line(2)
            .row_spacing(16)
            .column_spacing(16)
            .homogeneous(true)
            .build();
        let recent_tracks_box = gtk::Box::new(gtk::Orientation::Vertical, 2);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
        content.set_margin_start(28);
        content.set_margin_end(28);
        content.set_margin_top(20);
        content.set_margin_bottom(28);
        content.set_hexpand(true);
        content.append(&heading);
        content.append(&blurb);
        content.append(&stats_label);
        content.append(&actions);
        content.append(&section_label("Continue Listening"));
        content.append(&continue_box);
        content.append(&section_label("Recent Albums"));
        content.append(&recent_albums_box);
        content.append(&section_label("Recently Added"));
        content.append(&recent_tracks_box);

        let widget = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&content)
            .build();

        Self {
            widget,
            continue_box,
            recent_albums_box,
            recent_tracks_box,
            stats_label,
            scan_button,
            organise_button,
            lookup_button,
            on_play: Rc::new(RefCell::new(None)),
            on_album: Rc::new(RefCell::new(None)),
            on_favorite: Rc::new(RefCell::new(None)),
            on_context: Rc::new(RefCell::new(None)),
            on_album_context: Rc::new(RefCell::new(None)),
            continue_tracks: Rc::new(RefCell::new(Vec::new())),
            recent_tracks: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn connect_play<F: Fn(Vec<TrackDisplay>, usize) + 'static>(&self, f: F) {
        *self.on_play.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_album<F: Fn(AlbumId) + 'static>(&self, f: F) {
        *self.on_album.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_favorite<F: Fn(TrackId, bool) + 'static>(&self, f: F) {
        *self.on_favorite.borrow_mut() = Some(Rc::new(f));
    }

    pub fn connect_context<F: Fn(Track, ContextAction) + 'static>(&self, f: F) {
        *self.on_context.borrow_mut() = Some(Rc::new(f));
    }

    pub fn connect_album_context<F: Fn(AlbumId, ContextAction) + 'static>(&self, f: F) {
        *self.on_album_context.borrow_mut() = Some(Rc::new(f));
    }

    pub fn favorite_handler(&self) -> Option<FavoriteHandler> {
        self.on_favorite.borrow().clone()
    }

    pub fn context_handler(&self) -> Option<ContextHandler> {
        self.on_context.borrow().clone()
    }

    pub fn set_stats(&self, artists: u64, albums: u64, songs: u64) {
        self.stats_label.set_label(&format!(
            "{artists} artists  ·  {albums} albums  ·  {songs} songs"
        ));
    }

    pub fn set_continue(&self, tracks: Vec<TrackDisplay>) {
        fill_track_list(
            &self.continue_box,
            &self.continue_tracks,
            tracks,
            &self.on_play,
            self.favorite_handler().as_ref(),
            self.context_handler().as_ref(),
            true,
        );
    }

    pub fn set_recent(&self, tracks: Vec<TrackDisplay>) {
        fill_track_list(
            &self.recent_tracks_box,
            &self.recent_tracks,
            tracks,
            &self.on_play,
            self.favorite_handler().as_ref(),
            self.context_handler().as_ref(),
            true,
        );
    }

    pub fn set_recent_albums(&self, albums: Vec<(Album, String)>) {
        while let Some(child) = self.recent_albums_box.first_child() {
            self.recent_albums_box.remove(&child);
        }
        let on_album = Rc::clone(&self.on_album);
        let on_album_context = Rc::clone(&self.on_album_context);
        for (album, artist) in albums {
            let id = album.id;
            let card = home_album_card(&album, &artist);
            let child = gtk::FlowBoxChild::new();
            child.set_child(Some(&card));
            self.recent_albums_box.append(&child);

            let gesture = gtk::GestureClick::new();
            gesture.set_button(1);
            let on_album = Rc::clone(&on_album);
            gesture.connect_released(move |_, _, _, _| {
                if let Some(cb) = on_album.borrow().as_ref() {
                    cb(id);
                }
            });
            card.add_controller(gesture);

            let on_album_context = Rc::clone(&on_album_context);
            attach_context_menu(&card, move |action| {
                if let Some(cb) = on_album_context.borrow().as_ref() {
                    cb(id, action);
                }
            });
        }
    }
}

impl Default for LibraryHome {
    fn default() -> Self {
        Self::new()
    }
}

fn section_label(text: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(text)
        .xalign(0.0)
        .css_classes(["title-4", "cadence-home-section"])
        .build()
}

fn pill_button(label: &str, icon: &str) -> gtk::Button {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.append(&gtk::Image::from_icon_name(icon));
    row.append(&gtk::Label::new(Some(label)));
    gtk::Button::builder()
        .child(&row)
        .css_classes(["pill"])
        .build()
}

fn home_album_card(album: &Album, artist: &str) -> gtk::Box {
    let (frame, picture) = artwork_frame(148, &["card", "cadence-artwork"]);
    set_artwork_file(&picture, album.artwork_path.as_deref(), 148);
    let title = gtk::Label::builder()
        .label(&album.name)
        .xalign(0.0)
        .wrap(false)
        .lines(1)
        .max_width_chars(16)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .css_classes(["heading"])
        .build();
    let sub = gtk::Label::builder()
        .label(artist)
        .xalign(0.0)
        .wrap(false)
        .lines(1)
        .max_width_chars(16)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .css_classes(["dim-label", "caption"])
        .build();
    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.set_size_request(148, 40);
    text.append(&title);
    text.append(&sub);
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 6);
    box_.add_css_class("cadence-album-card");
    box_.set_size_request(148, 148 + 48);
    box_.set_halign(gtk::Align::Center);
    box_.set_valign(gtk::Align::Start);
    box_.append(&frame);
    box_.append(&text);
    box_
}

fn fill_track_list(
    container: &gtk::Box,
    store: &Rc<RefCell<Vec<TrackDisplay>>>,
    tracks: Vec<TrackDisplay>,
    on_play: &Rc<RefCell<Option<Box<dyn Fn(Vec<TrackDisplay>, usize)>>>>,
    on_favorite: Option<&FavoriteHandler>,
    on_context: Option<&ContextHandler>,
    show_art: bool,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
    if tracks.is_empty() {
        container.append(
            &gtk::Label::builder()
                .label("Nothing here yet — play something to fill this shelf.")
                .xalign(0.0)
                .css_classes(["dim-label", "caption"])
                .build(),
        );
        *store.borrow_mut() = tracks;
        return;
    }

    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .css_classes(["boxed-list"])
        .build();

    for (i, item) in tracks.iter().enumerate() {
        list.append(&rich_song_row(
            item,
            i + 1,
            show_art,
            on_favorite,
            on_context,
        ));
    }

    let store = Rc::clone(store);
    let on_play = Rc::clone(on_play);
    *store.borrow_mut() = tracks;
    list.connect_row_activated(move |_, row| {
        let index = row.index() as usize;
        if let Some(cb) = on_play.borrow().as_ref() {
            cb(store.borrow().clone(), index);
        }
    });
    container.append(&list);
}

/// Song row. Set `show_art` false under album headings.
pub fn rich_song_row(
    item: &TrackDisplay,
    track_no: usize,
    show_art: bool,
    on_favorite: Option<&FavoriteHandler>,
    on_context: Option<&ContextHandler>,
) -> gtk::ListBoxRow {
    let title = gtk::Label::builder()
        .label(&item.track.title)
        .xalign(0.0)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .hexpand(true)
        .build();
    let meta = gtk::Label::builder()
        .label(format!(
            "{}  ·  {}",
            item.artist_label(),
            item.album_label()
        ))
        .xalign(0.0)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .css_classes(["dim-label", "caption"])
        .build();

    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.append(&title);
    text.append(&meta);

    let num = gtk::Label::builder()
        .label(
            item.track
                .track_number
                .map(|n| format!("{n}"))
                .unwrap_or_else(|| format!("{track_no}")),
        )
        .width_chars(3)
        .css_classes(["dim-label", "numeric", "caption"])
        .build();
    let duration = gtk::Label::builder()
        .label(format_duration_ms(item.track.duration_ms))
        .css_classes(["dim-label", "numeric"])
        .build();

    let fav = gtk::ToggleButton::builder()
        .icon_name(if item.track.favorite {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        })
        .active(item.track.favorite)
        .tooltip_text("Favourite")
        .css_classes(["flat", "circular"])
        .valign(gtk::Align::Center)
        .build();
    let track_id = item.track.id;
    if let Some(cb) = on_favorite {
        let cb = Rc::clone(cb);
        fav.connect_toggled(move |btn| {
            let active = btn.is_active();
            btn.set_icon_name(if active {
                "starred-symbolic"
            } else {
                "non-starred-symbolic"
            });
            cb(track_id, active);
        });
    }

    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row_box.add_css_class("cadence-song-row");
    row_box.set_margin_start(12);
    row_box.set_margin_end(12);
    row_box.set_margin_top(8);
    row_box.set_margin_bottom(8);
    row_box.append(&num);
    if show_art {
        let (art_frame, art) = artwork_frame(48, &["card"]);
        set_artwork_file(&art, item.artwork_path.as_deref(), 48);
        row_box.append(&art_frame);
    }
    row_box.append(&text);
    row_box.append(&fav);
    row_box.append(&duration);

    let row = gtk::ListBoxRow::builder()
        .child(&row_box)
        .activatable(true)
        .build();
    if let Some(cb) = on_context {
        let cb = Rc::clone(cb);
        let track = item.track.clone();
        attach_context_menu(&row, move |action| cb(track.clone(), action));
    }
    row
}
