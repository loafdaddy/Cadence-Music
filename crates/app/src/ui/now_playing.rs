//! Immersive Now Playing overlay with optional vinyl presentation.

use std::cell::Cell;
use std::f64::consts::PI;
use std::path::Path;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gdk, glib};

use super::{artwork_frame, format_duration_ms, set_artwork_file};

const DISC_SIZE: i32 = 160;
const ART_SIZE: i32 = 110;

/// Full-window overlay for focused listening.
pub struct NowPlaying {
    pub widget: gtk::Box,
    pub close_button: gtk::Button,
    pub play_button: gtk::Button,
    pub prev_button: gtk::Button,
    pub next_button: gtk::Button,
    pub vinyl_toggle: gtk::Switch,
    title: gtk::Label,
    subtitle: gtk::Label,
    artwork: gtk::Picture,
    disc: gtk::Box,
    tonearm: gtk::DrawingArea,
    position_label: gtk::Label,
    duration_label: gtk::Label,
    seek: gtk::Scale,
    playing: Cell<bool>,
    vinyl_enabled: Rc<Cell<bool>>,
    seeking: Rc<Cell<bool>>,
}

impl NowPlaying {
    #[must_use]
    pub fn new() -> Self {
        let close_button = gtk::Button::builder()
            .icon_name("go-down-symbolic")
            .tooltip_text("Close")
            .css_classes(["flat", "circular"])
            .halign(gtk::Align::Start)
            .build();

        let vinyl_label = gtk::Label::builder()
            .label("Vinyl animation")
            .css_classes(["caption"])
            .build();
        let vinyl_toggle = gtk::Switch::builder()
            .active(true)
            .valign(gtk::Align::Center)
            .build();
        let vinyl_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        vinyl_row.set_halign(gtk::Align::End);
        vinyl_row.append(&vinyl_label);
        vinyl_row.append(&vinyl_toggle);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        header.set_margin_start(16);
        header.set_margin_end(16);
        header.set_margin_top(12);
        header.append(&close_button);
        header.append(
            &gtk::Label::builder()
                .label("Now Playing")
                .hexpand(true)
                .css_classes(["heading"])
                .build(),
        );
        header.append(&vinyl_row);

        let (art_frame, artwork) = artwork_frame(ART_SIZE, &["cadence-vinyl-label"]);
        art_frame.remove_css_class("cadence-art-square");
        art_frame.add_css_class("cadence-vinyl-art-wrap");
        art_frame.set_halign(gtk::Align::Center);
        art_frame.set_valign(gtk::Align::Center);
        art_frame.set_margin_top(25);
        art_frame.set_margin_bottom(25);
        art_frame.set_margin_start(25);
        art_frame.set_margin_end(25);

        let disc = gtk::Box::new(gtk::Orientation::Vertical, 0);
        disc.add_css_class("cadence-vinyl-disc");
        disc.set_halign(gtk::Align::Center);
        disc.set_valign(gtk::Align::Center);
        disc.set_size_request(DISC_SIZE, DISC_SIZE);
        disc.set_overflow(gtk::Overflow::Hidden);
        disc.append(&art_frame);

        let tonearm = gtk::DrawingArea::builder()
            .content_width(90)
            .content_height(140)
            .halign(gtk::Align::End)
            .valign(gtk::Align::Start)
            .margin_end(4)
            .margin_top(4)
            .css_classes(["cadence-tonearm"])
            .build();
        tonearm.set_draw_func(|_, cr, w, h| {
            draw_tonearm(cr, w as f64, h as f64, false);
        });

        let stage = gtk::Overlay::new();
        stage.set_halign(gtk::Align::Center);
        stage.set_size_request(220, 180);
        stage.set_child(Some(&disc));
        stage.add_overlay(&tonearm);

        let title = gtk::Label::builder()
            .label("Nothing playing")
            .wrap(true)
            .justify(gtk::Justification::Center)
            .halign(gtk::Align::Center)
            .css_classes(["title-2"])
            .build();
        let subtitle = gtk::Label::builder()
            .label("")
            .wrap(true)
            .justify(gtk::Justification::Center)
            .halign(gtk::Align::Center)
            .css_classes(["dim-label"])
            .build();

        let prev_button = gtk::Button::builder()
            .icon_name("media-skip-backward-symbolic")
            .css_classes(["flat", "circular"])
            .build();
        let play_button = gtk::Button::builder()
            .icon_name("media-playback-start-symbolic")
            .css_classes(["circular", "suggested-action", "cadence-play"])
            .build();
        play_button.set_size_request(48, 48);
        let next_button = gtk::Button::builder()
            .icon_name("media-skip-forward-symbolic")
            .css_classes(["flat", "circular"])
            .build();

        let controls = gtk::Box::new(gtk::Orientation::Horizontal, 18);
        controls.set_halign(gtk::Align::Center);
        controls.append(&prev_button);
        controls.append(&play_button);
        controls.append(&next_button);

        let seeking = Rc::new(Cell::new(false));
        let seek = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.001);
        seek.set_draw_value(false);
        seek.set_hexpand(true);
        seek.add_css_class("cadence-seek");
        let seeking_press = Rc::clone(&seeking);
        seek.connect_change_value(move |_, _, _| {
            seeking_press.set(true);
            glib::Propagation::Proceed
        });

        let position_label = gtk::Label::builder()
            .label("0:00")
            .css_classes(["caption", "dim-label", "numeric"])
            .build();
        let duration_label = gtk::Label::builder()
            .label("0:00")
            .css_classes(["caption", "dim-label", "numeric"])
            .build();
        let seek_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        seek_row.set_halign(gtk::Align::Center);
        seek_row.set_size_request(320, -1);
        seek_row.append(&position_label);
        seek_row.append(&seek);
        seek_row.append(&duration_label);

