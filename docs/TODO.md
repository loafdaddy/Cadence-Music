# Cadence — TODO

Single living status list. Keep it honest: only check items when verified in the running app.

Related: [ARCHITECTURE.md](ARCHITECTURE.md) · [ROADMAP.md](ROADMAP.md) · [INSTALL.md](INSTALL.md) · [RELEASES.md](RELEASES.md) · [CONTRIBUTING.md](../CONTRIBUTING.md)

---

## Done

### Shell and playback
- [x] GTK4 + libadwaita shell; library-first layout
- [x] Compact playback dock (fixed height; artwork cannot grow the bar)
- [x] Now Playing overlay (opt-in from dock artwork)
- [x] GStreamer play/pause/seek; queue with shuffle and repeat (Off / All / One)
- [x] Native context menus (queue / playlist / delete from library and disk)
- [x] Favourites on home, songs, search, and dock

### Library
- [x] Recursive folder scan + live folder watching
- [x] Scan Library in the app menu — adds new files, removes missing ones, toasts added/removed; Banner for progress
- [x] Orphan albums/artists pruned on track removal and after scan
- [x] SQLite library + FTS5 search (artists, albums, songs, genres, years, folders)
- [x] Library home (Continue, recent albums/tracks, stats)
- [x] Artists master-detail (albums + singles sections)
- [x] Albums cover grid (opens flat song list)
- [x] Songs browser with sort + load-more pagination
- [x] Playlists: create, add from context menu, open and play
- [x] Organise: Artist/Album, or Artist/Singles when album is missing — preview, apply, undo; empty dirs cleaned up
- [x] Single-track metadata dialog (menu / queue-current targeting)
- [x] Album artwork extract + local cache
- [x] MPRIS: play / pause / next / previous / stop (media keys)

### Docs hygiene
- [x] README and docs match implementation (no separate spec checklist)
- [x] CONTRIBUTING + INSTALL for Flatpak beta and from-source development
- [x] Cadence. wordmark in header; brand SVGs for GitHub README

---

## Partial / known gaps

These work enough to ship, but behaviour is incomplete or imprecise.

- [ ] Vinyl animation in Now Playing is coarse (optional; keep restrained)
- [ ] Home “Recent albums” uses highest album IDs, not real listen/add chronology
- [ ] Artist-detail track rows: favourite icon is display-only (not wired)
- [ ] Find Missing Metadata: album/CAA art pass works; MusicBrainz genre always unset; artist portraits never downloaded or shown
- [ ] Lookup progress is a header spinner (scan uses Banner)
- [ ] Search: no playlists group; folder click is toast-only; genre drill-down is client-side (capped)
- [ ] Edit Metadata does not target a right-clicked row (uses queue current / first context track); no context-menu Edit
- [ ] Albums: no dedicated album page (grid → flat songs only)
- [ ] Queue list is display-only (no jump / remove / reorder in UI)
- [ ] Playlists: no delete or rename UI; opened playlist rows omit artist/album labels
- [ ] MPRIS: dock pause does not update playback status; `can_seek` / `can_raise` advertised but unwired
- [ ] Preferences: colour-scheme row inert; folders are add-only (no remove)
- [ ] Flatpak: manifest exists; day-to-day is still `cargo run`; portals beyond `xdg-music` incomplete
- [ ] Large libraries: songs paginated; search / organise / genre paths can still load large sets
- [ ] Organise undo is in-memory last-apply only (lost on quit)

---

## Next (priority)

Finish dead wiring before new surfaces:

1. [ ] Artist portraits end-to-end — download from fill; show image/initials in Artists UI
2. [ ] MPRIS honesty — mirror pause/play; wire seek/Raise or stop advertising them
3. [ ] Playlist delete / rename; interactive queue (jump / remove)
4. [ ] Context-menu Edit Metadata; optional per-track lookup that writes tags
5. [ ] Search: Folder activation; playlists group; genre query that scales
6. [ ] Dedicated album page
7. [ ] Genres sidebar browser (then Years / Folders if useful)
8. [ ] Home Recent albums by real chronology
9. [ ] List virtualization before claiming 100k readiness
10. [ ] Vinyl polish only if it stays optional and restrained

---

## Later

- [ ] Batch metadata editor
- [ ] Gapless playback / ReplayGain / crossfade
- [ ] Smart playlists / M3U import-export
- [ ] Mini player window
- [ ] Track-change notifications
- [ ] Open containing folder / drag-and-drop
- [ ] Persist organise undo across sessions
- [ ] Flatpak verified end-to-end on a clean user install; portals beyond `xdg-music`
- [ ] MPRIS Raise / desktop entry activation
- [ ] Publish to Flathub (after beta stabilises)

---

## Explicitly deferred

Lyrics, visualiser, scrobbling, Chromecast, cloud sync.

---

## Dead wiring (do not claim done)

Code exists but is unused or incomplete:

- Portrait pipeline: `artists_missing_image`, `download_artist_image`, `set_artist_image` — fill never calls them; UI never reads `image_path`
- Per-track `lookup_and_fill` — never called from UI; would not write tags if called
- `delete_playlist`, `remove_folder` — no prefs/UI entry points
- Queue `jump_to` / `remove` — unused by UI
- Search → Folder activation — toast only
