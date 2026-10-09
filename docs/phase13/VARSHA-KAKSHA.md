# Phase 13F/13G design: kaksha transit, and the annual chart

Status: built and through the QA gate. Two small groups, both derivable, specified together.

## 1. Scope, and the line through each group

### 13G Varshaphal

| Built here | Why it is safe |
|---|---|
| The annual (solar return) chart | found by solving for the moment the Sun regains its exact natal longitude. The solve is self-checking: the Sun at the answer must equal the natal Sun |
| Muntha | one sign per completed year from the natal lagna. Arithmetic |

| Held | What is missing |
|---|---|
| Varshesha (the year lord) | the Panchadhikari procedure ranks five candidates by Panchavargeeya bala, which is a table-driven strength scheme this engine does not have |
| The sahams | roughly fifty separate formulas. Each is a specific arithmetic on specific points, and recalling fifty of them is exactly the failure this project is built to avoid |
| Patyayini and Mudda dasha | the starting lord differs between expositions |
| Harsha and Panchavargeeya bala | table-driven, like Shadbala's components |

### 13F Ashtakavarga depth

| Built here | Why it is safe |
|---|---|
| Kaksha transit | each sign divides into eight kakshas of 3 degrees 45 minutes, one per Ashtakavarga contributor. Whether a transit is supported is read straight off the `prastara` bitmaps this engine already computes and cross-validates |

| Held | What is missing |
|---|---|
| Sodhya pinda | the rasi and graha multiplier tables |
| Trikona and ekadhipatya sodhana | the reduction procedures vary, and the published worked examples disagree with one another |

## 2. The annual chart

For the year in which the native completes age `n`, the annual chart is cast
for the moment the Sun's sidereal longitude next equals its natal value, on or
after the `n`th birthday.

**Solving it.** The Sun advances slightly under a degree a day and never
reverses, so the moment is unique within a year and can be bracketed. The
engine brackets a day either side of the nominal anniversary and bisects on
the signed difference from the natal longitude, wrapped to `[-180, 180)` so
the crossing of 360 is not a discontinuity.

**The check that makes this safe.** The answer is verified, not assumed: the
Sun's longitude at the returned moment must equal the natal Sun's to within a
milliarcsecond. A solver that converged on the wrong root, or a bracket that
missed, fails that test rather than producing a plausible chart.

**Place.** The annual chart is cast for the birth place (V-13-22). Casting it
for where the native now lives is also practised, and the engine will take any
place once there is somewhere to ask for one.

## 3. Muntha

Muntha sits in the natal lagna at birth and advances one sign for each
completed year:

```
Muntha sign for age n = natal lagna + n signs
```

So the annual chart for age 1 has Muntha in the 2nd from the natal lagna, and
after twelve years it returns. The engine reports the sign and the house it
occupies in the annual chart, which is what practice reads.

## 4. Kaksha transit

Each sign divides into eight equal kakshas of 3 degrees 45 minutes. The eight
belong, in order from the start of the sign, to:

| Kaksha | Owner |
|---|---|
| 1 | Shani |
| 2 | Guru |
| 3 | Kuja |
| 4 | Surya |
| 5 | Shukra |
| 6 | Budha |
| 7 | Chandra |
| 8 | Lagna |

A transiting graha stands in one kaksha of the sign it occupies. The transit is
**supported** when that kaksha's owner is one of the contributors that gave a
bindu in the transiting graha's own Bhinnashtakavarga for that sign, and
**unsupported** when it is not. That is read directly from `prastara`, which
records exactly which contributor gave each bindu.

Eight kakshas of 3 degrees 45 minutes tile 30 degrees exactly, and the eight
owners are exactly the eight Ashtakavarga contributors - both properties a
test asserts rather than assumes.

## 5. Variant register

| ID | Question | Default | Alternatives | Source of default |
|---|---|---|---|---|
| V-13-22 | Place for the annual chart | the birth place | where the native now lives | reproducible without asking for a second place; the engine will take one when there is somewhere to ask |
| V-13-23 | Which solar return begins the year | the first on or after the birthday | the one nearest the birthday | "the year beginning at age n" is the plain reading |
| V-13-24 | Muntha counted from | completed years | the year in progress | Muntha is in the lagna at birth, so age 0 is the lagna |
| V-13-25 | Kaksha owner order | Shani, Guru, Kuja, Surya, Shukra, Budha, Chandra, Lagna | the reverse; starting from the sign lord | the order given wherever the eight are listed from the start of the sign |

## 6. Acceptance criteria (the QA gate)

1. **The solar return is exact.** The Sun at the returned moment equals the
   natal Sun to within a milliarcsecond, for every age in a long sweep.
2. **It is the right return.** The moment falls on or after the `n`th
   birthday and within a year and a day of it.
3. **Muntha advances one sign a year** and returns to the natal lagna every
   twelve years.
4. **Eight kakshas tile a sign.** Each is 3 degrees 45 minutes, they sum to
   30, and the eight owners are exactly the eight contributors.
5. **A kaksha verdict agrees with `prastara`.** For every graha and every
   longitude in a sweep, "supported" is true exactly when the kaksha owner's
   bit is set in that graha's prastara for that sign.
6. **An independent reimplementation agrees**
   (`scripts/qa_varsha_oracle.py`), parsing sections 2 to 4.
7. **The variants are on the output**, by ID.
8. **Lite is unchanged.**

## 7. What the gate actually proved

- `crates/lagn-core/tests/varsha.rs`, 9 tests. The headline one checks the
  solar return against itself for every age from 0 to 90: the Sun at the
  returned moment equals the natal Sun to within a milliarcsecond. A solver
  that converged on the wrong root fails there rather than producing a
  plausible chart.
- `scripts/qa_varsha_oracle.py` verifies the return rather than re-solving it
  (a root-finder is right exactly when its answer satisfies the equation), and
  recomputes Muntha and every kaksha reading from scratch, diffing the verdict
  against the `ashtakavarga` command's own prastara - a different code path
  from the one that produced it. 1,920 compared values, 0 disagreements.
- The WebAssembly engine answers `lagn_varsha`, checked by the smoke test.

One independent corroboration worth recording. The sidereal year is 365.2564
days against the calendar's 365.2425, so a solar return should drift about
0.0139 days later each year. For a birth at 14:10 on 21 December, the 45th
return should therefore land about 0.63 days later, near 03:08 on 22 December
- and it does. That is a check on the solve from outside the solve.

**A mutation found a self-referential test.** Swapping the first two kaksha
owners was caught by the oracle (35 disagreements) and *missed entirely* by
the Rust suite, because the suite compared `kaksha_of` against
`KAKSHA_ORDER` - the constant it was meant to be testing. A test that reads
the constant under test proves only that the code is self-consistent. Section
4's order is now pinned literally in the suite, which catches the same
mutation.

## 8. Revision history

| Rev | Change | Raised by |
|---|---|---|
| 1 | Initial design | Architect |
| 2 | Section 7 added. Kaksha readings moved into the serialised result so the oracle can diff them at all | QA |
