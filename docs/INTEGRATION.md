# lagn: what is built, and how to drive it from another app

Written for an engineer building a native client (iOS or otherwise) against
this engine. It covers every feature that exists today, the two surfaces
(Lite and Pro), the integration surface, and the professional-surface kill
switch.

Canonical as of the commit that contains it. The authority for any detail is
the code and the phase designs under `docs/`; where this document and the
code disagree, the code is right and this document is stale.

---

## 1. Shape of the thing

```
  Swift / Kotlin / JS UI
            │   JSON in, JSON out
  ┌─────────┴──────────┐
  │  C ABI (lagn-ffi)  │   include/lagn.h — 20 functions
  └─────────┬──────────┘
            │
   lagn-rules   layer 4: rule language, review gate, poruthams, readings
   lagn-core    layers 2-3: chart derivation, dashas, varga, Jaimini, KP…
   lagn-ephem   layer 1: Swiss Ephemeris, time, timezones
```

Three facts that shape any client:

1. **The engine is local.** There is no server in production. A client links
   the engine and calls it directly; the web app compiles the same engine to
   WebAssembly. Birth details never leave the device.
2. **Everything is deterministic.** Same inputs, same bytes out, forever. No
   clock, no randomness, no network inside a reading.
3. **Reference data ships with the engine**: the Swiss Ephemeris `.se1`
   files, the IANA timezone database, the reviewed rule corpus and a place
   index. A client must bundle these and tell the engine where they are.

---

## 2. The C ABI

`include/lagn.h` is the contract and is kept in exact parity with the Rust by
a test (`crates/lagn-ffi/tests/parity.rs`), so it cannot drift.

Every call takes a NUL-terminated UTF-8 JSON string and returns a
heap-allocated NUL-terminated UTF-8 JSON string **that the caller must free
with `lagn_string_free`**. An error comes back as `{"error": "..."}` with the
same shape, so there is one error path.

### Lifecycle

| Function | Purpose |
|---|---|
| `lagn_init(json)` | Once at start-up. Give it the paths to the bundled data. |
| `lagn_ready()` | Non-zero once init has succeeded. |
| `lagn_version()` | Engine, Swiss Ephemeris, tz database and corpus versions. |
| `lagn_string_free(ptr)` | Free any returned string. |
| `lagn_buffer_alloc` / `lagn_buffer_free` | Only needed by the WebAssembly loader; a native caller passes C strings directly. |

`lagn_init` takes:

```json
{ "ephemeris": "<dir>/ephe", "corpus": "<dir>/corpus", "places": "<dir>/data/places.tsv" }
```

### Lite features — safe for any audience

| Function | Request | Returns |
|---|---|---|
| `lagn_chart` | `{"birth": {...}}` | the chart, 16 vargas, the Vimshottari tree, Ashtakavarga summary |
| `lagn_topics` | — | the topic catalogue and bhava meanings |
| `lagn_topic(name, json)` | `{"birth":…, "sex"?, "from_age"?, "to_age"?}` | one topic's reading, as justified prose |
| `lagn_periods` | `{"birth":…, "from_age":n, "to_age":n}` | every antardasha in range, with transits |
| `lagn_days` | `{"latitude":…, "longitude":…, "utc_offset_hours":…, "days"?, "offset_days"?}` | panchanga, Rahu kalam, Yama gandam, Kuligai, Abhijit. Needs no chart. |
| `lagn_family` | `{"native":…, "member":…, "relation":"…"}` | a family member read beside the native |
| `lagn_match` | `{"bride":…, "groom":…}` | all twelve poruthams, doshams, pariharams-free |
| `lagn_places` | `{"q":"…", "limit":n}` | place search |
| `lagn_offset` | `{"tz":"…", "date":"…", "time":"…", "longitude"?}` | the UTC offset in force, with DST gaps and overlaps surfaced |

### Pro features — professional surface only

