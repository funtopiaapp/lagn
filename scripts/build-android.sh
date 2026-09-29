#!/usr/bin/env bash
# Build the engine as a shared library for Android, with the JNI entry points,
# and stage the data the app packages. Needs the NDK and cargo-ndk.
#
#   ANDROID_NDK_HOME=... bash scripts/build-android.sh
set -euo pipefail
cd "$(dirname "$0")/.."

: "${ANDROID_NDK_HOME:?set ANDROID_NDK_HOME to the installed NDK}"
command -v cargo-ndk >/dev/null || { echo "cargo-ndk is not installed: cargo install cargo-ndk" >&2; exit 1; }

# arm64 covers modern devices; x86_64 is for the emulator.
ABIS=(arm64-v8a x86_64)
OUT=build/android
mkdir -p "$OUT/jniLibs" "$OUT/assets"

echo "== building ${ABIS[*]} with the jni feature"
cargo ndk $(printf -- "-t %s " "${ABIS[@]}") -o "$OUT/jniLibs" \
  build --release -p lagn-ffi --features jni

cp -R ephe corpus "$OUT/assets/"
mkdir -p "$OUT/assets/data"
cp data/places.tsv "$OUT/assets/data/"
cp -R data/zoneinfo "$OUT/assets/data/"

echo
echo "staged in $OUT:"
du -sh "$OUT/jniLibs"/* "$OUT/assets" 2>/dev/null || true
cat <<'NOTE'

In the Android project:
  1. Copy build/android/jniLibs into src/main/jniLibs.
  2. Copy build/android/assets into src/main/assets.
  3. On first run copy the assets to filesDir (they are inside the APK, which
     is not a real filesystem, and Swiss Ephemeris opens files), then call
     LagnNative.init with those paths.
NOTE
