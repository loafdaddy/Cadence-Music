# Contributing to Cadence

Thanks for considering a contribution. Cadence is an early public beta — we need help on code, packaging, design, and docs.

## Quick start (from source)

```bash
git clone https://github.com/loafdaddy/Cadence-Music.git
cd Cadence-Music

# Fedora
sudo dnf install gtk4-devel libadwaita-devel \
  gstreamer1-devel gstreamer1-plugins-base-devel \
  gstreamer1-plugins-good gstreamer1-plugins-bad-free \
  rust cargo

cargo run -p cadence
```

Optional local deps prefix:

```bash
source .envrc.build   # if present
cargo run -p cadence
# or after building: ./scripts/run-debug.sh
```

See [docs/INSTALL.md](docs/INSTALL.md) for Flatpak and troubleshooting.

## Development loop

```bash
# Run the app
cargo run -p cadence

# Tests (core library)
cargo test -p cadence-core

# Format + lint
cargo fmt
cargo clippy -p cadence-core -p cadence -- -D warnings
```

Typical change flow:

1. Create a branch from `main` (`git checkout -b feature/short-name`)
2. Make a focused change
3. Run tests / clippy where relevant
4. Open a pull request against `main` with a short “why” in the description

## Project map

| Path | What it is |
|------|------------|
| `crates/core` | Database, scanner, tags, organise, lookup |
| `crates/app` | GTK UI, playback, MPRIS, library worker bridge |
| `data/` | Desktop entry, metainfo, icons, brand SVGs |
| `build-aux/` | Flatpak manifest |
| `docs/` | Install, architecture, roadmap, TODO |
| `scripts/` | Helper scripts (Flatpak build, debug run) |

Status of features: [docs/TODO.md](docs/TODO.md). Do not claim something done unless it works in the running app.
Releases: [docs/RELEASES.md](docs/RELEASES.md).

## What we need most right now

- Bug reports with steps to reproduce (Fedora / Flatpak / from-source)
- Flatpak install testing via `./scripts/build-flatpak.sh`
- UI polish that fits Adwaita (no Electron-style chrome)
- Finishing items under **Next** in `docs/TODO.md` (portraits, MPRIS honesty, playlist/queue UX)

## AI-assisted contributions

**AI tools are welcome.** You can use Cursor, Copilot, ChatGPT, Claude, or similar to help write code, docs, tests, or Flatpak packaging.

A few expectations so reviews stay useful:

- You understand and stand behind the change — if asked, you can explain what it does and why
- You have built and/or run the relevant bits (or say clearly what you could not verify)
- Do not paste large generated dumps that rewrite unrelated files
- Prefer small PRs; call out in the description if AI helped in a substantial way (optional but appreciated)

Parts of Cadence itself may have been written or edited with AI assistance. That is intentional for an early project moving quickly. Human review still applies to every merge.

## Code style

- Prefer small, focused PRs
- Match existing naming and module layout
- Avoid drive-by refactors unrelated to the change
- No emoji in docs or UI strings
- GPL-3.0-or-later for contributions (same as the project)

## Brand

App icon and wordmark live under `data/brand/` and `data/icons/`. Visual direction is dark + purple accent with the wordmark **Cadence.** — see `data/brand/README.md`.

## Communication

- Issues and PRs: https://github.com/loafdaddy/Cadence-Music
- Be respectful; this is an early project and maintainers may move slowly
