# Cadence — Specification Checklist

Compared against the original project brief and the codebase on `master`
(post Milestone 4 polish, plus library scan / organisation updates).

Status: ✅ Complete · 🚧 Partial · ❌ Missing

## Design principle

Library first. Player as a compact dock. Immersive listening is opt-in via Now Playing.

## UX / shell

| Item | Status | Notes |
|------|--------|-------|
| Compact playback dock (~90–120px) | ✅ | Fixed height (96px). Artwork loads at display size; wrapper does not propagate `Picture` natural size. CSS `max-height` clamp. |
| Title / artist / album / favourite in dock | ✅ | Labels ellipsize; layout stable across tracks |
| Click artwork → Now Playing overlay | ✅ | Slide-up revealer; `can_target` gated when closed |
| Optional vinyl animation in Now Playing | 🚧 | Disc spin + tonearm cue; toggleable; still coarse |
| Library home full width | ✅ | Master column hidden when unused |
| Continue / Recent albums / Recently added / stats | 🚧 | Continue + recently added OK; “Recent albums” is highest album IDs, not listen/add chronology |
| Artist detail: artwork on album header only | ✅ | Tracks are title + duration; fav icon on those rows is display-only |
| Scan Library (menu) | ✅ | App menu action; reconciles disk ↔ DB (add new, remove missing); toast shows added/removed counts; Banner for progress |
| Organise preview / apply / undo | ✅ | Single layout: Artist/Album or Artist/Singles; empty dirs pruned on apply/undo; undo is last-apply in memory (lost on quit) |
| Library-wide Find Missing Metadata | 🚧 | Album pass + spinner/toast; MB genre field always unset; portraits never downloaded in this pass |
| Artist portraits from lookup | 🚧 | Schema + download helpers exist; **fill never calls download**; UI never shows `image_path` or initials |
| Global grouped search | 🚧 | Artists / Albums / Songs / Genres / Years / Folders; no playlists; folder click is toast-only; genre drill-down is client-side ≤500 songs |
| Lookup progress UI | 🚧 | Header spinner + tooltip (not Banner); scan uses Banner |
| Favourite toggle on song rows + dock | ✅ | Home / Songs / Search / dock; not on artist-detail track rows |
| Context menus (queue / playlist / delete) | ✅ | Native `PopoverMenu` + `gio::Menu` (Milestone 4) |

## Core philosophy & stack

| Item | Status | Notes |
|------|--------|-------|
| GTK4 + libadwaita + Rust | ✅ | |
| Flatpak-first | 🚧 | Manifest + finish-args present; local `cargo run` still primary for day-to-day |
| Offline-first | ✅ | Network only for optional lookup |
| 100k library performance | 🚧 | Songs paginated; search/organise/genre paths can load large sets |

## Library / organisation / metadata

| Item | Status | Notes |
|------|--------|-------|
| Folder scan + watch | ✅ | Debounced `notify` watcher; watches new folders on add; full rescan also prunes deleted files and orphan artists/albums |
| Tag read (lofty) | ✅ | |
| Organise preview + apply + undo | ✅ | One scheme only — album tracks → `Artist/Album/…`, no-album → `Artist/Singles/…`; multi-root via longest folder prefix |
| Single-track metadata edit | 🚧 | Dialog works; menu targets queue current / `context_tracks.first()`, not a right-clicked row; no context “Edit” |
| Batch metadata edit UI | ❌ | |
| MusicBrainz / CAA | 🚧 | Library-wide album pass + CAA art; per-track `lookup_and_fill` unwired and does not write tags |

## Browsing / playback / playlists

| Item | Status | Notes |
|------|--------|-------|
| Artists master-detail | ✅ | List + album sections / singles; empty artists pruned when their last track is removed |
| Albums browser | 🚧 | Cover grid → song list (no dedicated album page) |
| Songs browser | ✅ | Sort + load-more pagination |
| Genres / Years / Folders views | ❌ | Appear in search only; no sidebar destinations |
| Queue / shuffle / repeat | 🚧 | Logic OK (Off / All / One); queue list is display-only — no jump/remove/reorder |
| Gapless / ReplayGain / crossfade | ❌ | Preferences copy acknowledges later |
| Manual playlists | 🚧 | Create, add-from-context, play work; **no delete/rename UI**; opened playlist rows omit artist/album labels |
| Smart playlists / M3U | ❌ | |
| Mini player window | ❌ | |

## Linux integration

| Item | Status | Notes |
|------|--------|-------|
| MPRIS basics | 🚧 | Play / pause / next / previous / stop; metadata on track start; **pause from dock does not update status**; `can_seek` / `can_raise` advertised but unwired |
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
| Lookup felt dead | No progress UI | Fixed (M3) — spinner path; still not Banner |
| Ghost artists after wipe/rescan | Track removal pruned albums only, not artists | **Fixed** — `prune_orphans` on remove and after scan |
| Organise left empty Album/Singles dirs | Execute/undo did not walk empty parents | **Fixed** — prune empty ancestors on apply and undo |
| Rescan left deleted files in DB | `scan_folder` only upserted | **Fixed** — reconcile disk ↔ DB; toast added/removed |

## Audit notes (Milestone 4)

Verified by reading wiring in `window.rs`, UI modules, playback, MPRIS, core library worker, and Flatpak manifest — not by assuming a control implies a finished feature.

### Over-claimed before this pass
- Artist portraits marked ✅ while download is never invoked and the UI never reads `image_path`.
- Lookup progress marked ✅ while only a header spinner exists (scan Banner is separate).
- Manual playlists / queue / single-track edit marked fully done despite missing delete, interactive queue, and weak edit targeting.
- Dock marked fully fixed while high-res covers could still grow the bar (fixed in M4).

### Dead or incomplete wiring
- Portrait pipeline: `artists_missing_image`, `download_artist_image`, `set_artist_image` — unused by fill.
- Per-track MusicBrainz: `lookup_and_fill` never called from UI; would not write tags if called.
- `delete_playlist`, `remove_folder` (prefs Add-only), queue `jump_to` / `remove`.
- Search → Folder activation is a toast only.
- Preferences Playback “Colour scheme” row inert.

## Do not add yet

Lyrics, visualiser, scrobbling, Chromecast — keep the foundation calm and handcrafted.
