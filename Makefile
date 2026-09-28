.PHONY: build test verify oracle clean fmt qa release-check tzdb

build:
	cargo build --release -p lagn-cli -p lagn-server

test:
	cargo test

oracle:
	bash tools/swetest/build.sh

verify: build oracle
	cargo test
	python3 scripts/crossvalidate.py

fmt:
	cargo fmt --all

clean:
	cargo clean
	rm -f tools/swetest/swetest

# Full QA gate: lint, release + debug tests, cross-validation.
qa: build oracle
	cargo clippy --workspace --all-targets --release -- -D warnings
	cargo test --release
	cargo test
	python3 scripts/crossvalidate.py
	python3 scripts/qa_phase2_oracle.py --charts 2000
	python3 scripts/qa_phase3_oracle.py --charts 2000
	python3 scripts/qa_porutham_oracle.py
	python3 scripts/qa_transit_oracle.py
	./target/release/lagn rules validate
	bash scripts/qa_web.sh

# Refresh data/zoneinfo from IANA's latest release.
tzdb:
	bash scripts/update_tzdb.sh

# Gate before any release: full QA, plus tz data must be IANA's latest.
release-check: qa
	python3 scripts/check_tzdb.py
