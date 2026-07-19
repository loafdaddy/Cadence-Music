#!/usr/bin/env bash
# Build and install the Cadence Flatpak (user install) for local beta testing.
# Also exports a release-ready .flatpak bundle with a Flathub runtime-repo hint.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/build-aux/org.cadence.Cadence.yml"
BUILD_DIR="${CADENCE_FLATPAK_BUILD_DIR:-$ROOT/build-dir}"
REPO_DIR="${CADENCE_FLATPAK_REPO:-$ROOT/.flatpak-repo}"
FLATHUB_REPO_URL="${CADENCE_FLATPAK_RUNTIME_REPO:-https://flathub.org/repo/flathub.flatpakrepo}"

if ! command -v flatpak-builder >/dev/null 2>&1; then
  echo "flatpak-builder not found. On Fedora: sudo dnf install flatpak-builder" >&2
  exit 1
fi

if ! command -v flatpak >/dev/null 2>&1; then
  echo "flatpak not found. On Fedora: sudo dnf install flatpak" >&2
  exit 1
fi

VERSION="$(
  awk '
    $0 == "[workspace.package]" { in_pkg = 1; next }
    in_pkg && /^\[/ { exit }
    in_pkg && $1 == "version" {
      gsub(/"/, "", $3)
      print $3
      exit
    }
  ' "$ROOT/Cargo.toml"
)"
if [[ -z "${VERSION}" ]]; then
  echo "Could not read workspace.package version from Cargo.toml" >&2
  exit 1
fi

BUNDLE="${CADENCE_FLATPAK_BUNDLE:-$ROOT/cadence-${VERSION}.flatpak}"

echo "==> Building org.cadence.Cadence ${VERSION} from $MANIFEST"
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
echo "==> Exporting bundle $BUNDLE"
echo "    runtime-repo: $FLATHUB_REPO_URL"
flatpak build-bundle \
  "$REPO_DIR" \
  "$BUNDLE" \
  org.cadence.Cadence \
  --runtime-repo="$FLATHUB_REPO_URL"

echo
echo "Installed. Run with:"
echo "  flatpak run org.cadence.Cadence"
echo
echo "Bundle ready for GitHub release attach:"
echo "  $BUNDLE"
echo
echo "Clean-machine install (Flathub + GNOME Platform once, then the bundle):"
echo "  flatpak remote-add --if-not-exists --user flathub $FLATHUB_REPO_URL"
echo "  flatpak install --user -y org.gnome.Platform//49"
echo "  flatpak install --user ./$(basename "$BUNDLE")"
