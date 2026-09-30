#!/usr/bin/env bash
# Build the engine as a static library for iOS, and stage the data the app
# bundles. Needs Xcode (for the linker and the SDKs).
#
#   bash scripts/build-ios.sh            device + simulator
#   bash scripts/build-ios.sh device     device only
set -euo pipefail
cd "$(dirname "$0")/.."

command -v xcrun >/dev/null || { echo "Xcode is not installed: xcrun not found." >&2; exit 1; }

WHICH="${1:-all}"
TARGETS=()
case "$WHICH" in
  device) TARGETS=(aarch64-apple-ios) ;;
  sim)    TARGETS=(aarch64-apple-ios-sim) ;;
  all)    TARGETS=(aarch64-apple-ios aarch64-apple-ios-sim) ;;
  *) echo "usage: $0 [device|sim|all]" >&2; exit 2 ;;
esac

for t in "${TARGETS[@]}"; do
  rustup target list --installed | grep -qx "$t" || rustup target add "$t"
  echo "== building $t"
  cargo build --release -p lagn-ffi --target "$t"
done

OUT=build/ios
mkdir -p "$OUT/lib" "$OUT/include" "$OUT/data"
cp include/lagn.h "$OUT/include/"
for t in "${TARGETS[@]}"; do
  mkdir -p "$OUT/lib/$t"
  cp "target/$t/release/liblagn_ffi.a" "$OUT/lib/$t/"
done

# The reference data the app bundles and copies to its own storage on first run.
cp -R ephe corpus "$OUT/data/"
mkdir -p "$OUT/data/data"
cp data/places.tsv "$OUT/data/data/"
cp -R data/zoneinfo "$OUT/data/data/"

echo
echo "staged in $OUT:"
du -sh "$OUT"/lib/* "$OUT/data" 2>/dev/null || true
cat <<'NOTE'

In Xcode:
  1. Add build/ios/lib/<target>/liblagn_ffi.a to "Link Binary With Libraries",
     and build/ios/include to the header search paths.
  2. Add build/ios/data as a folder reference so it ships in the bundle.
  3. On first run copy it into Application Support, then call lagn_init with
     those paths. The ephemeris is opened as files, so it cannot stay in a
     read-only bundle on every platform; copying keeps one code path.
NOTE