| Function | Returns | Spec |
|---|---|---|
| `lagn_jaimini` | 8 chara karakas, 12 arudha padas, argala/virodhargala | `docs/phase13/DESIGN.md` |
| `lagn_chara` | Chara dasha, maha and antar, to a 120-year horizon | `CHARA-DASHA.md` |
| `lagn_upagraha` | 10 upagrahas (incl. Gulika) and 3 time lagnas | `UPAGRAHA.md` |
| `lagn_yogi` | Yoga sphuta, Yogi, Avayogi | `YOGI.md` |
| `lagn_kp` | four lords per point, 12 Placidus cusps, 5 ruling planets, house significators | `KP.md` |
| `lagn_varsha` | `{"birth":…, "age":n}` → annual (solar return) chart, Muntha, kaksha transit | `VARSHA-KAKSHA.md` |
| `lagn_bala` | Shadbala, **five of six components** — see §5 | `SHADBALA.md` |
| `lagn_yogini` | Yogini dasha, the 36-year cycle | `YOGINI.md` |
| `lagn_ashtakavarga` | the full BAV/SAV grid with per-cell contributors | `VARSHA-KAKSHA.md` |

### The `birth` object

```json
{
  "date": "1981-12-21",          // YYYY-MM-DD, Julian calendar before 1582-10-15
  "time": "14:10:00",            // local wall clock, HH:MM or HH:MM:SS
  "latitude": 8.8932,            // north positive
  "longitude": 76.6141,          // east positive
  "utc_offset_hours": 5.5,       // the CONFIRMED offset; never looked up silently
  "place": "Kollam",             // optional, for display only
  "settings": {                  // optional
    "ayanamsa": "lahiri",        // lahiri | lahiri_icrc | true_chitra | raman |
                                 // krishnamurti | true_revati | yukteshwar
    "node_type": "mean",         // mean | true
    "house_system": "whole_sign",
    "year_length": "julian"
  }
}
```

**`utc_offset_hours` is deliberately required.** The engine will not guess a
historical offset: `lagn_offset` exists so a client can *ask* and then show
the answer for confirmation. A wrong offset silently rotates the whole chart,
so this is the one place the design refuses to be convenient.

### An HTTP mirror

`lagn-server` exposes the same operations over HTTP at the same JSON shapes
(`/api/chart`, `/api/kp`, `/api/bala`, …). It exists for development and
parity testing; a shipped client does not need it. A test asserts the two
transports return byte-identical output.

---

## 3. Lite and Pro

Two surfaces over one engine. The rules, from `docs/phase13/DESIGN.md` §2:

| | Lite (default) | Pro |
|---|---|---|
| Audience | the general public | astrologers |
| Language | plain, justified prose | technical terms, unglossed |
| Numbers | scores and verdicts, explained | raw values, intermediate components, variant IDs |
| Unverified output | never shown | shown, labelled |

1. **Lite is the default.** A first-time user never lands in Pro.
2. **Pro is presentation, not computation.** Both surfaces call the same
   functions. A number cannot differ between them; Pro shows more of the same
   numbers.
3. **The choice is device-local** and must not travel in a URL or a share
   link — a shared link must not drop a lay reader into the professional
   surface.
4. **Anything whose variant is unsigned-off is Pro-only and labelled.**
5. **No Lite feature regresses when Pro is enabled.**

A native client should mirror this: one setting, stored on the device,
defaulting to Lite, absent from any deep link.

### The Pro tabs, in the web app's order

Jaimini · Chara dasha · Points (upagrahas, time lagnas, Yogi/Avayogi) · KP ·
Annual chart · Shadbala · Yogini dasha · Ashtakavarga

---

## 4. The Pro kill switch

The owner can turn the professional surface off entirely, and it stays off
until explicitly turned back on.

### Web

One environment variable at **build** time:

```
VITE_PRO=off     # off | 0 | false | no   → surface off
VITE_PRO=on      # anything else, or unset → surface on
```

Unset means **on**, so forgetting the variable can never silently remove
features; only an explicit off does anything.

To flip it for the hosted app, set it in the build step of
`.github/workflows/pages.yml` and let the deploy run:

```yaml
      - name: Build the web app
        env:
          VITE_PRO: off          # ← add or remove this line
        run: npm run build
```

### What "off" actually does

| | Pro on | Pro off |
|---|---|---|
| Lite/Pro toggle in the header | shown | **not rendered** |
| Professional tabs | 8 | **0** |
| Professional panels | reachable | **unreachable** |
| `ProPanels` JavaScript chunk | emitted, ~36 KB, fetched on first use | **not emitted at all** |
| Tab names in the bundle | present | **absent** |
| A device that remembers `pro` | honoured | **put back on Lite** |
| Writing `pro` to storage | honoured | **ignored** |

