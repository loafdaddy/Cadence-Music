# Cadence — Specification Checklist (Milestone 4)

Compared against the original project brief and the codebase after the Milestone 4 polish pass
(`feature/milestone-4-player-polish`).

Status: ✅ Complete · 🚧 Partial · ❌ Missing

## Design principle

Library first. Player as a compact dock. Immersive listening is opt-in via Now Playing.

## UX / shell

| Item | Status | Notes |
|------|--------|-------|
| Compact playback dock (~90–120px) | ✅ | Fixed height (96px). Artwork loads at display size; wrapper does not propagate `Picture` natural size. CSS `max-height` clamp. |
| Title / artist / album / favourite in dock | ✅ | Labels ellipsize; layout stable across tracks |
| Click artwork → Now Playing overlay | ✅ | Slide-up revealer |
| Optional vinyl animation in Now Playing | 🚧 | Disc spin + tonearm cue; toggleable; still coarse |
| Library home full width | ✅ | Master column hidden when unused |
| Continue / Recent albums / Recently added / stats | ✅ | |
| Artist detail: artwork on album header only | ✅ | Tracks are title + duration |
| Organise preview / apply / undo | ✅ | DB-backed preview; undo via menu |
| Library-wide Find Missing Metadata | 🚧 | Album-oriented pass with progress banner; not a full per-track fill |
| Artist portraits from lookup | 🚧 | MB → Wikidata → Commons download + DB `image_path`; **UI does not show portraits or initials yet** |
| Global grouped search | 🚧 | Artists / Albums / Songs / Genres / Years / Folders; playlists not included |
| Lookup progress UI | ✅ | Banner / spinner phase + done/total |
| Favourite toggle on song rows + dock | ✅ | |
| Context menus (queue / playlist / delete) | ✅ | Native `PopoverMenu` + `gio::Menu` (Milestone 4) |

## Core philosophy & stack

| Item | Status | Notes |
|------|--------|-------|
| GTK4 + libadwaita + Rust | ✅ | |
| Flatpak-first | 🚧 | Manifest + finish-args present; local `cargo run` still primary for day-to-day |
| Offline-first | ✅ | Network only for optional lookup |
| 100k library performance | 🚧 | Songs paginated; lists are not virtualized |

## Library / organisation / metadata

| Item | Status | Notes |
|------|--------|-------|
| Folder scan + watch | ✅ | Debounced `notify` watcher; watches new folders on add |
| Tag read (lofty) | ✅ | |
| Organise preview + apply + undo | ✅ | |
| Single-track metadata edit | ✅ | Menu → Edit Metadata (current track) |
| Batch metadata edit UI | ❌ | |
| MusicBrainz / CAA | 🚧 | Per-track helper + limited library pass; rate-limited |

## Browsing / playback / playlists

| Item | Status | Notes |
|------|--------|-------|
| Artists master-detail | ✅ | List + album sections / singles |
| Albums browser | 🚧 | Cover grid → song list (no dedicated album page) |
| Songs browser | ✅ | Sort + load-more pagination |
| Genres / Years / Folders views | ❌ | Appear in search only; no sidebar destinations |
| Queue / shuffle / repeat | ✅ | Off / All / One |
| Gapless / ReplayGain / crossfade | ❌ | Preferences copy acknowledges later |
| Manual playlists | 🚧 | Create, add-from-context, play work; **no delete/rename UI** (service APIs exist) |
| Smart playlists / M3U | ❌ | |
| Mini player window | ❌ | |

## Linux integration

| Item | Status | Notes |
|------|--------|-------|
| MPRIS basics | 🚧 | Play / pause / next / previous + metadata; `can_seek` advertised but **seek not wired**; no Raise |
| Notifications | ❌ | Portal talked in Flatpak; unused |
| Open folder / DnD | ❌ | |
| Flatpak portals | 🚧 | FileChooser used via `gtk::FileDialog`; Documents / Notification / OpenURI declared only |

## Previously broken (and current status)

| Feature | Root cause | Status |
|---------|------------|--------|
| Giant player / growing dock | `gtk::Picture` preferred size = full image; `size_request` is a minimum only | **Re-fixed in M4** — scaled textures + non-propagating clip + dock `max-height` |
| Context menu item border | Custom `gtk::Button`s inside `Popover` fought Adwaita | **Fixed in M4** — `PopoverMenu` |
| Library left void | Master pane always in layout | Fixed (M3) |
| Organise empty preview | Disk `read_metadata` dropped tracks | Fixed (M3) |
| Lookup felt dead | No progress UI | Fixed (M3) |

## Audit notes (Milestone 4)

Verified by reading wiring in `window.rs`, UI modules, playback, MPRIS, core library worker, and Flatpak manifest — not by assuming a control implies a finished feature.

### Over-claimed before this pass
- Artist portraits marked ✅ while the app crate never reads `image_path`.
- Dock marked fully fixed while high-res covers could still grow the bar.
- README listed capabilities (e.g. MPRIS, Flatpak) without noting partial depth.

### Unused / incomplete service surface
- `delete_playlist`, `rescan_path`, `lookup_and_fill` (direct) exist on `LibraryService` but lack full UI paths.
- Preferences Playback page is a placeholder (colour scheme row non-interactive).

## Do not add yet

Lyrics, visualiser, scrobbling, Chromecast — keep the foundation calm and handcrafted.
