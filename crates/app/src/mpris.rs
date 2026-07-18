//! MPRIS D-Bus integration via `mpris_server::Player`.

use std::cell::RefCell;
use std::rc::Rc;

use cadence_core::APP_ID;
use gtk::glib;
use mpris_server::{Metadata, PlaybackStatus, Player, Time};

use crate::playback::PlaybackState;

/// Thin handle that owns the optional MPRIS player and mirrors playback info.
pub struct MprisService {
    player: RefCell<Option<Rc<Player>>>,
}

impl Default for MprisService {
    fn default() -> Self {
        Self {
            player: RefCell::new(None),
        }
    }
}

impl std::fmt::Debug for MprisService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MprisService").finish_non_exhaustive()
    }
}

/// Commands forwarded from MPRIS into the UI.
#[derive(Debug, Clone, Copy)]
pub enum MprisCommand {
    Play,
    Pause,
    PlayPause,
    Stop,
    Next,
    Previous,
}

/// Start the MPRIS player on the glib main context and route controls through `on_command`.
pub fn start_mpris(on_command: impl Fn(MprisCommand) + 'static) -> Rc<MprisService> {
    let service = Rc::new(MprisService::default());
    let service_set = Rc::clone(&service);
    let on_command = Rc::new(on_command);

    glib::spawn_future_local(async move {
        let player = match Player::builder("Cadence")
            .identity("Cadence")
            .desktop_entry(APP_ID)
            .can_raise(true)
            .can_play(true)
            .can_pause(true)
            .can_seek(true)
            .can_control(true)
            .can_go_next(true)
            .can_go_previous(true)
            .supported_uri_schemes(vec!["file".to_owned()])
            .supported_mime_types(vec![
                "audio/mpeg".to_owned(),
                "audio/flac".to_owned(),
                "audio/ogg".to_owned(),
                "audio/mp4".to_owned(),
            ])
            .build()
            .await
        {
            Ok(player) => Rc::new(player),
            Err(err) => {
                tracing::warn!(%err, "MPRIS unavailable");
                return;
            }
        };

        {
            let on_command = Rc::clone(&on_command);
            player.connect_play(move |_| on_command(MprisCommand::Play));
        }
        {
            let on_command = Rc::clone(&on_command);
            player.connect_pause(move |_| on_command(MprisCommand::Pause));
        }
        {
            let on_command = Rc::clone(&on_command);
            player.connect_play_pause(move |_| on_command(MprisCommand::PlayPause));
        }
        {
            let on_command = Rc::clone(&on_command);
            player.connect_stop(move |_| on_command(MprisCommand::Stop));
        }
        {
            let on_command = Rc::clone(&on_command);
            player.connect_next(move |_| on_command(MprisCommand::Next));
        }
        {
            let on_command = Rc::clone(&on_command);
            player.connect_previous(move |_| on_command(MprisCommand::Previous));
        }

        tracing::info!("MPRIS server started");
        *service_set.player.borrow_mut() = Some(Rc::clone(&player));
        player.run().await;
    });

    service
}

impl MprisService {
    /// Push title/artist/album/art and playback status to MPRIS clients.
    pub fn update_track(
        &self,
        title: &str,
        artist: &str,
        album: &str,
        art_url: Option<&str>,
        length_us: i64,
        state: PlaybackState,
    ) {
        let Some(player) = self.player.borrow().clone() else {
            return;
        };
        let mut builder = Metadata::builder()
            .title(title.to_owned())
            .artist([artist.to_owned()])
            .album(album.to_owned());
        if length_us > 0 {
            builder = builder.length(Time::from_micros(length_us));
        }
        if let Some(url) = art_url {
            builder = builder.art_url(url.to_owned());
        }
        let meta = builder.build();
        let status = match state {
            PlaybackState::Playing => PlaybackStatus::Playing,
            PlaybackState::Paused => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        };
        glib::spawn_future_local(async move {
            let _ = player.set_metadata(meta).await;
            let _ = player.set_playback_status(status).await;
        });
    }
}
