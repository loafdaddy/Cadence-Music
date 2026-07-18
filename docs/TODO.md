# Cadence — TODO

Living list aligned with [SPEC_CHECKLIST.md](SPEC_CHECKLIST.md) and [ROADMAP.md](ROADMAP.md).
Check items off only when behaviour is verified in the running app, not when a stub lands.

## Polish / correctness

- [x] Playback dock fixed height (artwork must not grow the bar)
- [x] Context menu native Adwaita styling
- [x] Spec checklist matches implementation
- [ ] Artist portraits / initials visible in Artists UI
- [ ] MPRIS seek handler (or set `can_seek` false)
- [ ] Playlist delete / rename in UI
- [ ] Preferences Playback page: remove or implement placeholder rows

## Features (next milestones)

- [ ] Album detail view
- [ ] Genres sidebar browser
- [ ] Years / Folders sidebar browsers (optional after Genres)
- [ ] Search: include playlists group
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
