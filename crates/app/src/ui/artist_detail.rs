//! Artist detail pane — albums with songs grouped beneath, plus singles.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use cadence_core::models::{Album, Artist, Track, TrackDisplay};

use super::library_home::ContextHandler;
use super::{
    artwork_frame, attach_context_menu, format_duration_ms, set_artwork_file, ContextAction,
};

pub struct ArtistDetail {
    pub widget: gtk::ScrolledWindow,
    header_box: gtk::Box,
    albums_box: gtk::Box,
    on_play: Rc<RefCell<Option<Box<dyn Fn(Vec<Track>, usize)>>>>,
    on_context: Rc<RefCell<Option<ContextHandler>>>,
}

impl ArtistDetail {
    #[must_use]
    pub fn new() -> Self {
        let header_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        header_box.set_margin_start(24);
        header_box.set_margin_end(24);
        header_box.set_margin_top(20);

        let albums_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
        albums_box.set_margin_start(24);
        albums_box.set_margin_end(24);
        albums_box.set_margin_bottom(24);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
        content.append(&header_box);
        content.append(&albums_box);

        let widget = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .child(&content)
            .build();

        Self {
            widget,
            header_box,
            albums_box,
            on_play: Rc::new(RefCell::new(None)),
            on_context: Rc::new(RefCell::new(None)),
        }
    }

    pub fn connect_play<F: Fn(Vec<Track>, usize) + 'static>(&self, f: F) {
        *self.on_play.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_context<F: Fn(Track, ContextAction) + 'static>(&self, f: F) {
        *self.on_context.borrow_mut() = Some(Rc::new(f));
    }

    pub fn show_placeholder(&self, message: &str) {
        while let Some(child) = self.header_box.first_child() {
            self.header_box.remove(&child);
        }
        while let Some(child) = self.albums_box.first_child() {
            self.albums_box.remove(&child);
        }
        self.header_box.append(
            &adw::StatusPage::builder()
                .icon_name("audio-x-generic-symbolic")
                .title("Select an artist")
                .description(message)
                .build(),
        );
    }

    pub fn set_artist(
        &self,
        artist: &Artist,
        duration_ms: u64,
        albums: Vec<(Album, Vec<TrackDisplay>)>,
        singles: Vec<TrackDisplay>,
    ) {
        while let Some(child) = self.header_box.first_child() {
            self.header_box.remove(&child);
        }
        while let Some(child) = self.albums_box.first_child() {
            self.albums_box.remove(&child);
        }

        let name = gtk::Label::builder()
            .label(&artist.name)
            .xalign(0.0)
            .css_classes(["title-1"])
            .build();
        let album_label = if artist.album_count == 1 {
            "1 album".into()
        } else {
            format!("{} albums", artist.album_count)
        };
        let stats = gtk::Label::builder()
            .label(format!(
                "{album_label}  ·  {} songs  ·  {}",
                artist.track_count,
                format_duration_ms(Some(duration_ms))
            ))
            .xalign(0.0)
            .css_classes(["dim-label"])
            .build();
        self.header_box.append(&name);
        self.header_box.append(&stats);

        if albums.is_empty() && singles.is_empty() {
            self.albums_box.append(
                &gtk::Label::builder()
                    .label("No songs for this artist yet.")
                    .xalign(0.0)
                    .css_classes(["dim-label"])
                    .build(),
            );
            return;
        }

        for (album, tracks) in albums {
            self.albums_box.append(&album_section(
                &album,
                &tracks,
                artist.name.as_str(),
                &self.on_play,
                self.on_context.borrow().clone(),
            ));
        }

        if !singles.is_empty() {
            self.albums_box.append(&singles_section(
                &singles,
                &self.on_play,
                self.on_context.borrow().clone(),
            ));
        }
    }
}

impl Default for ArtistDetail {
    fn default() -> Self {
        Self::new()
    }
}

