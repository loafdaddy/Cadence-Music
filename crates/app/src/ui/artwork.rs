//! Sized album / track artwork that cannot blow out the layout.
//!
//! GTK4 ignores CSS `max-width` / `max-height` for size negotiation. A bare
//! `gtk::Picture` reports the image's natural pixel size as its *preferred*
//! size even when `can_shrink` is true — so high-res covers expand parents
//! unless we (1) load a display-sized texture and (2) wrap in a widget that
//! does not propagate the child's natural size.

use std::path::Path;

use adw::prelude::*;
use gtk::gdk;
use gtk::gdk_pixbuf::Pixbuf;

/// Fixed-size square artwork. Returns `(wrapper, picture)` — put `wrapper` in
/// the UI and call [`set_artwork_file`] on `picture` to update.
pub fn artwork_frame(size: i32, css_classes: &[&str]) -> (gtk::Widget, gtk::Picture) {
    let picture = gtk::Picture::builder()
        .can_shrink(true)
        .content_fit(gtk::ContentFit::Cover)
        .width_request(size)
        .height_request(size)
        .build();
    for class in css_classes {
        picture.add_css_class(class);
    }

    // ScrolledWindow with propagate_natural_* = false reports only its
    // size_request upward — Picture's huge preferred size cannot leak out.
    let clip = gtk::ScrolledWindow::builder()
        .can_focus(false)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .propagate_natural_width(false)
        .propagate_natural_height(false)
        .kinetic_scrolling(false)
        .child(&picture)
        .width_request(size)
        .height_request(size)
        .build();
    clip.set_size_request(size, size);
    clip.set_hexpand(false);
    clip.set_vexpand(false);
    clip.set_halign(gtk::Align::Center);
    clip.set_valign(gtk::Align::Center);
    clip.set_overflow(gtk::Overflow::Hidden);
    clip.add_css_class("cadence-art-square");

    (clip.upcast(), picture)
}

/// Load artwork scaled to about `size` CSS pixels (2× for HiDPI). Never uses
/// `Picture::set_file`, which would reintroduce full-resolution natural size.
pub fn set_artwork_file(picture: &gtk::Picture, path: Option<&Path>, size: i32) {
    match path {
        Some(path) => match load_scaled_texture(path, size) {
            Some(texture) => picture.set_paintable(Some(&texture)),
            None => picture.set_paintable(gdk::Paintable::NONE),
        },
        None => picture.set_paintable(gdk::Paintable::NONE),
    }
}

fn load_scaled_texture(path: &Path, size: i32) -> Option<gdk::Texture> {
    let edge = size.saturating_mul(2).max(size).max(1);
    let pixbuf = Pixbuf::from_file_at_scale(path, edge, edge, true).ok()?;
    Some(gdk::Texture::for_pixbuf(&pixbuf))
}
