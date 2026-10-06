# Phase 3 design: rule engine and marriage corpus

Status: built and QA-passed. Revision 2 (2026-09-21): two QA corrections, see section 13.
Depends on Phase 1 and Phase 2A/2B. Doesn't use Shadbala (2C is blocked).

## 1. What Phase 3 is, and what "accurate" means here

Phases 1 and 2 compute facts: where a graha is and what state it's in. Phase 3
**interprets** them. It has two very different parts, and they carry
different accuracy guarantees.

| Part | Nature | How accuracy is established |
|---|---|---|
| **Rule engine**: language, evaluator, cancellation, gating, timing | Software | Proved: tests, an N-version oracle, mutation testing, as in Phase 2 |
| **Porutham procedure** (section 8) | Fixed tables | Proved against this spec by an exhaustive oracle; each table needs astrologer sign-off |
| **Rule corpus**: what each configuration *means* | Interpretive content | **Only by astrologer review.** QA can prove a rule does what it says. It can't prove what it says is true. |

To meet the requirement that no mistakes reach a user, **the review gate
(section 5) is the central design decision.** No rule and no porutham reaches
production output until a named reviewer has approved it. Everything written
in Phase 3 starts as a draft.

**Citations.** A rule may cite a text only if the citation has been checked
against the text. Drafts written from general knowledge set `reference` to
null and say so in `note`. Inventing chapter and verse numbers is forbidden.

## 2. Scope

**Build:**

1. The rule language (section 3) and its three-valued evaluator (section 4).
2. The review gate (section 5), cancellation resolution (section 6) and dasha
   timing windows (section 7).
3. Ten Poruthams (section 8). All ten are built; Vasya was blocked on a
   transcribed table.
4. A marriage corpus of about 35 draft rules, including Kuja (Chevvai) dosha and
   its cancellations, expressed in the rule language (section 9).
5. A corpus validator, and a review-sheet generator that produces the document
   the astrologer approves from.
6. CLI: `rules validate`, `topic`, `match` and `review-sheet`.

**Not built:** rules needing strength (Shadbala), Jaimini karakas and arudhas,
other topics. The engine is topic-agnostic, so new topics are corpus-only
additions.

## 3. Rule language

The corpus is JSON: one file per topic area, each holding an array of rules.
JSON keeps the parser dependency-free and deterministic. The astrologer
reviews through the generated review sheet (section 10), not raw JSON.

### 3.1 Rule

| Field | Type | Meaning |
|---|---|---|
| `id` | string | Unique, dotted, e.g. `marriage.l7.dusthana` |
| `topic` | string | e.g. `marriage` |
| `title` | string | One line |
| `tradition` | enum | `parashari`, `tamil`, `kerala` |
| `when` | Condition | When the rule fires |
| `polarity` | integer, -3 to 3 | Direction and weight. 0 means informational or cancelling |
| `overrides` | ids | Rules this one cancels when both fire (section 6) |
| `timing` | GrahaRefs | Dasha lords that activate the rule (section 7) |
| `tags` | strings | Free grouping, e.g. `dosha:kuja` |
| `text.en` | string | What the rule says, in plain language |
| `source.reference` | string or null | A checked textual citation, or null |
| `source.note` | string | Provenance in words |
| `review.status` | `draft`, `approved`, `rejected` | Section 5 |
| `review.reviewer`, `review.date`, `review.comment` | string or null | Required when approved |

Integers only, so scores are exact sums with no floating-point drift.

### 3.2 References

A **GrahaRef** is either a graha name (`"mars"`) or
`{"lord_of": H, "varga": "d1"}`: the lord of house H counted from that
varga's lagna. The varga defaults to D-1.

A **GrahaSet** is one of `"benefics"`, `"malefics"`, `"any"` or
`{"grahas": [GrahaRef, ...]}`.

Benefic and malefic classification (variant R-1): Jupiter, Venus and Mercury
are benefic. The Moon is benefic when waxing, that is when its elongation from
the Sun is in [0, 180). The Sun, Mars, Saturn, Rahu and Ketu are malefic, and
so is the Moon when waning. Every graha is exactly one of the two.

