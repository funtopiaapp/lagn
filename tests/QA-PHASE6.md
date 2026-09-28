# QA report: Phase 6, the complete reading

Date: 2026-09-22. Spec: `docs/phase6/DESIGN.md`. Scope: 6A engine extensions,
6B content (9 new topics, bhava meanings, pariharams), 6C family readings and
6D UI (all topics, sensitive-periods timeline, family tab).

## Verdict

**Pass.** Every acceptance criterion is met, and every earlier gate is green.
`make qa` passes on macOS; for Linux, see criterion 6.

## Acceptance criteria

| # | Criterion | Result |
|---|---|---|
| 1 | Functional nature, exhaustive over 12 lagnas x 7 grahas; known yogakarakas asserted | pass (`functional_nature_exhaustively_matches_an_independent_derivation`) |
| 2 | New conditions in the Phase 3 N-version oracle, with synthetic corpora | pass: 2,000 charts, 37,338 rule evaluations, 0 disagreements. The oracle now covers **all 9 shipped natal topics** plus the shipped period rules, and compares each result's resolved subject and tags |
| 3 | Transits vs an independent swetest-driven search, 100 years, 4 grahas, within 1 minute | pass: every ingress agrees (0 problems); sade sati invariants hold |
| 4 | Period rules: validator, per-window evaluation, oracle coverage | pass: 15,454 period windows compared, 0 disagreements |
| 5 | Guardrails | pass, and wider than specified (below) |
| 6 | Every earlier gate green on macOS and Linux | macOS: `make qa` green, 582 Rust tests (release + debug), 56 web unit, 28 browser. Linux: see the addendum |

## Content (6B)

| Topic | Rules | Notes |
|---|---|---|
| career | 28 | 10th lord, dig bala, dharma-karmadhipati, yogakaraka, D-10, 9 unscored field indications |
| wealth | 16 | 2nd/11th lords, dhana yoga (8 lord pairs), Lakshmi-type condition, SAV |
| education | 13 | 4th/5th lords, Jupiter, D-24, Mercury |
| health | 21 | tendencies only, each pointing to care, never a diagnosis |
| vitality | 10 | constitution and care periods; **no lifespan is computed** |
| progeny | 15 | 5th lord, Jupiter, D-7, Kerala sarpa dosha (tagged); gentle wording |
| parents | 14 | 4th/Moon (mother), 9th/Sun (father), tagged per parent |
| later_life | 9 | 4th, 11th, 12th, Saturn and Jupiter |
| periods | 14 | period-scope rules: functional nature, dusthana, debility (with neecha bhanga), combustion, shashtashtaka, harmony, Jupiter aspect |

All 140 new rules have an AI review with reasoning, reviewer and date (as
authorised on 2026-09-21), `source.reference: null` (no citation verified),
and a provenance note saying so. Review sheets with firing rates over 1,000
sample births: `docs/phase6/review-sheets/`. A firing-rate screen flagged no
rule outside 1.5%–85% except the always-true timing rules, which is by design.

## Guardrails (criterion 5)

- **Banned wording** in every user-visible field: rule titles and texts, topic
  titles and summaries, bhava meanings, every pariharam field, and the
  generated period explanations and transit labels. The list covers death and
  lifespan, gemstones, fatalistic statements about children ("childless",
  "barren", ...), certainty and cure, payment, and "danger", "loss",
  "disaster" and "accident". Matching is on whole words.
- Health and vitality disclaimers must say "not a medical" and name a doctor;
  vitality's must also say it never estimates a lifespan.
- Every graha has a pariharam, and every dosha pariharam can be triggered by a
  shipped rule. Negative rules with no subject graha are pinned to a reviewed
  list of 10 (house and period-relation rules), so a new rule cannot silently
  lose its pariharam.
- Pariharams: no gemstones, no paid services; the UI states this beside every list.

## Mutation testing

10 mutants in the new reading and family code; **all 10 caught**. The first
pass missed 3 (node focus via dispositor, repeated explanation lines, transit
pariharams dropped), because the server parity test compares against the same
library. I added `period_readings_match_independent_expectations`, which checks
focus houses against a hand-written sign-lord table and asserts the other two
properties directly; all 3 are now caught.

