use adw::prelude::*;
use cadence_core::models::Track;

use super::format_duration_ms;

pub struct QueueView {
    pub widget: gtk::Box,
    list: gtk::ListBox,
}

impl QueueView {
    #[must_use]
    pub fn new() -> Self {
        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["navigation-sidebar"])
            .build();
        let scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .child(&list)
            .build();

        let header = gtk::Label::builder()
            .label("Play Queue")
            .css_classes(["title-2"])
            .margin_start(12)
            .margin_top(12)
            .margin_bottom(8)
            .xalign(0.0)
            .build();

        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.set_width_request(280);
        widget.append(&header);
        widget.append(&scrolled);

        Self { widget, list }
    }

    pub fn set_tracks(&self, tracks: &[Track], current: Option<usize>) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        for (i, track) in tracks.iter().enumerate() {
            let title = gtk::Label::builder()
                .label(&track.title)
                .xalign(0.0)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .hexpand(true)
                .build();
            if Some(i) == current {
                title.add_css_class("heading");
            }
            let duration = gtk::Label::builder()
                .label(format_duration_ms(track.duration_ms))
                .css_classes(["dim-label", "numeric"])
                .build();
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            row.set_margin_start(12);
            row.set_margin_end(12);
            row.set_margin_top(6);
            row.set_margin_bottom(6);
            row.append(&title);
            row.append(&duration);
            self.list
                .append(&gtk::ListBoxRow::builder().child(&row).build());
        }
    }
}

impl Default for QueueView {
    fn default() -> Self {
        Self::new()
    }
}
