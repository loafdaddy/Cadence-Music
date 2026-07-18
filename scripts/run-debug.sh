#!/usr/bin/env bash
# Run Cadence with the local .deps libs + system GStreamer plugins.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEPS_ROOT="$ROOT/.deps/root"
export PKG_CONFIG_PATH="$DEPS_ROOT/usr/lib64/pkgconfig:${PKG_CONFIG_PATH:-/usr/lib64/pkgconfig}"
export LIBRARY_PATH="$DEPS_ROOT/usr/lib64${LIBRARY_PATH:+:$LIBRARY_PATH}"
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH:+$LD_LIBRARY_PATH:}$DEPS_ROOT/usr/lib64"
export GST_PLUGIN_SYSTEM_PATH="${GST_PLUGIN_SYSTEM_PATH:-/usr/lib64/gstreamer-1.0}"
export GST_PLUGIN_PATH="${GST_PLUGIN_PATH:-/usr/lib64/gstreamer-1.0}"
BIN="$ROOT/target/debug/cadence"
if [[ ! -x "$BIN" ]]; then
  echo "missing $BIN — run: cargo build -p cadence" >&2
  exit 1
fi
exec "$BIN" "$@"
