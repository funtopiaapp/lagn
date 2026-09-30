#!/usr/bin/env bash
# Build the engine as WebAssembly and stage it for the web app, so the browser
# computes every reading locally and no server is needed.
#
#   bash scripts/build-wasm.sh
#
# Needs wasi-sdk for the C (Swiss Ephemeris). Set WASI_SDK to override.
set -euo pipefail
cd "$(dirname "$0")/.."
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

WASI_SDK="${WASI_SDK:-$HOME/.local/lagn-tools/wasi-sdk-34.0-arm64-macos}"
if [ ! -x "$WASI_SDK/bin/clang" ]; then
  echo "wasi-sdk not found at $WASI_SDK." >&2
  echo "Download it from https://github.com/WebAssembly/wasi-sdk/releases and set WASI_SDK." >&2
  exit 1
fi

rustup target list --installed | grep -qx wasm32-wasip1 || rustup target add wasm32-wasip1

echo "== compiling the engine to WebAssembly"
# wasi-sdk compiles the C; rustc links with its own lld and wasi-libc. Passing
# the external linker as well makes it build a PIC shared library, which fails.
# The larger stack is for Swiss Ephemeris, whose frames overflow the 1 MB default.
CC_wasm32_wasip1="$WASI_SDK/bin/clang" \
AR_wasm32_wasip1="$WASI_SDK/bin/llvm-ar" \
CFLAGS_wasm32_wasip1="--sysroot=$WASI_SDK/share/wasi-sysroot" \
RUSTFLAGS="-C link-arg=-zstack-size=8388608" \
  cargo build --release -p lagn-ffi --target wasm32-wasip1

OUT=web/public/engine
rm -rf "$OUT"
mkdir -p "$OUT/ephe" "$OUT/corpus" "$OUT/data"

cp target/wasm32-wasip1/release/lagn_ffi.wasm "$OUT/lagn.wasm"

# The ephemeris ships in 600-year blocks. _18 covers 1800-2400: every living
# person, and their transits to the end of a long life. Shipping _12 and _24 as
# well would add 3.8 MB to every first visit for births before 1800.
cp ephe/seas_18.se1 ephe/semo_18.se1 ephe/sepl_18.se1 "$OUT/ephe/"
cp ephe/seleapsec.txt "$OUT/ephe/" 2>/dev/null || true

cp -R corpus/. "$OUT/corpus/"
cp data/places.tsv "$OUT/data/"

# The manifest tells the loader what to fetch and where the module sees it.
python3 - "$OUT" <<'PY'
import json, os, sys
out = sys.argv[1]
files = []
for base in ("ephe", "corpus", "data"):
    for dirpath, _dirs, names in os.walk(os.path.join(out, base)):
        for n in sorted(names):
            files.append(os.path.relpath(os.path.join(dirpath, n), out).replace(os.sep, "/"))
files.sort()
json.dump({"root": "/lagn", "wasm": "lagn.wasm", "files": files},
          open(os.path.join(out, "manifest.json"), "w"), indent=2)
print(f"  {len(files)} data files")
PY

echo
echo "staged in $OUT:"
du -sh "$OUT"/lagn.wasm "$OUT"/ephe "$OUT"/corpus "$OUT"/data | sed 's/^/  /'
echo "  total: $(du -sh "$OUT" | cut -f1)"
