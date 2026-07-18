# Cadence

A modern, lightweight, native music library for Linux.

Cadence aims to feel like it ships with Fedora Workstation: **GTK4**, **libadwaita**, Wayland-first, Flatpak-first, offline-first — no Electron, no embedded browser.

> Windows Media Player (Windows 7 era) + GNOME HIG + modern design + native Linux performance.

## Features (MVP)

- Recursive library scanning with live folder watching
- SQLite-backed library with instant FTS5 search
- Library home, Artists, Albums, Songs, Playlists, Favourites, Recently Added
- GStreamer playback, queue, shuffle, repeat
- Album artwork extraction and local cache
- Metadata editing and optional MusicBrainz / Cover Art Archive lookup
- Non-destructive library organisation with preview and undo
- MPRIS media-key integration
- Flatpak packaging

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

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
