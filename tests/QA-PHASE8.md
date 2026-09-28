# QA report: Phase 8, the past-life reading

Date: 2026-09-27. Spec: `docs/phase8/DESIGN.md`. Scope: a `past_life` topic
(21 rules), two new write-up sections (the Ketu-Rahu axis, and the bridge into
the native's other readings), and the guardrails that topic needs.

## Verdict

**Pass.** 301 Rust tests, 67 web unit, 38 browser, all green; `make qa` and the
Linux container as recorded in the addendum.

## What it reads

The 5th house (purva punya, merit carried forward), the 9th (dharma and
fortune), the 12th (what is being set down), Ketu by house and sign with its
dispositor, Rahu opposite it, the D-60 shashtiamsa through the lagna lord, and
Jupiter as karaka of dharma. Variants recorded: **PL-1** the axis is read from
the rasi chart; **PL-2** D-60 through the lagna lord only, because of its
sensitivity to birth time, which the text says out loud; **PL-3** Jaimini
atmakaraka and karakamsa are not modelled.

## The two new sections

- **"What you carry, and what is asked now."** Ketu's house and sign with what
  that house governs; the lord of Ketu's sign, since Ketu gives results through
  it; Rahu's house opposite, framed as the direction of growth; and a closing
  line that does not tell the native to abandon what comes easily.
- **"Where this shows in the rest of your chart."** For Ketu's house, Rahu's
  house and the 5th, the catalogue topics whose focus covers that house, each
  with the verdict that topic's own rules produce. This is the interpretation
  against the present life, and it introduces no new judgement: every verdict
  is recomputed from the existing rules.

## Guardrails particular to this topic

Tested over the corpus and over generated text on 30 charts:

- **No biography.** Banned: "you were", "previous birth", "former life",
  "reincarnation", "rebirth", "in that life", "kingdom", "dynasty", "century",
  "medieval" and similar. The reading never names an identity, place or era.
- **No blame.** Banned: "sin", "punishment", "karmic debt", "paying for",
  "deserve", "retribution", "curse", "bad karma". Difficulty is described as
  unfinished work, never as something earned by wrongdoing.
- **The frame is declared.** The disclaimer states that this is a traditional
  interpretive frame, not a claim about events, that it names no past identity,
  and that nothing in it can be tested. A test asserts it appears in every
  reading of the topic.
- The phase 6 rules still apply: plain language, tendencies not fate, and a
  constructive close on every negative rule.

## Defects found during QA

1. `past_life.l9.dusthana` used the phrase "beliefs you were handed" - innocent
   in meaning, but exactly the biography pattern being banned. Reworded rather
   than narrowing the ban.
2. The shared summary template said findings "deserve attention"; "deserve" is
   on the blame list, so the template now says "are worth reading". This
   changed wording in every topic, not only this one.
3. The pinned list of negative rules without a subject graha needed the two new
   house-occupancy rules; that test exists to force exactly this review.

## Mutation testing

4 of 4 caught: the axis swapping Ketu's and Rahu's houses, the axis naming the
wrong dispositor, the bridge reporting on the past-life topic itself, and the
bridge inverting a verdict.

## Addendum: the gates

macOS `make qa`: **pass** — 602 Rust tests (release and debug), 67 web unit
tests, 38 browser tests; every oracle agreeing with zero disagreements, with
the Phase 3 N-version oracle now exercising the past-life topic alongside the
other ten.
`docker run lagn-qa` (Linux aarch64): **LINUX QA PASSED**, 602 Rust tests, 0
failed.
