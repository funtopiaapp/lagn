#!/usr/bin/env bash
# Fetch the latest IANA tz database, build zic from the same release, and
# compile the zones into data/zoneinfo. Run before every release; the server
# uses whichever of this copy, the OS copy and jiff's bundled copy is newest.
#
# Usage: scripts/update_tzdb.sh [VERSION]    (default: IANA's latest)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
V="${1:-$(curl -fsS https://data.iana.org/time-zones/tzdb/version)}"
[[ "$V" =~ ^[0-9]{4}[a-z]$ ]] || { echo "bad version: $V" >&2; exit 1; }
WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
cd "$WORK"
for part in tzcode tzdata; do
  curl -fsSLO "https://data.iana.org/time-zones/releases/${part}${V}.tar.gz"
done
shasum -a 256 tzcode${V}.tar.gz tzdata${V}.tar.gz > SHA256SUMS
mkdir src && tar -xzf tzcode${V}.tar.gz -C src && tar -xzf tzdata${V}.tar.gz -C src
( cd src && make -s zic >/dev/null )
[[ "$(cat src/version)" == "$V" ]] || { echo "tarball version mismatch" >&2; exit 1; }

OUT="$ROOT/data/zoneinfo"
rm -rf "$OUT.new" && mkdir -p "$OUT.new"
( cd src && ./zic -b slim -d "$OUT.new" africa antarctica asia australasia europe northamerica southamerica etcetera backward )
echo "$V" > "$OUT.new/+VERSION"
cp SHA256SUMS "$OUT.new/SOURCE-SHA256SUMS"
rm -rf "$OUT" && mv "$OUT.new" "$OUT"
echo "data/zoneinfo: IANA tzdb $V ($(find "$OUT" -type f | wc -l | tr -d ' ') files, $(du -sh "$OUT" | cut -f1))"
