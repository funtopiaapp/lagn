#!/usr/bin/env bash
# The QA suite inside a Linux container (Dockerfile target "qa"). Everything
# that touches numerics or the server; the browser tests are platform
# independent and run on the development machine.
set -euo pipefail
cd "$(dirname "$0")/.."
echo "== platform: $(uname -sm), glibc $(ldd --version | head -1 | awk '{print $NF}')"
cargo test --release --locked
cargo test --locked
bash tools/swetest/build.sh
python3 scripts/crossvalidate.py
python3 scripts/qa_phase2_oracle.py --charts 3000
python3 scripts/qa_phase3_oracle.py --charts 3000
python3 scripts/qa_porutham_oracle.py
python3 scripts/qa_transit_oracle.py
TOKEN_FILE=$(mktemp); printf 'linux-qa-token-0123456789abcdef\n' > "$TOKEN_FILE"
./target/release/lagn-server --addr 127.0.0.1:8797 --review-token-file "$TOKEN_FILE" >/tmp/srv.log 2>&1 &
PID=$!; trap 'kill $PID 2>/dev/null' EXIT
for _ in $(seq 100); do curl -sf http://127.0.0.1:8797/api/health >/dev/null && break; sleep 0.1; done
python3 scripts/check_tzdb.py --url http://127.0.0.1:8797 || echo "(tz freshness: see above)"
python3 scripts/qa_offset_oracle.py --url http://127.0.0.1:8797 --random 3000 --zones 60
python3 scripts/qa_api_parity.py --url http://127.0.0.1:8797 --token-file "$TOKEN_FILE" --charts 200
if [ -d /out ]; then python3 scripts/qa_cross_platform.py --emit "/out/linux-$(uname -m)"; fi
echo "== LINUX QA PASSED on $(uname -m)"
