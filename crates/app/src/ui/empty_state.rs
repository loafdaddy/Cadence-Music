use adw::prelude::*;
use cadence_core::{APP_ID, APP_WORDMARK};

/// Shown when the library has no folders configured.
pub struct EmptyState {
    pub widget: adw::StatusPage,
    pub add_button: gtk::Button,
}

impl EmptyState {
    #[must_use]
    pub fn new() -> Self {
        let add_button = gtk::Button::builder()
            .label("Add Music Folder")
            .css_classes(["suggested-action", "pill"])
            .halign(gtk::Align::Center)
            .build();

        let widget = adw::StatusPage::builder()
            .icon_name(APP_ID)
            .title(APP_WORDMARK)
            .description(
                "A modern, native music library for Linux.\n\
                 Add a folder to start building your library.",
            )
            .child(&add_button)
            .build();
        widget.add_css_class("cadence-empty");

        Self { widget, add_button }
    }
}

impl Default for EmptyState {
    fn default() -> Self {
        Self::new()
    }
}
