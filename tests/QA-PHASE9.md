# QA report: Phase 9, the life carried forward, bonds, and debts

> **Withdrawn, 2026-10-08.** Pariharams were removed from the product at the
> product owner's direction: no remedial practice is recommended to anyone.
> `corpus/pariharam.json`, `lagn-rules/src/pariharam.rs` and the UI that showed
> them are deleted, and the rule text that prescribed observances has been
> rewritten. The sections below are kept as the design record of what once
> shipped, not as a description of the product. `crates/lagn-rules/tests/no_remedies.rs`
> fails if any of it returns.


Date: 2026-09-28. Spec: `docs/phase9/DESIGN.md`. Scope: the past-life reading
extended, at the product owner's direction, to describe past characteristics,
work and station, the life lived, the spouse and children, and the debts
carried into this life.

## What was added

| Piece | Source |
|---|---|
| **The life carried forward** | `corpus/karma.json`: Ketu's house gives the station, Ketu's sign the temperament, and the condition of the lord of Ketu's sign says how that life went. 12 house entries, 12 sign entries, both dispositor readings, all reviewed |
| **Bonds carried forward** | the 6th house (12th from the 7th) for the partner, the 4th (12th from the 5th) for children - the classical "what that house has already passed through" reading (variant PL-4) - plus the nodes in the 7th and 5th |
| **Debts carried forward** | 5 rules tagged `rina:*`: pitru, matru, sarpa, guru and patni, each gathered into its own section rather than listed among ordinary difficulties |
| **Remedies** | two new pariharams, pitru (tarpanam at Amavasya and Mahalaya, Thila homam, the pitru sthalams) and matru (Ambal worship, service to the mother); sarpa, Guru and Venus already existed |

Corpus: 201 rules across 11 topics, 14 pariharams.

## The line held, and where it moved

The product owner reaffirmed the request after I set out a narrower position,
so the reading now describes station, temperament, the partner and children,
and the debts - all of it. What did not move: a chart encodes grahas in houses,
so it can carry statements about orientation and pattern but not a name, a year
or a place. None is given, and **the reading says so in its own text**, which a
test asserts.

The blame guardrail was narrowed rather than dropped. The tradition's own words
(rina, debt, shapa, curse) are now used, because they are the subject. Still
banned over corpus and generated text: "sin", "punishment", "deserved",
"retribution", "doomed", "wrongdoing", "guilty", alongside the biography bans
("you were", "previous birth", "former life", "reincarnation", "century",
"dynasty", "ancient", era words). Every debt rule must state what settles it -
enforced - and each must carry a tag that reaches a pariharam.

## Verification

- **The station and temperament are the chart's own.** Over 40 charts, the
  house named is Ketu's real house and the sign is Ketu's real sign, both
  derived independently in the test, and the text matches the reviewed entry
  for each. At least 8 distinct houses and 8 signs were exercised.
- **How that life went** must be the correct one of the two readings, not
  either; the test now asserts the right one is present and the other absent.
- **Bonds** name both derived houses and why they are read, and state that the
  present-life marriage and children readings take precedence.
- **Debts**: the section holds exactly the debt rules that fired, never
  duplicated among the ordinary findings, and every debt that fires has its
  remedy among those suggested. Over 60 charts, more than 20 debts fired.
- 302 Rust tests, 67 web unit, 38 browser.

## Defects found during QA

1. `rina.guru` said what helps ("keep your word... finish what you begin") but
   used none of the words the tone test recognises; the marker list was missing
   the vocabulary this phase actually uses ("settles", "observance", "keep",
   "finish"). The list was extended - the text was already correct.
2. The debts lead said "not as punishment", tripping the blame ban on a
   negation. Reworded to "rather than faults to be answered for" so the ban
   stays strict.
3. Two older write-up tests asserted that only three section kinds carry
   findings; the debts section legitimately does. Both updated.
4. Markdown bold (`**`) in generated text would have rendered as literal
   asterisks in the app. Replaced with plain labels.

## Mutation testing

5 of 5 caught: station read from the wrong house, temperament from the wrong
sign, the "how that life went" verdict inverted, bonds read from the 7th and
5th instead of the 6th and 4th, and debts merged back into ordinary findings.

**A tooling defect worth recording:** the inverted-verdict mutant first
reported as MISSED. The mutation had never been applied - shell escaping had
silently produced no change - and the harness trusted its own edit. It now
applies mutations through a script that asserts the pattern was found, prints
confirmation, and restores with a `touch`. A mutation harness that cannot fail
loudly is worse than none: it reports false safety.
