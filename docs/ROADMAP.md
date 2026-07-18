# Cadence — Development Roadmap

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

### Milestone 4 — Polish (this branch)
- Dock height locked; artwork cannot resize the shell
- Native context menus (`PopoverMenu`)
- Full audit vs brief; documentation brought in sync with reality

## Next — proposed Milestone 5 (finish dead wiring first)

Priority order for the next development branch:

1. **Artist portraits end-to-end** — call download from fill; show `image_path` + initials in Artists UI
2. **MPRIS honesty** — mirror pause/play status; wire seek or stop advertising it; same for Raise
3. **Playlist + queue UX** — delete/rename playlists; jump/remove in queue
4. **Metadata targeting** — context-menu Edit; optional per-track lookup that writes tags
5. **Search stubs** — Folder activation; playlist group; genre query that scales
6. **Dedicated album page** — replace “grid → flat songs”
7. **Genres browser** — first-class sidebar view
8. **Home recency** — Recent albums by real chronology
9. **Large-library lists** — virtualization before claiming 100k readiness
10. **Vinyl polish** — only if it stays optional and restrained

## Later

- Gapless playback / ReplayGain / crossfade (or drop from marketing copy permanently)
- Batch metadata editor
- Smart playlists / M3U import-export
- Mini player window
- Desktop notifications on track change
- “Open containing folder”
- Stronger Flatpak portal coverage (sandbox music dirs beyond `xdg-music`)

## Explicitly deferred

Lyrics, visualiser, scrobbling, Chromecast, cloud sync.
