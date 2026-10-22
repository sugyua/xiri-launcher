#!/usr/bin/env bash
# Reproduce Xiri app icons from the original geometric vector; needs resvg and Rust.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/assets/app-icon"
SOURCE="$ROOT/assets/xiri/mark.svg"
ID="io.github.sugyua.xiri-launcher"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
command -v resvg >/dev/null || { echo "resvg is required" >&2; exit 1; }
render() { resvg -w "$2" -h "$2" "$1" "$3"; }
render "$SOURCE" 1024 "$DIR/xiri-launcher-1024.png"
for size in 16 24 32 48 64 128 256 512; do
  mkdir -p "$DIR/hicolor/${size}x${size}/apps"
  render "$SOURCE" "$size" "$DIR/hicolor/${size}x${size}/apps/$ID.png"
done
mkdir -p "$DIR/hicolor/scalable/apps"
cp "$SOURCE" "$DIR/hicolor/scalable/apps/$ID.svg"
cp "$SOURCE" "$DIR/xiri-launcher.svg"
ICO_PNGS=()
for size in 16 20 24 32 40 48 64 128 256; do
  render "$SOURCE" "$size" "$TMP/ico-$size.png"
  ICO_PNGS+=("$TMP/ico-$size.png")
done
(cd "$ROOT" && cargo run -q -p xtask -- ico "$DIR/xiri-launcher.ico" "${ICO_PNGS[@]}")
ICNS_PNGS=()
for size in 16 32 64 128 256 512 1024; do
  render "$SOURCE" "$size" "$TMP/icns-$size.png"
  ICNS_PNGS+=("$TMP/icns-$size.png")
done
(cd "$ROOT" && cargo run -q -p xtask -- icns "$DIR/xiri-launcher.icns" "${ICNS_PNGS[@]}")
render "$SOURCE" 64 "$ROOT/assets/xiri/mark.png"
