# Cadence — Specification Checklist (Milestone 3)

Compared against the original project brief and the current codebase after the UX redesign.
Status: ✅ Complete · 🚧 Partial · ❌ Missing

## Design principle

Library first. Player as a compact dock. Immersive listening is opt-in via Now Playing.

## UX / shell (Milestone 3 focus)

| Item | Status | Notes |
|------|--------|-------|
| Compact playback dock (~90–120px) | ✅ | Fixed `Picture` natural-size blow-up (`can_shrink`) |
| Title / artist / album / favourite in dock | ✅ | |
| Click artwork → Now Playing overlay | ✅ | Slide-up revealer |
| Optional vinyl animation in Now Playing | 🚧 | Disc spin + tonearm cue; toggleable; refine later |
| Library home full width | ✅ | Master column hidden when unused |
| Continue / Recent albums / Recently added / stats | ✅ | |
| Artist detail: artwork on album header only | ✅ | Tracks are title + duration |
| Organise preview functional | ✅ | Was empty because disk `read_metadata` dropped all tracks; now DB-backed + auto-preview |
| Library-wide Find Missing Metadata | 🚧 | Scans albums, rate-limited MB lookups, summary toast; artist photos not yet |
| Artist portraits from lookup | ✅ | MB → Wikidata → Commons; initials fallback; schema v2 |
| Global grouped search | 🚧 | Artists / Albums / Songs / Genres / Years / Folders; playlists later |
| Lookup progress UI | ✅ | Banner phase + done/total during library fill |
| Favourite toggle on song rows | ✅ | Toggle in lists + dock |

## Core philosophy & stack

| Item | Status |
|------|--------|
| GTK4 + libadwaita + Rust | ✅ |
| Flatpak-first | 🚧 |
| Offline-first | ✅ |
| 100k library performance | 🚧 |

## Library / organisation / metadata

| Item | Status | Notes |
|------|--------|-------|
| Folder scan + watch | ✅ | |
| Tag read (lofty) | ✅ | Unknown Artist only when tags truly missing |
| Organise preview + apply + undo | ✅ | Multi-root path selection by longest folder prefix |
| Single-track metadata edit | ✅ | |
| Batch metadata edit UI | ❌ | |
| MusicBrainz / CAA | 🚧 | Per-track + limited library pass |

## Browsing / playback / playlists

| Item | Status |
|------|--------|
| Artists / Albums / Songs master-detail | ✅ / 🚧 |
| Genres / Years / Folders views | ❌ |
| Queue / shuffle / repeat | ✅ |
| Gapless / ReplayGain / crossfade | ❌ |
| Manual playlists | ✅ |
| Smart playlists / M3U | ❌ |
| Mini player window | ❌ |

## Linux integration

| Item | Status |
|------|--------|
| MPRIS basics | 🚧 |
| Notifications / DnD / open folder | ❌ |
| Flatpak portals | 🚧 |

## Previously “implemented but broken”

| Feature | Root cause | Fix |
|---------|------------|-----|
| Giant player | `gtk::Picture` `can_shrink(false)` expanded to image natural size | Compact dock + shrink + CSS max size |
| Library left void | Master pane + separator always in layout | Hide master pane/sep except Artists |
| Organise empty preview | `read_metadata().ok()?` dropped every track | Build plan from DB metadata; auto-preview on open |
| Lookup felt dead | No progress; required open list | Toasts + library-wide pass from home |

## Recommended next (before more features)

1. Artist photo download + cache (keep initials fallback)
2. Progress UI for long metadata/organise jobs (`adw::Banner` or dialog)
3. Favourite toggle on song rows (dock toggle exists)
4. Genres browse + search groups
5. Gapless playback or remove from README claims
6. Polish vinyl (shared-element / proper tonearm drawing)

## Do not add yet

Lyrics, visualiser, scrobbling, Chromecast — keep the foundation calm and handcrafted.
