# Phase 2 design: chart derivation

Status: 2A and 2B built and QA-passed. 2C and 2D design only.
Revision 2 (2026-09-21): three QA corrections, listed in section 12.
Source text: *Brihat Parashara Hora Shastra* (BPHS), Santhanam translation,
unless a row says otherwise.

## 1. Scope and split

Phase 2 is too variant-heavy to validate as one block, so it ships in four
gated sub-phases. Each one passes architect, developer and QA before the next
starts.

| Sub-phase | Contents | Variant risk | Status |
|---|---|---|---|
| **2A** | 16 Shodasavargas; dignity; natural/temporary/compound relationships; graha and rasi drishti; Baladi and Jagradadi avasthas; combustion; graha yuddha detection | Low, and fully specified below | **Build** |
| **2B** | Ashtakavarga: BAV, SAV, prastara | Low, with fixed tables and fixed totals | **Build** |
| 2C | Shadbala (6 components, ~20 sub-components), Vimshopaka, sphuta drishti | **High** | Design only; see section 9 |
| 2D | Deeptadi, Lajjitadi and Shayanadi avasthas; ashtakavarga shodhana | High | Design only |

**Why 2C is held.** Several Shadbala sub-components (Kala bala's abda and masa
lords, Cheshta bala, and the special-aspect handling in sphuta drishti) have
more than one published formula. No independent reference output is available
to us: JHora can't be run here, and test values must never come from memory of
a book. Building them now would produce numbers we can't verify, which the
99.9% target rules out. Section 9 lists exactly what is needed to unblock it.

## 2. Definition of "accurate"

A computation is accurate when it matches this specification, including the
variant defaults in section 8. Where BPHS gives one unambiguous rule, the
specification *is* BPHS. Where sources disagree, the specification records the
choice and the alternatives, and the astrologer signs off.

QA can prove conformance to the specification. It can't prove the
specification matches a given practitioner's paramparā; the variant register
and astrologer sign-off cover that.

## 3. Shodasavargas (2A)

Notation: `s` = rasi index of the longitude (Mesha = 0), `k` = 0-based part
index within the sign, `n` = number of parts. Result sign = `(start + k) mod 12`.

**Grid rule.** For every equal-part varga, `s` and `k` are read off one floor:
`K = floor(lon / (30/n))`, `s = K div n`, `k = K mod n`. Computing them
separately lets them disagree at a boundary, which is the defect class fixed
in Phase 1 QA for nakshatra and pada. For n = 9 this is bit-identical to the
Phase 1 navamsa and pada grid, and must stay so.

"Odd" means odd-numbered sign: Mesha, Mithuna, Simha and so on (even index).

| Varga | n | Start sign rule | Signification |
|---|---|---|---|
| D-1 Rasi | 1 | `s` | body, overall |
| D-2 Hora | 2 | odd: k=0 Simha, k=1 Karka. even: k=0 Karka, k=1 Simha | wealth |
| D-3 Drekkana | 3 | `s + 4k` (1st, 5th, 9th from sign) | siblings |
| D-4 Chaturthamsa | 4 | `s + 3k` (1st, 4th, 7th, 10th) | property, fortune |
| D-7 Saptamsa | 7 | odd: `s`. even: `s + 6` | progeny |
| D-9 Navamsa | 9 | chara `s`; sthira `s + 8`; dvisvabhava `s + 4` | spouse, dharma |
| D-10 Dasamsa | 10 | odd: `s`. even: `s + 8` | career |
| D-12 Dwadasamsa | 12 | `s` | parents |
| D-16 Shodasamsa | 16 | chara Mesha; sthira Simha; dvisvabhava Dhanus | vehicles, comforts |
| D-20 Vimsamsa | 20 | chara Mesha; sthira Dhanus; dvisvabhava Simha | spiritual practice |
| D-24 Chaturvimsamsa | 24 | odd Simha; even Karka | learning |
| D-27 Bhamsa | 27 | Agni Mesha; Prithvi Karka; Vayu Tula; Jala Makara | strengths, weaknesses |
| D-30 Trimsamsa | 5, unequal | table below | misfortune |
| D-40 Khavedamsa | 40 | odd Mesha; even Tula | maternal legacy |
| D-45 Akshavedamsa | 45 | chara Mesha; sthira Simha; dvisvabhava Dhanus | paternal legacy, character |
| D-60 Shashtiamsa | 60 | `s` | past karma, the finest division |

**D-30 (Parashari), degrees within the sign, half-open intervals `[a, b)`:**

