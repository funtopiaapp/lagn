# Phase 1 QA report

Scope: layers 1-3 (ephemeris, rasi/bhava/D-9, Vimshottari x3) and the CLI.
Date: 2026-09-20.

## Result

7 defects found and fixed, each pinned by a regression test. 127 tests pass in
release and debug (overflow checks on). No clippy warnings. The 45-chart swetest
cross-validation still agrees to 0.000965 arcsec after the fixes.

## Defects

| ID | Severity | Defect | Consequence before fix |
|---|---|---|---|
| QA-1 | Medium | `norm360` could return exactly 360.0 | Broke the documented `[0,360)` contract. Masked in practice because every consumer re-normalised; would have hit any external consumer of the JSON |
| QA-2 | **High** | A NaN or infinite Julian Day was accepted | Returned a complete, plausible chart: Mesha lagna, Ashwini pada 1, no error |
| QA-3 | Low | The exact poles returned one identical ascendant for N and S | Wrong output rather than an error. Nobody is born at exactly ±90, so low impact |
| QA-4 | **High** | Impossible dates accepted (30 Feb, 31 Apr, year 0) | A typo in a birth date silently produced a chart for a different day |
| QA-5 | **High** | JSON lost up to 1 ULP on floats | Broke the bit-identical reproducibility contract at the API and fixture boundary |
| QA-6 | Medium | The strict-mode Moshier guard had never been tested | Unverified safety net |
| QA-7 | Medium | Pre-1582 dates were printed in the wrong calendar; dates in the 1582 reform gap were accepted | A 1400 birth listed its first dasha 9 days after the birth date; a 1582-10-10 birth moved 10 days forward |

Also hardened (no defect observed): nakshatra and pada are now read off a single
floor on the 108-cell grid. Before, they came from two independent divisions,
held consistent only by a `.min(4)` clamp. A 2M-sample sweep showed no
disagreement either way. The change makes consistency structural instead of
empirical.

## Suites

| Suite | Tests | Technique |
|---|---|---|
| `lagn-core/tests/regression.rs` | 23 | One or more per defect |
| `lagn-core/tests/boundaries.rs` | 16 | Boundary value analysis: every rasi, nakshatra, pada and navamsa edge ±1 arcsec, lagna cusp crossing, retrograde stations, clock and offset limits |
| `lagn-core/tests/invariants.rs` | 15 | Property-based, 600 seeded random charts, 1300-2390 AD, all latitudes |
| `lagn-ephem/tests/concurrency.rs` | 6 | 24-thread contention with mixed ayanamsas, lock poisoning, order independence |
| `lagn-cli/tests/cli.rs` | 21 | End to end on the real binary: exit codes, error messages, JSON contract, byte-identical output |
| existing unit, oracle and golden tests | 46 | |

### Checks on the tests themselves

- **Concurrency suite, mutation-tested.** With the mutex disabled, it failed in
  3 of 3 runs: one SIGABRT, one SIGSEGV, and one run with 4 of 6 tests failing
  on contaminated results. Without the lock, Swiss Ephemeris crashes the process.
- **Golden harness, mutation-tested.** A corrupted Moon pada is caught.
- **Seeded generator.** The random tests use a fixed-seed xorshift, so any
  failure reproduces exactly without a `rand` dependency.

## Known limitations (accepted, not defects)

- **Calendar cutover is fixed at 1582.** Britain and British India used the
  Julian calendar until 1752, so an Indian birth recorded 1582-1752 in a British
  document means a Julian date. The civil path reads it as Gregorian. Callers
  who know the source calendar should use `julian_day_ut(.., Calendar::Julian)`.
  Phase 4 input UI should ask for the calendar on pre-1752 dates.
- **Time resolution is about 40 µs** (one ULP of a modern Julian Day). That is
  under a milliarcsecond of lagna, so it doesn't matter. The floor is pinned by
  a test so it can't silently degrade.
- **An exact f64 boundary is ambiguous.** 10/3° has no binary representation.
  Boundary tests therefore probe ±1 arcsec. That is 1/12000 of a pada, and far
  finer than any real birth-time uncertainty.
- **Supported years are 1200-3000**, limited by the bundled `.se1` files.

## Not covered: JHora verification (open)

Every check above establishes internal consistency, agreement with Swiss
Ephemeris, or conformance to classical rules as written. None of them
establishes agreement with the software practitioners actually use. The Phase 1
exit criterion (50 JHora-verified golden charts) is still open. See
`tests/golden/README.md`.
