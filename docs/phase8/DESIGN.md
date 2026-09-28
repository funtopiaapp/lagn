# Phase 8 design: the past-life reading (purva janma)

Status: approved for build (product owner asked for it on 2026-09-27, to the
same model as the other topics).

## 1. Scope

A reading of what the chart is traditionally held to carry over from before
this birth, and then an interpretation of that against the present life.

Two halves, both in one topic (`past_life`):

1. **What is carried over:** merit (5th, purva punya bhava), dharma and
   fortune (9th), what is being released (12th), and the shashtiamsa (D-60),
   which Parashara gives the greatest weight for past-life karma.
2. **The bridge to this life:** the Ketu-Rahu axis - what the chart says you
   arrive already carrying (Ketu) against what this life asks you to develop
   (Rahu) - and which of the native's other readings the same houses drive.

## 2. Guardrails (in addition to phase 6 section 2)

This is the most speculative topic in the app, so it is the most constrained.

1. **No biography.** The reading never names a past identity, occupation,
   place, era, country, caste, name or event. The tradition describes
   tendencies and merit carried forward; anything more is invention. Tests ban
   the vocabulary of biography ("you were", "previous birth you", "in a former
   life you", "reincarnat...", century and era words).
2. **No blame.** Nothing is framed as punishment, debt owed for wrongdoing, or
   suffering deserved. Words such as "sin", "punish", "karmic debt" and
   "paying for" are banned. Difficulty is described as unfinished work.
3. **A frame, not a fact.** Every reading of this topic carries a disclaimer
   saying so plainly: this is a traditional interpretive frame, not a claim
   about events, and it is not testable.
4. The phase 6 tone rules apply in full: tendencies not fate, plain language,
   and every difficult finding closes constructively.

## 3. What the engine reads

| Source | Read as |
|---|---|
| 5th house, its lord, occupants, aspects | merit carried forward (purva punya) |
| 9th house and its lord | dharma, and fortune that arrives unearned in this life |
| 12th house | what is being set down: release, retreat, the inner life |
| Ketu by house and sign | what is already done, where the chart is disengaged, the settled skill |
| Rahu by house (always opposite Ketu) | what this life asks the native to take up |
| D-60 shashtiamsa, lagna lord | the varga Parashara gives for past-life karma |
| Jupiter | karaka of dharma and merit |

Variants: **PL-1** the Ketu axis is read from the rasi chart, not from
karakamsa. **PL-2** D-60 is read through the lagna lord only, since its
sensitivity to birth time makes finer claims unsafe. **PL-3** the Jaimini
atmakaraka and karakamsa are *not* modelled; they need a variant decision on
whether Rahu is counted, and they are not part of this phase.

## 4. Two new write-up sections

Both are template-generated, deterministic, and appear only for this topic.

- **The karmic axis (Ketu and Rahu):** Ketu's house and sign, what that house
  signifies, and the same for Rahu's house opposite it, stated as "already
  carried" against "asked of you now". It names the dispositor of Ketu's sign,
  since Ketu gives its results through it.
- **Where this shows in the rest of your chart:** for each of the two axis
  houses and the 5th, the topics in the catalogue whose focus includes that
  house, with how each of those readings currently comes out. This is the
  interpretation against the present life, and it is computed from the same
  rules those topics already use - no new judgements.

## 5. Acceptance

1. Every rule in the topic carries a review, a plain-language impact line and a
   constructive close where negative, as in phase 6.
2. The biography and blame bans hold over generated text as well as corpus
   text, across many charts.
3. The karmic axis names Ketu's and Rahu's real houses, checked against an
   independent derivation, and the two are always opposite.
4. The bridge lists exactly the catalogue topics whose focus houses include the
   axis houses or the 5th, with the lean each of those readings produces.
5. Deterministic; the disclaimer is always present; `make qa` and the Linux
   container stay green.
