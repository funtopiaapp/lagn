# QA: validating the independent 1.0 report

An independent pass tested 1.0 against thirteen living public figures with
published birth data (2026-10-01/02), grading every reading against researched
life events. Its raw report is not in this repository; the subjects stay
anonymised here as Subject-01 … Subject-13, as that report left them.

This document records what happened when each of its findings was reproduced
against the engine, which of them survived, and what now tests them.

**Summary: the report's data is excellent and its diagnoses are mostly wrong.**
All thirteen charts reproduce exactly, and 128 of its 130 topic scores match
the engine cell for cell. But of eleven logged defects, three were real, one
was already fixed, five were not reproducible, and one was backwards — it
advised doing the wrong thing. Meanwhile the thing it ranked below its own
release blocker, the scoring model, is the one real problem.

Everything below was reproduced with the release CLI:

```bash
cargo build --release -p lagn-cli
export LAGN_EPHE_PATH=ephe
./target/release/lagn topic marriage --date 1988-11-05 --time 07:30 \
    --lat 28.65195 --lon 77.23149 --tz 5.5 --sex male --writeup
```

## 1. What reproduced

Every lagna, to the arcsecond, and every score checked. The two cells that
differ are both ones where the report's own parenthetical number contradicts
its own verbal label (Subject-09 education and later life, written "+1" but
labelled "Even"); the engine agrees with the label. The fixture in
`tests/validation13.json` now pins all 130 cells, and
`crates/lagn-rules/tests/validation13.rs` replays them on every `make qa`.

The fixture locks the numbers. It does not bless them: several are wrong about
the documented life, which is the subject of section 4.

## 2. The defect log, finding by finding

| ID | Severity claimed | Verdict | What is actually true |
|----|------------------|---------|-----------------------|
| **D1** | **High, release blocker** | **Not reproducible** | The claim was that Because-citations name house placements contradicting the chart — Saturn cited in the 10th while shown in the 3rd. The engine says the opposite. Its own write-up for Subject-06 reads *"Saturn (Shani) … is in your 3rd house, in Dhanus"* and *"Mercury (Budha) … is in your 1st house, in Tula"*, both matching the chart table. An extraction artifact in the tester's pipeline, which that report's own Appendix B had flagged as possible. **The release blocker does not exist.** |
| **D2** | Medium | **Not reproducible** | No zero-length period exists on any of the thirteen charts: 910 periods each, shortest 131 hours. The *shape* that would cause one is real — `view.rs` drops periods ending before birth and clamps the rest up to the birth moment, so a period ending later the same day would render start == end — so the absence is now pinned by a test rather than left to luck. |
| **D3** | Medium | **Not reproducible** | The count is arithmetically correct: 13 windows render as 12 listed plus "1 more". Two real but cosmetic faults were found in that same line and fixed: it said "1 more periods", and it never said the table holds all of them, which is what made the number look wrong. |
| **D4** | Low | **Confirmed** | The marriage subtitle promised a Chevvai dosha verdict unconditionally. It was a static `summary` string in `corpus/topics.json`. Fixed — see section 3. |
| **D5** | Low | **Not reproducible** | `DashaTimeline.tsx` always renders `.lord`; no CSS hides it at any depth, and no period in any fixture has an empty lord. |
| **D6** | Low | **Misdiagnosed** | Health carries **22 rules and zero timing rules**; parents 15 and zero. Every other topic has exactly one. Health is therefore uniformly untimed *by construction*, not inconsistently. The message the report quotes, "Health is not timed by dasha in this engine", **does not exist anywhere in the codebase**. The real fault was smaller and different: the timing section simply vanished, with no explanation. Fixed — see section 3. |
| **D7** | Medium | **Confirmed, and understated** | There was no birth-time confidence anywhere; the only confidence badge in the app is for the *timezone* suggestion. This was the report's best finding and section 4 shows it mattered more than the report knew. Fixed — see section 3. |
| **D8** | Low | **Confirmed, wrong cause** | Paramakudi **is** in the place data — as "Paramagudi", population 95,579, at the correct coordinates — with an empty `alternates` field. So it was never a data gap, and the recommended data refresh would have changed nothing. It was a search defect: exact and prefix matching cannot cross a k↔g transliteration. Fixed — see section 3. |
| **D9** | Low | **Not a defect** | The claim was that "Shani in Kanya" is cited for lagna-, 6th- and 8th-house factors though Saturn occupies only the 1st. Saturn *is* in the 1st (Kanya lagna), and "the 6th lord is in the lagna" is a different rule: Saturn lords Kumbha, the 6th from Kanya. Both citations are correct, and no 8th-house factor appears in that reading at all. |
| **D10** | Low | **Already fixed** | The footer reads, in one sentence: *"Visits are counted by a third party, which sees your address and browser. No cookies, no accounts."* The report quoted the second half. |
| **U1** | Note | **Backwards** | The app suggested UTC+6:30 for a 1942 Indian birth and the report called that "historically dubious", overriding it to +5:30. IANA has India on **+6:30 from 1942-09-01 to 1945-10-15** — wartime Indian Summer Time. The app was right and the override introduced a one-hour error. Acting on this recommendation would make the engine wrong for every Indian birth in that three-year window. |

On U1, the error happens not to have changed Subject-03's grades: the lagna
moves from Kumbha 21°33′ to Kumbha 01°56′, staying in the same rasi, so the
whole-sign houses are identical and marriage scores −10 at both offsets. The
finding survives; the advice must not be followed.

## 3. What was fixed, and the test for each