Turning it off and back on does **not** silently restore Pro for readers who
had chosen it before: they choose again. "Explicitly enable" applies at both
ends.

### What "off" does *not* do, stated plainly

- **The engine keeps its Pro exports.** `lagn_kp`, `lagn_bala` and the rest
  remain in the compiled engine. Gating them would mean a second engine build,
  and the engine is shared with the native clients — so the flag is a product
  switch, not a capability switch.
- **A few API-client method names remain** in the web bundle (`/api/kp` and
  similar), because they live on one shared `api` object that Lite also uses.
  No UI reaches them.

If you need the capability itself gone — for a paid tier, say — that is a
different job: a Cargo feature on `lagn-ffi` that omits the Pro `endpoint!`
macros, producing a genuinely smaller engine. Not built.

### iOS

Mirror the same semantics: a build setting (or remote config) that, when off,
hides the mode control, removes the professional tabs, and coerces a stored
`pro` back to Lite. Honour the "does not silently restore" rule. Keep the
default on so a missing setting never removes features.

---

## 5. Where the numbers stand — read this before shipping a Pro UI

Not every professional figure has the same standing, and a client must not
flatten the difference.

### Verified the same way as everything else

Jaimini core, Chara dasha, upagrahas and the time lagnas, Yogi/Avayogi, KP,
the annual chart and Muntha, kaksha transit, Yogini dasha, the Ashtakavarga
grid.

Each has: a spec with a variant register, a Rust suite, and an **independent
reimplementation in Python** that parses the spec at run time and is diffed
over thousands of charts, plus deliberate mutations proving both catch errors.

Several carry self-checks worth surfacing in a UI, because they let a reader
verify rather than trust:

- **KP's 249 sub-divisions** are *derived*, not quoted: 243 subs, 6 straddling
  a sign boundary. Both implementations land on 249 independently.
- **Upaketu is always exactly 30° behind the Sun** — a consequence of the five
  upagraha offsets composing.
- **The Avayogi's lord is always five places past the Yogi's** in the
  Vimshottari cycle, for every chart.
- **Yogini's periods are 1–8 summing to 36**, which is the cycle length.
- **Every BAV comes to a fixed total, and the SAV always to 337.** The web UI
  prints the fixed total beside the computed one for this reason, and raises an
  alarm if they ever differ. A native UI should do the same.
- **The annual chart's Sun equals the natal Sun** to a milliarcsecond. The
  residual is in the response (`sun_error`); show it.

### Labelled, and must stay labelled: Shadbala

`lagn_bala` returns **five of the six** components. Drik bala is **not
computed**: it needs sphuta drishti, which is degree-based and disputed, and a
sign-based substitute would produce a plausible number with no error bar.

The response carries `drik_included: false`, `components_computed: 5` and a
`caveat` string. **A client must show the caveat and must not present these as
verified numbers.** The `customary_minimum_rupas` field is the threshold for a
*complete* Shadbala and is therefore **not comparable** to these totals — show
it greyed, for reference, and never test against it.

No reviewed interpretation anywhere in the app rests on a bala. That is
enforced by `crates/lagn-rules/tests/no_unverified_strength.rs`, which fails if
any corpus rule names a strength or if the rule language gains a way to express
one. Preserve that boundary: do not feed balas into anything a user reads as a
conclusion.

### Variant registers

Forty variant choices (V-13-1 … V-13-40) are recorded across the phase 13
designs. **None has been signed off by an astrologer.** Each response carries
the IDs it depends on, and the web UI prints them under every Pro view. A
native client should do the same — a practitioner comparing against other
software needs to know which scheme produced a number.

The ones most likely to explain a difference from other software:

| ID | What it decides |
|---|---|
| V-13-1 | 8 chara karakas or 7 |
| V-13-6 / V-13-12 | the lord of a dual-lorded sign (Vrischika, Kumbha) |
| V-13-9 | Chara dasha's direction: lagna parity, or navamsa foot |
| V-13-14 | which moment of its part gives an upagraha |
| V-13-18 | KP's ayanamsa: the chart's own, or forced Krishnamurti |
| V-13-33 | Yogini's starting rule |

### Not built, and why

Each of these needs **one transcribed worked example, read off a page** — not
recalled, and not assembled from websites:

