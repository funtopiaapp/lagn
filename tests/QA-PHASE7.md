# QA report: Phase 7, the interface

Date: 2026-09-23. Spec: `docs/phase7/DESIGN.md`. Scope: the "modern app" shell,
light and dark with a manual choice, and the family view rebuilt as
"verdict first, then tabs" with new explanatory text from the engine.

## Verdict

**Pass.** macOS `make qa` green; Linux container green (see the addendum).

## What changed

| Area | Before | Now |
|---|---|---|
| Shell | plain page, links | sticky header, pill tab bar, cards with a status chip, shadows |
| Theme | followed the OS only | System / Light / Dark, remembered, `theme-color` kept in step |
| Findings | uniform grey blocks | colour-coded by kind (supports, calls for care, noted), from a typed `SectionKind` the engine now emits |
| Family | two columns of rule titles | verdict card (both leans, each labelled with whose chart and what it was read for), then *Your chart* / *<Name>'s chart* / *Together* |
| Together | did not exist | engine-generated text: why two charts are read, what each says, what agreement means, and which chart to follow for what |

Family names never leave the device: the engine emits a `{member}` placeholder
that the app fills in. A browser test asserts the request body carries no name.

## Acceptance criteria

| # | Criterion | Result |
|---|---|---|
| 1 | Light/Dark overrides the OS, survives a reload, updates `theme-color`; System follows the OS live | pass: 5 unit tests plus a browser test that emulates a dark OS, chooses Light, reloads, and checks the painted background |
| 2 | Contrast measured on the rendered page passes in both themes | pass: a browser test walks every visible text node, resolves the painted background behind it (including `color-mix`), and requires 4.5:1, or 3:1 for large text. Zero failures in light and dark |
| 3 | Family shows the verdict then three tabs, and *Together* equals the engine's output | pass, unit and browser |
| 4 | `comparison` is deterministic, passes the phase 6 wording and tone scans, and states both readings without merging them | pass: 48 readings across all four relations; it never says "average", "combined" or "overall verdict", and the verdict sentence always matches the computed agreement |
| 5 | Every earlier test still passes | pass: 596 Rust, 67 web unit, 36 browser (up from 30: theme and contrast, on both desktop and phone) |

## Defects found during QA

1. **The colour-coding did not appear.** The running server predated the
   `SectionKind` change, so the page had nothing to colour by. Caught by
   looking at a screenshot rather than at the CSS.
2. **Theme buttons were 40 px tall on phones**, below the 44 px touch
   standard, caught by the existing ergonomics test. The header height is now
   a token so the sticky tab bar stays aligned when it grows.
3. **A type error in my own test code** failed the build, so the browser tests
   ran against a stale bundle and reported a missing theme control rather than
   the real cause.
4. **The contrast helper crashed** on `color-mix()` colours, which Chromium
   reports as `color(srgb ...)`. The parser now handles both forms and skips
   colours it cannot parse rather than failing silently.
5. The verdict chip specified for card headers was missing; added.

## Mutation testing

3 of 3 caught: the agreement sentence contradicting the computed agreement, the
member's lean forced to "favourable", and a supporting section typed as "care".

## Addendum: Linux and the final gate

macOS `make qa`: **pass** — 598 Rust tests (release and debug), 67 web unit
tests, 36 browser tests; every oracle agreeing with zero disagreements
(116,640 porutham verdicts, all transit ingresses against swetest, 56,494
timezone cases, 200 births across 7 endpoints).
`docker run lagn-qa` (Linux aarch64): **LINUX QA PASSED**, 598 Rust tests, 0
failed.

### A QA-tooling note

A gate run appeared to hang for 90 minutes in the Phase 3 oracle. It was not a
regression: the machine was busy with the screenshot runs used to review the
new interface, and the oracle is CPU-bound (2,000 charts, ~10 minutes when the
machine is quiet). While investigating, `lagn periods` gained `--no-transits`,
which skips the gochara overlay for long ranges; a CLI test asserts it changes
nothing but the transit lines.
