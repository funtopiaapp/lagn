# Phase 6 design: the complete reading

> **Withdrawn, 2026-10-08.** Pariharams were removed from the product at the
> product owner's direction: no remedial practice is recommended to anyone.
> `corpus/pariharam.json`, `lagn-rules/src/pariharam.rs` and the UI that showed
> them are deleted, and the rule text that prescribed observances has been
> rewritten. The sections below are kept as the design record of what once
> shipped, not as a description of the product. `crates/lagn-rules/tests/no_remedies.rs`
> fails if any of it returns.


Status: approved for build.

## 1. Scope

Every topic the product owner listed, for the native and for the family:

| Topic | Read from | Sub-phase |
|---|---|---|
| Career | 10th house and lord, Sun/Saturn/Mercury, D-10, raja yogas | 6B |
| Wealth | 2nd and 11th, dhana yogas, Jupiter/Venus, SAV of the 11th | 6B |
| Education | 4th and 5th, Mercury and Jupiter, D-24 | 6B |
| Health (tendencies) | lagna and its lord, 6th and 8th, Sun and Moon, afflicted signs (Kalapurusha) | 6B |
| Vitality and care (longevity) | lagna lord, 8th house and lord, Saturn (ayushkaraka) | 6B |
| Progeny | 5th house and lord, Jupiter (putrakaraka), D-7 | 6B |
| Parents | 4th (mother, Moon) and 9th (father, Sun) | 6B |
| Later life and retirement | dashas from about age 55, 11th/12th, Saturn and Jupiter | 6B |
| Marriage | existing corpus | done |
| Pariharams | a catalogue linked to the afflictions found | 6B |
| Sensitive periods | dasha periods and transits, with amplifiers and negators explained | 6A engine, 6D timeline |
| Family: spouse, children, parents | each member's own chart plus the native's relational houses | 6C |
| Children's education and marriage; family health | the education, marriage and health topics on each child's or parent's own chart, beside the native's 5th, 4th and 9th | 6C |

## 2. Guardrails (content policy)

1. **No lifespan and no death timing.** "Vitality and care" reports
   constitutional strength and periods that call for extra care. It never gives
   a number of years, an age or a date of death. The classical ayurdaya methods
   are not computed.
2. **Health is tendencies, not diagnosis.** Every health reading, for the
   native or a family member, says the chart is not a medical assessment and
   points to a doctor. No rule may name a disease as a prediction.
3. **Pariharams are optional traditional practices:** worship, temple visits,
   charity, fasting. No gemstones (costly, and disputed even within the
   tradition), no paid services, no promise of results, and never a substitute
   for medical, legal or financial advice.
4. **Sensitive periods explain; they don't threaten.** Text says which areas of
   life a period emphasises and what tends to help, never "danger" or "loss" as
   a certainty.
5. **Family data stays on the device.** The server remains stateless.
6. Every rule goes through the review gate (AI review, as authorised on
   2026-09-21), with reviewer, date and reasoning.

## 3. Engine extensions (6A)

### 3.1 Functional nature per lagna

Every graha except the nodes gets exactly one class, from the D-1 houses it
rules. Evaluate in this order:

1. **Yogakaraka:** rules a kendra other than the 1st (4, 7, 10) *and* a trikona (5, 9).
2. **Benefic:** rules a trikona (1, 5, 9). The lagna lord is always benefic,
   even when it also rules the 8th.
3. **Malefic:** rules any of 3, 6, 8, 11, 12 and no trikona.
4. **Neutral:** otherwise; in practice, kendra-only lords (kendradhipati).

Variant F-1: the texts differ on dual lordships such as 2 and 7, which are
maraka houses. This scheme treats them as neutral.

### 3.2 New conditions

| Condition | Arguments | True when |
|---|---|---|
| `functional` | graha, is: [yogakaraka, benefic, malefic, neutral] | the graha's class (3.1) is listed; **Unknown** for nodes |
| `rules_house` | graha, houses | the graha is lord of any listed house (D-1) |
| `sambandha` | a, b, kinds? (conjunct, mutual_aspect, exchange) | the two grahas are related in any listed way (default: all three); false if a and b are the same graha |
| `neecha_bhanga` | graha | the graha is debilitated in D-1 **and** at least one cancellation (3.3) holds; **Unknown** for nodes |

*Mutual aspect*: each casts graha drishti on the other's sign.
*Exchange* (parivartana): each occupies a sign the other rules.

### 3.3 Neecha bhanga (variant N-1)

A debilitated graha's debilitation is cancelled if any of these hold:

1. the lord of its debilitation sign is in a kendra (1, 4, 7, 10) from the lagna or the Moon;
2. the lord of its exaltation sign is in a kendra from the lagna or the Moon;
3. it is conjunct or aspected by the lord of its debilitation sign;
4. it is exalted in the navamsa.

### 3.4 Period rules

A rule with `"scope": "period"` is evaluated once per Vimshottari antardasha,
with two extra GrahaRefs: `{"period": "maha"}` and `{"period": "antar"}`.
Natal rules (`scope` absent, or `"natal"`) cannot use them; the validator enforces this.

