use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use cadence_core::models::Playlist;

pub struct PlaylistsView {
    pub widget: gtk::Box,
    list: gtk::ListBox,
    pub new_button: gtk::Button,
    playlists: Rc<RefCell<Vec<Playlist>>>,
    on_activate: Rc<RefCell<Option<Box<dyn Fn(i64)>>>>,
}

impl PlaylistsView {
    #[must_use]
    pub fn new() -> Self {
        let new_button = gtk::Button::builder()
            .label("New Playlist")
            .css_classes(["pill"])
            .halign(gtk::Align::Start)
            .margin_start(12)
            .margin_top(12)
            .margin_bottom(8)
            .build();

        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .css_classes(["navigation-sidebar"])
            .build();
        let scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&list)
            .build();

        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.append(&new_button);
        widget.append(&scrolled);

        let playlists = Rc::new(RefCell::new(Vec::<Playlist>::new()));
        let on_activate: Rc<RefCell<Option<Box<dyn Fn(i64)>>>> = Rc::new(RefCell::new(None));

        {
            let playlists = Rc::clone(&playlists);
            let on_activate = Rc::clone(&on_activate);
            list.connect_row_activated(move |_, row| {
                let index = row.index() as usize;
                if let Some(pl) = playlists.borrow().get(index) {
                    if let Some(cb) = on_activate.borrow().as_ref() {
                        cb(pl.id);
                    }
                }
            });
        }

        Self {
            widget,
            list,
            new_button,
            playlists,
            on_activate,
        }
    }

    pub fn connect_activate<F: Fn(i64) + 'static>(&self, f: F) {
        *self.on_activate.borrow_mut() = Some(Box::new(f));
    }

    pub fn set_playlists(&self, playlists: Vec<Playlist>) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        for pl in &playlists {
            let title = gtk::Label::builder()
                .label(&pl.name)
                .xalign(0.0)
                .hexpand(true)
                .build();
            let count = gtk::Label::builder()
                .label(format!("{} tracks", pl.track_ids.len()))
                .css_classes(["dim-label", "caption"])
                .build();
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            row.set_margin_start(12);
            row.set_margin_end(12);
            row.set_margin_top(8);
            row.set_margin_bottom(8);
            row.append(&title);
            row.append(&count);
            self.list
                .append(&gtk::ListBoxRow::builder().child(&row).build());
        }
        *self.playlists.borrow_mut() = playlists;
    }

    pub fn playlist(&self, id: i64) -> Option<Playlist> {
        self.playlists.borrow().iter().find(|p| p.id == id).cloned()
    }
}

impl Default for PlaylistsView {
    fn default() -> Self {
        Self::new()
    }
}
