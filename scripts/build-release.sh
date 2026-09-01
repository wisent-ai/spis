#!/bin/sh
# Stage the Spis CLI: the Rust binary plus the catalogs it operates on.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
OUT="$ROOT/.wisent-output/release"
VERSION=$(sed -n 's/.*print("spis \([0-9]*\.[0-9]*\.[0-9]*\)").*/\1/p' "$ROOT/bin/spis")
[ -n "$VERSION" ] || { echo "cannot determine version from bin/spis" >&2; exit 1; }
STAGE="$OUT/spis-$VERSION"

cargo build --release --manifest-path "$ROOT/Cargo.toml"

rm -rf "$OUT"
mkdir -p "$STAGE/bin" "$STAGE/docs" "$OUT/bin"
install -m 0755 "$ROOT/target/release/spis" "$STAGE/bin/spis"
install -m 0755 "$ROOT/target/release/spis" "$OUT/bin/spis"
cp "$ROOT/example-catalogs.json" "$STAGE/example-catalogs.json"
cp "$ROOT"/docs/*.md "$STAGE/docs/" 2>/dev/null || true

TARBALL="$OUT/spis-darwin-arm64.tar.gz"
tar -czf "$TARBALL" -C "$(dirname "$STAGE")" "$(basename "$STAGE")"
(cd "$(dirname "$TARBALL")" && shasum -a 256 "$(basename "$TARBALL")" > "$(basename "$TARBALL").sha256")
echo "staged $TARBALL"
