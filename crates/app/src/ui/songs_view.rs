use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use cadence_core::db::SongSort;
use cadence_core::models::{Track, TrackDisplay, TrackId};

use super::library_home::{rich_song_row, ContextHandler, FavoriteHandler};
use super::ContextAction;

/// Paginated songs list with rich rows.
pub struct SongsView {
    pub widget: gtk::Box,
    list: gtk::ListBox,
    sort_dropdown: gtk::DropDown,
    load_more: gtk::Button,
    tracks: Rc<RefCell<Vec<TrackDisplay>>>,
    on_activate: Rc<RefCell<Option<Box<dyn Fn(Vec<Track>, usize)>>>>,
    on_load_more: Rc<RefCell<Option<Box<dyn Fn(SongSort, usize)>>>>,
    on_favorite: Rc<RefCell<Option<FavoriteHandler>>>,
    on_context: Rc<RefCell<Option<ContextHandler>>>,
}

impl SongsView {
    #[must_use]
    pub fn new() -> Self {
        let sort_dropdown = gtk::DropDown::from_strings(&[
            "Title",
            "Artist",
            "Album",
            "Recently Added",
            "Most Played",
        ]);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header.set_margin_start(12);
        header.set_margin_end(12);
        header.set_margin_top(8);
        header.set_margin_bottom(8);
        header.append(&gtk::Label::new(Some("Sort")));
        header.append(&sort_dropdown);

        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();

        let scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .child(&list)
            .build();

        let load_more = gtk::Button::builder()
            .label("Load more")
            .margin_bottom(12)
            .halign(gtk::Align::Center)
            .css_classes(["pill"])
            .build();

        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.append(&header);
        widget.append(&scrolled);
        widget.append(&load_more);

        let tracks = Rc::new(RefCell::new(Vec::<TrackDisplay>::new()));
        let on_activate: Rc<RefCell<Option<Box<dyn Fn(Vec<Track>, usize)>>>> =
            Rc::new(RefCell::new(None));
        let on_load_more: Rc<RefCell<Option<Box<dyn Fn(SongSort, usize)>>>> =
            Rc::new(RefCell::new(None));
        let on_favorite: Rc<RefCell<Option<FavoriteHandler>>> = Rc::new(RefCell::new(None));
        let on_context: Rc<RefCell<Option<ContextHandler>>> = Rc::new(RefCell::new(None));

        {
            let tracks = Rc::clone(&tracks);
            let on_activate = Rc::clone(&on_activate);
            list.connect_row_activated(move |_, row| {
                let index = row.index() as usize;
                if let Some(cb) = on_activate.borrow().as_ref() {
                    let plain: Vec<Track> =
                        tracks.borrow().iter().map(|t| t.track.clone()).collect();
                    cb(plain, index);
                }
            });
        }

        {
            let tracks = Rc::clone(&tracks);
            let on_load_more = Rc::clone(&on_load_more);
            let sort_dropdown = sort_dropdown.clone();
            load_more.connect_clicked(move |_| {
                if let Some(cb) = on_load_more.borrow().as_ref() {
                    cb(
                        Self::sort_from_index(sort_dropdown.selected()),
                        tracks.borrow().len(),
                    );
                }
            });
        }

        {
            let on_load_more = Rc::clone(&on_load_more);
            let tracks = Rc::clone(&tracks);
            sort_dropdown.connect_selected_notify(move |drop| {
                tracks.borrow_mut().clear();
                if let Some(cb) = on_load_more.borrow().as_ref() {
                    cb(Self::sort_from_index(drop.selected()), 0);
                }
            });
        }

        Self {
            widget,
            list,
            sort_dropdown,
            load_more,
            tracks,
            on_activate,
            on_load_more,
            on_favorite,
            on_context,
        }
    }

    pub fn connect_activate<F: Fn(Vec<Track>, usize) + 'static>(&self, f: F) {
        *self.on_activate.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_load_page<F: Fn(SongSort, usize) + 'static>(&self, f: F) {
        *self.on_load_more.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_favorite<F: Fn(TrackId, bool) + 'static>(&self, f: F) {
        *self.on_favorite.borrow_mut() = Some(Rc::new(f));
    }

    pub fn connect_context<F: Fn(Track, ContextAction) + 'static>(&self, f: F) {
        *self.on_context.borrow_mut() = Some(Rc::new(f));
    }

    pub fn current_sort(&self) -> SongSort {
        Self::sort_from_index(self.sort_dropdown.selected())
    }

    pub fn clear(&self) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        self.tracks.borrow_mut().clear();
    }

    pub fn replace_display(&self, page: Vec<TrackDisplay>) {
        self.clear();
        self.append_display(page);
    }

    pub fn append_display(&self, page: Vec<TrackDisplay>) {
        let start = self.tracks.borrow().len();
        let fav = self.on_favorite.borrow().clone();
        let ctx = self.on_context.borrow().clone();
        for (i, item) in page.iter().enumerate() {
            self.list.append(&rich_song_row(
                item,
                start + i + 1,
                true,
                fav.as_ref(),
                ctx.as_ref(),
            ));
        }
        self.tracks.borrow_mut().extend(page);
    }

    pub fn set_has_more(&self, has_more: bool) {
        self.load_more.set_visible(has_more);
    }

    fn sort_from_index(index: u32) -> SongSort {
        match index {
            1 => SongSort::ArtistAsc,
            2 => SongSort::AlbumAsc,
            3 => SongSort::RecentlyAdded,
            4 => SongSort::PlayCountDesc,
            _ => SongSort::TitleAsc,
        }
    }
}

impl Default for SongsView {
    fn default() -> Self {
        Self::new()
    }
}
