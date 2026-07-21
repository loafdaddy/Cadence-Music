# Cadence releases

Track every published version here. Update this file when cutting a release, then tag and publish on GitHub.

Current version in tree: **0.1.1** (`Cargo.toml` workspace package + AppStream metainfo).

## Versioning

Cadence uses [Semantic Versioning](https://semver.org/) while still in early beta:

| Part | Meaning for Cadence |
|------|---------------------|
| **MAJOR** | Breaking library/DB/on-disk behaviour users must migrate for |
| **MINOR** | New features or UI surfaces (still may be rough) |
| **PATCH** | Fixes and small polish |

Pre-1.0: expect breaking changes in minor releases. Mark development/beta builds clearly in release notes.

## How to cut a release

1. Update version in `Cargo.toml` (`[workspace.package] version`)
2. Add a matching `<release>` entry in `data/org.cadence.Cadence.metainfo.xml`
3. Add a section below in this file; bump version mentions in `README.md` / `SETUP.md` if needed
4. Commit on `main` (or merge the release PR)
5. Tag: `git tag -a v0.1.1 -m "Cadence 0.1.1"`
6. Push: `git push origin main --tags`
7. Create the GitHub release (notes can mirror the section below)
8. Attach the Flatpak bundle:
   - Ensure Flathub runtimes for the manifest are installed (`org.gnome.Platform` / `Sdk` + `rust-stable` — see [SETUP.md](../SETUP.md))
   - Run `./scripts/build-flatpak.sh` — this installs locally **and** writes `cadence-<version>.flatpak` with `--runtime-repo` pointing at Flathub
   - Upload that `.flatpak` on the GitHub release
9. Sanity-check on a machine that does **not** already have the app: Flathub remote + Platform install (once), then `flatpak install --user ./cadence-<version>.flatpak`

### Flatpak bundle notes

- The `.flatpak` file is **app-only** (a few MB). It does **not** embed the GNOME Platform (~1 GB).
- Clean installs need Flathub once so Flatpak can pull `org.gnome.Platform` for the version in `build-aux/org.cadence.Cadence.yml`.
- Always export with `--runtime-repo=https://flathub.org/repo/flathub.flatpakrepo` (the build script does this). Without it, double-click / Software installs often fail on machines that lack the runtime.
- Keep the manifest on a **supported** GNOME runtime (not EOL). EOL platforms disappear from Flathub and break new installs.

## Releases

### 0.1.1 — 2026-07-19 (Flatpak clean-install fix)

**Status:** early public beta · not on Flathub yet

**Highlights**
- Flatpak targets **GNOME Platform 49** (48 is end-of-life and no longer usable for clean Flathub installs)
- `scripts/build-flatpak.sh` exports `cadence-0.1.1.flatpak` with a Flathub `--runtime-repo` hint
- Docs clarify that the bundle is app-only and needs the Platform runtime once
- GNOME Software: if two **Local file** targets appear, choose **USER** (system-wide default often hangs on fresh installs)

**Install**
- Flatpak bundle: download `cadence-0.1.1.flatpak` from the [GitHub release](https://github.com/loafdaddy/Cadence-Music/releases/tag/v0.1.1)
- Prefer the terminal (`--user`). In Software, pick the **USER** target if offered.
- One-time runtime (if needed):

```bash
flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --user -y org.gnome.Platform//49
flatpak install --user ./cadence-0.1.1.flatpak
flatpak run org.cadence.Cadence
```

- From source / local Flatpak build: see [SETUP.md](../SETUP.md)
- GitHub: https://github.com/loafdaddy/Cadence-Music/releases/tag/v0.1.1

**Known gaps:** see [TODO.md](TODO.md)

**AI note:** Substantial parts of this project were developed with AI assistance. AI-assisted contributions remain welcome — see [CONTRIBUTING.md](../CONTRIBUTING.md#ai-assisted-contributions).

### 0.1.0 — 2026-07-19 (first public beta)

**Status:** early public beta · not on Flathub yet · **superseded for Flatpak installs by 0.1.1**

**Highlights**
- Native GTK4 / libadwaita music library for local files
- Library scan (menu) with add/remove reconcile and orphan artist/album pruning
- Organise: Artist/Album, or Artist/Singles when album is missing
- Playback dock, Now Playing, queue / shuffle / repeat, MPRIS basics
- Cadence. wordmark and app icon; Flatpak beta via `./scripts/build-flatpak.sh`

**Install**
- Prefer **0.1.1** — the 0.1.0 bundle targets EOL GNOME Platform 48 and fails on clean machines
- From source: see [SETUP.md](../SETUP.md)
- GitHub: https://github.com/loafdaddy/Cadence-Music/releases/tag/v0.1.0

**Known gaps:** see [TODO.md](TODO.md) (queue UI, playlist delete/rename, MPRIS pause status, portraits, etc.)

**AI note:** Substantial parts of this release were developed with AI assistance. AI-assisted contributions remain welcome — see [CONTRIBUTING.md](../CONTRIBUTING.md#ai-assisted-contributions).
