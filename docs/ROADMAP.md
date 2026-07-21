# Cadence — roadmap

High-level history and direction. Day-to-day status: [TODO.md](TODO.md).  
Install / contribute: [SETUP.md](../SETUP.md) · [CONTRIBUTING.md](../CONTRIBUTING.md).  
Version history: [RELEASES.md](RELEASES.md).

## Direction

Stay a **small native music library**: local files, calm GNOME UI, honest playback chrome. Prefer finishing dead wiring and large-library readiness over becoming a streaming client or lyrics/visualiser suite.

## Themes

1. **Library honesty** — scan, search, and browsers that match what is on disk
2. **Playback polish** — dock + Now Playing + MPRIS that stay in sync
3. **Operator UX** — boring Flatpak installs, clear docs, predictable portals
4. **Project hygiene** — SemVer releases, contributor-friendly Rust layout, brand consistency

## Milestones

- **Milestone 1–2** — foundation (shell, SQLite, scanner, GStreamer, organise)
- **Milestone 3** — UX redesign (library-first, dock, Now Playing, home)
- **Milestone 4** — polish (dock height, context menus, docs)
- **v0.1.0** — first public beta — see [RELEASES.md](RELEASES.md)
- **v0.1.1** — Flatpak clean-install fix (GNOME 49) — see [RELEASES.md](RELEASES.md)
- **v0.1.2** — studio branding + docs polish — see [RELEASES.md](RELEASES.md)
- **Next** — items under **Next** in [TODO.md](TODO.md)

## Non-goals (for now)

- Lyrics, visualiser, scrobbling
- Chromecast / cloud sync
- Replacing a full DAW or tagger suite
