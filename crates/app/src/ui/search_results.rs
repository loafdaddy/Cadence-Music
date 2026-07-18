//! Grouped global search results.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use cadence_core::models::{Album, AlbumId, Artist, ArtistId, Track, TrackDisplay, TrackId};

use super::library_home::{rich_song_row, ContextHandler, FavoriteHandler};
use super::ContextAction;

pub struct SearchResults {
    pub widget: gtk::ScrolledWindow,
    content: gtk::Box,
    on_artist: Rc<RefCell<Option<Box<dyn Fn(ArtistId)>>>>,
    on_album: Rc<RefCell<Option<Box<dyn Fn(AlbumId)>>>>,
    on_play: Rc<RefCell<Option<Box<dyn Fn(Vec<Track>, usize)>>>>,
    on_favorite: Rc<RefCell<Option<FavoriteHandler>>>,
    on_context: Rc<RefCell<Option<ContextHandler>>>,
    on_genre: Rc<RefCell<Option<Box<dyn Fn(String)>>>>,
    on_year: Rc<RefCell<Option<Box<dyn Fn(i32)>>>>,
    on_folder: Rc<RefCell<Option<Box<dyn Fn(PathBuf)>>>>,
}

impl SearchResults {
    #[must_use]
    pub fn new() -> Self {
        let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(20);
        content.set_margin_bottom(24);

        let widget = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .child(&content)
            .build();

        Self {
            widget,
            content,
            on_artist: Rc::new(RefCell::new(None)),
            on_album: Rc::new(RefCell::new(None)),
            on_play: Rc::new(RefCell::new(None)),
            on_favorite: Rc::new(RefCell::new(None)),
            on_context: Rc::new(RefCell::new(None)),
            on_genre: Rc::new(RefCell::new(None)),
            on_year: Rc::new(RefCell::new(None)),
            on_folder: Rc::new(RefCell::new(None)),
        }
    }

