//! GTK / libadwaita widgets for Cadence.

mod albums_view;
mod artist_detail;
mod artists_view;
mod artwork;
mod context_menu;
mod empty_state;
mod library_home;
mod metadata_dialog;
mod now_playing;
mod organize_dialog;
mod player_bar;
mod playlists_view;
mod preferences;
mod queue_view;
mod search_results;
mod songs_view;

pub use artwork::{artwork_frame, set_artwork_file};
pub use context_menu::{attach_context_menu, ContextAction};

pub use albums_view::AlbumsView;
pub use artist_detail::ArtistDetail;
pub use artists_view::ArtistsView;
pub use empty_state::EmptyState;
pub use library_home::LibraryHome;
pub use metadata_dialog::MetadataDialog;
pub use now_playing::NowPlaying;
pub use organize_dialog::OrganizeDialog;
pub use player_bar::PlayerBar;
pub use playlists_view::PlaylistsView;
pub use preferences::PreferencesWindow;
pub use queue_view::QueueView;
pub use search_results::SearchResults;
pub use songs_view::SongsView;

/// Format milliseconds as `m:ss` or `h:mm:ss`.
#[must_use]
pub fn format_duration_ms(ms: Option<u64>) -> String {
    let Some(ms) = ms else {
        return "—:—".into();
    };
    let total_secs = ms / 1000;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    if hours > 0 {
        format!("{hours}:{mins:02}:{secs:02}")
    } else {
        format!("{mins}:{secs:02}")
    }
}