For each antardasha within the requested ages, the engine evaluates every
admitted period rule, applies cancellation within that window, and reports the
effective rules as **amplifiers** (polarity < 0) and **negators** (polarity > 0).
A window is **sensitive** when its net score is at most -2.

### 3.5 Transits (gochara)

New `lagn-core` module. For Saturn, Jupiter, Rahu and Ketu: every sidereal
sign ingress within a date range, found by stepping every half day and
bisecting to one minute of time. Retrograde
re-entries are handled naturally. Periods counted from the natal Moon sign:

| Transit | Houses from the Moon | Nature |
|---|---|---|
| Sade sati, rising / peak / setting | 12 / 1 / 2 | challenging |
| Ardhashtama shani | 4 | challenging |
| Kantaka shani (variant T-1) | 7, 10 | challenging |
| Ashtama shani | 8 | challenging |
| Guru transit | 2, 5, 7, 9, 11 | supportive |
| Rahu / Ketu | 3, 6, 11 | supportive |

**Ashtakavarga support:** a Saturn or Jupiter transit through a sign with 4 or
more bindus in that graha's own BAV is reported as *supported*. That reads
*milder* for Saturn and *fuller* for Jupiter; a Jupiter transit with fewer
than 4 is reported as *weaker*.
Vedha (transit obstruction) is not modelled (variant T-2).

### 3.6 Reading assembly (`lagn_rules::reading`)

One module builds the sensitive-periods reading for both the CLI and the
server, so the two cannot drift. For each antardasha in the age range:

- **Period rules** (3.4) give the score, amplifiers, negators and
  cancellations; a net score at most -2 is *sensitive* (P-9).
- **Focus houses:** the house the antardasha lord occupies and the houses it
  rules from the lagna. Rahu and Ketu rule no sign, so they take the houses
  ruled by their dispositor (variant F-2). Each house carries its reviewed
  significations from `corpus/bhava.json`.
- **Transits** overlapping the window, clipped to it, split into pressures
  (challenging) and supports. A transit that re-enters its sign during
  retrogression stays as separate stretches in the data, but the explanation
  names each kind once.
- **Pariharams** from the effective amplifiers (their tags and resolved
  subject graha) and the challenging transits' tags.
- **Explanation:** fixed sentence templates only, filled from computed facts
  and reviewed rule text. There is no generated prose and no event prediction.
  A sensitive period is described as calling for care, never as an outcome.

The topic endpoint defaults its age range to the topic's catalogue ages and
returns the topic's metadata (title, summary, disclaimer) and pariharams.
`/api/topics` lists only approved topics that have approved natal rules.

## 4. Family (6C)

Members are stored on the device: relationship (spouse, child, father, mother)
and birth details. The family report shows, for each member:

- **their own chart,** read with the topics relevant to the relationship
  (children: education, marriage, health; parents and spouse: health);
- **the native's relational reading** of them: 7th for spouse, 5th for
  children, 4th for mother, 9th for father;
- **agreement or disagreement** between the two. It never averages them away.

Implementation (`lagn_rules::family`, `POST /api/family`):

| Relation | Native's relational reading | Member's own topics (first one is compared) |
|---|---|---|
| Spouse | marriage topic | marriage, health |
| Child | progeny topic | health, education, marriage |
| Mother | parents topic, rules tagged `parent:mother` | health, later life |
| Father | parents topic, rules tagged `parent:father` | health, later life |

Each reading's *lean* is the sign of its net score: favourable, balanced or
calls for care. With no admitted rule, the lean is none. **Agreement**
(variant FA-1): *agree* when both lean the same non-balanced way; *differ*
when they lean opposite ways; *inconclusive* otherwise. Each own topic is
read over that topic's catalogue ages, counted from the member's own birth.
Members are kept in the browser's localStorage (`lagn.family.v1`), with a
validated export and import format (`lagn-family/1`); the server stores
nothing.

## 4a. Write-ups (6E)

Product owner feedback (2026-09-22): readings were lists of rule verdicts, and
the per-view sex question was repeated. Every topic now gets a written
interpretation a lay reader can follow, each statement justified by the chart
fact behind it. Templates only: no generated prose, same input gives the same text.

**Focus** (new, reviewed, in `topics.json`): per topic, the bhavas to explain,
the karakas with their role (optionally only for one sex), and divisional
charts to confirm with (varga + house). E.g. career: houses [10], karakas Sun
(authority), Saturn (work and service), Mercury (commerce and skill), D-10 for
the 10th.

**Structure** (`lagn_rules::writeup::WriteUp`):

1. **In brief:** the conclusion from the net score and weights of supporting
   and care-calling factors. It says "evenly balanced" when they cancel, and
   says so when nothing fired. The topic's disclaimer follows.
