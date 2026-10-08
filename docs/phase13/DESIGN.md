# Phase 13 design: the professional surface

Status: 13A and the Chara dasha half of 13B built and through the QA gate.
The remaining five Jaimini dashas, and 13C onwards, are specified only as
scope. The product owner asked for JHora parity "for
professional astrologers", and for the professional and general-public
surfaces to be *clearly separated*. This phase is both: the pro computations,
and the mode boundary that keeps them out of a lay reader's way.

## 1. What this phase is, and what "accurate" means here

Everything here is a computation, not an interpretation. A chara karaka, an
arudha pada and an argala are each a deterministic function of positions the
engine already has. So "accurate" means what it has meant since Phase 2: the
output matches this specification, including the variant defaults in section 7,
and an independent reimplementation agrees cell for cell.

What this phase does *not* claim is that the specification matches any one
practitioner's paramparā. Jaimini has more live disagreement than Parashara
does, so the variant register here is larger, and every variant is recorded on
the output rather than hidden inside it.

## 2. Pro and Lite are separate surfaces

The general public gets a reading. A professional gets the apparatus. Mixing
them makes the first group anxious and the second group impatient, so they are
two surfaces over one engine.

| | Lite (default) | Pro |
|---|---|---|
| Who it is for | the general public | astrologers |
| Tabs | Chart, Readings, Sensitive periods, Day timings, Family, Match, Saved profiles | the Lite tabs, plus Jaimini and Chara dasha today, and Balas, Vargas, Ashtakavarga, Varshaphal, KP and Yogas as they land |
| Language | plain, justified prose | technical terms, unglossed |
| Numbers shown | scores and verdicts, explained | raw values, intermediate components, variant IDs |
| Unverified output | never shown | shown, labelled |

Rules the boundary has to keep:

1. **Lite is the default.** A first-time visitor never lands in Pro.
2. **Pro is a presentation mode, not a second engine.** Both surfaces call the
   same functions. A number cannot differ between them; Pro shows more of the
   same numbers, never different ones.
3. **The mode is device-local**, stored like the theme, and survives a reload.
   It is not in the URL: a shared link must not drop a lay reader into Pro.
4. **Anything whose variant is unsigned-off is Pro-only and labelled.** This is
   how Shadbala ships (section 6) without putting an unverified number in
   front of someone who cannot tell.
5. **No Lite feature regresses.** Turning Pro on adds tabs and detail; it
   removes nothing and changes no wording a Lite reader sees.

## 3. Scope, in build order

13A is this increment. The rest is sequenced but not yet specified in detail.

| | Group | Contents | Blocked? |
|---|---|---|---|
| 13A | Jaimini core | 8 chara karakas; arudha padas A1-A12 incl. Arudha Lagna and Upapada; argala and virodhargala | **built** |
| 13B | Jaimini dashas | Chara (**built**, see `CHARA-DASHA.md`); Narayana, Sthira, Shoola, Brahma, Varnada each need a transcribed worked example first | partly |
| 13C | Foundations | upagrahas (Gulika, Mandi, Kaala, Mrityu, Ardhaprahara, Yamaghantaka, Dhuma, Vyatipata, Parivesha, Indrachapa, Upaketu); special lagnas (Bhava, Hora, Ghati, Vighati, Pranapada, Sree, Indu); vargas D-5, D-6, D-8, D-11, D-81, D-108, D-144; Vimshopaka and Dasavarga/Shodasavarga weights; avasthas | no |
| 13D | Dasha library | Ashtottari, Yogini, Kalachakra, Dwisaptati, Shattrimsa, Dwadasottari, Chaturaseeti, Shashtihayani, Shodasottari, Panchottari, Satabdika, Tribhagi, and the applicability rules that pick one | no |
| 13E | Balas | Shadbala, Bhava bala, Ishta/Kashta phala, sphuta drishti | variants unsigned; ships labelled (section 6) |
| 13F | Ashtakavarga depth | sodhya pindas (rasi, graha, sodhya), kaksha transit, trikona and ekadhipatya sodhana | no |
| 13G | Varshaphal | Muntha, Varshesha, sahams, Tri-pataki, Patyayini and Mudda dasha, Harsha and Panchavargeeya bala | no |
| 13H | KP | 249 sub-lords, cuspal sub-lords, Placidus cusps, ruling planets, significators, horary | no |
| 13I | Yogas | named yogas as reviewed corpus rules | no (needs review capacity, not data) |

