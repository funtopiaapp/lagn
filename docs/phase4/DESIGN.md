# Phase 4 design: web application

Status: approved for build. Revision 4 (2026-09-21): newest of shipped, OS and bundled tz data, with a release freshness gate; revision 3: tz data from the OS (QA-P4-3); place-search ranking specified (QA-P4-1, QA-P4-2).
Depends on Phases 1-3. Adds no astrology.

## 1. Principle: the web layer computes nothing

Every number, sign, date and verdict a user sees was produced by the kernel
already verified in Phases 1-3. The web layer only transports and displays.
That makes Phase 4's accuracy question simple: **does the web app show exactly
what the verified kernel produced?** Every part of this design serves that
question.

Three rules follow:

1. **No astrology in the browser.** The frontend never computes a sign, house,
   dignity or aspect.
2. **No calendar arithmetic in the browser.** Every date is formatted by the
   engine (`jd_to_civil`, in the birth's UTC offset), including dasha
   boundaries, timing windows and "running now". JavaScript's `Date` uses the
   proleptic Gregorian calendar, so it would reintroduce the pre-1582 defect
   fixed in Phase 1 (QA-7).
3. **No number formatting that changes a value.** Degrees reach the browser as
   engine-formatted strings, truncated rather than rounded, so 29 deg 59'59.9"
   never displays as 30 deg 00'00".

## 2. Architecture decision: server-side kernel, not WebAssembly

| Option | Verdict |
|---|---|
| **Rust HTTP server running the verified kernel, serving a static SPA** | **Chosen** |
| Kernel compiled to WebAssembly in the browser | Deferred |

**Why WebAssembly is deferred.** Swiss Ephemeris calls the platform's
`sin`/`cos`/`atan2`. A browser build would link a different maths library
(musl through Emscripten), which can differ in the last bit. That is a new
platform needing its own full validation run, and Emscripten isn't available
here. A browser build would also ship 5.8 MB of ephemeris to every visitor.
Server-side keeps the exact binary that passed QA.

**Deployment platform caveat.** All verification so far ran on macOS arm64. A
Linux server links glibc's maths library. Determinism per platform holds, but
**`make qa` must pass on the deployment platform before going live.** Docker
isn't available here, so that step is recorded as an open item, not claimed.

## 3. Time zone and place

The user supplies a date, a time and a place. The kernel needs a UTC offset,
and Phase 1 deliberately kept time-zone lookup out of the kernel. Phase 4
adds it as a **suggestion the user confirms**, never a silent conversion.

| Component | Choice |
|---|---|
| Places | GeoNames `cities15000` (every place with population 15,000 or more), with ASCII alternate names so Madras, Bombay, Calcutta, Trivandrum and Cochin are found. Licence CC BY 4.0, attribution shown in the app. |
| Offset | IANA tz database, read **from the operating system** via `jiff` (a bundled copy only on systems without one), resolved for the place's zone at the local birth date and time. The database in use and its version are reported by `/api/version` and in every suggestion. |
| Search ranking | Exact matches on the current or a historical name first, then prefix matches; population order within each tier; id as the final tie-break. Airport and station codes (3 or 4 capitals) are excluded from alternate names. |
| Manual entry | Latitude, longitude and offset can always be typed directly |

**Suggested offsets are labelled by confidence:**

| Case | Label | Shown to the user |
|---|---|---|
| Date on or after 1970-01-01 | `reliable` | the offset |
| Before 1970 | `historical` | IANA only guarantees data after 1970; confirm against the birth record |
| India, before 1970 | `historical` plus a note | the tz database models all of India as a single zone (`Asia/Kolkata`), so it can't represent local times that differed from that zone in particular cities; confirm against the birth record |
| Time falls in a DST gap | `nonexistent` | the time didn't exist; ask the user |
| Time falls in a DST overlap | `ambiguous` | both offsets shown; the user picks |

The confirmed offset is what's sent to the kernel, and it is echoed in every
result.

**Why the operating system's tz database (revision 3).** IANA publishes
corrections to *historical* offsets several times a year. QA found the first
design, which compiled `chrono-tz`'s copy (2025b) into the binary, wrong
for Tijuana births in 1953-1975. tzdb 2025c corrected Baja California's DST
history, and no newer `chrono-tz` release carries it. A compiled-in copy is
only as current as the last release of the app; the OS copy is kept current
by routine OS updates. Reproducibility is unaffected: the offset is a
suggestion, and the value the user confirms is stored with the chart.

**Freshness (revision 4).** Relying on the OS copy alone just moves the staleness
risk to the host: this Mac had 2026b while IANA was at 2026d, which corrects
Colombia 1992 and Iran 1979. The server now considers three copies and uses the
**newest by IANA release name**:

1. `data/zoneinfo`, compiled from IANA's latest sources by
   `scripts/update_tzdb.sh`, which builds `zic` from the same release and
   records the source checksums;
2. the operating system's copy;
3. jiff's bundled copy.

`make release-check` fails if the copy in use is older than IANA's latest, so
a stale release can't ship. `/api/version` reports every copy considered and
the one in use.

## 4. HTTP API

JSON in and out. Every error is `{"error": "..."}` with a meaningful status.

| Method, path | Purpose |
|---|---|
| `GET /api/health` | liveness |
| `GET /api/version` | engine, Swiss Ephemeris and tz database versions, target platform, corpus summary, whether review is enabled |
| `GET /api/places?q=&limit=` | place search |
| `GET /api/offset?tz=&date=&time=` | offset suggestion (section 3) |
| `POST /api/chart` | chart, 16 vargas with dignity, details, Ashtakavarga, dasha with engine-formatted dates, and the chain running now |
| `POST /api/topic/{name}` | topic report, production by default |
| `POST /api/match` | porutham report, production by default |
| `GET /api/review-sheet?topic=` | review sheet (review token required) |

**Birth input:**

```json
{"date": "1985-06-21", "time": "14:30:00", "latitude": 13.0827,
 "longitude": 80.2707, "utc_offset_hours": 5.5, "place": "Chennai"}
```

**The review gate at the API boundary.** A request for review mode must carry
`X-Review-Token` matching the token the server was started with. With no
token configured, review mode is disabled entirely. Drafts can never reach a
public visitor, however the request is crafted. Missing or wrong tokens get
403.

**Hardening.** 16 KB request body limit. Compute runs on a blocking pool. A
panic returns 500 without taking the process down. No CORS, because the SPA
is same-origin. Strict JSON: unknown fields are rejected.

## 5. Frontend

Vite, React and TypeScript, built to static files that the server serves.

**Screens:**

1. **Birth details.** Date, time, and a place search with autocomplete, or
   manual coordinates. The suggested offset is shown with its confidence label
   and must be confirmed or edited.
2. **Chart.** South Indian square chart (SVG) for any of the 16 vargas; a
   graha table with sign, degree, nakshatra and pada, house, dignity, avasthas
   and combustion; the dasha timeline with the running period highlighted; the
   Ashtakavarga table. Sanskrit and Tamil names throughout.
3. **Marriage.** The topic report. In production it says plainly that the
   interpretations are awaiting astrologer review. In review mode (token
   entered) drafts carry a DRAFT badge and show their traces, so the web app
   doubles as the astrologer's review tool.
4. **Match.** Two birth forms and the porutham table, gated the same way.

**Design:** readable over decorative. Light and dark themes, works at phone
width, keyboard accessible, no content conveyed by colour alone.

## 6. Architecture

```
crates/lagn-server/     axum HTTP server, view models, place index, tz suggestions
  src/view.rs           display-ready view models built from kernel output
  src/places.rs         gazetteer load and search
  src/offset.rs         IANA offset suggestions
data/places.tsv         derived from GeoNames (script in scripts/)
web/                    Vite + React + TS SPA
```

`lagn-core` gains `format::dms`, so the CLI and the server share one
truncating formatter.

## 7. Acceptance criteria (the QA gate)

1. **Parity.** For at least 1,000 random births, every raw value in the API
   response equals the verified library output bit for bit, and equals the
   CLI's JSON through a real HTTP server. The web layer adds or changes
   nothing.
2. **Gate at the boundary.** Review mode is impossible without the token (no
   token configured, missing, wrong, empty, differently cased) on every
   gated endpoint.
3. **Offset N-version.** Suggestions for at least 3,000 random (zone, date,
   time) cases agree with Python `zoneinfo`, an independent reader of the tz
   database. Any disagreement is traced to a tz database version difference
   or treated as a defect. Asia/Kolkata's transitions are tested directly,
   with expected values taken from the system tz database (2026b), not from
   memory: LMT, HMT +5:53:20, MMT +5:21:10 until 1906, IST, and +6:30 in
   1941-10 to 1942-05-15 and 1942-09 to 1945-10-15.
4. **Places.** Known cities resolve to correct coordinates (within 0.05
   degrees) and zones; historical names resolve; ranking is deterministic.
5. **Robustness.** Malformed, oversized, truncated and random JSON never
   crashes the server and always gets a 4xx.
6. **Formatting.** `dms` never shows 60 seconds or minutes, and truncates.
   Every date the UI shows comes from the API.
7. **Frontend.** Unit tests for the South Indian cell layout and the API
   client. An end-to-end browser test of the full flow, asserting that the
   rendered values equal the API's.
8. **No regressions:** `make qa` green.
9. **Not claimable here:** Linux deployment validation (no Docker). Recorded
   as open, not waived.