    pub fn connect_artist<F: Fn(ArtistId) + 'static>(&self, f: F) {
        *self.on_artist.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_album<F: Fn(AlbumId) + 'static>(&self, f: F) {
        *self.on_album.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_play<F: Fn(Vec<Track>, usize) + 'static>(&self, f: F) {
        *self.on_play.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_favorite<F: Fn(TrackId, bool) + 'static>(&self, f: F) {
        *self.on_favorite.borrow_mut() = Some(Rc::new(f));
    }

    pub fn connect_context<F: Fn(Track, ContextAction) + 'static>(&self, f: F) {
        *self.on_context.borrow_mut() = Some(Rc::new(f));
    }

    pub fn connect_genre<F: Fn(String) + 'static>(&self, f: F) {
        *self.on_genre.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_year<F: Fn(i32) + 'static>(&self, f: F) {
        *self.on_year.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_folder<F: Fn(PathBuf) + 'static>(&self, f: F) {
        *self.on_folder.borrow_mut() = Some(Box::new(f));
    }

    pub fn show_empty(&self, query: &str) {
        while let Some(child) = self.content.first_child() {
            self.content.remove(&child);
        }
        self.content.append(
            &adw::StatusPage::builder()
                .icon_name("system-search-symbolic")
                .title("No results")
                .description(format!("Nothing matched “{query}”."))
                .build(),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_results(
        &self,
        query: &str,
        artists: Vec<Artist>,
        albums: Vec<(Album, String)>,
        songs: Vec<TrackDisplay>,
        genres: Vec<String>,
        years: Vec<i32>,
        folders: Vec<PathBuf>,
    ) {
        while let Some(child) = self.content.first_child() {
            self.content.remove(&child);
        }

        if artists.is_empty()
            && albums.is_empty()
            && songs.is_empty()
            && genres.is_empty()
            && years.is_empty()
            && folders.is_empty()
        {
            self.show_empty(query);
            return;
        }

        self.content.append(
            &gtk::Label::builder()
                .label(format!("Results for “{query}”"))
                .xalign(0.0)
                .css_classes(["title-2"])
                .build(),
        );

        if !artists.is_empty() {
            self.content.append(&section_heading("Artists"));
            let list = gtk::ListBox::builder()
                .selection_mode(gtk::SelectionMode::Single)
                .css_classes(["boxed-list"])
                .build();
            for artist in &artists {
                let title = gtk::Label::builder()
                    .label(&artist.name)
                    .xalign(0.0)
                    .hexpand(true)
                    .build();
                let sub = gtk::Label::builder()
                    .label(format!(
                        "{} albums · {} songs",
                        artist.album_count, artist.track_count
                    ))
                    .xalign(0.0)
                    .css_classes(["dim-label", "caption"])
                    .build();
                let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
                text.append(&title);
                text.append(&sub);
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                row.set_margin_start(12);
                row.set_margin_end(12);
                row.set_margin_top(8);
                row.set_margin_bottom(8);
                row.append(&text);
                list.append(&gtk::ListBoxRow::builder().child(&row).build());
            }
            let artists = artists.clone();
            let on_artist = Rc::clone(&self.on_artist);
            list.connect_row_activated(move |_, row| {
                let i = row.index() as usize;
                if let Some(a) = artists.get(i) {
                    if let Some(cb) = on_artist.borrow().as_ref() {
                        cb(a.id);
                    }
                }
            });
            self.content.append(&list);
        }

        if !albums.is_empty() {
            self.content.append(&section_heading("Albums"));
            let list = gtk::ListBox::builder()
                .selection_mode(gtk::SelectionMode::Single)
                .css_classes(["boxed-list"])
                .build();
            for (album, artist) in &albums {
                list.append(&album_result_row(album, artist));
            }
            let albums: Vec<Album> = albums.into_iter().map(|(a, _)| a).collect();
            let on_album = Rc::clone(&self.on_album);
            list.connect_row_activated(move |_, row| {
                let i = row.index() as usize;
                if let Some(a) = albums.get(i) {
                    if let Some(cb) = on_album.borrow().as_ref() {
                        cb(a.id);
                    }
                }
            });
            self.content.append(&list);
        }

        if !songs.is_empty() {
            self.content.append(&section_heading("Songs"));
            let list = gtk::ListBox::builder()
                .selection_mode(gtk::SelectionMode::Single)
                .css_classes(["boxed-list"])
                .build();
            let fav = self.on_favorite.borrow().clone();
            let ctx = self.on_context.borrow().clone();
            for (i, song) in songs.iter().enumerate() {
                list.append(&rich_song_row(
                    song,
                    i + 1,
                    true,
                    fav.as_ref(),
                    ctx.as_ref(),
                ));
            }
            let playlist: Vec<Track> = songs.iter().map(|s| s.track.clone()).collect();
            let on_play = Rc::clone(&self.on_play);
            list.connect_row_activated(move |_, row| {
                let i = row.index() as usize;
                if let Some(cb) = on_play.borrow().as_ref() {
                    cb(playlist.clone(), i);
                }
            });
            self.content.append(&list);
        }

        if !genres.is_empty() {
            self.content.append(&section_heading("Genres"));
            let list = simple_string_list(&genres);
            let genres = genres.clone();
            let on_genre = Rc::clone(&self.on_genre);
            list.connect_row_activated(move |_, row| {
                let i = row.index() as usize;
                if let Some(g) = genres.get(i) {
                    if let Some(cb) = on_genre.borrow().as_ref() {
                        cb(g.clone());
                    }
                }
            });
            self.content.append(&list);
        }

        if !years.is_empty() {
            self.content.append(&section_heading("Years"));
            let labels: Vec<String> = years.iter().map(|y| y.to_string()).collect();
            let list = simple_string_list(&labels);
            let years = years.clone();
            let on_year = Rc::clone(&self.on_year);
            list.connect_row_activated(move |_, row| {
                let i = row.index() as usize;
                if let Some(y) = years.get(i) {
                    if let Some(cb) = on_year.borrow().as_ref() {
                        cb(*y);
                    }
                }
            });
            self.content.append(&list);
        }

        if !folders.is_empty() {
            self.content.append(&section_heading("Folders"));
            let labels: Vec<String> = folders
                .iter()
                .map(|p| p.display().to_string())
                .collect();
            let list = simple_string_list(&labels);
            let folders = folders.clone();
            let on_folder = Rc::clone(&self.on_folder);
            list.connect_row_activated(move |_, row| {
                let i = row.index() as usize;
                if let Some(p) = folders.get(i) {
                    if let Some(cb) = on_folder.borrow().as_ref() {
                        cb(p.clone());
                    }
                }
            });
            self.content.append(&list);
        }
    }
}

impl Default for SearchResults {
    fn default() -> Self {
        Self::new()
    }
}

fn section_heading(text: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(text)
        .xalign(0.0)
        .css_classes(["title-4"])
        .margin_top(8)
        .build()
}

fn simple_string_list(labels: &[String]) -> gtk::ListBox {
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .css_classes(["boxed-list"])
        .build();
    for label in labels {
        let row = gtk::Label::builder()
            .label(label)
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .margin_start(12)
            .margin_end(12)
            .margin_top(10)
            .margin_bottom(10)
            .build();
        list.append(&gtk::ListBoxRow::builder().child(&row).build());
    }
    list
}

fn album_result_row(album: &Album, artist: &str) -> gtk::ListBoxRow {
    let (art_frame, art) = super::artwork_frame(44, &["card"]);
    super::set_artwork_file(&art, album.artwork_path.as_deref());
    let title = gtk::Label::builder()
        .label(&album.name)
        .xalign(0.0)
        .hexpand(true)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    let year = album.year.map(|y| format!(" · {y}")).unwrap_or_default();
    let sub = gtk::Label::builder()
        .label(format!("{artist}{year} · {} tracks", album.track_count))
        .xalign(0.0)
        .css_classes(["dim-label", "caption"])
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.append(&title);
    text.append(&sub);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.set_margin_start(12);
    row.set_margin_end(12);
    row.set_margin_top(8);
    row.set_margin_bottom(8);
    row.append(&art_frame);
    row.append(&text);
    gtk::ListBoxRow::builder().child(&row).build()
}
