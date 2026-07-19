//! `adw::Application` setup for Cadence.

use std::path::PathBuf;

use adw::prelude::*;
use cadence_core::{APP_ID, APP_NAME};
use gtk::gio;
use gtk::glib;

use crate::window::CadenceWindow;

pub fn run() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_startup(|app| {
        register_app_icons();
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default);
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_str!("ui/style.css"));
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        let _ = app;
    });

    app.connect_activate(|app| {
        let window = CadenceWindow::new(app);
        window.present();
    });

    let quit = gio::SimpleAction::new("quit", None);
    {
        let app = app.clone();
        quit.connect_activate(move |_, _| app.quit());
    }
    app.add_action(&quit);
    app.set_accels_for_action("app.quit", &["<Primary>q"]);
    app.set_accels_for_action("win.preferences", &["<Primary>comma"]);

    let about = gio::SimpleAction::new("about", None);
    {
        let app = app.clone();
        about.connect_activate(move |_, _| {
            let mut builder = adw::AboutWindow::builder()
                .application_name(APP_NAME)
                .application_icon(APP_ID)
                .developer_name("The Cadence Contributors")
                .version(env!("CARGO_PKG_VERSION"))
                .comments(
                    "A modern, native music library for Linux.\n\
                     Early public beta — contributions welcome.",
                )
                .license_type(gtk::License::Gpl30)
                .website("https://github.com/loafdaddy/Cadence-Music")
                .issue_url("https://github.com/loafdaddy/Cadence-Music/issues")
                .modal(true);
            if let Some(parent) = app.active_window() {
                builder = builder.transient_for(&parent);
            }
            builder.build().present();
        });
    }
    app.add_action(&about);

    app.run()
}

/// Register `data/icons` so `org.cadence.Cadence` resolves when running from cargo.
fn register_app_icons() {
    let candidates = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/icons"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/icons"),
    ];

    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };
    let theme = gtk::IconTheme::for_display(&display);
    for path in candidates {
        if path.is_dir() {
            let canonical = path.canonicalize().unwrap_or(path);
            theme.add_search_path(canonical);
            tracing::debug!("registered icon search path");
            return;
        }
    }
}
