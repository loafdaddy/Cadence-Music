//! Cadence — a native GTK4 music library for Linux.

mod application;
mod mpris;
mod playback;
mod services;
mod ui;
mod window;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    if let Err(err) = gstreamer::init() {
        eprintln!("Failed to initialize GStreamer: {err}");
        std::process::exit(1);
    }

    let code = application::run();
    std::process::exit(code.into());
}