| Odd signs | Sign | Lord | Even signs | Sign | Lord |
|---|---|---|---|---|---|
| 0-5 | Mesha | Mars | 0-5 | Vrishabha | Venus |
| 5-10 | Kumbha | Saturn | 5-12 | Kanya | Mercury |
| 10-18 | Dhanus | Jupiter | 12-20 | Meena | Jupiter |
| 18-25 | Mithuna | Mercury | 20-25 | Makara | Saturn |
| 25-30 | Tula | Venus | 25-30 | Vrischika | Mars |

**Derived facts QA must verify independently** (each follows from the rules,
so a mismatch means an implementation error):

- D-7, D-9 and D-27 each equal the continuous formula `floor(lon / (30/n)) mod 12`.
- Over a full circle, D-3, D-4, D-7, D-9, D-10, D-12, D-16, D-20, D-24, D-27 and D-60
  give every sign exactly `n` parts.
- Inside a sign, every part edge changes the varga sign. At a sign boundary
  the sign repeats in exactly three cases: D-2 at every boundary, and D-10
  and D-24 on entering each even sign. Nowhere else.
- D-2 only ever yields Simha or Karka.
- D-30 never yields a sign ruled by the Sun or Moon, and odd and even signs
  each use exactly five distinct signs.

## 4. Dignity (2A)

### 4.1 Rasi chart (degree-aware), half-open intervals

| Graha | Exalted | Debilitated | Moolatrikona | Own |
|---|---|---|---|---|
| Sun | Mesha (all) | Tula | Simha 0-20 | Simha 20-30 |
| Moon | Vrishabha 0-3 | Vrischika | Vrishabha 3-30 | Karka |
| Mars | Makara | Karka | Mesha 0-12 | Mesha 12-30, Vrischika |
| Mercury | Kanya 0-15 | Meena | Kanya 15-20 | Kanya 20-30, Mithuna |
| Jupiter | Karka | Makara | Dhanus 0-10 | Dhanus 10-30, Meena |
| Venus | Meena | Kanya | Tula 0-15 | Tula 15-30, Vrishabha |
| Saturn | Tula | Mesha | Kumbha 0-20 | Kumbha 20-30, Makara |

Deep exaltation points (for Uchcha bala in 2C): Sun Mesha 10, Moon Vrishabha 3,
Mars Makara 28, Mercury Kanya 15, Jupiter Karka 5, Venus Meena 27, Saturn Tula 20.
Deep debilitation is the exact opposite point.

If none of the above applies, dignity = the compound relationship (4.3) of the
graha to the lord of the occupied sign. This gives nine levels, best first:
Exalted, Moolatrikona, Own, Great friend, Friend, Neutral, Enemy, Great enemy,
Debilitated.

### 4.2 Divisional charts (sign-only)

Vargas carry no degrees. Exaltation sign gives Exalted, debilitation sign gives
Debilitated, and a moolatrikona sign gives Moolatrikona (V-4). Own sign, then
compound relationship. Moon in Vrishabha and Mercury in Kanya are Exalted at
sign level.

### 4.3 Relationships

**Natural (naisargika), BPHS ch. 3:**

| Graha | Friends | Neutral | Enemies |
|---|---|---|---|
| Sun | Moon, Mars, Jupiter | Mercury | Venus, Saturn |
| Moon | Sun, Mercury | Mars, Jupiter, Venus, Saturn | none |
| Mars | Sun, Moon, Jupiter | Venus, Saturn | Mercury |
| Mercury | Sun, Venus | Mars, Jupiter, Saturn | Moon |
| Jupiter | Sun, Moon, Mars | Saturn | Mercury, Venus |
| Venus | Mercury, Saturn | Mars, Jupiter | Sun, Moon |
| Saturn | Mercury, Venus | Jupiter | Sun, Moon, Mars |

**Temporary (tatkalika):** B is A's temporary friend when B occupies the 2nd,
3rd, 4th, 10th, 11th or 12th sign from A. Otherwise B is a temporary enemy.
Computed from D-1 positions (V-5).

**Compound (panchadha):**

| Natural + temporary | Compound |
|---|---|
| Friend + friend | Great friend |
| Friend + enemy | Neutral |
| Neutral + friend | Friend |
| Neutral + enemy | Enemy |
| Enemy + friend | Neutral |
| Enemy + enemy | Great enemy |

Rahu and Ketu have no natural relationships, dignity or lordship in 2A (V-1, V-2).

## 5. Drishti (2A)

**Graha drishti (sign-based, Parashari).** Counted from the graha's sign,
inclusive:

| Graha | Aspects houses |
|---|---|
| Sun, Moon, Mercury, Venus | 7 |
| Mars | 4, 7, 8 |
| Jupiter | 5, 7, 9 |
| Saturn | 3, 7, 10 |
| Rahu, Ketu | per V-3; default none |