Condition 3 of `docs/phase2/DESIGN.md` section 9 - sunrise and sunset in
layer 1 with an oracle test against `swetest` - is now **met**, which is why
13C and 13E are reachable at all.

## 4. Chara karakas

Eight karakas from eight candidates. Ketu is not a candidate.

**Advancement.** How far a graha has travelled through its sign:

| Graha | Advancement |
|---|---|
| Surya, Chandra, Mangala, Budha, Guru, Shukra, Shani | `degrees_in_rasi` |
| Rahu | `30 - degrees_in_rasi` |

Rahu is reversed because it moves anti-zodiacally: the sign it is leaving is
the one it has advanced through.

**Ranking.** Sort the eight by advancement, descending. The ranks, in order:

| Rank | Abbrev | Karaka | Signifies |
|---|---|---|---|
| 1 | AK | Atmakaraka | the self; the chart's own ruler |
| 2 | AmK | Amatyakaraka | career, the minister |
| 3 | BK | Bhratrikaraka | siblings, guru |
| 4 | MK | Matrikaraka | mother |
| 5 | PiK | Pitrikaraka | father |
| 6 | PuK | Putrakaraka | children |
| 7 | GK | Gnatikaraka | obstacles, disease, cousins |
| 8 | DK | Darakaraka | spouse |

**Ties.** Two grahas at the same advancement to the engine's precision is
possible in principle. The order is then the natural order Surya, Chandra,
Mangala, Budha, Guru, Shukra, Shani, Rahu, lower first. This is determinism,
not doctrine; see V-13-4.

## 5. Arudha padas and argala

### 5.1 Arudha pada

For bhava `H` in 1..12, with `S` the rasi occupying it and `L` the lord of `S`:

1. `n` = signs from `S` to the rasi of `L`, counting `S` as 1, so `n` in 1..12.
2. The arudha is the sign `n` from the rasi of `L`, counting that rasi as 1.
   Equivalently `index(rasi(L)) + n - 1` modulo 12.
3. **Exception.** If that sign is `S` itself, or the 7th from `S`, the arudha
   is the 10th sign from it instead.

Step 3 fires exactly when `n` is 1 or 7 (lands on `S`) or 4 or 10 (lands on
the 7th). A pada must not sit on its own bhava or opposite it; the texts give
the 10th as the remedy and that is what is implemented.

`A1` is the Arudha Lagna (AL) and `A12` the Upapada (UL); both are named in
the output because practice names them.

### 5.2 Argala and virodhargala

Argala is intervention in a sign's affairs; virodhargala is the intervention
that cancels it. Each argala has exactly one counter:

| Argala from | Countered by | Pair |
|---|---|---|
| 2nd | 12th | wealth |
| 4th | 10th | home |
| 11th | 3rd | gain |

**Strength.** Count the grahas occupying each sign. The argala stands if its
sign holds more grahas than its counter, is neutralised if they are equal, and
is overcome if the counter holds more. A sign with no grahas gives no argala
and no counter.

**Ketu.** Counted as an occupant, like any graha. See V-13-5.

## 6. Shadbala, and how 13E ships

The product owner chose: build it with the defaults, record the alternatives,
and label every number as unverified until sign-off. So 13E ships under three
constraints, which exist because `docs/phase2/DESIGN.md` section 9 condition 2
- reference output for five charts - is still unmet.

1. **Pro-only.** No Shadbala number appears on the Lite surface at all.
2. **Labelled at the number, not in a footnote.** Every displayed bala carries
   its unverified marking and the IDs of the variants it depends on, so a
   practitioner can see which choice produced it.
3. **No rule may consume it.** The corpus cannot condition on an unverified
   bala. A reviewed interpretation resting on an unverified number would
   launder the caveat away, and the review gate exists to stop exactly that.
   This is enforced by test, not convention.

When reference output arrives, the labels come off and constraint 3 lifts,
with no recomputation: the numbers do not change, only their standing.

## 7. Variant register

Each needs astrologer sign-off. The default applies until then and is recorded
on the output.

