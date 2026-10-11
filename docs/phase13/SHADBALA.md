# Phase 13E design: Shadbala

Status: built and through the QA gate, under the three constraints of
`DESIGN.md` section 6.

## 1. What this is, and the standing it ships with

`docs/phase2/DESIGN.md` held Shadbala from the start, for a reason worth
restating rather than paraphrasing: several sub-components have more than one
published formula, and no independent reference output is available here. The
99.9% target ruled out building numbers that could not be verified.

The product owner's decision was to build it anyway, with the alternatives
recorded and every number labelled until an astrologer signs it off. So this
ships under `DESIGN.md` section 6:

1. **Pro-only.** No bala appears on the Lite surface.
2. **Labelled at the number**, with the variant IDs it depends on, so a
   practitioner can see which choice produced what they are reading.
3. **No corpus rule may consume it.** Enforced by test. A reviewed
   interpretation resting on an unverified number would launder the caveat
   away, and the review gate exists to stop exactly that.

Nothing here claims agreement with JHora or with any book. What it claims is
that it matches this specification, which is a weaker and honest claim.

## 2. Units

Strength is in **virupas**. Sixty virupas make one **rupa**, and the component
maxima below are in virupas throughout. Totals are reported in both.

## 3. The six components

### 3.1 Sthana bala (positional), five parts

| Part | Rule | Max |
|---|---|---|
| Uchcha | `60 - (angular distance from the deep debilitation point) / 3`, the distance taken the short way round so it is 0 to 180 | 60 |
| Saptavargaja | the dignity ladder below, summed over D-1, D-2, D-3, D-7, D-9, D-12 and D-30 | 315 |
| Ojayugma | 15 for each of rasi and navamsa whose parity suits the graha: odd for Surya, Kuja, Guru, Budha and Shani, even for Chandra and Shukra | 30 |
| Kendra | 60 in a kendra (1, 4, 7, 10), 30 in a panapara (2, 5, 8, 11), 15 in an apoklima (3, 6, 9, 12) | 60 |
| Drekkana | 15 when the graha is in the drekkana of its own kind: 1st for the male Surya, Kuja and Guru; 2nd for the neutral Budha and Shani; 3rd for the female Chandra and Shukra | 15 |

**The dignity ladder** used by Saptavargaja. This is a table, not a rule: the
lower four rungs halve, but 45 to 30 to 22.5 does not, so the values are
stated rather than generated. (An earlier revision of this document described
the whole ladder as halving, which the test suite refuted.)

| Dignity | Virupas |
|---|---|
| Moolatrikona | 45 |
| Own sign | 30 |
| Great friend | 22.5 |
| Friend | 15 |
| Neutral | 7.5 |
| Enemy | 3.75 |
| Great enemy | 1.875 |

Exalted is read as moolatrikona and debilitated as great enemy, because the
seven-level ladder has no rung for either (V-13-26).

### 3.2 Dig bala (directional)

Each graha is strongest on one of the four angles and weakest on the opposite:

| Graha | Strongest at the | Weakest at the |
|---|---|---|
| Guru, Budha | 1st (east) | 7th |
| Surya, Kuja | 10th (south) | 4th |
| Shani | 7th (west) | 1st |
| Chandra, Shukra | 4th (north) | 10th |

`Dig bala = 60 x (1 - d / 180)`, where `d` is the angular distance from the
strongest point, taken the short way round. So 60 at the strongest point, 0 at
the weakest, linear between. Max 60.

### 3.3 Kala bala (temporal), eight parts

