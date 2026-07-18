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

## Next — proposed Milestone 5 (depth, not sprawl)

Priority order for the next development branch:

1. **Artist portraits in UI** — show cached `image_path` (and initials fallback) on the Artists list / detail header
2. **MPRIS seek + position** — wire seek; stop advertising capabilities we do not implement
3. **Playlist management** — delete / rename in UI (APIs already exist)
4. **Dedicated album page** — replace “grid → flat songs” with a real album detail
5. **Genres browser** — first-class sidebar view (search group already exists)
6. **Vinyl polish** — tonearm / shared-element motion only if it stays optional
7. **Large-library lists** — virtualization or stricter paging before claiming 100k readiness

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
