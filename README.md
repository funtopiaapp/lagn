# lagn

A deterministic Vedic astrology engine. No LLM anywhere in the calculation or
interpretation path: the same birth data produces byte-identical output, every
time, and every conclusion can be traced to a rule and a source.

South Indian conventions throughout - whole-sign bhavas, Lahiri ayanamsa, mean
node, South Indian square charts, Tamil/Kerala interpretive tradition.

## Status: Phase 6 (the complete reading)

Every topic is read on the chart: marriage, career, wealth, education, health
(tendencies), vitality and care, children, parents, later life, and the
past-life reading (what the chart is held to carry forward, and how it meets
this life). The app also
gives sensitive periods (each dasha bhukti, with what amplifies it, what eases
it and the transits running through it), pariharams, and family readings
(spouse, children and parents, each on their own chart beside the native's).
Each reading is a written interpretation - the conclusion, then the houses,
their lords and the significators, then every classical finding with the chart
fact that justifies it - generated from fixed templates, never by a model.
`lagn topic <name> ... --writeup` prints one; `lagn periods ... --no-transits`
skips the transit overlay on long ranges.
Specs: `docs/phase2/`, `docs/phase3/`, `docs/phase6/DESIGN.md`. QA: `tests/QA-PHASE6.md`.

| Layer | | Status |
|---|---|---|
| 1 | Sidereal ephemeris | done, validated vs swetest |
| 2 | Rasi, bhava, all 16 vargas, dignity, relationships, drishti, avasthas, combustion, yuddha | done, 13,000-chart N-version check |
| 2 | Ashtakavarga (BAV, SAV, prastara) | done |
| 2 | Functional nature, sambandha, neecha bhanga, gochara (transits) | done; transits within 1 minute of swetest |
| 2 | Shadbala, Vimshopaka, remaining avasthas, shodhana | designed; blocked on reference data |
| 3 | Vimshottari dasha, 3 levels | done |
| 4 | Rule engine: language, 3-valued evaluator, review gate, cancellation, timing, period rules | done, N-version checked |
| 4 | Ten poruthams | done (Vasya blocked), exhaustively verified |
| 4 | Corpus: 11 topics, 201 rules, bhava meanings, karma tables, 14 pariharams | 198 approved, 3 rejected, by **AI review** at the owner's direction |
| 5 | Web app: HTTP API + React UI, place search, tz suggestions | done, parity-verified against the kernel |
| 6 | Installable PWA, offline shell, CORS for wrapped apps; conversion kit | done: `docs/phase5/CONVERSION.md` |
| 6 | Readings for all topics, sensitive periods, pariharams, family | done: `tests/QA-PHASE6.md` |
| 6 | Written interpretations: every reading as justified prose, not rule lists | done, template-generated and deterministic |
| 6 | Plain-language "what this means for you" on every rule | done, tone rules enforced by tests |
| 7 | Interface: app shell, light/dark with a manual choice, family verdict and tabs | done: `docs/phase7/DESIGN.md`, `tests/QA-PHASE7.md` |
| 8 | Past life (purva janma): merit carried forward, the Ketu-Rahu axis, and the bridge to the present-life readings | done: `docs/phase8/DESIGN.md`, `tests/QA-PHASE8.md` |
| 9 | The life carried forward (station, temperament), bonds (partner, children), and debts (rina and shapa) with their remedies | done: `docs/phase9/DESIGN.md`, `tests/QA-PHASE9.md` |
| 10 | Runs on the device with no server: a C ABI, an Android JNI layer, and a client that uses whichever transport exists | engine done and parity-tested; iOS/Android builds need Xcode and the NDK: `docs/phase10/DESIGN.md` |
| 10 | Narration (Tamil, Malayalam) | not started |

## Layout

```
crates/
  swe-sys/      raw FFI + vendored Swiss Ephemeris C sources
  lagn-ephem/   layer 1 - the only crate that touches Swiss Ephemeris
  lagn-core/    layers 2-3 - pure, no I/O, compiles to wasm/iOS/Android unchanged
  lagn-rules/   layer 4 - rule language, evaluator, review gate, poruthams,
                readings (periods, family, pariharams)
  lagn-cli/     validation CLI
  lagn-server/  HTTP API (axum) serving the web app; `api` holds the
                operations both transports call
  lagn-ffi/     C ABI and Android JNI, so a phone runs the engine with no
                server (`include/lagn.h`)
corpus/         reviewed rules per topic, topics.json, bhava.json, pariharam.json
web/            React PWA
ephe/           .se1 ephemeris data, 1200-2400 AD
tests/golden/   pinned charts (see that directory's README)
tools/swetest/  the reference CLI, built from the vendored sources as an oracle
scripts/        cross-validation and fixture generation
```

## Build and run

```bash
cargo build --release -p lagn-cli

./target/release/lagn chart \
    --date 1985-06-21 --time 14:30 \
    --lat 13.0827 --lon 80.2707 --tz 5.5 \
    --place Chennai --navamsa

./target/release/lagn dasha \
    --date 1985-06-21 --time 14:30 \
    --lat 13.0827 --lon 80.2707 --tz 5.5 --levels 2

./target/release/lagn dasha ... --on 2026-09-20   # what is running on a date
./target/release/lagn topic marriage ... --review  # draft rules, marked
./target/release/lagn match --bride "1992-03-15 06:20 13.08 80.27 5.5" --groom "..." --review
./target/release/lagn review-sheet > review.md     # for the astrologer
```

Add `--json` to either for machine-readable output.

## Run the web app

```bash
make build                                   # kernel CLI + server
(cd web && npm ci && npm run build)          # the SPA
./target/release/lagn-server --static web/dist
# open http://127.0.0.1:8080
```

Add `--review-token-file PATH` (a file holding a token of at least 16
characters) to let an astrologer sign in and see draft rules. Without it,
review mode is disabled entirely. The server reads the operating system's tz
database, so keep the host's `tzdata` package current.

## Deploy

```bash
make tzdb                          # refresh data/zoneinfo from IANA (before each release)
docker build -t lagn .             # production image, ~46 MB
docker run -p 8080:8080 lagn
docker build --target qa -t lagn-qa . && docker run --rm lagn-qa   # full QA on the target platform
make release-check                 # QA plus the tz freshness gate
```

Validated on Linux arm64 and x86-64: see `tests/QA-DEPLOYMENT.md`.

## Verification

Three independent layers of checking, because each catches what the others
cannot:

```bash
cargo test                              # unit + oracle + golden
bash tools/swetest/build.sh             # build the reference CLI
python3 scripts/crossvalidate.py        # 45 charts vs swetest
```

1. **Unit tests** check layers 2-3 against classical rules stated
   independently - the navamsa closed form is checked against the
   three-case textbook rule, the dasha tree against the proportional rule.
2. **Oracle tests** check layer 1 against `swetest`, the reference front end
   that ships with Swiss Ephemeris. Same C library, separate code path, so
   agreement confirms our flags and conventions rather than our arithmetic.
   Current worst deviation across 45 edge-case charts: **0.001 arcsec**.
3. **Golden fixtures** pin whole charts. These catch drift, but only prove
   correctness once a human has confirmed them in JHora - see
   `tests/golden/README.md`.

## Conventions, and why they are settings

Three choices change the numbers and are recorded on every chart rather than
hard-coded, because reasonable practitioners disagree:

- **Ayanamsa** - Lahiri by default.
- **Node** - mean by default, matching Tamil/Kerala panchangam practice and
  JHora. True node can put Rahu in a different nakshatra, which changes the
  birth dasha lord.
- **Dasha year length** - 365.25 days by default. Conventions range from 360
  to 365.256 days; across a full tree the spread exceeds 600 days. When our
  output disagrees with someone's family astrologer, check this first.

Timezones are deliberately **not** looked up from a database. A birth time
enters as a wall-clock reading plus the UTC offset in force at that place on
that date, and the offset is stored alongside the raw input. India had
multiple local times before 1906, Bombay kept its own until 1955, and wartime
IST was +6:30 - silently resolving that from a city name hides exactly the
ambiguity that most needs to be visible.

## Licence

AGPL-3.0-or-later, inherited from Swiss Ephemeris. **See `NOTICE.md` - this
must be resolved before any closed-source deployment.**
