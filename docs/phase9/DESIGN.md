# Phase 9 design: the life carried forward, bonds, and debts (rina)

> **Withdrawn, 2026-10-08.** Pariharams were removed from the product at the
> product owner's direction: no remedial practice is recommended to anyone.
> `corpus/pariharam.json`, `lagn-rules/src/pariharam.rs` and the UI that showed
> them are deleted, and the rule text that prescribed observances has been
> rewritten. The sections below are kept as the design record of what once
> shipped, not as a description of the product. `crates/lagn-rules/tests/no_remedies.rs`
> fails if any of it returns.


Status: approved for build. The product owner asked on 2026-09-27 for the
past-life reading to describe past characteristics, work, the life lived, the
spouse and children, and the debts carried forward into this life. This
supersedes the narrower phase 8 position, at their direction.

## 1. What is added

| Section | Read from |
|---|---|
| **The life carried forward** (temperament, work and station) | Ketu by house (station), Ketu by sign (temperament), the lord of Ketu's sign and its condition (how that life went), the 12th house lord |
| **Bonds carried forward** (spouse, children) | the 6th house, which stands 12th from the 7th and is read as what the house of marriage has already been through; the 4th, which stands 12th from the 5th, for children; the nodes in the 5th and 7th; Venus and the 7th lord |
| **Debts carried forward (rina and shapa)** | rules tagged `rina:*`: pitru (ancestors: the 9th, the Sun, the nodes), matru (mother: the 4th, the Moon), sarpa (the nodes on the 5th), guru (Jupiter with the nodes), patni (Venus or the 7th lord with the nodes) |

The 12th-from-a-bhava technique (variant **PL-4**) is the classical
"what that house has already passed through" reading. It is not universal
practice, so it is recorded as a variant and named in the text.

## 2. Line held

A chart encodes grahas in houses. It can support statements about temperament,
station, orientation and pattern. It cannot encode a personal name, a date, a
place or a verifiable event, so the reading gives none: no proper nouns, no
years, no centuries, no "you were X". Tests enforce that. Everything else the
product owner asked for is delivered.

The blame guardrail is narrowed rather than dropped: the tradition's own words
(rina, debt, shapa, curse) are used, because the request is precisely for them,
but nothing is framed as punishment for wrongdoing, and every debt is stated
with what settles it. Banned: "sin", "punishment", "deserved", "retribution",
"doomed", "cursed to". Required on every debt rule: a remedy or an action.

## 3. Reviewed data, not code

`corpus/karma.json`, validated and reviewed like the rest:

- `ketu_house[1..12]`: the station a life took, read from Ketu's house.
- `ketu_sign[1..12]`: the temperament carried, read from Ketu's sign.
- `dispositor[strong | weak]`: how that life is read to have gone, from the
  condition of the lord of Ketu's sign.

The write-up assembles these with the chart's own facts. No text is invented at
run time and none is stored in code.

## 4. Debts feed the remedies already in the app

Each debt rule carries `dosha:` tags, so the existing pariharam machinery
selects the traditional remedy. Two new pariharam entries are needed: **pitru**
(tarpanam on Amavasya and in Mahalaya paksha, Thila homam, the pitru sthalams)
and **matru** (worship of Ambal, service to the mother). Sarpa, Guru and Venus
remedies already exist.

## 5. Acceptance

1. Every new rule and every karma entry is reviewed, with a plain-language
   impact line and, for debts, a stated settlement.
2. The station and temperament named are the ones the chart's Ketu actually
   gives, checked against an independent derivation over many charts.
3. Bonds name the 6th and 4th houses and say why they are read.
4. Every debt that fires reaches its pariharam.
5. No proper nouns, dates or "you were" anywhere in generated text; no blame
   vocabulary; the frame disclaimer still appears in every reading.
6. `make qa` and the Linux container stay green.
