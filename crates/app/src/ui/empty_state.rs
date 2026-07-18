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
            .icon_name("folder-music-symbolic")
            .title("Welcome to Cadence")
            .description("Add a folder to start building your music library.")
            .child(&add_button)
            .build();

        Self { widget, add_button }
    }
}

impl Default for EmptyState {
    fn default() -> Self {
        Self::new()
    }
}
