use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use adw::prelude::*;
use cadence_core::models::{Album, AlbumId};

use super::{artwork_frame, attach_context_menu, set_artwork_file, ContextAction};

const COVER_SIZE: i32 = 160;

pub struct AlbumsView {
    pub widget: gtk::ScrolledWindow,
    flow: gtk::FlowBox,
    albums: Rc<RefCell<Vec<Album>>>,
    on_activate: Rc<RefCell<Option<Box<dyn Fn(AlbumId)>>>>,
    on_context: Rc<RefCell<Option<Box<dyn Fn(AlbumId, ContextAction)>>>>,
}

impl AlbumsView {
    #[must_use]
    pub fn new() -> Self {
        let flow = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .homogeneous(false)
            .max_children_per_line(8)
            .min_children_per_line(2)
            .row_spacing(16)
            .column_spacing(16)
            .margin_start(20)
            .margin_end(20)
            .margin_top(16)
            .margin_bottom(20)
            .build();

        let widget = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .child(&flow)
            .build();

        let albums = Rc::new(RefCell::new(Vec::<Album>::new()));
        let on_activate: Rc<RefCell<Option<Box<dyn Fn(AlbumId)>>>> = Rc::new(RefCell::new(None));
        let on_context: Rc<RefCell<Option<Box<dyn Fn(AlbumId, ContextAction)>>>> =
            Rc::new(RefCell::new(None));

        {
            let albums = Rc::clone(&albums);
            let on_activate = Rc::clone(&on_activate);
            flow.connect_child_activated(move |_, child| {
                let index = child.index() as usize;
                if let Some(album) = albums.borrow().get(index) {
                    if let Some(cb) = on_activate.borrow().as_ref() {
                        cb(album.id);
                    }
                }
            });
        }

        Self {
            widget,
            flow,
            albums,
            on_activate,
            on_context,
        }
    }

    pub fn connect_activate<F: Fn(AlbumId) + 'static>(&self, f: F) {
        *self.on_activate.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_context<F: Fn(AlbumId, ContextAction) + 'static>(&self, f: F) {
        *self.on_context.borrow_mut() = Some(Box::new(f));
    }

    pub fn set_albums(&self, albums: Vec<Album>, artist_names: &[String]) {
        while let Some(child) = self.flow.first_child() {
            self.flow.remove(&child);
        }
        for (i, album) in albums.iter().enumerate() {
            let artist = artist_names
                .get(i)
                .cloned()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Unknown Artist".into());
            let card = album_card(album, &artist, album.artwork_path.as_deref());
            let album_id = album.id;
            let on_context = Rc::clone(&self.on_context);
            attach_context_menu(&card, move |action| {
                if let Some(cb) = on_context.borrow().as_ref() {
                    cb(album_id, action);
                }
            });
            self.flow.append(&card);
        }
        *self.albums.borrow_mut() = albums;
    }
}

impl Default for AlbumsView {
    fn default() -> Self {
        Self::new()
    }
}

fn album_card(album: &Album, artist: &str, art: Option<&Path>) -> gtk::Box {
    let (frame, picture) = artwork_frame(COVER_SIZE, &["card", "cadence-artwork"]);
    set_artwork_file(&picture, art);

    let title = gtk::Label::builder()
        .label(&album.name)
        .wrap(false)
        .justify(gtk::Justification::Center)
        .width_chars(18)
        .max_width_chars(18)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .css_classes(["heading"])
        .build();
    let year = album.year.map(|y| format!("{y}")).unwrap_or_default();
    let genre = album.genre.clone().unwrap_or_default();
    let subtitle = gtk::Label::builder()
        .label(format!(
            "{artist}\n{}{}{} tracks",
            if year.is_empty() {
                String::new()
            } else {
                format!("{year} · ")
            },
            if genre.is_empty() {
                String::new()
            } else {
                format!("{genre} · ")
            },
            album.track_count
        ))
        .justify(gtk::Justification::Center)
        .width_chars(18)
        .max_width_chars(18)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .css_classes(["dim-label", "caption"])
        .build();

    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 8);
    box_.add_css_class("cadence-album-card");
    box_.set_halign(gtk::Align::Center);
    box_.set_valign(gtk::Align::Start);
    box_.set_size_request(COVER_SIZE, -1);
    box_.append(&frame);
    box_.append(&title);
    box_.append(&subtitle);
    box_
}
