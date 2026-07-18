//! Background services that keep the UI thread free of disk I/O.

mod library_service;

pub use library_service::{LibraryEvent, LibraryService};