| Mutant | Caught |
|---|---|
| transit overlap ignores the window end | yes |
| node focus drops the dispositor | yes (after the fix) |
| duplicate pressure lines | yes (after the fix) |
| topic default ages ignored | yes |
| transit pariharams dropped | yes (after the fix) |
| balanced counted as agreement | yes |
| parent tag filter ignored | yes |
| cancelled rules scored | yes |
| mother read as father | yes |
| member given the native's sex | yes |

## Defects found and fixed during QA

1. **Review sheet for period rules** showed 0% firing, because it evaluated
   them as natal rules. Now each sample chart is evaluated under all 81
   mahadasha/antardasha lord pairs.
2. **Review-sheet test** compared one topic's sheet with the corpus-wide rule
   count; it now checks each topic against its own count.
3. **Repeated transit lines** in the explanation, when a transit re-enters its
   sign during retrogression. Each kind is now named once; the structured data
   keeps every stretch.
4. **Vitality summary** contained "Never a lifespan." in user-facing text. It
   was moved to the disclaimer, where it is a policy statement.
5. **Design doc** said transits step 5 or 3 days; the code steps half a day.
   The doc is corrected.
6. **Mutation harness** (a QA tool, not the product): restoring files with an
   older mtime left a stale mutant build, which failed `make qa` on correct
   source. Restored files are now touched before any gate; the gate was re-run
   green.
7. **Family ID fallback** used `Date.now()`, which the frontend's no-date-math
   guard rightly rejected. It now uses `crypto.getRandomValues`.

## API added

| Endpoint | Purpose | Verified |
|---|---|---|
| `GET /api/topics` | catalogue: approved topics with approved natal rules, plus bhava meanings | every listed topic answers; `periods` is not listed and returns 404 |
| `POST /api/topic/{name}` | now defaults ages to the topic's catalogue range; adds `meta` and `pariharams` | equals the library over 160 readings |
| `POST /api/periods` | sensitive-periods reading | equals the library; windows tile the range; clipped transits; at most one current window |
| `POST /api/family` | member's own readings beside the native's relational reading | equals the library for all 4 relations; bad input returns 400; review mode needs the token |

## Known limitations (unchanged by QA; stated to users where relevant)

- The content review is an **AI review**, not a practising astrologer's.
  No textual citation has been verified.
- In period rules, Rahu and Ketu have no functional nature or dignity, so those
  conditions are "could not evaluate" in their periods (variant V-1). The
  focus houses use the dispositor instead (F-2).
- Vedha in gochara is not modelled (T-2).
- The family *agreement* is a product rule (FA-1: the sign of each net score).
  It is not a classical technique, and the UI shows both readings in full.
- Family members are stored in the browser; clearing site data removes them.
  Export/import is provided.

## Addendum: Linux

`docker build --target qa` then `docker run lagn-qa` (Colima, Linux aarch64):
**LINUX QA PASSED**. 582 Rust tests passed, 0 failed (release + debug). The Phase 2
oracle ran on 3,000 charts and the Phase 3 oracle on 3,000 charts across all topics and period rules, both with 0
disagreements. Poruthams, transits vs swetest, the tz oracle (56,494 cases) and
HTTP parity all agree.

## Phase 6E: write-ups and sex asked once (2026-09-22)

Product owner feedback: readings were rule lists with technical traces
("interpretations are missing"), and sex was asked on every view. Spec:
`docs/phase6/DESIGN.md` section 4a.

