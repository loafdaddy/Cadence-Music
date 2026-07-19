# Cadence — Development Roadmap

High-level history and direction. Day-to-day status: [TODO.md](TODO.md).
Install / contribute: [INSTALL.md](INSTALL.md) · [CONTRIBUTING.md](../CONTRIBUTING.md).

## Done

### Milestone 1–2 — Foundation
- GTK4 + libadwaita shell, SQLite library, scanner + watcher
- GStreamer playback, queue, shuffle, repeat
- Tag read/write, organise with preview/undo
- Basic Artists / Albums / Songs / Playlists / Favourites

### Milestone 3 — UX redesign
- Library-first layout; compact playback dock; Now Playing overlay
- Library home; grouped search; lookup progress; favourites

### Milestone 4 — Polish
- Dock height locked; native context menus; docs synced with reality

### Library scan and organisation
- Scan in app menu; disk/DB reconcile; Artist/Album or Singles; orphan prune

### Branding and first Flatpak beta
- Cadence. wordmark + app icon (dark / purple accent)
- Contributor docs; INSTALL from Flatpak and from source
- `scripts/build-flatpak.sh` for local beta installs

## Next

Finish dead wiring (see TODO “Next”), then browsers and large-library work.
Get Flatpak verified on clean installs; Flathub later if the beta holds up.

## Later

Gapless / ReplayGain, batch metadata, smart playlists / M3U, mini player,
notifications, open-folder, stronger portals, persisted organise undo.

## Explicitly deferred

Lyrics, visualiser, scrobbling, Chromecast, cloud sync.
