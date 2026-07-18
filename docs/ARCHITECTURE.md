# Cadence — Architecture

## Crates

| Crate | Path | Role |
|-------|------|------|
| `cadence-core` | `crates/core` | SQLite library DB, scanner, folder watcher, tag I/O (lofty), artwork cache, organise planner, MusicBrainz / CAA lookup |
| `cadence` | `crates/app` | GTK4 / libadwaita UI, GStreamer playback, queue, MPRIS |

## UI threading model

- The GTK main context owns all widgets and the GStreamer bus watch.
- A dedicated **library worker** thread owns the SQLite connection.
- `LibraryService` sends commands over an MPSC channel; results and `LibraryEvent`s return to the UI via glib-idle friendly replies.
- The UI must not open the database or perform blocking disk/network I/O on the main thread.

## Shell layout

```
ApplicationWindow
└─ ToastOverlay
   └─ Overlay
      ├─ ToolbarView
      │  ├─ HeaderBar (search, add folder, menu)
      │  └─ Library shell
      │     ├─ Banner (scan / lookup)
      │     ├─ Nav | [Master] | Detail stack
      │     └─ Playback dock (fixed height)
      └─ Now Playing revealer (overlay, opt-in)
```

- **Library first:** the detail stack is the primary surface.
- **Dock:** permanent compact player (~96px). Artwork is clipped and scaled; it must never change dock height.
- **Now Playing:** immersive overlay revealed from the dock artwork; not the default chrome.

## Playback

- `Player` wraps GStreamer `playbin` (audio only; video sink discarded).
- `Queue` holds ordered tracks plus shuffle / repeat mode.
- Dock and Now Playing share the same player/queue; they are two views, not two engines.

## Artwork

- Extracted embeds and downloaded covers live under the app cache (`cadence_core::paths`).
- UI uses `artwork_frame` + `set_artwork_file(picture, path, size)` so textures are loaded at display size and natural size cannot blow out parents.

## Flatpak

- Manifest: `build-aux/org.cadence.Cadence.yml`
- Runtime: GNOME 48; owns `org.mpris.MediaPlayer2.Cadence`
- Default music access: `xdg-music:rw`; other locations rely on document portal grants when chosen via `FileDialog`
