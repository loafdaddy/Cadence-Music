//! Sized album / track artwork that cannot blow out the layout.
//!
//! GTK4 ignores CSS `max-width` / `max-height` for size negotiation. A bare
//! `gtk::Picture` reports the image's natural pixel size as its *preferred*
//! size even when `can_shrink` is true — so high-res covers expand parents.
//!
//! Fix: load a display-sized texture **and** host the picture in `gtk::Fixed`,
//! which reports only its own size request (children never inflate it).

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
        .build();
    // Ask for exactly `size`; Fixed will allocate this and clip overflow.
    picture.set_size_request(size, size);
    for class in css_classes {
        picture.add_css_class(class);
    }

    // GtkFixed's preferred size is only its size_request — Picture's natural
    // size cannot propagate to parents (unlike Box / AspectFrame / ScrolledWindow).
    let fixed = gtk::Fixed::new();
    fixed.set_size_request(size, size);
    fixed.set_hexpand(false);
    fixed.set_vexpand(false);
    fixed.set_halign(gtk::Align::Center);
    fixed.set_valign(gtk::Align::Center);
    fixed.set_overflow(gtk::Overflow::Hidden);
    fixed.add_css_class("cadence-art-square");
    fixed.put(&picture, 0.0, 0.0);

    (fixed.upcast(), picture)
}

/// Load artwork scaled to about `size` CSS pixels (2× for HiDPI). Never uses
/// `Picture::set_file`, which would reintroduce full-resolution natural size.
pub fn set_artwork_file(picture: &gtk::Picture, path: Option<&Path>, size: i32) {
    match path {
        Some(path) => match load_scaled_texture(path, size) {
            Some(texture) => {
                picture.set_paintable(Some(&texture));
                // Re-assert after paintable change — some GTK versions remeasure.
                picture.set_size_request(size, size);
            }
            None => picture.set_paintable(gdk::Paintable::NONE),
        },
        None => picture.set_paintable(gdk::Paintable::NONE),
    }
}

fn load_scaled_texture(path: &Path, size: i32) -> Option<gdk::Texture> {
    let edge = size.saturating_mul(2).max(size).max(1);
    // Force a square raster so natural width/height never disagree with the frame.
    let pixbuf = Pixbuf::from_file_at_scale(path, edge, edge, false).ok()?;
    Some(gdk::Texture::for_pixbuf(&pixbuf))
}