### 3.3 Conditions

Houses are counted from the lagna of the given varga (default D-1). All sign
tests are whole-sign.

| Condition | Arguments | True when |
|---|---|---|
| `all` | [Condition] | Kleene AND |
| `any` | [Condition] | Kleene OR |
| `not` | Condition | Kleene NOT |
| `always` | `{}` | Always true, for timing-only rules |
| `graha_in_house` | graha, houses, varga? | the graha occupies one of the houses |
| `graha_in_sign` | graha, signs, varga? | the graha occupies one of the signs |
| `graha_from` | graha, from, houses | the graha is in one of the houses counted from `from` (`"lagna"` or a GrahaRef), in D-1 |
| `dignity` | graha, in, varga? | the graha's Phase 2 dignity is one of `in`; **Unknown** for Rahu and Ketu |
| `house_occupied` | house, by, min?, varga? | at least `min` (default 1) grahas from set `by` occupy the house |
| `house_aspected` | house, by, min? | at least `min` grahas from `by` cast graha drishti on the house (D-1, node aspects per Phase 2 V-3) |
| `conjunct` | a, b | a and b share a D-1 sign; **false if they resolve to the same graha** |
| `aspects_graha` | from, to | `from` casts graha drishti on `to`'s D-1 sign |
| `combust` | graha | Phase 2 combustion |
| `retrograde` | graha | vakri; always false for the nodes |
| `sav` | house, min?, max? | the SAV of that house is within [min, max], inclusive |
| `native` | sex | the native's stated sex matches; **Unknown** if not supplied |

## 4. Evaluation semantics

Three-valued (Kleene) logic, because a missing fact must never quietly
become false.

| | all | any | not |
|---|---|---|---|
| any operand False | **False** | if no True and no Unknown: False | T becomes F |
| any operand True | if all are True: **True** | **True** | F becomes T |
| otherwise | Unknown | Unknown | Unknown stays Unknown |

- A rule **fires** only when `when` evaluates to True.
- A rule that evaluates to Unknown is reported under "could not evaluate",
  with the missing fact named. It is never silently dropped.
