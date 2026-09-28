# Phase 3 QA report

Scope: the rule engine (language, three-valued evaluator, review gate,
cancellation, timing), the ten-porutham procedure, the corpus validator, the
review-sheet generator, and the draft marriage corpus.
Specification: `docs/phase3/DESIGN.md` revision 2. Date: 2026-09-21.
Verdict: **engine passes. Content is not yet approved**, by design.

## Acceptance criteria

| # | Criterion | Result |
|---|---|---|
| 1 | Kleene truth tables, exhaustive | Pass: every sequence of length 0 to 4 (121) for AND and OR, NOT, and both De Morgan laws |
| 2 | Every atomic condition against `lagn-core` | Pass: 150 charts, all nine grahas, 5 vargas, all 12 houses |
| 3 | N-version evaluator, 2,000+ charts, zero disagreements | **Pass: 4,500 charts, 97,604 rule evaluations, 0 disagreements.** Covered the real corpus in review mode, the real corpus under random approvals in production mode, and random synthetic corpora exercising every condition type. Outcomes: 26,003 True, 70,012 False, 1,589 Unknown. Also 2,227 cancellations and 40,345 timing windows. |
| 4 | Porutham, exhaustive | **Pass: all 11,664 pada pairs (1,296 distinct), 116,640 verdicts, 0 mismatches**, against tables parsed from the spec |
| 5 | Gate never leaks | Pass: 300 random corpora in Rust, every production-mode oracle run, and CLI end-to-end |
| 6 | Cancellation semantics and order independence | Pass: cancel-of-cancel, non-firing cancellers, tie-break, draft cancellers, 500 shuffled orderings |
| 7 | One validator negative test per rule | Pass: 27 error classes |
| 8 | Mutation check | Pass: 7 of 7 planted engine bugs caught by `cargo test` and by the oracles |
| 9 | No regressions | Pass: 232 tests in release and debug; swetest, the Phase 2 oracle and the golden fixtures are green; clippy clean; `make qa` exits 0 |

## Findings

**Engine defects found by QA: none.** The developer caught and fixed five
problems in self-review before QA began:

| Found in | Problem |
|---|---|
| `eval.rs` | a closure holding a mutable borrow across recursion (would not have compiled) |
| `facts.rs` | two dead lines in `moon_waxing` |
| `porutham.rs` | a convoluted `all_padas` with a meaningless `0 * 0` term and a pointless trait |
| workspace | the declared MSRV of 1.80 was below the 1.82 APIs in use |
| `eval.rs` | `Truth::not` shadowing `std::ops::Not`; replaced with a trait impl |

QA findings:

| ID | Type | Finding | Action |
|---|---|---|---|
| QA-P3-S1 | Spec gap | Section 6 didn't say which canceller is reported when several are effective. Two conforming implementations could differ. | Spec now says: the smallest id. The code already did this; a test pins it. |
| QA-P3-S2 | Spec error | Section 12 described 11,664 porutham pairs as the input space. The true distinct space is 1,296, because 9 nakshatras straddle two signs. | Spec corrected; the oracle asserts both numbers. |
| QA-P3-T1 | Test assumption | The porutham oracle first asserted 108 distinct (nakshatra, rasi) pairs; there are 36. | Assertion corrected. Recorded because it produced S2. |

## Mutation results

| Mutant | Planted defect | `cargo test` | Oracle |
|---|---|---|---|
| M1 | Kleene AND lets Unknown dominate False | caught | 38/120 charts |
| M2 | Mula moved from Pada to Kati rajju | caught | 352 wrong verdicts |
| M3 | Cancellation ignored | caught | 39/120 charts |
| M4 | Gate admits drafts in production | caught | 56/120 charts |
| M5 | A graha counted as conjunct with itself | caught | 8/120 charts |
| M6 | Moon treated as benefic when waning | caught | 5/120 charts |
| M7 | Validator accepts override cycles | caught | not applicable |

## What this does and does not establish

**Established:** the engine evaluates any valid rule exactly as the spec
defines, for every chart tested. It never shows unapproved content in
production, never turns a missing fact into a false one, and reports
cancellations and unknowns instead of hiding them. The porutham procedure
matches its specified tables over its entire input space.

**Not established, by design:**

1. **Whether any rule is astrologically correct.** All 34 marriage rules and
   all 10 poruthams are `draft`. They were written from general knowledge of
   commonly stated principles, with no textual citations, because none could
   be verified. Production output is therefore empty until the astrologer
   reviews `docs/phase3/REVIEW-SHEET.md` and approves rules. A test fails if
   anyone commits an approval or a citation without going through that
   process.
2. **Vasya porutham:** blocked until the reviewer supplies the table (P-7).
3. **Variants P-1 to P-8 and R-1 to R-4:** defaults pending sign-off.
4. The shared-author limit from Phases 1 and 2 applies here too.

## Reproduce

```bash
make qa
python3 scripts/qa_phase3_oracle.py --charts 4500 --seed 11
python3 scripts/qa_porutham_oracle.py
```
