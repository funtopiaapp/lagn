# Deployment validation: Linux and tz data freshness

Date: 2026-09-21. Closes Phase 4 open items 1 (Linux validation) and 2 (tz data currency).

## Linux validation

Environment: Colima 0.10.3 / Lima 2.2.0 VM (Apple Virtualization, Rosetta for
x86-64), Docker 29.8 CLI with buildx 0.37.1. All binaries were installed in
`~/.local/lagn-tools` with checksums verified; no admin rights were used.
Images are built from the repository `Dockerfile` (target `qa`) on
`rust:1.98-trixie`, glibc 2.41.

| Check | Linux arm64 | Linux x86-64 |
|---|---|---|
| Rust tests, release and debug | 520 passed | 520 passed |
| swetest cross-validation (45 charts) | worst 0.000965 arcsec | worst 0.000965 arcsec |
| Phase 2 oracle, 3,000 charts | 2,004,000 checks, 0 failing | 2,004,000 checks, 0 failing |
| Phase 3 oracle, 3,000 charts | 66,007 evaluations, 0 failing | 66,007 evaluations, 0 failing |
| Porutham, full input space | 116,640 verdicts, 0 mismatches | 116,640 verdicts, 0 mismatches |
| tz offsets vs zoneinfo | 56,494 cases, 0 disagreements | 56,494 cases, 0 disagreements |
| Live HTTP parity, 8 concurrent clients | 200 x 7, 0 failing | 200 x 7, 0 failing |

x86-64 ran under Rosetta, which executes x86-64 instructions faithfully
(including the glibc x86-64 maths library), but it isn't physical x86-64
hardware. Run `docker run lagn-qa` once on the real server as the final check.

### Defect found: Swiss Ephemeris state was per-thread on Linux (QA-P4-7)

**Severity: release blocker, now fixed.** Upstream Swiss Ephemeris declares
its global state `__thread` on every platform except Apple
(`sweodef.h:86`). On Linux an ephemeris path set on one thread was invisible
to others: in the server, every request handled on a worker thread would have
failed. It would have **failed loudly, not silently**, because Phase 1's
strict mode refused the less accurate Moshier fallback. The macOS suite
could never see this, because Apple builds already use global state.

**Fix:** `swe-sys` compiles with `-DTLSOFF`, one global state on every
platform, serialised by `lagn-ephem`'s mutex, which is the configuration
validated on macOS. Regression test:
`the_ephemeris_path_applies_to_threads_spawned_after_it_was_set`.

## Cross-platform agreement

The same 500 births (every CLI output: chart, dasha, 16 vargas, details,
Ashtakavarga, marriage topic) on three platforms:

| Comparison | Discrete differences | Floats differing | Largest longitude difference |
|---|---|---|---|
| macOS arm64 vs Linux arm64 | **0** | 14,924 | 0.000126 arcsec |
| macOS arm64 vs Linux x86-64 | **0** | 11,295 | 0.000126 arcsec |
| Linux arm64 vs Linux x86-64 | **0** | 3,641 | 0.000029 arcsec |

This confirms, with measurement, the correction made at the start of Phase 4.
Results are **not bit-identical** across platforms: each maths library rounds
its last bits differently. But **no sign, nakshatra, pada, house, dasha lord,
dignity, bindu or rule verdict differs** in any of the 500 births, and the
largest positional difference is 100 million times smaller than a pada.

## tz data freshness (QA-P4-8)

This Mac had tzdb 2026b, Debian trixie ships 2026c, jiff bundles 2026c, and
IANA is at **2026d**, which corrects Colombia 1992 and Iran 1979. Fix:

- `scripts/update_tzdb.sh` downloads IANA's latest release, builds `zic`
  from the same release, compiles `data/zoneinfo` and records the source
  checksums. It is now at 2026d.
- The server uses the **newest** of the shipped, OS and bundled copies. In
  the production container it chose shipped 2026d over Debian's 2026c.
- `make release-check` fails if the copy in use is older than IANA's latest.
- A regression test pins a 2026d correction (Iran 1979-05-26 00:30 didn't
  exist), and it was shown to fail on the 2026b data.

## Production image

`docker build -t lagn .` produces a 46 MB image. It runs as a non-root user
(uid 10001) with a healthcheck, serves the web app and the API, and keeps
review mode disabled unless a token file is mounted. A chart computed inside
the container matched the macOS output.