| Held | What is missing |
|---|---|
| Drik bala / sphuta drishti | the degree-based aspect-strength curve |
| Narayana, Sthira, Shoola, Brahma, Varnada dashas | progression and start rules |
| Ashtottari and the other conditional dashas | the periods are corroborated, but the nakshatra→lord mapping is not; the most detailed source says it is pada-based over **28** nakshatras, which this engine does not model |
| Kalachakra dasha | the whole scheme |
| Varshaphal sahams (~50), Varshesha | fifty formulas; and Panchavargeeya bala |
| Sodhya pindas, trikona/ekadhipatya sodhana | the multiplier tables |
| Sree, Pranapada, Indu lagnas | how the Moon's nakshatra fraction is applied |
| Vargas D-5, D-6, D-8, D-11, D-81, D-108, D-144 | start-sign conventions |
| Vimshopaka / Dasavarga weights | the weight sets differ |
| Duplicate Yogi | the accounts disagree |

**Explicitly out of scope** by the owner's instruction: Mrityu Bhaga, the 64th
navamsa, Tajaka, and dasa-pravesha charts.

### The withdrawn feature

**Pariharams (remedial measures) have been removed from the product** at the
owner's direction: nothing recommends a remedial practice to anyone. The
catalogue, the engine module, the API field, the UI and the PDF section are
all deleted, and the rule prose that prescribed observances was rewritten.

Four guards enforce it, over the shipped corpus text *and* the generated prose.
**A native client must not reintroduce remedies** — not from its own strings,
not from an older corpus copy. `crates/lagn-rules/tests/no_remedies.rs` is the
reference for the vocabulary that is banned.

Day timings (Rahu kalam, Yama gandam, Kuligai) are **kept**: those state which
hours tradition avoids, which is a panchangam fact rather than a practice to
perform.

---

## 6. Rules a client must not break

These are not style preferences; each exists because breaking it produced a
real bug.

1. **The client does no calendar arithmetic.** Every date a user sees is
   formatted by the engine. The web app enforces this with a test
   (`noDateMath.test.ts`) and exempts exactly one file. The engine returns
   Julian Days *and* formatted strings — use the strings.
2. **Never guess a UTC offset.** Ask `lagn_offset`, show the answer, let the
   user confirm. DST gaps and overlaps are surfaced rather than resolved.
3. **Free every returned string** with `lagn_string_free`.
4. **Check the engine's exports at start-up.** The web loader verifies every
   function it will call and fails with an actionable message, because a stale
   engine against new UI code once surfaced as "not a function" inside a
   feature. A native client should assert the same.
5. **Version the engine's data.** The web app appends a content hash to every
   engine asset URL, because a cache served the previous build's WebAssembly to
   new JavaScript. A native client bundles its engine, so this is moot — but
   if you ever fetch engine data over the network, version the URL.
6. **Review-gated content.** Rules carry a review status; production shows
   approved only. Do not add a path that shows draft content without the
   reviewer token.
7. **Health, legal and financial disclaimers** travel with the readings that
   need them (`disclaimer` on the topic metadata). Show them.

---

## 7. Verification, so you can judge the engine's claims

| Gate | What it covers |
|---|---|
| `make qa` | clippy, release and debug test suites, every oracle, corpus validation, the web suite, the browser suite |
| `make release-check` | the above, plus the tz database must be IANA's latest |

Current totals: **74 Rust suites, 248 web tests, 52 browser tests**, and
independent-reimplementation oracles agreeing on, among others: 1,336,000
phase-2 checks, 139,968 porutham verdicts, 304,020 Chara dasha values, 113,600
Jaimini values, 69,792 Yogini values, 14,400 KP values, 56,494 timezone cases —
all with **zero disagreements**.

The oracles matter more than the counts. Each is a second implementation
written against the specification text, which it **parses at run time** rather
than having the tables retyped into it — so it checks the Rust against the
document instead of against a second hand-copy that could share an error.
Several refuse to run at all if the spec they parse fails its own structural
check.

Mutation testing is part of the method: a deliberate error is introduced and
both the suite and the oracle must catch it. That has twice found a weakness in
the *tests* rather than the code — a self-referential assertion, and an
ordering that was not single-sourced — and both are recorded in the phase
designs.

---

## 8. Licence

AGPL-3.0. Section 13 applies: anyone using this over a network is entitled to
the source of the version they are using. A native app that merely links the
engine still distributes it, so the obligation stands — ship the licence and an
offer of source.
