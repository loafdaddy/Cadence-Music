# Cadence — Development Roadmap

High-level history and direction. Day-to-day status lives in [TODO.md](TODO.md).

## Done

### Milestone 1–2 — Foundation
- GTK4 + libadwaita shell, SQLite library, scanner + watcher
- GStreamer playback, queue, shuffle, repeat
- Tag read/write, organise with preview/undo
- Basic Artists / Albums / Songs / Playlists / Favourites

### Milestone 3 — UX redesign
- Library-first layout; compact playback dock; Now Playing overlay
- Library home (Continue, recent albums/tracks, stats)
- Grouped search; lookup progress; favourite toggles
- Artist detail without per-track cover spam

### Milestone 4 — Polish
- Dock height locked; artwork cannot resize the shell
- Native context menus (`PopoverMenu`)
- Documentation brought in sync with reality

### Library scan and organisation
- Scan Library in the app menu (removed from home)
- Rescan reconciles disk and DB: add, remove, toast delta; prune orphan artists/albums
- Single organise layout: Artist/Album, or Artist/Singles when album is missing
- Empty directories pruned on organise apply/undo

## Next

Proposed Milestone 5: finish dead wiring first (see [TODO.md](TODO.md) “Next”).

Then expand browsers (album page, Genres), search stubs, and large-library lists.

## Later

Gapless / ReplayGain / crossfade, batch metadata, smart playlists / M3U, mini player,
notifications, open-folder, stronger Flatpak portals, persisted organise undo.

## Explicitly deferred

Lyrics, visualiser, scrobbling, Chromecast, cloud sync.