| ID | Question | Default | Alternatives | Source of default |
|---|---|---|---|---|
| V-13-1 | Number of chara karakas | 8, Rahu included | 7, Rahu excluded, PiK and PuK merged | Parashara's scheme; JHora's default |
| V-13-2 | Rahu's advancement | reversed, `30 - deg` | direct, as other grahas | Rahu moves anti-zodiacally; near-universal |
| V-13-3 | Ketu as a karaka candidate | no | yes, as a 9th candidate | no classical scheme lists nine karakas |
| V-13-4 | Tie-break on equal advancement | natural graha order | by dignity; by sign order | determinism; texts do not consider the case |
| V-13-5 | Ketu counted when weighing argala | yes | no; nodes excluded from occupancy | a node in a sign is an occupant everywhere else in this engine |
| V-13-6 | Lord used for arudha of a dual-lorded sign | sole traditional lord: Mangala for Vrischika, Shani for Kumbha | Ketu for Vrischika and Rahu for Kumbha; the stronger of the two | matches the lordship table the rest of the engine uses |
| V-13-7 | Argala from the 5th when it outnumbers the 9th | not applied | applied as a fourth pair | the three-pair rule is the one both Jaimini sutra commentaries agree on |
| V-13-8 | Arudha exception when the pada falls on the bhava or its 7th | 10th from the computed sign | 4th from it; leave it in place | BPHS ch. 29 and the Jaimini sutras agree on the 10th |

## 8. Architecture

```
lagn-core/src/
  jaimini.rs       chara karakas, arudha padas, argala          (new, 13A)
  upagraha.rs      upagrahas and special lagnas                 (new, 13C)
  bala.rs          Shadbala and friends                         (new, 13E)

lagn-rules/
  (unchanged in 13A: nothing here is interpreted yet)

lagn-server/src/api.rs    POST /api/jaimini
lagn-ffi/src/lib.rs       lagn_jaimini
web/src/lib/mode.ts       Lite or Pro, device-local
web/src/components/JaiminiView.tsx
```

`jaimini.rs` depends only on `Chart`, `Rasi` and `Graha`. It computes nothing
from time, so it needs no ephemeris access and no exemption from the
no-date-arithmetic rule.

## 9. Acceptance criteria (the QA gate)

13A is done when all of these hold.

1. **Karaka ranking is a total order.** For any chart, the eight karakas are
   eight distinct grahas, each rank filled exactly once.
2. **Rahu's reversal is observable.** A chart with Rahu early in its sign
   ranks it near the bottom; the same chart with Rahu late ranks it near the
   top. A test pins both.
3. **No arudha sits on its own bhava or the 7th from it.** Asserted for all
   twelve padas across the golden charts, which is the property step 3 of
   section 5.1 exists to guarantee.
4. **The arudha exception is exercised.** The golden corpus contains charts
   where `n` is 1, 4, 7 and 10, and each is pinned.
5. **Argala pairs are exactly the three in the table**, and the strength
   verdict is one of stands, neutralised, overcome, or none.
6. **An independent reimplementation agrees.** `scripts/qa_jaimini_oracle.py`
   parses sections 4 and 5 of this document, recomputes karakas, all twelve
   padas and all twelve signs' argala from the chart JSON, and disagrees
   nowhere across the golden charts and a wide random sweep.
7. **The variant defaults are on the output**, by ID, so a reader can tell
   which scheme produced what they are looking at.
8. **Lite is unchanged.** The existing web suite passes untouched, and a test
   asserts the Lite tab list is exactly what it was before this phase.
9. **Pro is off by default**, persists when set, and is absent from the URL.

## 10. Revision history

| Rev | Change | Raised by |
|---|---|---|
| 1 | Initial design | Architect |
| 2 | 13A marked built; section 11 added recording what the gate proved | Dev |

## 11. What 13A's gate actually proved

Recorded here because "the tests pass" is not the same claim as "the tests
could have failed".

- `crates/lagn-core/tests/jaimini.rs`, 14 tests, covering criteria 1 to 5 and 7.
- `scripts/qa_jaimini_oracle.py`, an independent reimplementation that parses
  the karaka order, Rahu's reversal rule and the three argala pairs out of
  sections 4 and 5 of this document at run time. 300 charts, 85,200 compared
  values, 0 disagreements.
- `crates/lagn-server/tests/api.rs`, two tests: bit-for-bit parity between
  `/api/jaimini` and the library over 200 random births, and the absence of
  any rule, status or reviewer field in the response.
- `web/src/proMode.test.tsx` and `web/src/lib/mode.test.ts`, covering criteria
  8 and 9.
- Three deliberate mutations were introduced and each was caught independently
  by both the Rust suite and the oracle:

  | Mutation | Rust suite | Oracle |
  |---|---|---|
  | arudha exception removed | 2 tests fail | 134 disagreements |
  | Rahu's reversal removed | 1 test fails | 64 disagreements |
  | gain argala countered from the 9th instead of the 3rd | 1 test fails | 223 disagreements |

What none of this proves is that the specification matches any one
practitioner's paramparā. That is what section 7 exists to collect, and no
variant in it is signed off yet.
