#!/usr/bin/env bash
# Phase 4 QA gate: frontend unit tests, build, end-to-end browser tests, and
# the live-server oracles (offset N-version and HTTP parity).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/web"
npm run typecheck
npm test
npm run build
npm run build:xorigin
npx playwright test

cd "$ROOT"
TOKEN_FILE="$(mktemp)"; printf 'qa-web-token-0123456789abcdef\n' > "$TOKEN_FILE"
./target/release/lagn-server --addr 127.0.0.1:8798 --review-token-file "$TOKEN_FILE" >/tmp/lagn-qa-web.log 2>&1 &
PID=$!
trap 'kill $PID 2>/dev/null; rm -f "$TOKEN_FILE"' EXIT
for _ in $(seq 50); do curl -sf http://127.0.0.1:8798/api/health >/dev/null && break; sleep 0.1; done
python3 scripts/qa_offset_oracle.py --url http://127.0.0.1:8798 --random 3000 --zones 60
python3 scripts/qa_api_parity.py --url http://127.0.0.1:8798 --token-file "$TOKEN_FILE" --charts 200
