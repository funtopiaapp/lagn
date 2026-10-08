# Phase 13C design: upagrahas and special lagnas

Status: built and through the QA gate. Numbered separately from `DESIGN.md` so that document's
section numbers, which 13A's oracle and code cite, stay fixed.

## 1. Scope, and the line drawn through it

13C in `DESIGN.md` section 3 lists upagrahas, special lagnas, the remaining
vargas, Vimshopaka weights and avasthas. This increment delivers the two
groups whose arithmetic can be stated from first principles and checked:

| Built here | Why it is safe to build |
|---|---|
| Dhuma, Vyatipata, Parivesha, Indrachapa, Upaketu | fixed offsets from the Sun, and the chain closes on itself (section 3) |
| Kaala, Mrityu, Ardhaprahara, Yamaghantaka, Gulika | the eight-part division of the day, which this engine already computes and cross-validates for Rahu kalam |
| Bhava, Hora, Ghati lagnas | one sign per 2 hours, 1 hour and 24 minutes from sunrise |

Held back, and why:

| Held | What is missing |
|---|---|
| Sree lagna | how the Moon's nakshatra fraction is applied - as a fraction of a sign, of a nakshatra, or of the whole circle - differs between expositions |
| Pranapada lagna | the multiplier depends on the sign's mobility, and the published multipliers disagree |
| Indu lagna | the kala values assigned to each graha are a table, not a derivation |
| Vighati lagna | a sign per 24 seconds makes it acutely sensitive to birth time; it would be precise and meaningless without a rectified time |
| D-5, D-6, D-8, D-11, D-81, D-108, D-144 | each has a start-sign convention that varies by source; the grid rule alone does not settle them |
| Vimshopaka and Dasavarga weights | published weight sets differ, and a weighted total hides which weights produced it |
| Avasthas beyond Baladi and Jagradadi | Deeptadi and Lajjitadi need the dignity-plus-aspect conditions stated, which `docs/phase2/DESIGN.md` does not yet do |

Each held item needs one transcribed worked example, read off a page rather
than recalled. This is the same condition `docs/phase2/DESIGN.md` section 9
sets for Shadbala and `CHARA-DASHA.md` section 1 sets for the other five
Jaimini dashas.

## 2. Definition of "accurate"

Every value here is a longitude, and a longitude is right or wrong. So
"accurate" means it matches the arithmetic in sections 3 to 5, an independent
reimplementation agrees to within an arcsecond, and the internal identities in
section 3 hold exactly.

## 3. The Sun-offset upagrahas

Five shadowy points, each a fixed offset from the Sun's sidereal longitude.
All arithmetic is modulo 360.

| Upagraha | Longitude |
|---|---|
| Dhuma | `Sun + 133:20` |
| Vyatipata | `360 - Dhuma` |
| Parivesha | `Vyatipata + 180` |
| Indrachapa | `360 - Parivesha` |
| Upaketu | `Indrachapa + 16:40` |

**The chain closes.** Substituting through, `Upaketu = Sun - 30` exactly. That
identity is not a separate rule to be remembered - it is a consequence of the
five offsets above - which makes it a property a test can assert, and a check
on whether the table has been transcribed correctly. Two more fall out the
same way: `Parivesha = Dhuma + 180 - 2 x Sun` reduces to `Sun + 46:40`
reflected, and `Vyatipata` and `Parivesha` are always exactly 180 apart.

## 4. The day-division upagrahas

The daylight, sunrise to sunset, is divided into eight equal parts. Seven are
ruled; the eighth is not. The first part belongs to the lord of the weekday,
and the rest follow in the order Surya, Chandra, Kuja, Budha, Guru,
Shukra, Shani, wrapping round. (Kuja is Mangala; this engine uses the South
Indian name throughout, so the specification does too.)

An upagraha's longitude is **the ascendant at the moment its part begins**
(V-13-14).

| Upagraha | Ruling graha |
|---|---|
| Kaala | Surya |
| Mrityu | Kuja |
| Ardhaprahara | Budha |
| Yamaghantaka | Guru |
| Gulika | Shani |

Chandra's and Shukra's parts carry no named upagraha, and the eighth part is
unruled, so five of the eight parts are named.

**Night births.** For a birth between sunset and the next sunrise the night is
divided into eight the same way, and the sequence starts from the lord *fifth*
from the weekday lord (V-13-15). This engine already knows which it is: the
vara belongs to the sunrise that precedes the moment.

Gulika is the same point South Indian practice calls Mandi. It is also the
same division that produces Kuligai in `day.rs`, which is cross-validated
against `swetest` through sunrise - so this group inherits a verified
foundation rather than starting a new one.

## 5. The time lagnas

Three points that advance through the zodiac at a fixed rate from sunrise,
each starting from the Sun's longitude at sunrise.