| Part | Rule | Max |
|---|---|---|
| Nathonnatha | `60 x (1 - d / 12h)` where `d` is the distance in hours from the graha's weak moment: midnight for Surya, Guru and Shukra, midday for Chandra, Kuja and Shani. Budha always 60 | 60 |
| Paksha | the Moon's elongation from the Sun, divided by 3, for a benefic; `60 -` that for a malefic | 60 |
| Tribhaga | 60 to the ruler of the third of the day or night the birth falls in: by day Budha, Surya, Shani; by night Chandra, Shukra, Kuja. Guru always 60 | 60 |
| Abda | 15 to the lord of the year | 15 |
| Masa | 30 to the lord of the month | 30 |
| Vara | 45 to the lord of the weekday | 45 |
| Hora | 60 to the lord of the hora | 60 |
| Ayana | from declination; see below | 60 |

**Ayana bala.** `Ayana = 60 x (23.45 + k x decl) / 46.9`, where `decl` is the
graha's declination and `k` is `+1` for Surya, Kuja, Guru and Shukra, `-1` for
Chandra and Shani, and `+1` with the result doubled for Budha, which is strong
in either direction (V-13-30).

### 3.4 Cheshta bala (motional)

`60 x (|graha speed - mean speed| / mean speed)` capped at 60, with a
retrograde graha taking the full 60 (V-13-31). This is the sub-component
`docs/phase2/DESIGN.md` names first among the disputed ones, and the formula
above is one of several published. Surya and Chandra never retrograde and take
their Cheshta from Ayana instead, which is the common convention.

### 3.5 Naisargika bala (natural)

Fixed, and a clean series: `60 x n / 7` for `n` from 7 down to 1.

| Graha | n | Virupas |
|---|---|---|
| Surya | 7 | 60 |
| Chandra | 6 | 51.43 |
| Shukra | 5 | 42.86 |
| Guru | 4 | 34.29 |
| Budha | 3 | 25.71 |
| Kuja | 2 | 17.14 |
| Shani | 1 | 8.57 |

That it is exactly `60 n / 7` is a property a test asserts, rather than seven
numbers to transcribe.

### 3.6 Drik bala (aspectual) - held

**Not computed.** Drik bala is the net of *sphuta* drishti, which is
degree-based: each aspect's strength is a function of the exact longitudinal
difference, through a piecewise curve. That curve is the second item
`docs/phase2/DESIGN.md` section 9 names as disputed, and it is not something
this project can state from general knowledge.

The engine already has whole-sign graha drishti, and it would be easy to use
it here and call the result Drik bala. That would be wrong in a way nothing
could detect: a sign-based stand-in for a degree-based quantity produces a
plausible number with no error bar. So Drik bala is reported as absent, and
the total is reported as the sum of **five of the six** components, labelled
as such wherever it appears.

What unblocks it is one transcribed sphuta drishti table or formula, read off
a page.

## 4. Totals and thresholds

`Shadbala = Sthana + Dig + Kala + Cheshta + Naisargika`, in virupas, and in
rupas divided by 60 - **five of the six**, since Drik is held (section 3.6).
Every total says so.

The customary minimum strengths are listed for reference only, never as a
verdict, and they are the thresholds for a *complete* Shadbala - so a total
computed from five components must not be compared against them. The engine
reports them alongside but does not test against them:

| Graha | Rupas |
|---|---|
| Surya | 5 |
| Chandra | 6 |
| Kuja | 5 |
| Budha | 7 |
| Guru | 6.5 |
| Shukra | 5.5 |
| Shani | 5 |

## 5. Variant register

Every one of these is unsigned-off, and the engine labels each number with the
IDs it depends on.