        let column = gtk::Box::new(gtk::Orientation::Vertical, 14);
        column.set_halign(gtk::Align::Center);
        column.set_valign(gtk::Align::Center);
        column.set_hexpand(true);
        column.set_vexpand(true);
        column.append(&stage);
        column.append(&title);
        column.append(&subtitle);
        column.append(&controls);
        column.append(&seek_row);

        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.add_css_class("cadence-now-playing");
        widget.set_hexpand(true);
        widget.set_vexpand(true);
        widget.append(&header);
        widget.append(&column);

        let vinyl_enabled = Rc::new(Cell::new(true));
        {
            let vinyl_enabled = Rc::clone(&vinyl_enabled);
            let disc = disc.clone();
            let tonearm = tonearm.clone();
            vinyl_toggle.connect_active_notify(move |sw| {
                let on = sw.is_active();
                vinyl_enabled.set(on);
                if !on {
                    disc.remove_css_class("playing");
                    tonearm.remove_css_class("lowered");
                }
                tonearm.queue_draw();
            });
        }

        Self {
            widget,
            close_button,
            play_button,
            prev_button,
            next_button,
            vinyl_toggle,
            title,
            subtitle,
            artwork,
            disc,
            tonearm,
            position_label,
            duration_label,
            seek,
            playing: Cell::new(false),
            vinyl_enabled,
            seeking,
        }
    }

    pub fn set_track_info(&self, title: &str, artist: &str, album: &str, art: Option<&Path>) {
        self.title.set_label(title);
        self.subtitle.set_label(&format!("{artist}  ·  {album}"));
        if let Some(path) = art {
            set_artwork_file(&self.artwork, Some(path));
        } else {
            set_artwork_file(&self.artwork, None);
            if let Some(display) = gdk::Display::default() {
                let p = gtk::IconTheme::for_display(&display).lookup_icon(
                    "folder-music-symbolic",
                    &[],
                    ART_SIZE,
                    1,
                    gtk::TextDirection::None,
                    gtk::IconLookupFlags::empty(),
                );
                self.artwork
                    .set_paintable(Some(&p.upcast::<gdk::Paintable>()));
            }
        }
    }

    pub fn set_playing(&self, playing: bool) {
        self.playing.set(playing);
        self.play_button.set_icon_name(if playing {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        });
        self.refresh_vinyl_classes();
    }

    pub fn finish_seek(&self) {
        self.seeking.set(false);
    }

    pub fn is_seeking(&self) -> bool {
        self.seeking.get()
    }

    pub fn update_position(&self, position_ms: u64, duration_ms: u64) {
        if self.is_seeking() {
            return;
        }
        self.position_label
            .set_label(&format_duration_ms(Some(position_ms)));
        self.duration_label
            .set_label(&format_duration_ms(Some(duration_ms)));
        if duration_ms > 0 {
            self.seek
                .set_value(position_ms as f64 / duration_ms as f64);
        }
    }

    pub fn seek_widget(&self) -> &gtk::Scale {
        &self.seek
    }

    fn refresh_vinyl_classes(&self) {
        let spin = self.playing.get() && self.vinyl_enabled.get();
        let lowered = spin;
        if spin {
            self.disc.add_css_class("playing");
        } else {
            self.disc.remove_css_class("playing");
        }
        if lowered {
            self.tonearm.add_css_class("lowered");
        } else {
            self.tonearm.remove_css_class("lowered");
        }
        let lowered = lowered;
        self.tonearm.set_draw_func(move |_, cr, w, h| {
            draw_tonearm(cr, w as f64, h as f64, lowered);
        });
        self.tonearm.queue_draw();
    }
}

impl Default for NowPlaying {
    fn default() -> Self {
        Self::new()
    }
}

fn draw_tonearm(cr: &gtk::cairo::Context, w: f64, h: f64, lowered: bool) {
    let pivot_x = w * 0.82;
    let pivot_y = h * 0.14;
    let angle: f64 = if lowered { 2.15 } else { 2.55 };
    let arm_len = h * 0.58;
    let end_x = pivot_x + arm_len * angle.cos();
    let end_y = pivot_y + arm_len * angle.sin();

    cr.set_source_rgba(0.0, 0.0, 0.0, 0.28);
    cr.set_line_width(4.0);
    cr.set_line_cap(gtk::cairo::LineCap::Round);
    cr.move_to(pivot_x + 1.5, pivot_y + 2.0);
    cr.line_to(end_x + 1.5, end_y + 2.0);
    let _ = cr.stroke();

    cr.set_source_rgb(0.78, 0.80, 0.84);
    cr.set_line_width(3.2);
    cr.move_to(pivot_x, pivot_y);
    cr.line_to(end_x, end_y);
    let _ = cr.stroke();

    cr.set_source_rgb(0.55, 0.58, 0.62);
    cr.arc(pivot_x, pivot_y, 8.0, 0.0, PI * 2.0);
    let _ = cr.fill();
    cr.set_source_rgb(0.32, 0.34, 0.38);
    cr.arc(pivot_x, pivot_y, 3.5, 0.0, PI * 2.0);
    let _ = cr.fill();

    cr.set_source_rgb(0.90, 0.91, 0.93);
    cr.arc(end_x, end_y, 6.0, 0.0, PI * 2.0);
    let _ = cr.fill();

    cr.set_source_rgb(0.95, 0.55, 0.35);
    cr.set_line_width(2.0);
    cr.move_to(end_x, end_y + 2.0);
    cr.line_to(end_x - 1.0, end_y + 12.0);
    let _ = cr.stroke();
}
