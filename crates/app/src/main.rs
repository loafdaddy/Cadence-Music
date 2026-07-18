//! Cadence — a native GTK4 music library for Linux.

mod application;
mod mpris;
mod playback;
mod services;
mod ui;
mod window;

use std::path::Path;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Bundled/dev `.deps` GStreamer libs often ship an incomplete plugin tree.
    // Point the registry at the host plugins before `gst_init` so playback works.
    configure_gstreamer_plugins();

    if let Err(err) = gstreamer::init() {
        eprintln!("Failed to initialize GStreamer: {err}");
        std::process::exit(1);
    }

    let code = application::run();
    std::process::exit(code.into());
}

/// Ensure decoder/sink plugins are discoverable when linked against a sparse
/// GStreamer prefix (local `.deps`). Respects any path the user or Flatpak
/// already set.
fn configure_gstreamer_plugins() {
    if std::env::var_os("GST_PLUGIN_PATH").is_some()
        || std::env::var_os("GST_PLUGIN_SYSTEM_PATH").is_some()
    {
        return;
    }

    let candidates = [
        "/usr/lib64/gstreamer-1.0",
        "/usr/lib/x86_64-linux-gnu/gstreamer-1.0",
        "/usr/lib/gstreamer-1.0",
    ];

    for candidate in candidates {
        let dir = Path::new(candidate);
        if !(dir.join("libgstmpg123.so").exists()
            || dir.join("libgstflac.so").exists()
            || dir.join("libgstpulseaudio.so").exists()
            || dir.join("libgstplayback.so").exists())
        {
            continue;
        }

        // Called once before any other threads touch the environment.
        // SAFETY: single-threaded startup; no concurrent env access yet.
        unsafe {
            std::env::set_var("GST_PLUGIN_SYSTEM_PATH", candidate);
            std::env::set_var("GST_PLUGIN_PATH", candidate);
        }
        tracing::info!(path = candidate, "using system GStreamer plugin path");
        return;
    }

    tracing::warn!("no system GStreamer plugin directory found; playback may fail");
}
