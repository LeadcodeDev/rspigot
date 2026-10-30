#!/bin/sh
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
RE="${RE_ROOT:-/Users/baptiste.parmantier/Development/rspigot/re}"
CLASSES="${CLASSES:-$RE/server}"
JAVAP="${JAVAP:-/opt/homebrew/opt/openjdk@25/bin/javap}"
DUMP="${DUMP:-$RE/datagen/generated/reports/block_properties.json}"
TAGS="${TAGS:-$RE/server/data/minecraft/tags/block}"
BLOCKS_MOD="${BLOCKS_MOD:-$ROOT/crates/rspigot-blocks/src/blocks/mod.rs}"
OUT="${1:-$ROOT/crates/rspigot-ai/src/classify.rs}"
case "$OUT" in
  /*) ;;
  *) OUT="$(pwd)/$OUT" ;;
esac

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

python3 -I "$HERE/hier.py" "$CLASSES" "$DUMP" "$JAVAP" "$WORK/hier.txt"
python3 -I "$HERE/classes.py" "$DUMP" "$WORK/hier.txt" "$TAGS" "$BLOCKS_MOD" "$WORK/table.rs"
python3 -I "$HERE/emit.py" "$WORK/table.rs" "$WORK/classify.rs"
(cd "$ROOT" && rustfmt --edition 2024 --emit stdout < "$WORK/classify.rs" > "$WORK/formatted.rs")
cp "$WORK/formatted.rs" "$OUT"
echo "wrote $OUT"
