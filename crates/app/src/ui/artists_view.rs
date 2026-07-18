use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use cadence_core::models::{Artist, ArtistId};

/// Master list of artists for the three-pane layout.
pub struct ArtistsView {
    pub widget: gtk::ScrolledWindow,
    list: gtk::ListBox,
    artists: Rc<RefCell<Vec<Artist>>>,
    on_select: Rc<RefCell<Option<Box<dyn Fn(ArtistId)>>>>,
}

impl ArtistsView {
    #[must_use]
    pub fn new() -> Self {
        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .css_classes(["navigation-sidebar"])
            .build();
        let widget = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .css_classes(["cadence-master"])
            .child(&list)
            .build();

        let artists = Rc::new(RefCell::new(Vec::<Artist>::new()));
        let on_select: Rc<RefCell<Option<Box<dyn Fn(ArtistId)>>>> = Rc::new(RefCell::new(None));

        {
            let artists = Rc::clone(&artists);
            let on_select = Rc::clone(&on_select);
            list.connect_row_selected(move |_, row| {
                let Some(row) = row else { return };
                let index = row.index() as usize;
                if let Some(artist) = artists.borrow().get(index) {
                    if let Some(cb) = on_select.borrow().as_ref() {
                        cb(artist.id);
                    }
                }
            });
        }

        Self {
            widget,
            list,
            artists,
            on_select,
        }
    }

    pub fn connect_select<F: Fn(ArtistId) + 'static>(&self, f: F) {
        *self.on_select.borrow_mut() = Some(Box::new(f));
    }

    pub fn set_artists(&self, artists: Vec<Artist>) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        for artist in &artists {
            let title = gtk::Label::builder()
                .label(&artist.name)
                .xalign(0.0)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .hexpand(true)
                .css_classes(["heading"])
                .build();
            let subtitle = gtk::Label::builder()
                .label(format!(
                    "{} albums · {} songs",
                    artist.album_count, artist.track_count
                ))
                .xalign(0.0)
                .css_classes(["dim-label", "caption"])
                .build();
            let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
            text.append(&title);
            text.append(&subtitle);
            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            row_box.set_margin_start(12);
            row_box.set_margin_end(12);
            row_box.set_margin_top(8);
            row_box.set_margin_bottom(8);
            row_box.append(&text);
            self.list.append(
                &gtk::ListBoxRow::builder()
                    .child(&row_box)
                    .activatable(true)
                    .build(),
            );
        }
        *self.artists.borrow_mut() = artists;
        if let Some(row) = self.list.row_at_index(0) {
            self.list.select_row(Some(&row));
        }
    }
}

impl Default for ArtistsView {
    fn default() -> Self {
        Self::new()
    }
}
