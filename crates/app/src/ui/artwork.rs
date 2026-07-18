//! Sized album / track artwork that cannot blow out the layout.
//!
//! `gtk::Picture` reports a paintable's intrinsic pixel size as preferred size,
//! which on HiDPI (and inside `GtkFixed`) leaves covers tiny in the corner or
//! blows out parents. `gtk::Image` with [`gtk::ImageExt::set_pixel_size`]
//! always paints at the requested CSS size.

use std::path::Path;

use adw::prelude::*;
use gtk::gdk;
use gtk::gdk_pixbuf::Pixbuf;

/// Fixed-size square artwork. Returns `(wrapper, image)` — put `wrapper` in
/// the UI and call [`set_artwork_file`] on `image` to update.
pub fn artwork_frame(size: i32, css_classes: &[&str]) -> (gtk::Widget, gtk::Image) {
    let image = gtk::Image::builder()
        .pixel_size(size)
        .width_request(size)
        .height_request(size)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();
    for class in css_classes {
        image.add_css_class(class);
    }
    image.add_css_class("cadence-art-square");

    // Outer box owns the exact square and clips; Image's pixel_size keeps the
    // painted cover filling that square without leaking preferred size upward.
    let wrapper = gtk::Box::new(gtk::Orientation::Vertical, 0);
    wrapper.set_size_request(size, size);
    wrapper.set_hexpand(false);
    wrapper.set_vexpand(false);
    wrapper.set_halign(gtk::Align::Center);
    wrapper.set_valign(gtk::Align::Center);
    wrapper.set_overflow(gtk::Overflow::Hidden);
    wrapper.add_css_class("cadence-art-frame");
    wrapper.append(&image);

    (wrapper.upcast(), image)
}

/// Load artwork for a frame of `size` CSS pixels (decoded at 2× for sharpness).
pub fn set_artwork_file(image: &gtk::Image, path: Option<&Path>, size: i32) {
    image.set_pixel_size(size);
    match path {
        Some(path) => match load_scaled_texture(path, size) {
            Some(texture) => image.set_paintable(Some(&texture)),
            None => image.set_paintable(gdk::Paintable::NONE),
        },
        None => image.set_paintable(gdk::Paintable::NONE),
    }
}

fn load_scaled_texture(path: &Path, size: i32) -> Option<gdk::Texture> {
    let edge = size.saturating_mul(2).max(size).max(1);
    let pixbuf = Pixbuf::from_file_at_scale(path, edge, edge, false).ok()?;
    Some(gdk::Texture::for_pixbuf(&pixbuf))
}
