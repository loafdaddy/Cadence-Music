//! # cadence-core
//!
//! The engine behind [Cadence](https://github.com/loafdaddy/Cadence-Music), a
//! native GTK4 music library application for Linux.
//!
//! This crate contains everything that is *not* the GUI: the SQLite-backed
//! library database, the recursive folder [`scanner`], audio [`metadata`]
//! reading/writing, and the optional, non-destructive [`organization`] engine.
//!
//! It has no dependency on GTK, libadwaita or GStreamer, which keeps the core
//! fast to compile and trivial to unit-test in isolation.
//!
//! ## Module map
//!
//! - [`models`] — plain data types (`Track`, `Album`, `Artist`, …).
//! - [`db`] — the [`db::Database`] and all queries.
//! - [`scanner`] — recursive discovery, parallel parsing, folder watching.
//! - [`metadata`] — tag reading/writing and missing-field detection.
//! - [`organization`] — path templates, preview plans and undo.
//! - [`artwork`] — embedded cover-art extraction and caching.
//! - [`lookup`] — MusicBrainz / Cover Art Archive helpers.
//! - [`paths`] — XDG data/cache locations.
//! - [`error`] — the shared [`error::Error`] and [`error::Result`].

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

pub mod artwork;
pub mod db;
pub mod error;
pub mod lookup;
pub mod metadata;
pub mod models;
pub mod organization;
pub mod paths;
pub mod scanner;

pub use error::{Error, Result};

/// The application's reverse-DNS identifier, shared by the GUI, the desktop
/// entry, GSettings and MPRIS.
pub const APP_ID: &str = "org.cadence.Cadence";

/// Human-readable application name.
pub const APP_NAME: &str = "Cadence";

/// Wordmark used in the header (“Cadence.”).
pub const APP_WORDMARK: &str = "Cadence.";
