#!/usr/bin/env bash
# Build and install the Cadence Flatpak (user install) for local beta testing.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/build-aux/org.cadence.Cadence.yml"
BUILD_DIR="${CADENCE_FLATPAK_BUILD_DIR:-$ROOT/build-dir}"
REPO_DIR="${CADENCE_FLATPAK_REPO:-$ROOT/.flatpak-repo}"

if ! command -v flatpak-builder >/dev/null 2>&1; then
  echo "flatpak-builder not found. On Fedora: sudo dnf install flatpak-builder" >&2
  exit 1
fi

if ! command -v flatpak >/dev/null 2>&1; then
  echo "flatpak not found. On Fedora: sudo dnf install flatpak" >&2
  exit 1
fi

echo "==> Building org.cadence.Cadence from $MANIFEST"
echo "    build dir: $BUILD_DIR"

# --install puts the app in the user installation so `flatpak run` works immediately.
flatpak-builder \
  --user \
  --install \
  --force-clean \
  --repo="$REPO_DIR" \
  "$BUILD_DIR" \
  "$MANIFEST"

echo
echo "Installed. Run with:"
echo "  flatpak run org.cadence.Cadence"
echo
echo "Optional: export a single-file bundle for sharing:"
echo "  flatpak build-bundle $REPO_DIR cadence-0.1.0.flatpak org.cadence.Cadence"
