# Phase 13D design: Yogini dasha

Status: built and through the QA gate. The first of the nakshatra dashas, built to the bar
`CHARA-DASHA.md` set: spec with a variant register, independent oracle,
mutation testing, Pro surface only.

## 1. Why Yogini is buildable and most of 13D is not

`DESIGN.md` section 3 lists a dozen conditional dashas under 13D. Yogini is
the one whose arithmetic polices itself.

The eight yoginis take **1, 2, 3, 4, 5, 6, 7 and 8 years** - consecutive
integers, summing to **36**, which is the cycle length by definition. A
mistranscribed period breaks two things at once: the run of consecutive
integers, and the total. Neither can be fudged, and both are properties a test
asserts rather than values it trusts.

Ashtottari, Dwadasottari, Shashtihayani and the rest have no such structure:
their periods are irregular, so a wrong one produces a wrong total that looks
as plausible as the right one. They stay held until a worked example arrives.

## 2. Definition of "accurate"

A Yogini dasha is accurate when the starting yogini, the balance at birth, the
sequence and every boundary match sections 3 to 5, and an independent
reimplementation agrees on every boundary.

Boundaries are dates, so they depend on the year length the chart was computed
with. Yogini uses the chart's own `YearLength`, the same one Vimshottari and
Chara use, so no two dashas of one chart can disagree about how long a year is.

## 3. The eight yoginis

| # | Yogini | Lord | Years |
|---|---|---|---|
| 1 | Mangala | Chandra | 1 |
| 2 | Pingala | Surya | 2 |
| 3 | Dhanya | Guru | 3 |
| 4 | Bhramari | Kuja | 4 |
| 5 | Bhadrika | Budha | 5 |
| 6 | Ulka | Shani | 6 |
| 7 | Siddha | Shukra | 7 |
| 8 | Sankata | Rahu | 8 |

Total 36 years. The sequence runs in this order and wraps.

## 4. Where it starts

Take the janma nakshatra's serial number, Ashwini being 1. Add 3. Divide by 8.
The remainder names the yogini:

```
start = (janma nakshatra number + 3) mod 8,  with 0 meaning the 8th
```

So Ashwini gives 4, which is Bhramari; Mrigashira gives 0, which is Sankata.

**The balance at birth.** The first yogini's period is already running when the
native is born, and what remains is proportional to the part of the janma
nakshatra still untraversed - exactly as Vimshottari's balance works:

```
balance years = (that yogini's years) x (1 - fraction of the nakshatra traversed)
```

The first period in the emitted list therefore begins *before* birth, and
keeping that opening portion is what makes the antardasha boundaries inside it
correct. This mirrors how `dasha.rs` handles the Vimshottari birth mahadasha.

## 5. Antardashas

A mahadasha of `Y` years divides among the eight yoginis in the same order,
beginning with the mahadasha's own yogini, each in proportion to its own years
out of 36:

```
antardasha of yogini k inside mahadasha of yogini j  =  Y x years(k) / 36
```

The eight sum to `Y` because the eight period lengths sum to 36, which is the
same self-check the cycle rests on, applied one level down.

## 6. Variant register

Continuing the series in `DESIGN.md` section 7.

| ID | Question | Default | Alternatives | Source of default |
|---|---|---|---|---|
| V-13-33 | The starting rule | `(nakshatra + 3) mod 8`, 0 meaning the 8th | `(nakshatra + 3) mod 8` with 0 meaning the 1st; counting from a different nakshatra | the statement common to the published expositions |
| V-13-34 | Balance at birth | proportional to the untraversed part of the janma nakshatra | the whole period from birth | Vimshottari's balance works this way and one chart should not hold two conventions |
| V-13-35 | Antardasha order | the eight from the mahadasha's own yogini, proportional to their years | always from Mangala; equal eighths | the sub-period follows the period it sits in, as in Vimshottari |
| V-13-36 | Sankata's lord | Rahu | Ketu | the lord given wherever the eight are tabulated |
| V-13-37 | When Yogini applies | always shown, as an alternative to Vimshottari | only under the classical applicability conditions | this engine shows what it computes and leaves the choice of dasha to the astrologer |

## 7. Architecture

```
lagn-core/src/yogini.rs      the dasha                       (new)
lagn-cli                     `lagn yogini`                   (extends)
lagn-server/src/api.rs       POST /api/yogini                (extends)
lagn-ffi/src/lib.rs          lagn_yogini                     (extends)
web/src/components/YoginiView.tsx   Pro surface only         (new)
```

It depends on `Chart`, `NakshatraPosition` and `YearLength`, and reads no
clock.

## 8. Acceptance criteria (the QA gate)

1. **The periods are 1 to 8 and sum to 36.** Asserted as the structure, not as
   eight transcribed numbers.
2. **Every cycle visits all eight yoginis once**, in the order section 3 gives.
3. **The starting yogini follows section 4**, checked for all 27 nakshatras.
4. **The balance is proportional.** A birth at the very start of a nakshatra
   gets very nearly the whole first period; one at the very end gets very
   nearly none.
5. **Periods tile the timeline**, each ending exactly where the next begins,
   bitwise, with the first beginning at or before birth and the second at or
   after it.
6. **Eight antardashas fill each mahadasha**, proportional to their years,
   opening on the mahadasha's own yogini and closing exactly on its end.
7. **The horizon is covered.** The emitted periods span at least 120 years from
   birth, so a reading never runs off the end of the list.
8. **The year length is the chart's**, and changing it scales every boundary in
   proportion.
9. **An independent reimplementation agrees** on every boundary
   (`scripts/qa_yogini_oracle.py`), parsing sections 3 to 5.
10. **The variants are on the output**, by ID.
11. **Lite is unchanged**, and Yogini appears only under Pro.

## 9. What the gate actually proved

- `crates/lagn-core/tests/yogini.rs`, 13 tests covering criteria 1 to 8 and 10.
- `scripts/qa_yogini_oracle.py`: 120 charts across three year lengths, 69,792
  compared values, 0 disagreements. It **refuses to run** unless the periods
  it parses out of section 3 are the consecutive integers 1 to 8 summing to 36
  - so a corrupted specification fails the oracle before it can agree with a
  corrupted implementation.
- Three deliberate mutations, each caught by both the suite and the oracle:

  | Mutation | Rust suite | Oracle |
  |---|---|---|
  | start offset +3 changed to +4 | 1 test fails | every chart disagrees |
  | Sankata lorded by Ketu instead of Rahu | 1 test fails | 25 of 4,657 |
  | antardashas made equal eighths rather than proportional | 1 test fails | 1,382 of 4,657 |

- The WebAssembly engine answers `lagn_yogini`, and the smoke test re-checks
  the 1-to-8-summing-to-36 structure in the browser.

## 10. Revision history

| Rev | Change | Raised by |
|---|---|---|
| 1 | Initial design | Architect |
| 2 | Section 9 added | Dev |
