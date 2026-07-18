# Cadence — TODO

Living list aligned with [SPEC_CHECKLIST.md](SPEC_CHECKLIST.md) and [ROADMAP.md](ROADMAP.md).
Check items off only when behaviour is verified in the running app, not when a stub lands.

## Polish / correctness

- [x] Playback dock fixed height (artwork must not grow the bar)
- [x] Context menu native Adwaita styling
- [x] Spec checklist matches implementation (post deep audit)
- [ ] Wire artist portrait download into library fill + show portraits/initials in Artists UI
- [ ] MPRIS: update status on pause/play; seek handler or clear `can_seek`; Raise or clear `can_raise`
- [ ] Playlist delete / rename in UI
- [ ] Interactive queue (jump to track; remove)
- [ ] Edit Metadata targets the intended track (context menu action)
- [ ] Search → Folder opens a real browse/filter (not toast)
- [ ] Recent albums ordered by real recency
- [ ] Preferences: implement or remove placeholder Playback rows; allow removing folders

## Features (next milestones)

- [ ] Album detail view
- [ ] Genres sidebar browser
- [ ] Years / Folders sidebar browsers (optional after Genres)
- [ ] Search: include playlists group
- [ ] Per-track MusicBrainz lookup that writes tags
- [ ] Batch metadata editor
- [ ] Gapless playback
- [ ] ReplayGain
- [ ] Crossfade
- [ ] Smart playlists
- [ ] M3U import / export
- [ ] Mini player window
- [ ] Track-change notifications
- [ ] Open folder in file manager
- [ ] List virtualization for large libraries

## Packaging / Linux

- [ ] Flatpak build verified end-to-end on a clean user install
- [ ] Portal-backed folder grants beyond `xdg-music`
- [ ] MPRIS Raise / desktop entry activation

## Docs hygiene

- [x] README feature list honest about partial items
- [x] Roadmap + TODO + architecture notes
- [ ] Keep checklist updated at the end of every milestone branch
