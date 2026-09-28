#!/usr/bin/env bash
# Build swetest, the reference CLI that ships with Swiss Ephemeris.
#
# We use it as an INDEPENDENT oracle for layer 1: it is the same C library our
# FFI calls, driven through a completely separate front end, so agreement
# proves our flags, sidereal mode and coordinate conventions are right. It
# cannot validate layers 2-3 (rasi, navamsa, dasha) - those are checked
# against classical rules in unit tests and against JHora in golden fixtures.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
VENDOR="$HERE/../../crates/swe-sys/vendor"
OUT="$HERE/swetest"

if [[ -x "$OUT" && "$OUT" -nt "$HERE/swetest.c" ]]; then
  echo "swetest already built: $OUT"; exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
cc -O2 -w -I"$VENDOR" -o "$OUT" \
  "$HERE/swetest.c" \
  "$VENDOR"/swedate.c "$VENDOR"/swehouse.c "$VENDOR"/swejpl.c \
  "$VENDOR"/swemmoon.c "$VENDOR"/swemplan.c "$VENDOR"/sweph.c \
  "$VENDOR"/swephlib.c "$VENDOR"/swecl.c "$VENDOR"/swehel.c \
  -lm
echo "built $OUT"
