# Cadence brand

Visual direction: dark, calm, purple accent — rhythm and library, with an original mark for Cadence. Sibling feel to Discoverr (dark + accent period), distinct palette.

| File | Use |
|------|-----|
| `cadence-mark.svg` | Icon / avatar / favicon-style mark |
| `cadence-lockup.svg` | README and marketing (“Cadence.”) |
| `cadence-social-banner.svg` | Social / Open Graph style banner |
| `../icons/hicolor/scalable/apps/org.cadence.Cadence.svg` | Desktop / Flatpak app icon |

## Palette

| Token | Hex | Role |
|-------|-----|------|
| Accent | `#A882FF` | Period, pulse strokes, highlights |
| Accent soft | `#C4A8FF` | Lighter pulse stop |
| Accent deep | `#7C5CFF` | Darker pulse stop |
| Deep | `#1A102F` | Mark background mid-stop |
| Deep top | `#2A1848` | Mark background highlight |
| Deep bottom | `#0E0A1A` | Mark background shadow |
| Soft text | `#F4F0FF` | Wordmark |

Wordmark ends with a purple period at normal font spacing.

## Typography

Lockup wordmark is **Cantarell Extra Bold** (GNOME’s classic UI face), outlined as SVG paths so GitHub and other hosts render the same weight without needing the font installed.

The purple period uses the font’s normal advance after `Cadence` (as in typed `Cadence.`).

Fallback stack if you re-edit as live text: `Cantarell Extra Bold, Cantarell, Adwaita Sans, Inter, Segoe UI, Ubuntu, system-ui, sans-serif` at weight **800**.

In the GTK app, the header wordmark uses the same stack with a purple period via markup.

## Usage notes

- Prefer the **lockup** in README heroes and marketing.
- Prefer the **mark** alone for app icons, About dialog, empty states, and square crops.
- Prefer the **social banner** for repository social previews and share cards.
- Do not recolor the accent to teal (reserved for Discoverr sibling branding).
- Export PNG from the SVG if a host does not accept SVG uploads.