2. **One section per focus house:** its name and significations; its sign and
   lord; where the lord sits (house, sign, dignity, combust, retrograde) and
   what that position means (kendra, trikona, gains house, upachaya or
   dusthana); occupants and aspects, each labelled natural benefic or malefic;
   and its SAV bindus against the average of 28.
3. **Karakas:** position and dignity of each, with its role.
4. **Divisional confirmation:** the rasi lord's sign and dignity in the varga.
5. **What the classical rules find:** supporting, calling for care, eased
   (cancelled, naming the canceller), and could not judge. Each point carries a
   *because*: a minimal justification from the condition tree. That is every
   child of a true `all`, the first true child of a true `any`, and the fact
   behind a `not`. It is never the full trace.
6. **Timing:** the dasha windows from the timing rules, with dates.
7. Pariharams, as before.

Neecha bhanga is named where it applies. Nodes have no dignity and are
described as shadow grahas. Family readings reuse the same write-up: a member's
own topics, and the native's relational reading restricted to the relation's
house and karaka.

**Sex** is asked once on the birth form (optional), kept with the chart in the
session, and sent with every reading; family members carry their own.

Acceptance: (a) every topic write-up names, for each focus house, the lord and
the lord's house, checked against an independent sign-lord table over many
charts; (b) generated text passes the banned-wording scan over many charts and
all topics; (c) each "because" consists only of true facts, each point matches
an effective result, and nothing is missing; (d) determinism; (e) the UI shows
the write-up first, with the rule cards under "Details"; (f) no sex selector
outside the birth and member forms.

## 4b. Plain-language impact (6F)

Product owner feedback (2026-09-22): a finding such as "a natural malefic sits
in the 4th house" tells a reader nothing about their life, and a bare "-1"
reads like a verdict. Every rule now carries a second line saying what it tends
to mean in practice, in softer language.

`text.impact` (required on every rule, reviewed like the rest):

1. **Plain words.** What this tends to look like in someone's life. No house
   numbers, no graha names as jargon, no technical terms.
2. **Tendency, not fate.** "tends to", "often", "may". Never "will", "always",
   "guaranteed", and never a stated outcome.
3. **Challenging findings end constructively:** what helps, or what the
   tradition itself says softens it. Every rule with a negative polarity must
   carry such a clause; a test enforces it.
4. **Never frightening.** The banned-wording scan applies, and the tone rules
   of section 2 apply doubly here: health is tendencies with a doctor named,
   children are never denied, doshas are described as friction to work through
   with the customary remedies, not as a bar to marriage.

It is shown directly under the rule's own line, as "What this means for you",
in the write-up and in the rule cards. Acceptance: every rule has one; the
scans pass; negative rules carry a constructive clause; no absolutes; and each
is distinct from the technical text.

## 5. Pariharam catalogue (6B)

`corpus/pariharam.json`, reviewed like rules. An entry has a trigger (a graha
that is afflicted, or a rule tag such as `dosha:kuja`), the practice (deity,
day, mantra name, charity, temple) and its provenance. The per-graha set is:
the deity, the weekday, the Tamil Nadu Navagraha sthalam (Suryanar Kovil,
Thingalur, Vaitheeswaran Koil, Thiruvenkadu, Alangudi, Kanjanur, Thirunallar,
Thirunageswaram, Keezhperumpallam), and the navadhanya grain for charity.
Dosha entries cover Chevvai dosha, sarpa/naga dosha (Kerala: Mannarasala,
Ayilyam puja) and sade sati.

## 6. Variant register

| ID | Question | Default |
|---|---|---|
| F-1 | Functional nature of dual lordships | section 3.1 ordering |
| N-1 | Neecha bhanga conditions | the four in 3.3 |
| T-1 | Kantaka shani houses | 7 and 10 from the Moon |
| T-2 | Vedha in gochara | not modelled |
| T-3 | Transit reference point | natal Moon sign |
| P-9 | Sensitive-period threshold | net score at most -2 |
| F-2 | Focus houses of a Rahu/Ketu period | occupied house plus the dispositor's houses |
| FA-1 | Family agreement | sign of net score, compared as in section 4 |

## 7. Acceptance criteria

1. **Functional nature:** exhaustive over all 12 lagnas x 7 grahas against a
   table derived independently from rulership; the known yogakarakas
   (Saturn for Vrishabha and Tula, Mars for Karka and Simha, Venus for Makara
   and Kumbha) asserted directly.
2. **New conditions** in the Phase 3 N-version oracle, including synthetic
   corpora, with zero disagreements.
3. **Transits:** every ingress checked against an independent search driven by
   `swetest`, over a century for all four grahas, to within one minute.
   Sade sati windows checked by invariant: continuous, 12 then 1 then 2, with
   retrograde re-entries handled.
4. **Period rules:** the validator rejects period refs in natal rules;
   evaluation is per window and deterministic; the oracle covers period rules.
5. **Guardrails:** tests fail if any shipped rule or pariharam text names a
   death age or date, uses "death" predictively, recommends a gemstone, or
   lacks a medical disclaimer on health and vitality topics.
6. Every earlier QA gate stays green on macOS and Linux.