**Rasi drishti (Jaimini).** A chara sign aspects every sthira sign except the
next sign. A sthira sign aspects every chara sign except the previous sign.
A dvisvabhava sign aspects the other three dvisvabhava signs. Every sign
aspects exactly three signs, and the relation is symmetric. QA must verify both
properties.

## 6. Avasthas, combustion, war (2A)

**Baladi**, by degrees within sign. Odd signs: 0-6 Bala, 6-12 Kumara, 12-18
Yuva, 18-24 Vriddha, 24-30 Mrita. Even signs use the reverse order. All nine
grahas.

**Jagradadi.** Exalted, moolatrikona or own sign gives Jagrat. Great friend,
friend or neutral gives Swapna. Enemy, great enemy or debilitated gives
Sushupti (V-6). Seven grahas only.

**Combustion (asta).** A graha is combust when its longitude difference from
the Sun is at most the orb. Orbs: Moon 12, Mars 17, Mercury 14 (12 if
retrograde), Jupiter 11, Venus 10 (8 if retrograde), Saturn 15. The Sun and
the nodes are never combust (V-7).

**Graha yuddha, detection only.** Any two of Mars, Mercury, Jupiter, Venus and
Saturn separated by at most 1.0 degree of longitude (inclusive), measured as
the shortest arc. Choosing the
winner has several conflicting rules and is deferred to 2C.

## 7. Ashtakavarga (2B)

For each of the seven grahas P, and each contributor C in {Sun, Moon, Mars,
Mercury, Jupiter, Venus, Saturn, Lagna}, P's BAV gets one bindu in every sign
that is the h-th from C's D-1 sign, for each h in the table. SAV[sign] is the
sum of the seven BAVs.

| BAV of | Sun | Moon | Mars | Mercury | Jupiter | Venus | Saturn | Lagna | Total |
|---|---|---|---|---|---|---|---|---|---|
| Sun | 1 2 4 7 8 9 10 11 | 3 6 10 11 | 1 2 4 7 8 9 10 11 | 3 5 6 9 10 11 12 | 5 6 9 11 | 6 7 12 | 1 2 4 7 8 9 10 11 | 3 4 6 10 11 12 | **48** |
| Moon | 3 6 7 8 10 11 | 1 3 6 7 10 11 | 2 3 5 6 9 10 11 | 1 3 4 5 7 8 10 11 | 1 4 7 8 10 11 12 | 3 4 5 7 9 10 11 | 3 5 6 11 | 3 6 10 11 | **49** |
| Mars | 3 5 6 10 11 | 3 6 11 | 1 2 4 7 8 10 11 | 3 5 6 11 | 6 10 11 12 | 6 8 11 12 | 1 4 7 8 9 10 11 | 1 3 6 10 11 | **39** |
| Mercury | 5 6 9 11 12 | 2 4 6 8 10 11 | 1 2 4 7 8 9 10 11 | 1 3 5 6 9 10 11 12 | 6 8 11 12 | 1 2 3 4 5 8 9 11 | 1 2 4 7 8 9 10 11 | 1 2 4 6 8 10 11 | **54** |
| Jupiter | 1 2 3 4 7 8 9 10 11 | 2 5 7 9 11 | 1 2 4 7 8 10 11 | 1 2 4 5 6 9 10 11 | 1 2 3 4 7 8 10 11 | 2 5 6 9 10 11 | 3 5 6 12 | 1 2 4 5 6 7 9 10 11 | **56** |
| Venus | 8 11 12 | 1 2 3 4 5 8 9 11 12 | 3 5 6 9 11 12 | 3 5 6 9 11 | 5 8 9 10 11 | 1 2 3 4 5 8 9 10 11 | 3 4 5 8 9 10 11 | 1 2 3 4 5 8 9 11 | **52** |
| Saturn | 1 2 4 7 8 10 11 | 3 6 11 | 3 5 6 10 11 12 | 6 8 9 10 11 12 | 5 6 11 12 | 6 11 12 | 3 5 6 11 | 1 3 4 6 10 11 | **39** |

**Chart-independent invariants.** Each graha's BAV total is fixed: 48, 49, 39,
54, 56, 52, 39. The SAV total is always **337**. Any sign's bindu count in one
BAV is between 0 and 8. These totals are a check on the transcription of the
table itself: a single wrong house number changes a total.

Shodhana (trikona and ekadhipatya reduction) and shodhya pinda are deferred to
2D; the ekadhipatya rules have several variants.

## 8. Variant register

Each variant needs astrologer sign-off before Phase 3 interprets it. The
default applies until then and is recorded on the chart.

