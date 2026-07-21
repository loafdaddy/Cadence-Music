# FAQ

Short answers for operators and contributors. Install steps: [SETUP.md](../SETUP.md).

## General

**What is Cadence?**  
A native GTK4 / libadwaita music library for local files on Linux. Offline-first, Flatpak-friendly, no Electron.

**Is this production-ready?**  
No — **early public beta** (**v0.1.1**). Core browse/play/scan paths work; expect rough edges. See [TODO.md](TODO.md).

**Is Cadence on Flathub?**  
Not yet. Use the GitHub release `.flatpak` or build from source.

## Install

**Why is the Flatpak only a few MB?**  
The release bundle is **app-only**. The shared GNOME Platform runtime (~1 GB) comes from Flathub once.

**GNOME Software hangs on Preparing**  
If you see two **Local file** targets, choose **USER**. Or install from a terminal with `flatpak install --user`. Details: [SETUP.md](../SETUP.md).

**Which release should I install?**  
Prefer **0.1.1+**. The 0.1.0 bundle targeted EOL GNOME Platform 48.

## Library and files

**Does Cadence upload or stream my music?**  
No. It indexes and plays files on disk.

**Will Organise Library rename files automatically?**  
No. You must open **Organise Library**, preview the plan, and apply. Undo is in-memory for the last apply only (lost on quit).

**Library looks empty after I deleted files on disk**  
Use menu **Scan Library** — it reconciles adds/removes and prunes orphan albums/artists.

## Playback

**No sound**  
Confirm GStreamer good/bad plugins are installed. From source, check `GST_PLUGIN_PATH` if you use a custom prefix. See [SETUP.md § Troubleshooting](../SETUP.md#troubleshooting).

**Do media keys work?**  
Basic MPRIS play/pause/next/previous/stop is wired. Pause status from the dock is still incomplete — see [TODO.md](TODO.md).

## Contributing

**Can I use AI tools?**  
Yes. Expectations: [CONTRIBUTING.md — AI-assisted contributions](../CONTRIBUTING.md#ai-assisted-contributions).

**Where should I start?**  
[TODO.md](TODO.md) **Next** list, Flatpak install testing, and focused bug reports with steps to reproduce.
