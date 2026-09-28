# Phase 4 QA report

Scope: `lagn-server` (HTTP API, place search, UTC offset suggestions, the
review gate at the boundary) and the web app in `web/`.
Specification: `docs/phase4/DESIGN.md` revision 3. Date: 2026-09-21.
Verdict: **pass**, with one item that can't be checked here (Linux deployment).

## Acceptance criteria

| # | Criterion | Result |
|---|---|---|
| 1 | Parity with the verified kernel | **Pass.** In-process: 1,000 random births, raw API output equals the library bit for bit (chart, analysis, Ashtakavarga, dasha), plus display strings checked against raw values. Live HTTP: 400 births x 7 endpoints equal the CLI's JSON with 8 concurrent clients. |
| 2 | Review gate at the boundary | **Pass.** Missing, empty, wrong, upper-cased, truncated and over-long tokens all get 403 on topic, match and review-sheet. A server without a token refuses review mode entirely. Production never returns drafts, whatever headers are sent. Tokens under 16 characters are refused at startup. |
| 3 | Offset N-version vs Python `zoneinfo` | **Pass after a design change: 127,214 cases, 0 disagreements.** 4,000 random plus 123,214 placed around transitions: 17,572 DST gaps, 26,090 overlaps, 530 local-mean-time cases. Asia/Kolkata's transitions are pinned from the system tz database. |
| 4 | Places | **Pass after two fixes.** Historical names resolve (Madras, Bombay, Calcutta, Trivandrum, Cochin). All 3,779 Indian places lie inside India's bounding box on Asia/Kolkata. Every zone name resolves. Search is deterministic and bounded. |
| 5 | Robustness | **Pass.** 400+ random byte mutations and 18 hand-written malformed bodies, on three endpoints: never a 5xx, always a JSON error. 413 for oversized bodies. JSON 404s. |
| 6 | Formatting and dates | **Pass.** A shared truncating `dms` never shows 60. A source scan fails the build if the frontend uses any `Date` API. Every date shown comes from the API. |
| 7 | Frontend | **Pass.** 37 unit tests. 12 browser end-to-end tests (desktop and phone) assert the rendered chart equals the API cell for cell, for all 16 vargas, the positions table, the dasha dates, Ashtakavarga and porutham. No horizontal page scroll. |
| 8 | No regressions | **Pass.** 257 Rust tests in release and debug; every earlier oracle green; clippy clean; `make qa` exits 0. |
| 9 | Linux deployment validation | **Open.** No Docker here. `make qa` must pass on the deployment host before going live. |

## Findings

| ID | Found by | Finding | Action |
|---|---|---|---|
| QA-P4-3 | Offset oracle | **The tz data compiled into the server was out of date.** `chrono-tz` carries tzdb 2025b, and it gave -08:00 for Tijuana in 1953 and 1961-75. tzdb 2025c corrected that to California's DST (-07:00 in summer). Traced to IANA's release notes, not assumed. No newer `chrono-tz` exists. | **Design changed (rev 3):** the server reads the operating system's tz database through `jiff`, which OS updates keep current. `/api/version` reports the version in use. Regression test pinned; it fails on a host with stale tzdata, by design. |
| QA-P4-1 | Rust suite | Airport codes indexed as place names: "mad" returned Madrid as an exact alias hit ("MAD") above Madurai. | Codes excluded at build time; regression test. |
| QA-P4-2 | Rust suite | "Calcutta" returned Calcutta, South Africa (pop. 35,864) above Kolkata (4.6 million): exact current names outranked exact historical names. | Current and historical names share a ranking tier, by population; the matched alias is shown. Spec updated. |
| QA-P4-4 | Developer visual check | The browser shows the date field in the OS locale, so 06/07/1985 means June 7 in a US locale and 6 July to an Indian reader, silently. | The form reads the entry back in words ("21 June 1985, 14:30:00, 24-hour clock"). |
| QA-P4-5 | Developer visual check | The web topic view omitted "Noted (not scored)" rules that the CLI shows, so a reviewer could never see them. | Added, with the CLI's exact selection rule; unit-tested. |
| QA-P4-6 | Developer | Oversized requests got a plain-text 413, against the all-errors-are-JSON rule. | JSON 413; tested. |
| QA-8 (Phase 1) | Developer | **The Phase 1 CLI could display 29°59'60.00"**: its degree formatter rounded. Display only; no computation, JSON value or fixture affected. | Replaced by the shared truncating `format::dms`; tested over a dense sweep. |
| - | Architect | Phase 1 docs gave Madras mean time as "+5:21:14 (5.3539 h)", unsourced. The tz database gives +5:21:10 (5.352778 h). | Corrected in code comments and CLI help. |

## Mutation checks

| Mutant | Caught by |
|---|---|
| `new Date()` added to the frontend | source-scan unit test |
| Historical offsets preselected | BirthForm unit test |
| Airport codes left in the index | `no_alternate_name_is_an_airport_code` |
| Stale tz data (2025b) | the offset oracle, and the pinned Tijuana test |

## Open items

1. **Linux deployment validation** (criterion 9). Run `make qa` on the target host.
2. **Host tzdata must be kept current**, an operational requirement now that
   the server reads the OS database.
3. **Licence** (unchanged): Swiss Ephemeris AGPL vs a hosted closed-source service.
4. **Content** (unchanged): all 34 rules and 10 poruthams are draft, so
   production shows nothing interpretive until astrologer review.

## Reproduce

```bash
make qa
python3 scripts/qa_offset_oracle.py --url http://127.0.0.1:PORT --random 4000 --zones 120
```