| ID | Question | Default | Alternatives | Source of default |
|---|---|---|---|---|
| V-13-26 | Exalted and debilitated on the seven-rung Saptavargaja ladder | read as moolatrikona and great enemy | a separate rung above and below | the ladder has seven rungs and nine dignities exist |
| V-13-27 | Mercury's nature for Paksha bala | benefic | malefic; benefic only when unassociated with a malefic | the first disputed item in phase 2 section 9 |
| V-13-28 | Lord of the year, for Abda bala | the weekday lord of the first day of the solar year containing the birth | several published reckonings | phase 2 section 9 names this as disputed |
| V-13-29 | Lord of the month, for Masa bala | the weekday lord of the first day of the solar month containing the birth | several published reckonings | as V-13-28 |
| V-13-30 | Hora length, and Budha's Ayana sign | equal horas of one hour; Budha doubled and always positive | unequal horas of one twelfth of the day; Budha signed like the rest | phase 2 section 9 names hora length as disputed |
| V-13-31 | Cheshta bala formula | speed deviation from the mean, retrograde taking 60 | the eight-fold Cheshta states; the arc-of-retrogression method | phase 2 section 9 names this as disputed |
| V-13-32 | Drik bala | **not computed**; the total is five of six and says so | a sign-based stand-in for sphuta drishti | phase 2 section 9 names sphuta drishti as disputed, and a sign-based substitute for a degree-based quantity cannot be checked |

## 6. Acceptance criteria (the QA gate)

1. **Every component is inside its range.** No sub-component exceeds the
   maximum section 3 gives it, and none is negative.
2. **Naisargika is exactly `60 n / 7`.** Asserted as the formula, not as seven
   transcribed numbers.
3. **Uchcha is 60 at deep exaltation and 0 at deep debilitation**, for all
   seven, and symmetric either side.
4. **Dig bala is 60 at the strongest angle and 0 at the opposite**, for all
   seven.
5. **The ladder is the table in section 3.1**, rung for rung, and the lower
   four rungs do halve - which is as much regularity as it has.
6. **The total is the sum of the five computed components**, in virupas, the
   rupa figure is the virupa figure over 60, and the result states that Drik
   is absent rather than silently omitting it.
7. **Every number carries its variant IDs.** A sub-component that depends on
   a disputed choice names it, and the test fails if any of the seven IDs in
   section 5 is unreferenced.
8. **No corpus rule consumes a bala.** Enforced by a test that reads the rule
   corpus and fails on any condition referring to strength.
9. **Lite never shows a bala**, and the Pro surface labels every one.
10. **An independent reimplementation agrees** on the components that are not
    variant-dependent (`scripts/qa_shadbala_oracle.py`).

## 7. What the gate actually proved

- `crates/lagn-core/tests/bala.rs`, 10 tests. No sub-component leaves its
  range over 120 charts; Naisargika is asserted as `60 n / 7` rather than as
  seven transcribed numbers; the deep exaltation points are pinned against
  phase 2's own table; Dig bala reaches both 60 and 0 across a sweep.
- `crates/lagn-rules/tests/no_unverified_strength.rs`, which enforces
  criterion 8 in both directions: no corpus file mentions a strength term,
  **and** the rule language contains no way to express one. The bar is a
  constraint rather than a convention.
- The WebAssembly engine answers `lagn_bala`, and the smoke test fails if the
  caveat or the held-component flag does not survive into the browser. A
  reader who sees these numbers must see their standing too.

**The declination transform is checked against the sky, not itself.** Ayana
bala needs a declination, which needed an ecliptic-to-equatorial conversion
this engine had no reason to compute before. The test asserts the Sun reaches
+obliquity at the June solstice, -obliquity in December and zero at the
equinoxes, and that a real 21 December chart puts the Sun within half a degree
of -23.44. That is a check from outside the code.

**A test refuted this document.** Revision 1 described the Saptavargaja ladder
as "halving from moolatrikona down", and the test written to that description
failed on 45 to 30, which is two thirds. The ladder is a table whose lower four
rungs halve, and section 3.1 now says so. It is worth recording that the
specification was wrong and the suite caught it, rather than quietly
correcting the sentence.

## 8. Revision history

| Rev | Change | Raised by |
|---|---|---|
| 1 | Initial design, under the three constraints of DESIGN.md section 6 | Architect |
| 2 | Section 3.1: the Saptavargaja ladder is a table, not a halving rule. The first revision claimed it halved throughout; 45 to 30 does not | QA |
| 3 | Section 3.6: Drik bala held rather than approximated with sign-based drishti | Architect |
| 4 | Section 7 added | Dev |
