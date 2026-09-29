#!/usr/bin/env bash
# Build the engine as a native library and call it from a plain C program,
# the way an iOS app will. No server, no network.
#
#   bash scripts/native-demo.sh
set -euo pipefail
cd "$(dirname "$0")/.."
# cargo may not be on PATH in a non-login shell.
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

# Keep the C objects and the link step on the same deployment target, or the
# linker warns about every Swiss Ephemeris object file.
if [ "$(uname -s)" = "Darwin" ]; then
  export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-$(sw_vers -productVersion | cut -d. -f1-2)}"
fi

echo "== building the library"
cargo build --release -p lagn-ffi

OUT=build/native-demo
mkdir -p "$OUT"

echo "== compiling the C program against include/lagn.h"
case "$(uname -s)" in
  Darwin) LINK=(-framework CoreFoundation -framework Security -liconv) ;;
  *)      LINK=(-lpthread -ldl -lm) ;;
esac
cc -O2 -Wall -Wextra -I include \
   crates/lagn-ffi/examples/demo.c \
   target/release/liblagn_ffi.a \
   "${LINK[@]}" -o "$OUT/demo"

echo "== running (the engine is inside this binary)"
"$OUT/demo" "$PWD"

echo "library: $(du -h target/release/liblagn_ffi.dylib 2>/dev/null | cut -f1 || echo n/a) shared, \
$(du -h "$OUT/demo" | cut -f1) for this statically linked program"
