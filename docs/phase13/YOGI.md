# Phase 13D design: Yogi, Avayogi and the Yoga sphuta

Status: built and through the QA gate. Small, and derivable; built to the
same bar as `YOGINI.md`.

## 1. Why this is safe to build

Three points, one offset, and two identities that hold for every chart.

The **Yoga sphuta** is the Sun's longitude plus the Moon's plus 93 degrees 20
minutes. That offset is not an arbitrary number: it is **exactly seven
nakshatras**, since a nakshatra spans 13 degrees 20 minutes and 7 x 13:20 =
93:20. So the Yogi's nakshatra is always the seventh past the nakshatra of
`Sun + Moon`, and the lord shifts seven places in the nine-lord Vimshottari
cycle.

The **Avayogi** is the lord of the sixth nakshatra from the Yogi's, which is
five nakshatras on - so the Avayogi's lord is **always five places past the
Yogi's lord** in that cycle, whatever the chart. Both facts are consequences
of the offsets rather than separate rules, which makes them properties a test
can assert.

## 2. The three quantities

| Quantity | Rule |
|---|---|
| Yoga sphuta | `Sun + Moon + 93:20`, modulo 360 |
| Yogi | the Vimshottari lord of the nakshatra containing the Yoga sphuta |
| Avayogi | the Vimshottari lord of the sixth nakshatra from the Yogi's, counting the Yogi's as the first |

The engine also reports the sphuta's sign and the nakshatra itself, because
practice quotes them alongside the lords.

**Held.** The *Duplicate Yogi* (Sahayogi) is not computed. Published accounts
differ on whether it is the graha conjoining the Yoga sphuta, the lord of its
sign, or the lord of its navamsa, and there is no identity to settle it. It
needs one transcribed worked example.

## 3. Variant register

| ID | Question | Default | Alternatives | Source of default |
|---|---|---|---|---|
| V-13-38 | The Yoga sphuta offset | 93 degrees 20 minutes, which is seven nakshatras exactly | no offset, taking `Sun + Moon` directly | the offset given wherever the Yogi is defined, and the only one that makes the seven-nakshatra identity hold |
| V-13-39 | Counting to the Avayogi | the 6th nakshatra from the Yogi's, counting the Yogi's as 1 | the 6th counting onward from the next; the 8th | the inclusive count is the convention used throughout this engine |
| V-13-40 | Duplicate Yogi | **not computed** | the graha conjoining the sphuta; the lord of its sign; the lord of its navamsa | the accounts disagree and no identity settles it |

## 4. Acceptance criteria (the QA gate)

1. **The offset is seven nakshatras exactly.** Asserted as the identity, so a
   mistyped 93:20 fails rather than shifting every chart quietly.
2. **The Yogi's nakshatra is seven past the sphuta's base** for a dense sweep
   of Sun and Moon positions.
3. **The Avayogi's lord is five places past the Yogi's** in the Vimshottari
   cycle, for every chart - an invariant that needs no reference output.
4. **Both lords are Vimshottari lords**, and the Avayogi differs from the Yogi
   (five places on in a nine-cycle can never return to the start).
5. **An independent reimplementation agrees** (`scripts/qa_yogi_oracle.py`),
   parsing section 2.
6. **The variants are on the output**, by ID, including the held one.
7. **Lite is unchanged.**

## 5. What the gate actually proved

- `crates/lagn-core/tests/yogi.rs`, 6 tests. The seven-nakshatra identity is
  asserted against the degrees-and-minutes value, so a mistyped 93.33 fails
  rather than shifting every chart by twelve arcseconds unnoticed. The
  five-place cycle shift is checked exhaustively across all 27 nakshatras and
  over 300 charts.
- `scripts/qa_yogi_oracle.py` builds the offset from degrees and arcminutes
  where the kernel builds it from nakshatra spans, and exits before comparing
  anything if the two do not agree. 150 charts, 600 values, 0 disagreements.
- `crates/lagn-server/tests/api.rs` re-checks the shift through the endpoint
  over 200 charts.
- Two deliberate mutations, each caught by both the suite and the oracle: the
  offset reduced to six nakshatras, and the Avayogi counted exclusively rather
  than inclusively.

## 6. Revision history

| Rev | Change | Raised by |
|---|---|---|
| 1 | Initial design | Architect |
| 2 | Section 5 added | Dev |