- Evaluation is pure and deterministic. Every evaluation produces a trace:
  each atomic condition, its value, and the facts it read (for example "lord
  of 7 = Mars; Mars in house 8"). The trace generates the explanation.

## 5. The review gate

| Mode | Rules evaluated and shown |
|---|---|
| **production** (default) | `approved` only |
| **review** | `approved` and `draft`, each draft visibly marked DRAFT |

`rejected` rules are never evaluated. They stay in the corpus as a record.
The same gate applies to each porutham (section 8.3). The corpus validator
rejects any `approved` rule that lacks a reviewer and a date.

**Consequence:** until the astrologer approves rules, production output is
deliberately empty apart from a count of rules withheld. That is the correct
behaviour under the no-mistakes requirement, not a defect.

## 6. Cancellation (bhanga)

Classical texts express exceptions as cancellations: a dosha stands unless a
more specific condition cancels it. The engine models this directly instead of
guessing from scores.

- `overrides: [ids]` means: when this rule is **effective**, the listed rules
  are cancelled.
- A rule is **effective** iff it fires and no effective rule cancels it. A
  cancellation can itself be cancelled.
- The validator requires the override graph to be acyclic, every target to
  exist, and no rule to override itself. Effectiveness is computed in
  topological order, so the result is unique.
- A cancelled rule is still reported, together with the rule that cancelled
  it. When several effective rules cancel it, the one with the smallest id
  (byte-wise string order) is reported. Cancellation is information, not deletion.

**Topic result.** For effective rules, list the supporting rules (polarity >
0) and the afflicting rules (< 0) separately, plus the score (the sum of
polarities). The label is `supportive`, `afflicted`, `mixed` (both sides
present) or `neutral`. The engine never collapses a mixed picture into a
single verdict.

## 7. Timing

A rule's `timing` lists GrahaRefs. For each effective rule with timing, the
engine lists every Vimshottari antardasha within a requested age range whose
mahadasha lord *or* antardasha lord is in the set. Output rows are start,
end, maha, antar and which lord matched. Ages are measured from birth in
the chart's dasha year length.

## 8. Poruthams (two-chart matching, Tamil 10-porutham system)

Inputs: the Moon's nakshatra and rasi for bride and groom. Counts go from the
**bride's** star or sign to the **groom's**, inclusively (1 to 27, or 1 to 12).
Let `n` be the nakshatra count and `r` the rasi count.

### 8.1 Rules

| # | Porutham | Matching when | Variant |
|---|---|---|---|
| 1 | Dina | `n mod 9` is 0, 2, 4, 6 or 8 | P-1 |
| 2 | Gana | same gana, or one Deva and one Manushya; not matching if exactly one is Rakshasa | P-2 |
| 3 | Mahendra | `n` is 4, 7, 10, 13, 16, 19, 22 or 25 | none |
| 4 | Stree Deergha | `n > 13` | P-3 |
| 5 | Yoni | the two yoni animals are not an enemy pair | P-4 |
| 6 | Rasi | `r` is not 2, 6, 8 or 12 | P-5 |
| 7 | Rasi Adhipati | the rasi lords are the same, or neither regards the other as a natural enemy (Phase 2 table) | P-6 |
| 8 | Vasya | built 2026-10-05; four of its twelve rows are the majority reading of disagreeing sources, and are marked as such | P-7 |
| 9 | Rajju | the two nakshatras are in different rajjus | P-8 |
| 10 | Vedha | the pair is not a vedha pair | none |

Rajju and Vedha failures are reported as **critical**. The summary gives
matched / evaluated and lists the critical failures. It never issues a
marriage recommendation; that judgment belongs to the astrologer.

### 8.2 Tables

**Gana:**

| Gana | Nakshatras |
|---|---|
| Deva | Ashwini, Mrigashira, Punarvasu, Pushya, Hasta, Swati, Anuradha, Shravana, Revati |
| Manushya | Bharani, Rohini, Ardra, Purva Phalguni, Uttara Phalguni, Purva Ashadha, Uttara Ashadha, Purva Bhadrapada, Uttara Bhadrapada |
| Rakshasa | Krittika, Ashlesha, Magha, Chitra, Vishakha, Jyeshtha, Mula, Dhanishta, Shatabhisha |

**Yoni:**

| Animal | Nakshatras |
|---|---|
| Horse | Ashwini, Shatabhisha |
| Elephant | Bharani, Revati |
| Sheep | Krittika, Pushya |
| Serpent | Rohini, Mrigashira |
| Dog | Ardra, Mula |
| Cat | Punarvasu, Ashlesha |
| Rat | Magha, Purva Phalguni |
| Cow | Uttara Phalguni, Uttara Bhadrapada |
| Buffalo | Hasta, Swati |
| Tiger | Chitra, Vishakha |
| Deer | Anuradha, Jyeshtha |
| Monkey | Purva Ashadha, Shravana |
| Mongoose | Uttara Ashadha |
| Lion | Dhanishta, Purva Bhadrapada |

**Yoni enemy pairs:** Horse and Buffalo; Elephant and Lion; Sheep and Monkey;
Serpent and Mongoose; Dog and Deer; Cat and Rat; Cow and Tiger.

**Rajju:**

| Rajju | Nakshatras |
|---|---|
| Pada | Ashwini, Ashlesha, Magha, Jyeshtha, Mula, Revati |
| Kati | Bharani, Pushya, Purva Phalguni, Anuradha, Purva Ashadha, Uttara Bhadrapada |
| Nabhi | Krittika, Punarvasu, Uttara Phalguni, Vishakha, Uttara Ashadha, Purva Bhadrapada |
| Kanta | Rohini, Ardra, Hasta, Swati, Shravana, Shatabhisha |
| Siro | Mrigashira, Chitra, Dhanishta |

**Vedha pairs:** Ashwini and Jyeshtha; Bharani and Anuradha; Krittika and
Vishakha; Rohini and Swati; Ardra and Shravana; Punarvasu and Uttara Ashadha;
Pushya and Purva Ashadha; Ashlesha and Mula; Magha and Revati; Purva Phalguni
and Uttara Bhadrapada; Uttara Phalguni and Purva Bhadrapada; Hasta and
Shatabhisha; Mrigashira and Chitra; Mrigashira and Dhanishta; Chitra and
Dhanishta.

**Vasya:** the signs each rasi holds sway over. The check is directional: the
bride's rasi must fall under the groom's sway, which is the reading Tamil
sources most often print rather than a mutual test. Printed tables agree on
eight rows and disagree on four - Tula, Vrischika, Makara and Kumbha - which
are recorded here as the majority reading and reported as disputed.

| Rasi | Holds sway over |
|---|---|
| Mesha | Simha, Vrischika |
| Vrishabha | Karka, Tula |
| Mithuna | Kanya |
| Karka | Vrischika, Dhanus |
| Simha | Tula |
| Kanya | Mithuna, Meena |
| Tula | Makara, Kanya |
| Vrischika | Kanya, Karka |
| Dhanus | Meena |
| Makara | Mesha, Kumbha |
| Kumbha | Mesha |
| Meena | Makara |

**Naadi:** sharing a naadi is what the texts object to, as with Rajju. The
assignment runs 1-2-3-3-2-1 along the twenty-seven, which is why it is quoted
as a zig-zag and not a repeating triple.

| Naadi | Nakshatras |
|---|---|
| Adi | Ashwini, Ardra, Punarvasu, Uttara Phalguni, Hasta, Jyeshtha, Mula, Shatabhisha, Purva Bhadrapada |
| Madhya | Bharani, Mrigashira, Pushya, Purva Phalguni, Chitra, Anuradha, Purva Ashadha, Dhanishta, Uttara Bhadrapada |
| Antya | Krittika, Rohini, Ashlesha, Magha, Swati, Vishakha, Uttara Ashadha, Shravana, Revati |

**Varna:** by the element of the moon-sign. The check is that the groom's varna
does not stand below the bride's, ranked Brahmin above Kshatriya above Vaishya
above Shudra.

| Varna | Rasis |
|---|---|
| Brahmin | Karka, Vrischika, Meena |
| Kshatriya | Mesha, Simha, Dhanus |
| Vaishya | Vrishabha, Kanya, Makara |
| Shudra | Mithuna, Tula, Kumbha |

**Self-checks QA must verify:** each gana has 9 nakshatras and the three are
disjoint. Each rajju follows the 1-2-3-4-5-4-3-2-1 zigzag through the 27. The
yoni table covers all 27 exactly once. Vedha is symmetric, and every
nakshatra has at least one vedha partner.

### 8.3 Gating

Each porutham carries its own review status in
`corpus/marriage/porutham.review.json`, with the same production and review
semantics as rules.

### 8.4 Variant register

| ID | Question | Default | Alternatives |
|---|---|---|---|
| P-1 | Dina when both have the same nakshatra (n = 1) | not matching, by the rule | several texts accept particular same-star pairs |
| P-2 | Gana cross-combinations | Deva with Manushya matches either way round | only groom Deva, bride Manushya matches; Manushya with Rakshasa is conditional |
| P-3 | Stree Deergha threshold | n > 13 | n > 9 (madhyama); n > 15 |
| P-4 | Yoni grading | enemy pair means no match; otherwise match | graded scale by male or female animals |
| P-5 | Rasi disqualifying counts | 2, 6, 8, 12 | 6 and 8 only; excuse 6/8 when the lords are friends |
| P-6 | Rasi Adhipati basis | natural relationship, both directions | compound; one direction only |
| P-7 | **Vasya table** | majority reading, four rows disputed | an astrologer confirms Tula, Vrischika, Makara and Kumbha against a panchangam they trust |
| P-8 | Rajju exceptions (aarohana/avarohana) | none; same rajju means no match | allow some same-rajju cases by direction |

### 8.5 Chart-level variants

| ID | Question | Default | Alternatives |
|---|---|---|---|
| R-1 | Benefic and malefic classification | section 3.2 | Mercury turns malefic when conjunct malefics |
| R-2 | Kuja dosha reference points | lagna and Moon approved-eligible; Venus as a separate draft | lagna only; lagna, Moon and Venus |
| R-3 | Kuja dosha houses | 2, 4, 7, 8, 12 | 1, 4, 7, 8, 12 (common in North India) |
| R-4 | Kuja cancellation list | section 9 | the reviewer edits the corpus |

## 9. Marriage corpus (draft content)

About 35 rules in these groups: the 7th house and its lord (D-1 and D-9),
Venus as kalatra karaka, delay indicators, Jupiter for a female native, SAV
of the 7th, Kuja dosha from lagna, Moon and Venus, and Kuja cancellations. The
cancellations are own sign or exaltation, Jupiter's conjunction or aspect,
and house-specific sign exceptions, all written as `overrides`. There is one
timing rule for marriage periods.

Every rule ships with `status: draft`, `reference: null` and a note saying the
textual source is for the reviewer to supply. Rule text is non-fatalistic:
it reports what classical texts associate with a configuration, never what
*will* happen.

## 10. Review sheet

`lagn review-sheet --topic marriage` writes a Markdown document for the
astrologer. For each rule and porutham it gives the id, title, status and the
condition in plain language, generated from the rule, not hand-written. It
also gives polarity, cancellations, the source note, and **how often the rule
fires across 1,000 sample charts**, which exposes rules that are too broad or
never fire. Approving a rule means setting `review.status`, `reviewer` and
`date` in the corpus.

## 11. Architecture

New crate `lagn-rules`, which depends on `lagn-core`. It is pure, with no I/O
except the corpus loader.

```
lagn-rules/src/
  model.rs      Rule, Condition, GrahaRef, GrahaSet, Review, Source
  facts.rs      FactBase: chart + analysis + ashtakavarga + optional native data
  eval.rs       three-valued evaluator and trace
  resolve.rs    gate, cancellation, topic result
  timing.rs     dasha windows
  explain.rs    plain-language rendering of conditions
  corpus.rs     loader and validator
  porutham.rs   section 8
corpus/marriage/*.json
```

## 12. Acceptance criteria (the QA gate)

1. Kleene truth tables tested exhaustively for `all`, `any` and `not`.
2. Every atomic condition tested against facts computed independently from
   `lagn-core`.
3. **N-version evaluator.** QA writes an independent Python evaluator for
   section 4, reading the same corpus JSON, and diffs every rule's outcome
   (True, False or Unknown) and every topic result over at least 2,000
   charts, in both modes. Zero disagreements.
4. **Porutham, exhaustive.** Python parses the section 8 tables from this
   document and checks all 108 x 108 bride and groom pada combinations
   (11,664 pairs). These collapse onto 36 x 36 = 1,296 distinct (nakshatra,
   rasi) pairs, because 9 of the 27 nakshatras straddle two signs, which
   makes 27 + 9 = 36. That is the complete input space. Zero disagreements.
5. **Gate property.** Across random corpora and charts, production mode never
   emits a draft or rejected rule, or an unapproved porutham.
6. **Cancellation.** Cycles, dangling targets and self-overrides are rejected;
   cancelling a cancellation works; results are independent of rule order.
7. **Validator negative tests,** one per validation rule.
8. **Mutation check:** one Kleene operator, one porutham table cell, one
   override edge and one gate check. Each must be caught.
9. No regressions: the 196 existing tests, the swetest cross-validation, the
   Phase 2 oracle, clippy clean, release and debug.

## 13. Revision history

| Rev | Change | Raised by |
|---|---|---|
| 1 | Initial design | Architect |
| 2 | Section 6: tie-break for the reported canceller (smallest id) | QA-P3-S1 |
| 2 | Section 12: porutham input space stated as 1,296 distinct pairs | QA-P3-S2 |