fn singles_section(
    tracks: &[TrackDisplay],
    on_play: &Rc<RefCell<Option<Box<dyn Fn(Vec<Track>, usize)>>>>,
    on_context: Option<ContextHandler>,
) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 10);
    section.add_css_class("cadence-album-card");

    let title = gtk::Label::builder()
        .label("Singles & other tracks")
        .xalign(0.0)
        .css_classes(["title-3"])
        .build();
    let sub = gtk::Label::builder()
        .label(format!(
            "{} tracks not on this artist’s albums",
            tracks.len()
        ))
        .xalign(0.0)
        .css_classes(["dim-label", "caption"])
        .build();
    section.append(&title);
    section.append(&sub);

    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .css_classes(["boxed-list"])
        .build();
    for (i, t) in tracks.iter().enumerate() {
        list.append(&album_track_row(t, i + 1, on_context.as_ref()));
    }

    let playlist: Vec<Track> = tracks.iter().map(|t| t.track.clone()).collect();
    let on_play = Rc::clone(on_play);
    list.connect_row_activated(move |_, row| {
        let index = row.index() as usize;
        if let Some(cb) = on_play.borrow().as_ref() {
            cb(playlist.clone(), index);
        }
    });
    section.append(&list);
    section
}

fn album_section(
    album: &Album,
    tracks: &[TrackDisplay],
    artist_name: &str,
    on_play: &Rc<RefCell<Option<Box<dyn Fn(Vec<Track>, usize)>>>>,
    on_context: Option<ContextHandler>,
) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 10);
    section.add_css_class("cadence-album-card");

    let (art_frame, art) = artwork_frame(96, &["card", "cadence-artwork"]);
    set_artwork_file(&art, album.artwork_path.as_deref(), 96);

    let title = gtk::Label::builder()
        .label(&album.name)
        .xalign(0.0)
        .css_classes(["title-3"])
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    let year = album
        .year
        .map(|y| y.to_string())
        .unwrap_or_else(|| "Year unknown".into());
    let genre = album
        .genre
        .clone()
        .filter(|g| !g.is_empty())
        .unwrap_or_else(|| "Unknown genre".into());
    let sub = gtk::Label::builder()
        .label(format!(
            "{year}  ·  {genre}  ·  {} tracks  ·  {artist_name}",
            album.track_count
        ))
        .xalign(0.0)
        .css_classes(["dim-label", "caption"])
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();

    let meta = gtk::Box::new(gtk::Orientation::Vertical, 4);
    meta.set_valign(gtk::Align::Center);
    meta.append(&title);
    meta.append(&sub);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    header.append(&art_frame);
    header.append(&meta);
    section.append(&header);

    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .css_classes(["boxed-list"])
        .build();
    for (i, t) in tracks.iter().enumerate() {
        list.append(&album_track_row(t, i + 1, on_context.as_ref()));
    }

    let playlist: Vec<Track> = tracks.iter().map(|t| t.track.clone()).collect();
    let on_play = Rc::clone(on_play);
    list.connect_row_activated(move |_, row| {
        let index = row.index() as usize;
        if let Some(cb) = on_play.borrow().as_ref() {
            cb(playlist.clone(), index);
        }
    });
    section.append(&list);
    section
}

fn album_track_row(
    item: &TrackDisplay,
    track_no: usize,
    on_context: Option<&ContextHandler>,
) -> gtk::ListBoxRow {
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
    let title = gtk::Label::builder()
        .label(&item.track.title)
        .xalign(0.0)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .hexpand(true)
        .build();
    let duration = gtk::Label::builder()
        .label(format_duration_ms(item.track.duration_ms))
        .css_classes(["dim-label", "numeric"])
        .build();
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.add_css_class("cadence-song-row");
    row.set_margin_start(12);
    row.set_margin_end(12);
    row.set_margin_top(8);
    row.set_margin_bottom(8);
    row.append(&num);
    row.append(&title);
    if item.track.favorite {
        row.append(&gtk::Image::from_icon_name("starred-symbolic"));
    }
    row.append(&duration);
    let list_row = gtk::ListBoxRow::builder()
        .child(&row)
        .activatable(true)
        .build();
    if let Some(cb) = on_context {
        let cb = Rc::clone(cb);
        let track = item.track.clone();
        attach_context_menu(&list_row, move |action| cb(track.clone(), action));
    }
    list_row
}
