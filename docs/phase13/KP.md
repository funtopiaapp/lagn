# Phase 13H design: the Krishnamurti Paddhati

Status: built and through the QA gate. Numbered separately from `DESIGN.md` so that document's
section numbers stay fixed.

## 1. Why KP is buildable without external reference output

KP is the one remaining JHora group that is pure derivation. Everything below
follows from two things this engine already computes and cross-validates: the
Vimshottari period lengths, and the Placidus cusps from Swiss Ephemeris.
Nothing here is a table to be recalled.

The keystone is section 3. KP is described everywhere as dividing the zodiac
into **249** sub-divisions, and that number is normally quoted rather than
derived. Building the division from the Vimshottari proportions and then
cutting it by the twelve sign boundaries yields:

- 27 nakshatras x 9 subs = **243** subs;
- of those, exactly **6** straddle a sign boundary and are therefore cut in
  two, because nine nakshatras straddle a boundary but in three of them the
  boundary falls exactly on a sub boundary;
- 243 + 6 = **249** cells.

That the construction lands on the traditional number is a real check on the
construction, not a coincidence to be assumed: a wrong sub width, a wrong
starting lord or a wrong cycle order all change the count. The arithmetic is
done in exact fractions so the six-versus-nine distinction is decided
arithmetically rather than by a tolerance.

## 2. Definition of "accurate"

A KP reading is accurate when, for any longitude, the four lords are the ones
sections 3 and 4 give; the cusps are the Placidus cusps Swiss Ephemeris
returns for the chart's own ayanamsa; and an independent reimplementation
agrees on every lord across a dense sweep of the whole zodiac.

## 3. The four lords of a longitude

| Lord | How it is found |
|---|---|
| Sign lord | the classical lord of the rasi containing the longitude |
| Star lord | the Vimshottari lord of the nakshatra containing it |
| Sub lord | see below |
| Sub-sub lord | the same division applied again, inside the sub |

**The sub.** A nakshatra spans 13 degrees 20 minutes. Divide it into nine
parts in the Vimshottari order, **beginning with the nakshatra's own lord**,
each part proportional to that lord's Vimshottari years out of 120:

```
width(lord) = (13 deg 20 min) x years(lord) / 120
```

The sub lord is the lord of the part containing the longitude. The nine parts
tile the nakshatra exactly, because the nine Vimshottari periods sum to 120 by
definition.

**The sub-sub (prana) lord.** The same rule once more: divide the sub into
nine parts in Vimshottari order beginning with the sub lord, each proportional
to its years out of 120.

The division is self-similar, which is the property that makes it checkable at
every depth without a new rule.

## 4. Cusps, ruling planets and significators

**Cusps.** KP reads houses as Placidus cusps, not as whole signs. Each of the
twelve cusps carries its own four lords, found exactly as in section 3. The
chart's own ayanamsa applies; KP practice normally uses the Krishnamurti
ayanamsa, which this engine already offers, but the choice stays the reader's
(V-13-18).

**Ruling planets.** The five, in the order practice lists them:

| # | Ruling planet |
|---|---|
| 1 | lord of the day (the vara lord) |
| 2 | sign lord of the Moon |
| 3 | star lord of the Moon |
| 4 | sign lord of the ascendant |
| 5 | star lord of the ascendant |

Taken at the moment the chart is cast. Each is reported with its own sub lord
alongside, because practice weighs the sub.

**Significators of a house.** The four groups, strongest first:

| Group | Who |
|---|---|
| 1 | grahas in the star of a graha occupying the house |
| 2 | grahas occupying the house |
| 3 | grahas in the star of the lord of the house |
| 4 | the lord of the house |

"In the star of X" means the graha's star lord is X. A graha may appear in
more than one group for the same house; it is listed in the strongest it
qualifies for and not repeated (V-13-19). House occupancy for significators
is read from the Placidus cusps, consistently with the rest of KP
(V-13-20).

## 5. Variant register

Continuing the series in `DESIGN.md` section 7, `CHARA-DASHA.md` section 6 and
`UPAGRAHA.md` section 6.

| ID | Question | Default | Alternatives | Source of default |
|---|---|---|---|---|
| V-13-18 | Ayanamsa for a KP reading | the chart's own | force Krishnamurti whenever KP is read | a chart should not silently change ayanamsa between tabs; the reader can select Krishnamurti and see every tab agree |
| V-13-19 | A graha qualifying in two significator groups | listed once, in the strongest | listed in every group it qualifies for | a significator list is read as a ranking, and a repeat would double-count |
| V-13-20 | House occupancy for significators | Placidus cusps | whole signs, as the rest of this engine uses | KP reads houses as cusps throughout, and mixing the two inside one method would be incoherent |
| V-13-21 | Rahu and Ketu as significators | included, by the sign lord of the sign they occupy | excluded; by their dispositor's star | the nodes are standard KP significators and are read through their dispositor |

