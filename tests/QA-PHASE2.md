# Phase 2A/2B QA report

Scope: the 16 Shodasavargas, dignity, relationships, drishti, Baladi and
Jagradadi avasthas, combustion, graha yuddha detection, and Ashtakavarga.
Specification: `docs/phase2/DESIGN.md` revision 2.
Date: 2026-09-21. Verdict: **pass**.

## Acceptance criteria

| # | Criterion | Result |
|---|---|---|
| 1 | Every rule restated longhand in a unit test | Pass: 58 unit tests in the new modules |
| 2 | N-version oracle, 2,000+ charts, zero disagreements | **Pass: 13,000 charts, 8.68M checks, 0 disagreements** (10,000 at seed 20260920, 3,000 at seed 777); 25% of charts under randomised non-default variants |
| 3 | Every derived fact and invariant tested | Pass: `tests/phase2.rs` |
| 4 | ±1 arcsec boundaries at every varga, D-30, MT, Baladi and combustion edge | Pass: every part edge of all 16 vargas |
| 5 | Mutation check (Ashtakavarga entry, D-30 edge, aspect house) | Pass after a gate fix; see below |
| 6 | No Phase 1 regressions | Pass: the 127 earlier tests, golden fixtures and swetest cross-validation (0.001 arcsec) |
| 7 | Clippy clean; release and debug pass | Pass: 196 tests in both, `-D warnings` clean |

## Findings

**Kernel defects found: none.** Every disagreement during QA traced back to a
wrong QA assumption, a gap in the test gate, or imprecise spec wording, never
to the kernel. Each one is recorded because each changed something.

| ID | Type | Finding | Action |
|---|---|---|---|
| QA-P2-G1 | **Gate gap** | Mutant M1 moved one Ashtakavarga house number while keeping the row total. It passed every Rust test, because they all read the same transcription they were meant to check. Only the oracle caught it. | Added `tests/spec_tables.rs`, which parses the Ashtakavarga, natural-relationship and D-30 tables out of DESIGN.md. `cargo test` alone now catches M1. |
| QA-P2-T1 | Test assumption | QA assumed a varga sign always changes at a part edge. D-10 legitimately repeats Makara across Mesha to Vrishabha. | Derived the exact repeat set from the independent oracle (D-2 at every boundary; D-10 and D-24 entering even signs) and asserted exactly that set. |
| QA-P2-S1 | Spec defect | Section 10 said every variant is switchable; only V-3 to V-6 are. | Spec corrected, rev 2. |
| QA-P2-S2 | Spec ambiguity | "Within 1 degree" didn't say whether the boundary counts. | Spec now says inclusive, matching the code. |
| QA-P2-S3 | Spec gap | The boundary-repeat fact wasn't documented. | Added to section 3. |

The developer also caught one defect in self-review before first compile:
D-2, D-3 and D-4 were routed into a match arm that would have panicked.

## Mutation results (final gate)

| Mutant | Planted defect | `cargo test` | Oracle |
|---|---|---|---|
| M1 | Moon's BAV from Jupiter, house 8 changed to 9 (total unchanged) | caught (after G1 fix) | 300/300 charts |
| M2 | D-30 odd-sign Jupiter band ends at 17 instead of 18 | caught | 42/300 |
| M3 | Saturn aspects the 11th instead of the 10th | caught | 300/300 |
| M4 | Saturn regards Jupiter as a friend | caught | not run |
| M5 | Venus exalted in Mesha | caught | not run |
| M6 | Mars combustion orb 16 instead of 17 | caught | not run |

## What this does and does not establish

**Established:** the kernel implements DESIGN.md exactly, for every chart
tested, under every combination of the switchable variants.

**Not established:**

1. **The spec's reading of BPHS.** The oracle and the kernel were written by
   the same author. Independent implementation catches coding errors, not a
   shared misreading of the source. JHora comparison and astrologer review
   close that gap.
2. **Astrologer sign-off on V-1 to V-9.** Pending. Until then Phase 3 treats
   every default as provisional.
3. **2C (Shadbala) and 2D.** Not built. DESIGN.md section 9 lists what is
   needed to unblock them.

## Reproduce

```bash
make qa        # clippy, release + debug tests, swetest, 2,000-chart oracle
python3 scripts/qa_phase2_oracle.py --charts 10000   # the full N-version run
```
