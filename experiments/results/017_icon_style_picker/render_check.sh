#!/usr/bin/env bash
# Rasterizes the picker sheets with headless Edge into out/017_icon_style_picker/ for a look.
set -e
cd "$(dirname "$0")/../../.."
EDGE="/c/Program Files (x86)/Microsoft/Edge/Application/msedge.exe"
OUT="out/017_icon_style_picker"
mkdir -p "$OUT"
for s in a b c d; do
  "$EDGE" --headless --disable-gpu --hide-scrollbars --window-size=940,604 \
    --screenshot="$(cygpath -w "$PWD/$OUT/style_$s.png")" \
    "file:///$(cygpath -m "$PWD/art/pickers/icon_style_r1/style_$s.svg")" 2>/dev/null
done
ls "$OUT"
