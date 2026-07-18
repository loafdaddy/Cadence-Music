//! Sized album / track artwork that cannot blow out the layout.
//!
//! GTK4 ignores CSS `max-width` / `max-height`. A bare `gtk::Picture` reports
//! the image's natural pixel size, so high-res covers expand the parent unless
//! we force a fixed allocation with overflow clipping.

use std::path::Path;

use adw::prelude::*;
use gtk::gio;

/// Fixed-size square artwork. Returns `(wrapper, picture)` — put `wrapper` in
/// the UI and call [`set_artwork_file`] on `picture` to update.
pub fn artwork_frame(size: i32, css_classes: &[&str]) -> (gtk::Box, gtk::Picture) {
    let picture = gtk::Picture::builder()
        .can_shrink(true)
        .content_fit(gtk::ContentFit::Cover)
        .hexpand(true)
        .vexpand(true)
        .build();
    for class in css_classes {
        picture.add_css_class(class);
    }

    // Outer box owns the exact square; AspectFrame alone can still stretch
    // inside a homogeneous FlowBox cell.
    let wrapper = gtk::Box::new(gtk::Orientation::Vertical, 0);
    wrapper.set_size_request(size, size);
    wrapper.set_hexpand(false);
    wrapper.set_vexpand(false);
    wrapper.set_halign(gtk::Align::Center);
    wrapper.set_valign(gtk::Align::Center);
    wrapper.set_overflow(gtk::Overflow::Hidden);
    wrapper.add_css_class("cadence-art-square");

    let frame = gtk::AspectFrame::builder()
        .ratio(1.0)
        .obey_child(false)
        .hexpand(true)
        .vexpand(true)
        .build();
    frame.set_child(Some(&picture));
    wrapper.append(&frame);
    (wrapper, picture)
}

pub fn set_artwork_file(picture: &gtk::Picture, path: Option<&Path>) {
    match path {
        Some(path) => picture.set_file(Some(&gio::File::for_path(path))),
        None => picture.set_file(None::<&gio::File>),
    }
}