**Built.** `lagn_rules::writeup` renders every topic as prose. It opens with a
conclusion; then each focus house (meaning, sign, lord, the lord's placement
and dignity and what that position means, occupants and aspects with their
natures, SAV against 28); the karakas; divisional confirmation; the findings,
each with a minimal *because*; and dated timing with the reason each lord
times the area. Focus houses, karakas and vargas are reviewed catalogue data in
`topics.json`. Family readings carry the same write-ups (the relational one is
restricted to the relation's house and karaka). The web shows the write-up
first, with the rule cards under Details, and loads a topic as soon as it is
opened. Sex is asked once on the birth form (and once per family member).
`lagn topic ... --writeup` prints the same text for review.

**QA found and fixed:** 4 user-facing rule texts held internal notes
("reviewer", "variant R-2", "337 / 12"). They were cleaned and re-reviewed, and
a new test bans internal jargon in rule text. Other fixes: the 12th-house
phrase used "loss" (banned; now "withdrawal"); cancellation rules announced
relief when nothing was cancelled (now shown only where they cancel); the
yogakaraka trace listed 14 facts where 2 decided it (the new `justify` cites
only the deciding branch); timing named lords without saying why; grammar in
lists; and the disclaimer showed twice.

**Verified:**
- 60 charts x 9 topics x 3 sex settings. Every focus house names the lord
  given by an independent sign-lord table, the lord's house and the house's
  SAV. Karakas appear exactly when applicable to the native's sex. Scored
  points equal the effective scored results exactly. Every *because* is a fact
  the rule read. The conclusion follows the weights. No banned wording or
  jargon appears, the disclaimer is carried, and the output is deterministic.
- The server's write-up equals the library's.
- Browser tests: the conclusion and first heading on screen equal the API's,
  for every topic. Sex chosen once reaches topic, period and family requests,
  and no later view asks for it.
- Mutation: 7 of 7 write-up mutants caught (wrong lord, wrong `any` branch,
  sex filter, lone cancellation, balanced shown as supportive, wrong-house
  SAV, ineffective rules shown).

### Linux re-run after 6E

`docker run lagn-qa` (Linux aarch64) after the write-ups: **LINUX QA PASSED**,
588 Rust tests passed, 0 failed (release + debug), with the Phase 2 and Phase 3
oracles at 3,000 charts each and 0 disagreements. macOS `make qa` is green with
the same 588 tests, 60 web unit tests and 30 browser tests.

## Phase 6F: plain-language impact (2026-09-22)

Product owner feedback: a finding such as "a natural malefic sits in the 4th
house" says nothing to a reader, and a bare "-1" reads like a verdict. Spec:
`docs/phase6/DESIGN.md` section 4b.

**Built.** Every one of the 175 rules now carries `text.impact`: one or two
sentences in everyday words saying what the indication tends to look like in a
person's life, and, where the indication is difficult, what helps. It appears
directly under the technical line as "What this means for you" in the
write-ups, in the rule cards and in the sensitive-period explanations, which
now lead with it. The validator requires it on every rule.

**Tone rules, enforced by tests over all 175 lines:** no technical vocabulary
(18 banned terms: kendra, dusthana, navamsa, bindus, antardasha and the rest);
no absolutes (will, always, certainly, guaranteed, must, cannot, doomed,
destined, fated and others), so every line reads as a tendency; the existing
banned-wording scan; 40 to 500 characters, distinct from the technical text;
and **every negative rule must close constructively** - naming what helps, or
the remedy the tradition itself offers. Doshas are described as friction to
work through with the customary matching and remedies, never as a bar to
marriage; progeny lines speak of timing and patience, and say plainly that
medical help is no quarrel of the tradition's; health lines name the habit that
helps.

**QA found and fixed:** 8 lines failed the tone tests on the first pass - 7
difficult findings that ended on the difficulty without saying what helps
(career, marriage, parents and period rules), and one that used "will". All
were rewritten rather than the tests weakened, except where the test's own
marker list was genuinely incomplete (persistence, suits, pacing).

**Verified:** all 175 lines pass the tone rules; results, write-up points and
period explanations carry the line wherever a rule is shown (checked over 12
charts x 9 topics, 200+ points and 200+ period lines); the browser test asserts
one visible "What this means for you" per finding shown, matching the API.
Mutation: 4 of 4 caught (results dropping the line, write-up points dropping
it, period lines dropping it, and the validator no longer requiring it - that
last one initially survived, so the validator's rejection is now tested).

### Linux re-run after 6F

`docker run lagn-qa` (Linux aarch64): **LINUX QA PASSED**, 592 Rust tests
passed, 0 failed. macOS `make qa` green with the same 592 tests, 60 web unit
tests and 30 browser tests. The Phase 3 oracle's synthetic corpora now carry
impact lines too, and it compares that field alongside the rest.
