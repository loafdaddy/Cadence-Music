//! Thin wrapper around a GStreamer `playbin` element.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use gstreamer as gst;
use gstreamer::prelude::*;
use gtk::glib;
use gtk::glib::ControlFlow;

/// High-level playback state for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
    Buffering,
}

/// Events emitted by the player on the glib main context.
#[derive(Debug, Clone)]
pub enum PlayerEvent {
    StateChanged(PlaybackState),
    PositionUpdated { position_ns: u64, duration_ns: u64 },
    EndOfStream,
    Error(String),
}

/// Audio player backed by GStreamer playbin.
pub struct Player {
    pipeline: gst::Element,
    volume: Rc<RefCell<f64>>,
    /// Kept alive so the bus watch remains registered.
    _bus_watch: gst::bus::BusWatchGuard,
}

impl std::fmt::Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player").finish_non_exhaustive()
    }
}

impl Player {
    /// Create a player and forward bus messages through `on_event`.
    pub fn new(on_event: impl Fn(PlayerEvent) + 'static) -> Self {
        let pipeline = gst::ElementFactory::make("playbin")
            .name("cadence-playbin")
            .build()
            .expect("GStreamer playbin element is required");

        // Audio-only: discard video to keep the pipeline light.
        if let Some(fakesink) = gst::ElementFactory::make("fakesink").build().ok() {
            pipeline.set_property("video-sink", &fakesink);
        }

        let bus = pipeline.bus().expect("playbin has a bus");
        let on_event = Rc::new(on_event);
        let bus_watch = bus
            .add_watch_local({
                let on_event = Rc::clone(&on_event);
                let pipeline = pipeline.clone();
                move |_, msg| {
                    use gst::MessageView;
                    match msg.view() {
                        MessageView::Eos(..) => on_event(PlayerEvent::EndOfStream),
                        MessageView::Error(err) => {
                            on_event(PlayerEvent::Error(err.error().to_string()));
                        }
                        MessageView::StateChanged(state) => {
                            if state.src() == Some(pipeline.upcast_ref()) {
                                let mapped = match state.current() {
                                    gst::State::Playing => PlaybackState::Playing,
                                    gst::State::Paused => PlaybackState::Paused,
                                    gst::State::Ready | gst::State::Null => PlaybackState::Stopped,
                                    gst::State::VoidPending => return ControlFlow::Continue,
                                };
                                on_event(PlayerEvent::StateChanged(mapped));
                            }
                        }
                        MessageView::Buffering(buf) => {
                            if buf.percent() < 100 {
                                on_event(PlayerEvent::StateChanged(PlaybackState::Buffering));
                            }
                        }
                        _ => {}
                    }
                    ControlFlow::Continue
                }
            })
            .expect("failed to attach bus watch");

        Self {
            pipeline,
            volume: Rc::new(RefCell::new(1.0)),
            _bus_watch: bus_watch,
        }
    }

    pub fn set_uri_from_path(&self, path: &Path) {
        let uri = glib::filename_to_uri(path, None)
            .unwrap_or_else(|_| glib::GString::from(format!("file://{}", path.display())));
        let _ = self.pipeline.set_state(gst::State::Null);
        self.pipeline.set_property("uri", uri.as_str());
    }

    pub fn play(&self) {
        let _ = self.pipeline.set_state(gst::State::Playing);
    }

    pub fn pause(&self) {
        let _ = self.pipeline.set_state(gst::State::Paused);
    }

    pub fn stop(&self) {
        let _ = self.pipeline.set_state(gst::State::Ready);
    }

    pub fn toggle(&self) {
        match self.state() {
            PlaybackState::Playing => self.pause(),
            _ => self.play(),
        }
    }

    pub fn seek_fraction(&self, fraction: f64) {
        let Some(duration) = self.pipeline.query_duration::<gst::ClockTime>() else {
            return;
        };
        let target = duration
            .nseconds()
            .saturating_mul((fraction.clamp(0.0, 1.0) * 1_000_000.0) as u64)
            / 1_000_000;
        let _ = self.pipeline.seek_simple(
            gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
            gst::ClockTime::from_nseconds(target),
        );
    }

    pub fn set_volume(&self, volume: f64) {
        let v = volume.clamp(0.0, 1.0);
        *self.volume.borrow_mut() = v;
        self.pipeline.set_property("volume", v);
    }

    #[must_use]
    pub fn volume(&self) -> f64 {
        *self.volume.borrow()
    }

    #[must_use]
    pub fn position_ns(&self) -> u64 {
        self.pipeline
            .query_position::<gst::ClockTime>()
            .map(|t| t.nseconds())
            .unwrap_or(0)
    }

    #[must_use]
    pub fn duration_ns(&self) -> u64 {
        self.pipeline
            .query_duration::<gst::ClockTime>()
            .map(|t| t.nseconds())
            .unwrap_or(0)
    }

    #[must_use]
    pub fn state(&self) -> PlaybackState {
        let (_, current, _) = self.pipeline.state(gst::ClockTime::ZERO);
        match current {
            gst::State::Playing => PlaybackState::Playing,
            gst::State::Paused => PlaybackState::Paused,
            _ => PlaybackState::Stopped,
        }
    }
}
