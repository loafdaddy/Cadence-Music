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
      │     ├─ Banner (scan progress)
      │     ├─ Nav | [Master] | Detail stack
      │     └─ Playback dock (fixed height)
      └─ Now Playing revealer (overlay, opt-in)
```

- **Library first:** the detail stack is the primary surface.
- **Dock:** permanent compact player (~96px). Artwork is clipped and scaled; it must never change dock height.
- **Now Playing:** immersive overlay revealed from the dock artwork; not the default chrome.
- **App menu:** Preferences, Scan Library, Organise Library, Edit / Lookup Metadata, Undo Organisation, About, Quit.
- **Header brand:** app icon + wordmark **Cadence.** (serif stack, purple period).
- **Home actions:** Organise Files and Find Missing Metadata (Scan lives in the menu only).
- **Lookup progress:** header spinner + tooltip (not the scan Banner).

## Library scan

- Startup and menu **Scan Library** both call `LibraryService::scan_all`.
- Each folder pass discovers files on disk, removes DB tracks whose files are gone, upserts new/changed files, then `prune_orphans` (empty albums + unreferenced artists).
- `LibraryEvent::ScanFinished` carries a `ScanSummary` (`added` / `removed` / `updated`); the UI toasts only when added or removed is non-zero.
- Live folder watching still handles incremental upsert / remove while the app is open.

## Organisation

- Single layout (`Preset::ArtistAlbum`): tracks with an album go under `Artist/Album/…`; without go under `Artist/Singles/…`.
- Preview builds an `OrganizationPlan` (no disk writes); Apply executes renames and returns an in-memory `UndoLog`.
- After each move (and on undo), empty parent directories are removed so leftover Album/Singles folders do not accumulate.

## Playback

- `Player` wraps GStreamer `playbin` (audio only; video sink discarded).
- `Queue` holds ordered tracks plus shuffle / repeat mode.
- Dock and Now Playing share the same player/queue; they are two views, not two engines.
- MPRIS exposes play/pause/next/previous/stop; status updates on track start are incomplete for dock pause (see [TODO.md](TODO.md)).

## Artwork

- Extracted embeds and downloaded covers live under the app cache (`cadence_core::paths`).
- UI uses `artwork_frame` + `set_artwork_file(picture, path, size)` so textures are loaded at display size and natural size cannot blow out parents.
- Artist portrait download helpers exist in core but are not wired into fill or the Artists UI yet.

## Flatpak

- Manifest: `build-aux/org.cadence.Cadence.yml`
- Local beta install: `./scripts/build-flatpak.sh` then `flatpak run org.cadence.Cadence`
- Runtime: GNOME 48; owns `org.mpris.MediaPlayer2.Cadence`
- Default music access: `xdg-music:rw`; other locations rely on document portal grants when chosen via `FileDialog`
- Finish-args also declare FileChooser, Documents, Notification, and OpenURI portals (Notification unused so far)
- Not on Flathub yet — see [INSTALL.md](INSTALL.md)
