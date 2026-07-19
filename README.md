# Cadence.

<p align="center">
  <img src="data/brand/cadence-lockup.svg" alt="Cadence." width="420"/>
</p>

<p align="center">
  <strong>A modern, native music library for Linux</strong><br/>
  GTK4 · libadwaita · offline-first · early public beta
</p>

<p align="center">
  <a href="https://github.com/loafdaddy/Cadence-Music/releases/tag/v0.1.0">v0.1.0</a>
  ·
  <a href="docs/RELEASES.md">Release history</a>
  ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

Cadence aims to feel like it ships with Fedora Workstation: Wayland-first, Flatpak-friendly, no Electron.

This is an **early build** (**v0.1.0**). Features work, but expect rough edges. **Contributors are very welcome** — design, Rust, packaging, docs, and bug reports all help.

> Windows Media Player (Windows 7 era) + GNOME HIG + modern design + native Linux performance.

## AI disclaimer

Parts of Cadence — including code, docs, branding, and packaging — have been written or edited with **AI assistance** (for example Cursor and similar tools). That is intentional for an early project moving quickly.

**AI-assisted contributions are welcome.** Use Cursor, Copilot, ChatGPT, Claude, or any other assistant if it helps you. You remain responsible for what you submit: understand the change, keep pull requests focused, and verify what you can.

Full expectations: [CONTRIBUTING.md — AI-assisted contributions](CONTRIBUTING.md#ai-assisted-contributions).

## Try it

### Flatpak (beta)

Easiest way to install and try Cadence without touching your system Rust toolchain:

```bash
# Once: install Flatpak builders (Fedora)
sudo dnf install flatpak-builder flatpak
flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --user -y org.gnome.Platform//48 org.gnome.Sdk//48 \
  org.freedesktop.Sdk.Extension.rust-stable//24.08

# From a clone of this repo
./scripts/build-flatpak.sh
flatpak run org.cadence.Cadence
```

Details: [docs/INSTALL.md](docs/INSTALL.md).

### From source (for development)

```bash
git clone https://github.com/loafdaddy/Cadence-Music.git
cd Cadence-Music

# Fedora dependencies
sudo dnf install gtk4-devel libadwaita-devel \
  gstreamer1-devel gstreamer1-plugins-base-devel \
  gstreamer1-plugins-good gstreamer1-plugins-bad-free \
  rust cargo

cargo run -p cadence
```

If you use the local `.deps` prefix for headers/libs, source `.envrc.build` (or run `./scripts/run-debug.sh` after `cargo build -p cadence`) before launching.

Full contributor workflow: [CONTRIBUTING.md](CONTRIBUTING.md) · [docs/INSTALL.md](docs/INSTALL.md).

## What works today

- Recursive library scan + live folder watching; menu **Scan Library** adds/removes and toasts the delta
- SQLite + FTS5 search (artists, albums, songs, genres, years, folders)
- Library home, Artists, Albums, Songs, Playlists, Favourites, Recently Added
- Compact playback dock + optional Now Playing overlay
- GStreamer playback with queue, shuffle, and repeat
- Album artwork cache; organise as **Artist / Album** or **Artist / Singles**
- MPRIS media keys (play/pause/next/previous)

## Known limitations

Queue UI is display-only; playlists lack delete/rename; metadata edit targeting is weak; no Genres sidebar, batch editor, gapless/ReplayGain, or notifications yet.

Living status list: [docs/TODO.md](docs/TODO.md). Direction: [docs/ROADMAP.md](docs/ROADMAP.md). Architecture: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Releases: [docs/RELEASES.md](docs/RELEASES.md).

## Contributing

We want help. Good first steps:

1. Read [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/TODO.md](docs/TODO.md)
2. Open an issue for bugs or ideas, or pick an item from TODO
3. Fork, branch from `main`, open a PR

No contribution is too small — docs and Flatpak testing count. AI-assisted PRs are fine; see the [AI disclaimer](#ai-disclaimer) above.

## Architecture

| Crate | Role |
|-------|------|
| `cadence-core` | SQLite library, scanner, metadata, artwork, organisation, MusicBrainz lookup |
| `cadence` | GTK4 / libadwaita UI, GStreamer playback, MPRIS |

The UI never blocks on disk I/O: a dedicated library worker owns the database.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