| Lagna | One sign per | Degrees per hour |
|---|---|---|
| Bhava | 2 hours (5 ghatis) | 15 |
| Hora | 1 hour (2.5 ghatis) | 30 |
| Ghati | 24 minutes (1 ghati) | 75 |

So for `h` hours elapsed since sunrise, with `S` the Sun's sidereal longitude
at that sunrise:

```
Bhava lagna = S + 15 h
Hora lagna  = S + 30 h
Ghati lagna = S + 75 h
```

All modulo 360. The rates are exact ratios: Hora advances twice as fast as
Bhava, and Ghati five times as fast as Hora. Those ratios are properties a
test can assert without re-deriving the rates.

## 6. Variant register

Continuing the series in `DESIGN.md` section 7 and `CHARA-DASHA.md` section 6.

| ID | Question | Default | Alternatives | Source of default |
|---|---|---|---|---|
| V-13-14 | Which moment of its part gives an upagraha | the start | the end; the midpoint | the start is the reading given in the expositions that state a moment at all |
| V-13-15 | Night-birth part sequence | starts from the lord 5th from the weekday lord | starts from the weekday lord again | the standard night rule, and the one that makes the eight night parts continue the day's cycle |
| V-13-16 | Sun longitude the time lagnas start from | the Sun at that day's sunrise | the Sun at the moment of birth | the lagnas are reckoned from sunrise, so their origin is the sunrise Sun |
| V-13-17 | Gulika and Mandi | the same point | distinct points, Mandi taken at the end of Shani's part | they are one point in South Indian practice, which is this engine's tradition |

## 7. Architecture

```
lagn-core/src/upagraha.rs    the ten upagrahas and three lagnas    (new)
lagn-cli                     `lagn upagraha`                       (extends)
lagn-server/src/api.rs       POST /api/upagraha                    (extends)
lagn-ffi/src/lib.rs          lagn_upagraha                         (extends)
web/src/components/UpagrahaView.tsx   Pro surface only             (new)
```

It needs sunrise, sunset and the ascendant at an arbitrary moment, all of
which layer 1 and `Chart` already provide.

## 8. Acceptance criteria (the QA gate)

1. **The Sun-offset chain closes.** `Upaketu == Sun - 30` to within a
   nanodegree, and `Vyatipata` and `Parivesha` are exactly 180 apart, for a
   wide sweep of Sun longitudes.
2. **Every upagraha is a valid longitude**, in `[0, 360)`, for every chart.
3. **The day-division parts tile the daylight** exactly, and each named
   upagraha's part is the one its ruling graha owns.
4. **A night birth uses the night division**, and the sequence starts from the
   fifth lord. A test pins a day birth and a night birth at the same place.
5. **The time lagnas keep their ratios.** Hora advances exactly twice as fast
   as Bhava and Ghati exactly five times as fast as Hora, measured by
   recomputing at two moments.
6. **At sunrise all three time lagnas equal the Sun**, which is what "reckoned
   from sunrise" means and the simplest case to get wrong.
7. **An independent reimplementation agrees** to an arcsecond
   (`scripts/qa_upagraha_oracle.py`), parsing sections 3, 4 and 5 of this
   document.
8. **The variants are on the output**, by ID.
9. **Lite is unchanged**, and this appears only under Pro.

## 9. What the gate actually proved

- `crates/lagn-core/tests/upagraha.rs`, 14 tests, covering criteria 1 to 6 and 8.
- `scripts/qa_upagraha_oracle.py`, which parses the Sun-offset table, the
  ruler mapping and the three rates out of this document at run time. It
  derives each Sun offset as one closed-form expression where the kernel walks
  the chain step by step, so a mistranscribed intermediate cannot cancel out.
  250 charts, 4,500 compared values, 0 disagreements.
- `crates/lagn-server/tests/api.rs`: the chain closes through the endpoint, no
  two named upagrahas share a part, and both the day and night branches were
  exercised (the test fails if the sweep produces only one).
- The WebAssembly engine answers `lagn_upagraha`, checked by the smoke test.
- Two deliberate mutations, each caught by both the suite and the oracle:

  | Mutation | Rust suite | Oracle |
  |---|---|---|
  | Dhuma's offset 133:20 changed to 133:00 | 2 tests fail | 50 of 180 values |
  | night sequence starting from the 4th lord instead of the 5th | 1 test fails | 65 of 360 values |

The first of those is the one worth noting: it was caught by the chain-closure
test, which does not know what Dhuma's offset should be. It only knows the
five offsets must compose into `Upaketu = Sun - 30`. That is why these five
were safe to build from a specification without external reference output,
and it is the property that distinguishes them from the tables held back in
section 1.

## 10. Revision history

| Rev | Change | Raised by |
|---|---|---|
| 1 | Initial design; section 1 records what is held and why | Architect |
| 2 | Sections 4's graha names aligned to the engine's own (Kuja, not Mangala) after the oracle could not match them | QA |
| 3 | Section 9 added | Dev |
