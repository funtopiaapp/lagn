# Phase 13B design: Chara dasha

Status: built and through the QA gate. Specification for the Jaimini rasi dasha. Numbered
separately from `DESIGN.md` so that document's section numbers, which
13A's oracle and code comments cite, stay fixed.

## 1. Scope, and why it is one dasha rather than six

Section 3 of `DESIGN.md` lists 13B as "Chara, Narayana, Sthira, Shoola,
Brahma, Varnada". This increment delivers **Chara dasha only**, and the
reason is the same one that holds Shadbala back in phase 2.

Chara dasha has a rule I can state exactly and a disagreement I can bound:
the mainstream statement is consistent across Raman, K. N. Rao and the
standard Jaimini expositions, and where Narasimha Rao's implementation (and
therefore JHora's default) differs, it differs in a way that can be named and
registered as a variant. That is enough to build to the 99.9% bar.

The other five are not in that position for us yet:

| Dasha | What is missing |
|---|---|
| Narayana | the progression and antardasha rules beyond the opening sign, and the strength comparison that picks the starting sign |
| Sthira | the period table per sign nature, which published sources state differently |
| Shoola | the starting sign rule, and whether the 9-year period is uniform in every exposition |
| Brahma | the derivation of the Brahma sign itself |
| Varnada | the Varnada lagna computation, which needs the Hora lagna from 13C |

Writing those from recall would produce numbers that look authoritative and
cannot be checked, which is the one failure mode this project is built to
avoid. What unblocks them is the same thing phase 2 section 9 asks for:
a transcribed worked example per dasha, read off a page rather than recalled.
Varnada additionally waits on 13C, which supplies the Hora lagna it needs.

## 2. Definition of "accurate"

A Chara dasha is accurate when its sign order, each sign's length in years,
and the antardashas inside it match sections 3 and 4 below, and an
independent reimplementation agrees on every boundary.

Boundaries are dates, so they also depend on the year length the chart was
computed with. Chara dasha uses the chart's own `YearLength` - the same one
Vimshottari uses - so the two dashas of one chart can never disagree about
how long a year is.

## 3. The sequence

Let `L` be the lagna rasi.

| Lagna | Direction of the sequence |
|---|---|
| odd (Mesha, Mithuna, Simha, Tula, Dhanus, Kumbha) | zodiacal (direct) |
| even (Vrishabha, Karka, Kanya, Vrischika, Makara, Meena) | anti-zodiacal (reverse) |

The first mahadasha is `L` itself and begins **at birth**. There is no
balance to carry, unlike Vimshottari: a rasi dasha is reckoned from the
lagna, not from a fraction of a nakshatra already traversed.

After twelve signs the cycle repeats in the same order. The engine emits as
many cycles as are needed to cover 120 years from birth, which is the horizon
Vimshottari uses, because a single Chara cycle can be as short as twelve
years and a reading has to reach past it.

## 4. The length of a sign's dasha

For sign `S`, let `P` be the rasi occupied by the lord of `S`.

1. Count from `S` to `P`, counting `S` itself as 1, so the count is 1 to 12.
   The counting direction is **the parity of `S`**, not of the lagna:
   zodiacal when `S` is odd, anti-zodiacal when `S` is even.
2. The dasha is `count - 1` years.
3. **Exception.** When the lord of `S` occupies `S` itself the count is 1,
   which would give nothing. The period is then **12 years**.

So a sign's dasha is 1 to 11 years, or 12 when its lord is at home, and a
full cycle runs between 12 and 144 years.

| Case | Count | Years |
|---|---|---|
| lord in the sign itself | 1 | 12 |
| lord in the next sign along the counting direction | 2 | 1 |
| lord in the twelfth along the counting direction | 12 | 11 |

## 5. Antardashas

A mahadasha of `Y` years divides into twelve equal antardashas of `Y / 12`
years each. They begin with the mahadasha's own sign and run in the **same
direction as the main sequence**, which is the lagna's parity (section 3).

Equal twelfths, not proportional ones: a rasi antardasha has no period of its
own to be proportional to, which is what distinguishes this from Vimshottari.

## 6. Variant register

Each needs astrologer sign-off. The default applies until then and is
recorded on the output, exactly as in `DESIGN.md` section 7. The IDs continue
that document's series.

