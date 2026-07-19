# Cadence

A modern, lightweight, native music library for Linux.

Cadence aims to feel like it ships with Fedora Workstation: **GTK4**, **libadwaita**, Wayland-first, Flatpak-first, offline-first — no Electron, no embedded browser.

> Windows Media Player (Windows 7 era) + GNOME HIG + modern design + native Linux performance.

## Features (current)

- Recursive library scanning with live folder watching (menu **Scan Library** adds new tracks, removes missing ones, and toasts the delta)
- SQLite-backed library with FTS5 search (grouped: artists, albums, songs, genres, years, folders)
- Library home, Artists, Albums, Songs, Playlists, Favourites, Recently Added
- Compact playback dock + optional Now Playing overlay
- GStreamer playback with queue, shuffle, and repeat
- Album artwork extraction and local cache
- Single-track metadata editing; optional MusicBrainz / Cover Art Archive lookup (library-wide album pass; portraits not wired)
- Non-destructive library organisation: **Artist / Album**, or **Artist / Singles** when album is missing — preview, apply, and undo
- MPRIS media keys (play/pause/next/previous; seek and Raise not wired)
- Flatpak packaging (manifest present; polish ongoing)

### Limitations (honest)

- Queue UI is display-only; playlists have no delete/rename UI
- Edit Metadata targets the current/queue track, not a right-clicked row
- No Genres/Years/Folders sidebar browsers, batch metadata editor, gapless/ReplayGain, smart playlists, M3U, mini player, or notifications yet

Status and priorities: [docs/TODO.md](docs/TODO.md). Direction: [docs/ROADMAP.md](docs/ROADMAP.md).

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
