//! Compact playback dock — supports the library, never dominates it.

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gdk, glib};

use super::{artwork_frame, format_duration_ms, set_artwork_file};

const ART_SIZE: i32 = 72;
const DOCK_HEIGHT: i32 = 96;

/// Bottom playback dock (~100px). Click artwork to open Now Playing.
pub struct PlayerBar {
    pub widget: gtk::Box,
    pub play_button: gtk::Button,
    pub prev_button: gtk::Button,
    pub next_button: gtk::Button,
    pub shuffle_button: gtk::ToggleButton,
    pub repeat_button: gtk::Button,
    pub queue_button: gtk::ToggleButton,
    pub favorite_button: gtk::ToggleButton,
    pub art_button: gtk::Button,
    pub seek: gtk::Scale,
    pub volume: gtk::Scale,
    pub title: gtk::Label,
    pub subtitle: gtk::Label,
    pub position_label: gtk::Label,
    pub duration_label: gtk::Label,
    pub artwork: gtk::Picture,
    seeking: Rc<Cell<bool>>,
    on_expand: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl PlayerBar {
    #[must_use]
    pub fn new() -> Self {
        let (art_frame, artwork) = artwork_frame(ART_SIZE, &["cadence-dock-art", "card"]);
        set_fallback_art(&artwork);

        let art_button = gtk::Button::builder()
            .child(&art_frame)
            .css_classes(["flat", "cadence-dock-art-btn"])
            .tooltip_text("Now Playing")
            .build();
        // size_request is a minimum only — also clip so artwork never grows the dock.
        art_button.set_size_request(ART_SIZE, ART_SIZE);
        art_button.set_hexpand(false);
        art_button.set_vexpand(false);
        art_button.set_valign(gtk::Align::Center);
        art_button.set_overflow(gtk::Overflow::Hidden);

        let title = gtk::Label::builder()
            .label("Nothing playing")
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .max_width_chars(28)
            .hexpand(true)
            .css_classes(["heading"])
            .build();
        let subtitle = gtk::Label::builder()
            .label("Choose something from your library")
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .max_width_chars(36)
            .hexpand(true)
            .css_classes(["dim-label", "caption"])
            .build();

        let favorite_button = gtk::ToggleButton::builder()
            .icon_name("non-starred-symbolic")
            .tooltip_text("Favourite")
            .css_classes(["flat", "circular"])
            .valign(gtk::Align::Center)
            .build();

        let meta = gtk::Box::new(gtk::Orientation::Vertical, 2);
        meta.set_valign(gtk::Align::Center);
        meta.set_hexpand(true);
        meta.append(&title);
        meta.append(&subtitle);

        let left = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        left.set_valign(gtk::Align::Center);
        left.set_size_request(280, -1);
        left.append(&art_button);
        left.append(&meta);
        left.append(&favorite_button);

        let prev_button = icon_button("media-skip-backward-symbolic", "Previous");
        let play_button = gtk::Button::builder()
            .icon_name("media-playback-start-symbolic")
            .tooltip_text("Play")
            .css_classes(["circular", "suggested-action", "cadence-play"])
            .build();
        let next_button = icon_button("media-skip-forward-symbolic", "Next");
        let shuffle_button = gtk::ToggleButton::builder()
            .icon_name("media-playlist-shuffle-symbolic")
            .tooltip_text("Shuffle")
            .css_classes(["flat", "circular"])
            .build();
        let repeat_button = gtk::Button::builder()
            .icon_name("media-playlist-repeat-symbolic")
            .tooltip_text("Repeat: Off")
            .css_classes(["flat", "circular"])
            .build();

        let controls = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        controls.set_halign(gtk::Align::Center);
        controls.append(&shuffle_button);
        controls.append(&prev_button);
        controls.append(&play_button);
        controls.append(&next_button);
        controls.append(&repeat_button);

        let seeking = Rc::new(Cell::new(false));
        let seek = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.001);
        seek.set_draw_value(false);
        seek.set_hexpand(true);
        seek.set_digits(3);
        seek.add_css_class("cadence-seek");

        let position_label = gtk::Label::builder()
            .label("0:00")
            .width_chars(5)
            .css_classes(["caption", "dim-label", "numeric"])
            .build();
        let duration_label = gtk::Label::builder()
            .label("0:00")
            .width_chars(5)
            .css_classes(["caption", "dim-label", "numeric"])
            .build();

        let seek_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        seek_row.set_hexpand(true);
        seek_row.append(&position_label);
        seek_row.append(&seek);
        seek_row.append(&duration_label);

        let center = gtk::Box::new(gtk::Orientation::Vertical, 2);
        center.set_hexpand(true);
        center.set_valign(gtk::Align::Center);
        center.set_margin_start(8);
        center.set_margin_end(8);
        center.append(&controls);
        center.append(&seek_row);

        let volume = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
        volume.set_draw_value(false);
        volume.set_value(1.0);
        volume.set_width_request(96);
        volume.add_css_class("cadence-volume");

        let queue_button = gtk::ToggleButton::builder()
            .icon_name("view-list-bullet-symbolic")
            .tooltip_text("Queue")
            .css_classes(["flat", "circular"])
            .build();

        let right = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        right.set_valign(gtk::Align::Center);
        right.set_halign(gtk::Align::End);
        right.set_size_request(160, -1);
        right.append(&gtk::Image::from_icon_name("audio-volume-high-symbolic"));
        right.append(&volume);
        right.append(&queue_button);

        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        widget.add_css_class("cadence-player");
        widget.set_margin_start(12);
        widget.set_margin_end(12);
        widget.set_margin_top(8);
        widget.set_margin_bottom(8);
        widget.set_hexpand(true);
        widget.set_vexpand(false);
        widget.set_valign(gtk::Align::End);
        // Fixed dock height: request is a floor; overflow + CSS max-height clamp growth.
        widget.set_size_request(-1, DOCK_HEIGHT);
        widget.set_overflow(gtk::Overflow::Hidden);
        widget.append(&left);
        widget.append(&center);
        widget.append(&right);

        let seeking_press = Rc::clone(&seeking);
        seek.connect_change_value(move |_, _, _| {
            seeking_press.set(true);
            glib::Propagation::Proceed
        });

        let on_expand: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));
        {
            let on_expand = Rc::clone(&on_expand);
            art_button.connect_clicked(move |_| {
                if let Some(cb) = on_expand.borrow().as_ref() {
                    cb();
                }
            });
        }

        Self {
            widget,
            play_button,
            prev_button,
            next_button,
            shuffle_button,
            repeat_button,
            queue_button,
            favorite_button,
            art_button,
            seek,
            volume,
            title,
            subtitle,
            position_label,
            duration_label,
            artwork,
            seeking,
            on_expand,
        }
    }

    pub fn connect_expand<F: Fn() + 'static>(&self, f: F) {
        *self.on_expand.borrow_mut() = Some(Box::new(f));
    }

    pub fn is_seeking(&self) -> bool {
        self.seeking.get()
    }

    pub fn finish_seek(&self) {
        self.seeking.set(false);
    }

    pub fn set_track_info(
        &self,
        title: &str,
        artist: &str,
        album: &str,
        artwork_path: Option<&Path>,
        favorite: bool,
    ) {
        self.title.set_label(title);
        let artist = if artist.is_empty() {
            "Unknown Artist"
        } else {
            artist
        };
        let line = if album.is_empty() {
            artist.to_string()
        } else {
            format!("{artist}  ·  {album}")
        };
        self.subtitle.set_label(&line);
        self.favorite_button.set_active(favorite);
        self.favorite_button.set_icon_name(if favorite {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        });

        if let Some(path) = artwork_path {
            set_artwork_file(&self.artwork, Some(path), ART_SIZE);
        } else {
            set_artwork_file(&self.artwork, None, ART_SIZE);
            set_fallback_art(&self.artwork);
        }
    }

    pub fn set_playing(&self, playing: bool) {
        self.play_button.set_icon_name(if playing {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        });
    }

    pub fn update_position(&self, position_ms: u64, duration_ms: u64) {
        if self.is_seeking() {
            return;
        }
        self.position_label
            .set_label(&format_duration_ms(Some(position_ms)));
        // Remaining time on the right feels more premium than total-only.
        let remaining = duration_ms.saturating_sub(position_ms);
        self.duration_label
            .set_label(&format!("-{}", format_duration_ms(Some(remaining))));
        if duration_ms > 0 {
            self.seek.set_value(position_ms as f64 / duration_ms as f64);
        }
    }
}

impl Default for PlayerBar {
    fn default() -> Self {
        Self::new()
    }
}

fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(tooltip)
        .css_classes(["flat", "circular"])
        .build()
}

fn set_fallback_art(artwork: &gtk::Picture) {
    if let Some(display) = gdk::Display::default() {
        let paintable = gtk::IconTheme::for_display(&display).lookup_icon(
            "folder-music-symbolic",
            &[],
            ART_SIZE,
            1,
            gtk::TextDirection::None,
            gtk::IconLookupFlags::empty(),
        );
        artwork.set_paintable(Some(&paintable.upcast::<gdk::Paintable>()));
    }
}