## 6. Architecture

```
lagn-core/src/kp.rs          the four lords, cusps, ruling planets,
                             significators                            (new)
lagn-cli                     `lagn kp`                                (extends)
lagn-server/src/api.rs       POST /api/kp                              (extends)
lagn-ffi/src/lib.rs          lagn_kp                                   (extends)
web/src/components/KpView.tsx   Pro surface only                       (new)
```

`kp.rs` needs the Placidus cusps, which `Ephemeris::angles` already returns
for any house system, and the Vimshottari tables in `dasha.rs`. It reads no
clock beyond the chart's own moment.

## 7. Acceptance criteria (the QA gate)

1. **The division tiles exactly.** The nine subs of every nakshatra sum to
   13 degrees 20 minutes, and the nine sub-subs of every sub sum to that sub,
   computed in exact rational arithmetic.
2. **The count is 249.** Building the 243 subs and cutting by the twelve sign
   boundaries yields exactly 249 cells, with exactly 6 subs straddling a
   boundary. Asserted, not assumed.
3. **Every longitude has four lords**, and a dense sweep of the whole zodiac
   finds no gap and no overlap: stepping across a sub boundary changes the sub
   lord exactly once.
4. **The first sub of a nakshatra belongs to that nakshatra's own lord**, and
   the first sub-sub of a sub belongs to the sub lord. This is the rule's
   starting point and the easiest thing to get wrong.
5. **Sub widths are proportional to the Vimshottari years.** Venus's sub is
   20/120 of a nakshatra and the Sun's is 6/120, checked as ratios rather than
   as absolute numbers.
6. **The twelve cusps are the Placidus cusps** Swiss Ephemeris returns for the
   chart, in order, and each carries four lords.
7. **The five ruling planets** are the ones section 4 names, in that order.
8. **No graha appears twice** in one house's significator list, and every
   graha listed qualifies under the group it is listed in.
9. **An independent reimplementation agrees** on all four lords for a dense
   sweep (`scripts/qa_kp_oracle.py`), parsing sections 3 and 4.
10. **The variants are on the output**, by ID.
11. **Lite is unchanged**, and KP appears only under Pro.

## 8. What the gate actually proved

- `crates/lagn-core/tests/kp.rs`, 13 tests, covering criteria 1 to 8 and 10.
- `scripts/qa_kp_oracle.py`, which builds the subs in exact rational
  arithmetic where the kernel uses f64, and rebuilds the 249 independently of
  the kernel's own assertion. 150 charts, 14,400 compared values, 0
  disagreements.
- `crates/lagn-server/tests/api.rs`: every point the endpoint returns has its
  sub recomputed from the longitude alone, over 80 charts.
- The WebAssembly engine answers `lagn_kp`, checked by the smoke test, which
  also asserts every house's significators come back ranked.

**Both implementations land on 249 independently.** That is the strongest
evidence here. The number is normally quoted; deriving it twice, by different
arithmetic, from the Vimshottari proportions and the sign boundaries, is a
check on the construction that no amount of self-consistency could give.

Three defects the tests found, none of which were visible by reading the code:

| Found by | Defect |
|---|---|
| the dense sweep | a float seam between nakshatras. One nakshatra's end was computed as `start + span` and the next's start as `SPAN * k`; they differ by an ulp and 93.33333333333333 degrees belonged to *no* sub. Fixed by dividing between two explicit endpoints |
| the dense sweep | at 226.66666666666666 degrees the kernel's own nakshatra floor said Jyeshtha while re-deriving the start by multiplication said Anuradha. `lords` now divides *within* the nakshatra using `degrees_within`, which comes from that same floor, so the disagreement is impossible rather than unlikely |
| a mutation | swapping significator ranks 1 and 2 was caught by the suite but **not** by the oracle, because the ordering keyed on the enum's declaration order while `rank()` was only a displayed number. Sorting by `rank()` made strength single-sourced; the same mutation now produces 39 disagreements where it produced 0 |

The third is worth recording as a finding about the test apparatus rather
than about the astrology: a mutation that only one of two independent checks
can see means the two were not as independent as intended.

## 9. Revision history

| Rev | Change | Raised by |
|---|---|---|
| 1 | Initial design. Section 1 derives the 249 rather than quoting it | Architect |
| 2 | Section 3: the division is measured within the nakshatra, from the same floor that decides which nakshatra a longitude is in | QA |
| 3 | Section 8 added | Dev |
