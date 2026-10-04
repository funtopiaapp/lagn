# Proposal: what to do about the headline score

Status: **proposal, nothing implemented.** The corpus and `resolve.rs` are
unchanged. This is the Phase 4 half of the work in
`tests/QA-VALIDATION-1.0.md`, held back because it changes interpretive
content, which `CONTRIBUTING.md` puts behind astrologer review.

Every number here was measured on the thirteen-chart validation corpus, 130
topic cells, by simulating each option against real engine output. Nothing is
asserted that was not computed.

## 1. The defect

`crates/lagn-rules/src/resolve.rs`:

```rust
score += r.polarity as i32;
```

An unbounded sum over effective rules, with no notion that two rules may
describe one condition. Subject-03's marriage score of −10 is ten afflicting
rules, in which:

- Venus is penalised three times — combust (−1), debilitated (−2), in the 6th
  or 8th (−1) — for one planet's one condition;
- Chevvai dosha is counted twice — from the lagna (−2) and from the Moon (−1)
  — for one dosha.

Against a marriage documented as intact for over fifty years. The report filed
this as failure pattern P1, "stacked afflictions overpower yogas", and
recommended capping stacked minor afflictions and widening the top of the
career scale. Both halves of that recommendation were measured. One works
partly; the other backfires.

## 2. Option A — one count per subject

Group effective rules by what they are about, and within a group of the same
polarity sign count only the strongest. The corpus already organises rules this
way in their ids, so no new metadata is needed:

```
marriage.venus.debilitated  marriage.venus.combust  marriage.venus.dusthana
        └──────────────── group "marriage.venus" ────────────────┘
```

**Measured across 130 cells: 36 move, mean absolute change 1.67.**

| | cells |
|---|---|
| unchanged | 94 |
| negative scores made less harsh | 4 |
| positive scores made less flattering | 28 |

Subject-03's marriage goes −10 → **−7**. Still strongly negative against a
fifty-year marriage, because the de-duplication only removes 3 of the 12
negative points; the remaining −9 is spread across five genuinely distinct
groups.

**This is the right change and it is not sufficient.** It is right because
counting one planet's condition three times is a defect on its own terms,
independent of any outcome. It is insufficient because it does not resolve P1,
and because 28 of its 36 movements *lower positive scores*, which makes the
report's second complaint worse rather than better.

## 3. Option B — a bounded balance, and why not

The obvious answer to "−10 is too scary a number" is to stop showing an
unbounded integer and show a balance instead: with P the positive weight and N
the negative, `(P − N) / (P + N)`, landing in [−1, +1].

It reads well on the failures. It destroys the top of the scale:

| Career | now | balance |
|---|---|---|
| Subject-01 | +10 | **+1.00** |
| Subject-02 | +6 | **+1.00** |
| Subject-04 | +6 | **+1.00** |
| Subject-05 | +1 | **+1.00** |
| Subject-06 | +2 | **+1.00** |
| Subject-07 | +7 | **+1.00** |
| Subject-08 | +4 | **+1.00** |
| Subject-12 | +10 | **+1.00** |

Eight of thirteen careers collapse onto the same value, because a chart with
no afflicting career factor scores +1.00 whether one favourable rule fires or
seven. A once-in-a-generation career and a quiet one become indistinguishable
— exactly the complaint the change was meant to answer, made worse.

**Rejected.** A single number cannot carry both how one-sided the evidence is
and how much evidence there is.

## 4. What the career measurement actually shows

The report read P2 as a compressed scale. It is not. Subject-05's career
scores +1 because **exactly one** scored career rule fires:

```
career.h10.benefic_occupant  +1  A natural benefic occupies the 10th house
```

One rule out of sixteen scored career rules in the corpus. Across the
thirteen:

| Topic | scored rules in corpus | fire per chart (mean) | range |
|---|---|---|---|
| marriage | 25 | 4.5 | 1–10 |
| career | 16 | 4.2 | 1–7 |
| wealth | 13 | 4.7 | 3–7 |
| health | 15 | 4.1 | 1–7 |
| parents | 14 | 3.8 | 3–6 |
| education | 11 | 3.5 | 2–6 |
| progeny | 11 | 2.6 | 1–4 |
| vitality | 8 | 2.2 | 0–4 |
| later_life | 7 | 2.2 | 0–5 |

A verdict resting on one rule and a verdict resting on seven are presented
identically, as a signed integer with a confident label. That is the real
defect behind P2, and it is the same *kind* of defect as D7 from the
validation report: the engine knows how much it knows and does not say.

## 5. Recommendation

Three changes, in this order. The first two are mechanical and need no
astrological judgment; only the third touches interpretation.

**1. De-duplicate (Option A).** Count one condition once. Defensible on its
own terms whatever it does to any grade. 36 cells move; `validation13.json`
must be regenerated and the movements reviewed subject by subject.

**2. Say what the verdict rests on.** Alongside the score, report how many
scored rules applied out of how many were checked — "4 of 16 classical factors
apply to this chart". Costs no interpretive decision, and it is what separates
Subject-05's career from Subject-01's. This also makes the sparse topics
honest: vitality and later life average 2.2 factors and sometimes zero.

**3. Then, and only then, consider the headline.** With de-duplication and an
evidence count in place, the question of whether −7 should be shown as "−7",
as "mostly adverse", or as a band is a presentation choice that can be made on
evidence rather than guesswork. Do not do this before 1 and 2: Option B shows
how a plausible-looking rescaling can destroy discrimination that the raw sum
actually had.

## 6. What not to do

**Do not recalibrate to fix Subject-06.** Its marriage −4 and children −2 are
the report's clearest directional failures, and both are artifacts of a birth
time one minute from a rasi boundary that conflicts with the commonly quoted
time. Two minutes later the children verdict is +2, which is correct. Fitting
rules to that chart would be fitting them to a wrong input.
`tests/QA-VALIDATION-1.0.md` section 4 separates the six robust failures from
these two.

**Do not treat the thirteen as validation.** All are famous and nearly all
wealthy, which is why wealth scored 11 Pass of 13 and why that number means
nothing. The corpus is a regression lock and a source of counter-examples, not
evidence that the engine is right.

## 7. If this is approved

```bash
# after changing resolve.rs and/or the corpus
cargo build --release -p lagn-cli
python3 scripts/make_validation13.py     # regenerate the expectations
git diff tests/validation13.json  # review every cell that moved
make qa
```

`crates/lagn-rules/tests/validation13.rs` will fail first, naming each cell
that moved and by how much. That failure is the review surface: every moved
cell should be a subject someone can explain. Rules changed in the corpus need
a `reviewer`, a date and reasoning, and the content remains AI-reviewed rather
than astrologer-reviewed until that gate is passed —
`lagn review-sheet --topic <name>` generates the sheet for a practitioner.
