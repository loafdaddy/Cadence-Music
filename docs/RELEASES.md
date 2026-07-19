# Cadence releases

Track every published version here. Update this file when cutting a release, then tag and publish on GitHub.

Current version in tree: **0.1.0** (`Cargo.toml` workspace package + AppStream metainfo).

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
3. Add a section below in this file
4. Commit on `main`
5. Tag: `git tag -a v0.1.0 -m "Cadence 0.1.0"`
6. Push: `git push origin main --tags`
7. Create the GitHub release (notes can mirror the section below)
8. Optional: attach a Flatpak bundle from `./scripts/build-flatpak.sh` / `flatpak build-bundle`

## Releases

### 0.1.0 — 2026-07-19 (first public beta)

**Status:** early public beta · not on Flathub yet

**Highlights**
- Native GTK4 / libadwaita music library for local files
- Library scan (menu) with add/remove reconcile and orphan artist/album pruning
- Organise: Artist/Album, or Artist/Singles when album is missing
- Playback dock, Now Playing, queue / shuffle / repeat, MPRIS basics
- Cadence. wordmark and app icon; Flatpak beta via `./scripts/build-flatpak.sh`

**Install**
- Flatpak bundle (easiest for personal use): download `cadence-0.1.0.flatpak` from the [GitHub release](https://github.com/loafdaddy/Cadence-Music/releases/tag/v0.1.0), then `flatpak install --user ./cadence-0.1.0.flatpak`
- From source: see [INSTALL.md](INSTALL.md)
- Flatpak (build from clone): `./scripts/build-flatpak.sh` then `flatpak run org.cadence.Cadence`
- GitHub: https://github.com/loafdaddy/Cadence-Music/releases/tag/v0.1.0

**Known gaps:** see [TODO.md](TODO.md) (queue UI, playlist delete/rename, MPRIS pause status, portraits, etc.)

**AI note:** Substantial parts of this release were developed with AI assistance. AI-assisted contributions remain welcome — see [CONTRIBUTING.md](../CONTRIBUTING.md#ai-assisted-contributions).
