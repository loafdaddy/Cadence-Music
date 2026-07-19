# Cadence

A modern, lightweight, native music library for Linux.

Cadence aims to feel like it ships with Fedora Workstation: **GTK4**, **libadwaita**, Wayland-first, Flatpak-first, offline-first — no Electron, no embedded browser.

> Windows Media Player (Windows 7 era) + GNOME HIG + modern design + native Linux performance.

## Features (current)

- Recursive library scanning with live folder watching (menu **Scan Library** adds new tracks, removes missing ones, and toasts the delta)
- SQLite-backed library with FTS5 search (grouped results: artists, albums, songs, genres, years, folders)
- Library home, Artists, Albums, Songs, Playlists, Favourites, Recently Added
- Compact playback dock + optional Now Playing overlay
- GStreamer playback with queue, shuffle, and repeat
- Album artwork extraction and local cache
- Single-track metadata editing; optional MusicBrainz / Cover Art Archive lookup (library-wide pass is partial)
- Non-destructive library organisation: **Artist / Album**, or **Artist / Singles** when album is missing — preview, apply, and undo (empty folders cleaned up)
- MPRIS media keys (play/pause/next/previous; seek not yet wired)
- Flatpak packaging (manifest present; polish ongoing)

### Not yet

Batch metadata editor, Genres/Years/Folders sidebar browsers, gapless / ReplayGain / crossfade, smart playlists, M3U, mini player, notifications, open-folder.

See [docs/SPEC_CHECKLIST.md](docs/SPEC_CHECKLIST.md) for the full status matrix and [docs/ROADMAP.md](docs/ROADMAP.md) for what’s next.

## Build

### Dependencies (Fedora)

```bash
sudo dnf install gtk4-devel libadwaita-devel \
  gstreamer1-devel gstreamer1-plugins-base-devel \
  gstreamer1-plugins-good gstreamer1-plugins-bad-free \
  rust cargo
```

### Run

```bash
cargo run -p cadence
```

If you use the local `.deps` prefix for headers/libs, source `.envrc.build` (or run `./scripts/run-debug.sh` after `cargo build -p cadence`) before launching.

### Test

```bash
cargo test -p cadence-core
cargo fmt
cargo clippy -p cadence-core -p cadence -- -D warnings
```

## Flatpak

```bash
flatpak-builder --user --install --force-clean build-dir \
  build-aux/org.cadence.Cadence.yml
flatpak run org.cadence.Cadence
```

## Architecture

| Crate | Role |
|-------|------|
| `cadence-core` | SQLite library, scanner, metadata, artwork, organization, MusicBrainz lookup |
| `cadence` | GTK4 / libadwaita UI, GStreamer playback, MPRIS |

The UI never blocks on disk I/O: a dedicated library worker owns the database and talks to the main loop through channels.

More detail: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