| Defect | Fix | Test |
|---|---|---|
| D4 | `TopicMeta` gained an optional `dosha` field; the write-up now states the verdict in all three states — applies, applies-but-cancelled, absent — and the subtitle no longer promises one. Data-driven: the corpus names the dosha and its rule prefix, so no topic name is written into the engine. | Subjects 02 and 09, the two the report named, now read "Chevvai dosha does not apply in this chart." |
| D6 | The timing section now always appears, saying either that no period in range was picked out, or that the engine does not time this area at all because no classical timing rule for it has been reviewed. | `validation13.rs` |
| D7 | `Chart::lagna_window()` computes, per chart, how many minutes the lagna holds its rasi either side of the birth moment. The UI shows it above the verdicts, and warns when the margin is under half an hour. | `the_lagna_margin_matches_a_recomputed_boundary`, `subject_06_is_the_knife_edge_chart`, `LagnaMargin.test.tsx` |
| D8 | A transliteration fold — aspirates, voicing, doubled letters, long vowels — tried *only* when exact and prefix matching find nothing, so an exact hit can never be displaced. Fixes the whole class, not one town. | `a_different_romanisation_still_finds_the_town`, `the_fold_never_displaces_an_exact_match` |
| D2 | No code change; the absence is now asserted across all thirteen charts and every level of the dasha tree. | `no_dasha_period_renders_as_zero_length` |
| D3 | Pluralisation, and the line now says how many the table holds. | — |
| (new) | The write-up said "Venus (Shukra), the lord of the 7th and Shukra" when a graha timed an area both as house lord and as karaka. A real role now always wins over the graha's own name. | — |

## 4. The finding the report did not make

Every house in every reading is counted from the lagna. `lagna_window()` makes
it possible to ask, per chart, how much the birth time actually matters:

| Subject | Lagna | Margin to the next rasi | Birth-time confidence |
|---|---|---|---|
| 06 | Tula 29°48′ | **1 minute** | C− (conflicts with the quoted 10:28) |
| 11 | Dhanus 00°42′ | **3 minutes** | D (noon default) |
| 05 | Vrischika 01°15′ | **6 minutes** | C+ (round number, "Approx") |
| 10 | Mesha 10°32′ | 16 minutes | D (noon default) |
| 02 | Kanya 04°21′ | 18 minutes | D (noon default) |
| 03 | Kumbha 21°33′ | 25 minutes | B (published) |
| 09, 07, 01, 12, 08, 13, 04 | — | 32–68 minutes | mixed |

Subject-06 is one minute from Vrischika, on a birth time the report itself
says conflicts with the commonly quoted one. Two minutes later:

| Subject-06 | as tested, 07:30 | 07:32 | quoted 10:28 |
|---|---|---|---|
| marriage | −4 | −1 | −5 |
| children | **−2** | **+2** | +1 |
| career | +2 | −2 | +3 |

Both of Subject-06's Fails — marriage −4 against a stable marriage, children
−2 against two children — soften or reverse across two minutes of clock time.
The children verdict becomes +2, which is the correct call. **Those two grades
are artifacts of the birth time, not of the rules.**

Sweeping every reported Fail across ±15 minutes separates the two kinds:

| Reported Fail | Robust to ±15 min? | Therefore |
|---|---|---|
| S03 marriage −10 (50-year marriage) | yes, and across both tz offsets | genuine scoring problem |
| S03 parents −2 (parents lived into their 90s) | yes | genuine |
| S05 progeny +2 (childless at 76) | yes | genuine |
| S05 marriage 0 (separated after ~3 months) | yes | genuine |
| S10 marriage −6 (stable 8-year marriage) | yes | genuine |
| S01 education +3 (schooling ended early) | yes | genuine |
| **S06 marriage −4** | **no, −1 at +5 min** | birth-time artifact |
| **S06 progeny −2** | **no, +2 at +5 min** | birth-time artifact |

Six of eight are real targets for recalibration. Two are not, and recalibrating
the corpus to "fix" them would be fitting rules to a wrong birth time.

## 5. What is actually wrong with 1.0

Not D1. `resolve.rs` sums polarities with no cap and no notion that two rules
may describe one condition:

```rust
score += r.polarity as i32;
```

Subject-03's marriage −10 is ten afflicting rules in five groups, in which
Venus is penalised three times over (combust −1, debilitated −2, in the 6th or
8th −1) and Chevvai dosha is counted twice (from the lagna −2, from the Moon
−1). That is the mechanism behind the report's failure pattern P1, and it is a
design fault, not a wording one.

Its pattern P2, that exceptional careers are under-discriminated, has a
different cause than the report supposed, and the measurements are in
`docs/SCORING-RECALIBRATION.md`. Subject-05's historic political career scores
+1 because **exactly one** career rule fires out of sixteen scored ones. That
is sparse coverage, not a compressed scale, and no rescaling fixes it.

Both are left unchanged in this pass, deliberately: they are interpretive
content, and `CONTRIBUTING.md` holds that behind astrologer review. The
proposal, with before-and-after numbers for all 130 cells, is in
`docs/SCORING-RECALIBRATION.md`.

## 6. Release assessment

The report's verdict was "not yet — fix D1 (High) and the Medium defects, add
the D7 confidence UX". D1 does not exist, the Medium defects were two
not-reproducible and one real, and D7 is done and goes further than asked.

What stands in the way of calling 1.0 validated is not a defect list. It is
that six of its topic verdicts demonstrably conflict with documented lives for
a structural reason that has not been changed yet, and that the sample is
thirteen famous people — every one of them wealthy, which is why wealth scored
11 Pass out of 13 and means nothing. Retrospective fit on a purposive sample of
thirteen is where this corpus starts, not where it finishes.