| ID | Question | Default | Alternatives | Source of default |
|---|---|---|---|---|
| V-1 | Rahu/Ketu exaltation and debilitation | none (no dignity) | Vrishabha/Vrischika; Mithuna/Dhanus | BPHS gives no firm verse; conservative |
| V-2 | Rahu/Ketu natural relationships | none | several published tables | as V-1 |
| V-3 | Rahu/Ketu graha drishti | none | 7th only; 5, 7, 9 | disputed verses in BPHS ch. 26 |
| V-4 | Moolatrikona in divisional charts | MT sign gives Moolatrikona | treat as Own | sign-only charts can't apply degree ranges |
| V-5 | Temporary relationships used for varga dignity | from D-1 | from each varga's own positions | Raman, *Graha and Bhava Balas* |
| V-6 | Jagradadi uses natural or compound relationship | compound | natural | BPHS ch. 45 doesn't say |
| V-7 | Combustion orbs | Surya Siddhanta orbs above | Mercury 13; no retrograde reduction | widely used, incl. JHora |
| V-8 | Moon moolatrikona range | Vrishabha 3-30 | 4-20; 4-30 | BPHS ch. 3 (Santhanam) |
| V-9 | Interval boundaries (MT, D-30, Baladi) | half-open `[a, b)` | closed | needed for determinism; texts silent |

## 9. What 2C needs before it can start

1. Astrologer decisions on the Shadbala variants: Kala bala abda/masa lord
   method, hora length (equal or unequal), Cheshta bala formula, sphuta drishti
   special-aspect handling, and Mercury's benefic or malefic status in Paksha bala.
2. **Reference output for at least 5 charts with every Shadbala sub-component
   shown**, from JHora or from B.V. Raman's worked example in *Graha and Bhava
   Balas*, transcribed from the book, not recalled.
3. Sunrise and sunset (`swe_rise_trans`) added to layer 1, with its own oracle
   test against `swetest`.

## 10. Architecture

```
lagn-core/src/
  varga.rs         16 vargas, grid rule, D-30 table              (extends)
  dignity.rs       exaltation/debilitation/MT/own tables, Dignity  (new)
  relationship.rs  natural, temporary, compound                   (new)
  drishti.rs       graha drishti, rasi drishti, NodeAspects        (new)
  condition.rs     Baladi, Jagradadi, combustion, graha yuddha     (new)
  ashtakavarga.rs  BAV/SAV/prastara from the table in section 7    (new)
```

- All new modules are pure functions over `Chart` or longitudes, with no I/O
  and no ephemeris access, so they compile to wasm, iOS and Android unchanged.
- `Varga` gains 14 variants. The `Placement.navamsa` field and `navamsa_sign`
  stay, with unchanged output, so Phase 1 fixtures replay untouched.
- V-3 to V-6 are fields on a new `DerivationSettings`, recorded on every
  result they affect. V-1, V-2 and V-7 to V-9 are fixed at their defaults
  until the astrologer asks for an alternative.
- CLI: `lagn vargas`, `lagn chart --varga D10`, `lagn details` (dignity,
  relationships, aspects, avasthas, combustion, war) and `lagn ashtakavarga`,
  each with `--json`.

## 11. Acceptance criteria (the QA gate)

1. Every rule in sections 3-7 has a unit test stating the rule longhand.
2. **N-version check.** QA writes an independent Python implementation of
   sections 3, 4, 5 and 7 from this document without reading the Rust, and
   diffs it against CLI JSON over at least 2,000 random charts. Zero
   disagreements are allowed.
3. Every derived fact and invariant in sections 3, 5 and 7 has its own test.
4. Boundary tests at every varga, D-30, MT, Baladi and combustion edge, ±1 arcsec.
5. Mutation check. Corrupting one entry in the Ashtakavarga table, one D-30
   boundary and one aspect house must each fail at least one test.
6. No Phase 1 regressions: all 127 tests, the golden fixtures and the swetest
   cross-validation stay green.
7. Clippy is clean, and both release and debug test runs pass.

The N-version check catches implementation errors. It can't catch a
misreading of BPHS shared by both implementations, because one person writes
both. JHora comparison and astrologer review close that gap.

## 12. Revision history

| Rev | Change | Raised by |
|---|---|---|
| 1 | Initial design | Architect |
| 2 | Section 10: only V-3 to V-6 are switchable (the text said every variant) | QA-P2-S1 |
| 2 | Section 6: graha yuddha orb stated as inclusive (was "within 1 degree") | QA-P2-S2 |
| 2 | Section 3: added the sign-boundary repeat fact (D-2, D-10, D-24) | QA-P2-S3 |
| 2 | Section 3: D-10 added to the uniform-distribution list | Architect self-check |
