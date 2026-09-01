#!/bin/sh
# Stage the Spis CLI: a pure-Python tool plus the catalogs it operates on.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
OUT="$ROOT/.wisent-output/release"
STAGE="$OUT/spis-1.0.0"
VERSION=$(sed -n 's/.*print("spis \([0-9]*\.[0-9]*\.[0-9]*\)").*/\1/p' "$ROOT/bin/spis")
[ -n "$VERSION" ] || { echo "cannot determine version from bin/spis" >&2; exit 1; }
STAGE="$ROOT/.wisent-output/release/spis-$VERSION"

rm -rf "$OUT"
mkdir -p "$STAGE/bin" "$STAGE/docs"
install -m 0755 "$ROOT/bin/spis" "$STAGE/bin/spis"
cp "$ROOT/example-catalogs.json" "$STAGE/example-catalogs.json"
cp "$ROOT"/docs/*.md "$STAGE/docs/" 2>/dev/null || true

TARBALL="$OUT/spis-darwin-arm64.tar.gz"
tar -czf "$TARBALL" -C "$(dirname "$STAGE")" "$(basename "$STAGE")"
(cd "$(dirname "$TARBALL")" && shasum -a 256 "$(basename "$TARBALL")" > "$(basename "$TARBALL").sha256")
echo "staged $TARBALL"
