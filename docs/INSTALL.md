# Installing and running Cadence

Cadence is an **early public beta**. Prefer Flatpak to try it; use a source checkout when you want to change code.

## Flatpak (try the beta)

### Prerequisites (Fedora)

```bash
sudo dnf install flatpak flatpak-builder
flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --user -y \
  org.gnome.Platform//48 \
  org.gnome.Sdk//48 \
  org.freedesktop.Sdk.Extension.rust-stable//24.08
```

The Rust SDK extension version must match what your Freedesktop SDK expects; if install fails, check `flatpak search rust-stable` and adjust the script/manifest accordingly.

### Build and install (from a git clone)

```bash
git clone https://github.com/loafdaddy/Cadence-Music.git
cd Cadence-Music
./scripts/build-flatpak.sh
flatpak run org.cadence.Cadence
```

The script runs `flatpak-builder` against `build-aux/org.cadence.Cadence.yml`, installs into your user Flatpak, and prints the run command.

### Uninstall

```bash
flatpak uninstall --user org.cadence.Cadence
```

### Notes

- Default music access is `xdg-music` (read/write). Folders outside that path need portal grants when you pick them in the app.
- First Flatpak build downloads dependencies and can take a while.
- This is not on Flathub yet — local install only for the first beta.

## From source (development)

Use this when you are changing Cadence or debugging.

### 1. Clone

```bash
git clone https://github.com/loafdaddy/Cadence-Music.git
cd Cadence-Music
```

### 2. System packages (Fedora)

```bash
sudo dnf install gtk4-devel libadwaita-devel \
  gstreamer1-devel gstreamer1-plugins-base-devel \
  gstreamer1-plugins-good gstreamer1-plugins-bad-free \
  rust cargo
```

Other distros: install equivalent GTK4, libadwaita, GStreamer (base/good/bad), and a recent Rust toolchain (edition 2021 / rustc 1.80+).

### 3. Run

```bash
cargo run -p cadence
```

Optional local `.deps` prefix (headers/libs):

```bash
source .envrc.build    # if the file exists in your tree
cargo run -p cadence
```

After a debug build you can also use:

```bash
./scripts/run-debug.sh
```

### 4. Test and lint

```bash
cargo test -p cadence-core
cargo fmt
cargo clippy -p cadence-core -p cadence -- -D warnings
```

### 5. Making changes

1. Branch from `main`
2. Edit code under `crates/`
3. `cargo run -p cadence` to verify
4. Open a PR — see [CONTRIBUTING.md](../CONTRIBUTING.md)

Data files (icons, desktop entry, metainfo) live under `data/`. When running from cargo, the app registers `data/icons` so the Cadence icon appears in the header and About dialog.

## Troubleshooting

| Symptom | What to try |
|---------|-------------|
| No sound | Confirm GStreamer good/bad plugins are installed; check `GST_PLUGIN_PATH` |
| Missing app icon when running from cargo | Ensure `data/icons/hicolor/scalable/apps/org.cadence.Cadence.svg` exists |
| Flatpak build fails on Rust extension | Align `sdk-extensions` / install version with your Freedesktop runtime |
| Library empty after wipe | Use menu **Scan Library** — it prunes orphans and reconciles disk |

## Related docs

- [CONTRIBUTING.md](../CONTRIBUTING.md) — how to contribute
- [ARCHITECTURE.md](ARCHITECTURE.md) — crates and threading
- [TODO.md](TODO.md) — status and priorities
- [ROADMAP.md](ROADMAP.md) — milestones
- [RELEASES.md](RELEASES.md) — version history and how to cut a release