| ID | Question | Default | Alternatives | Source of default |
|---|---|---|---|---|
| V-13-9 | Direction of the sequence | by the lagna's odd/even parity | by whether the lagna's navamsa is odd-footed (savya/apasavya), which is JHora's default | the statement common to Raman and the standard expositions |
| V-13-10 | Direction used for counting a sign's length | the parity of that sign | the parity of the lagna, for every sign | counting "from the sign" is counting in that sign's own direction |
| V-13-11 | Antardasha direction | the main sequence's direction | the parity of the mahadasha's own sign | the sub-period follows the period it sits in |
| V-13-12 | Lord of a dual-lorded sign | sole traditional lord: Mangala for Vrischika, Shani for Kumbha | Ketu for Vrischika and Rahu for Kumbha; the stronger of the two | consistent with V-13-6, so one chart cannot use two lordship schemes |
| V-13-13 | Period when the lord is in its own sign | 12 years | 0 years, skipping the sign | a sign cannot have a zero-length dasha and remain in the sequence |

V-13-9 is the one that will most often explain a difference from JHora, so it
is named on the output rather than left to be discovered.

## 7. Architecture

```
lagn-core/src/chara.rs       the dasha itself                    (new)
lagn-cli                     `lagn chara`                        (extends)
lagn-server/src/api.rs       POST /api/chara                     (extends)
lagn-ffi/src/lib.rs          lagn_chara                          (extends)
web/src/components/CharaView.tsx   Pro surface only              (new)
```

It depends on `Chart`, `Rasi` and `YearLength`, and on nothing that reads a
clock, so the browser does no calendar arithmetic here either.

## 8. Acceptance criteria (the QA gate)

1. **The sequence is a permutation.** Every cycle visits each of the twelve
   signs exactly once, in the direction section 3 gives.
2. **Lengths are in range.** Every mahadasha is 1 to 12 years, and is 12
   exactly when the sign's lord occupies it.
3. **Periods tile the timeline.** Each mahadasha's end is the next one's
   start, to within a second, with no gap and no overlap; the first starts at
   birth.
4. **Antardashas tile their mahadasha**, twelve of them, equal to within a
   second, starting with the mahadasha's own sign.
5. **The horizon is covered.** The emitted periods span at least 120 years
   from birth, however short one cycle turns out to be.
6. **The year length is the chart's.** Changing a chart's `YearLength`
   changes Chara dasha boundaries in the same proportion as Vimshottari's.
7. **An independent reimplementation agrees.** `scripts/qa_chara_oracle.py`
   parses sections 3, 4 and 5 of this document and disagrees on no boundary
   across a wide random sweep.
8. **The variants are on the output**, by ID.
9. **Lite is unchanged**, and Chara dasha appears only under Pro.

## 9. What the gate actually proved

- `crates/lagn-core/tests/chara.rs`, 14 tests, covering criteria 1 to 6 and 8.
- `scripts/qa_chara_oracle.py`, an independent reimplementation that parses
  section 3's odd-sign list and section 4's case table out of this document at
  run time. 250 charts across three year lengths, 255,030 compared values,
  0 disagreements.
- `crates/lagn-server/tests/api.rs`, two tests: every period's displayed date
  equals what its own Julian Day denotes in the birth's offset, and every
  length obeys the count rule.
- The WebAssembly engine answers `lagn_chara`, so this works in the browser
  and not only against the server.
- Two deliberate mutations, each caught by both the Rust suite and the oracle:

  | Mutation | Rust suite | Oracle |
  |---|---|---|
  | length counted in the lagna's direction instead of the sign's | 2 tests fail | 5,672 disagreements |
  | antardashas opening on the lagna instead of the period's sign | 1 test fails | 3,300 disagreements |

**The oracle found a real defect, which the Rust suite had passed.** Period
boundaries were accumulated in Julian days, so by the twenty-fourth period the
running total sat an ulp short of the 120-year horizon; on a chart whose cycle
divides 120 exactly - a 60-year cycle - that started a third cycle which was
not needed, giving 36 periods where 24 cover the horizon. The fix accumulates
in *years*, which are small integers and therefore exact, and computes every
boundary from birth. A period's end and the next one's start now come from the
identical expression and are bitwise equal rather than merely close, and the
absolute dates no longer drift with distance from birth. Both the regression
and the exactness property are pinned by tests.

That is the whole argument for N-version checking: the suite tested the
properties the specification states, and the specification does not mention
floating-point accumulation.

## 10. Revision history

| Rev | Change | Raised by |
|---|---|---|
| 1 | Initial design; 13B scoped to Chara dasha alone, with section 1 recording why | Architect |
| 2 | Section 9 added; boundaries accumulate in years after the oracle found day-accumulation drift | QA |
