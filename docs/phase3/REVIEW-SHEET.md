# Review sheet: marriage

For the reviewing astrologer. Each rule below is exactly what the engine evaluates:
the "Condition" line is generated from the rule itself, not written separately.

To approve a rule, set `review.status` to `approved` and fill in `reviewer` and
`date` in the corpus file. To reject, set `rejected`. Please correct anything wrong:
the condition, the polarity (-3 to +3), the cancellations, or the wording.

Firing rates come from 1000 sample births in South India, 1920-2025, evaluated in review mode.

| Rules | 35 |
|---|---|
| Approved | 32 |
| Draft | 0 |
| Rejected | 3 |

## A natural malefic occupies the 2nd house

- **Id:** `marriage.h2.malefic_occupant`
- **Status:** Rejected
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the 2nd house is occupied by malefics
- **Text shown to users:** A natural malefic sits in the 2nd house, the house of family (kutumba). Classical texts associate this with friction in family life after marriage.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 0.0% of samples (0.0% after cancellations)

## A natural benefic occupies the 7th house

- **Id:** `marriage.h7.benefic_occupant`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the 7th house is occupied by benefics
- **Text shown to users:** A natural benefic sits in the 7th house, the house of marriage. Classical texts read this as support for married life and for the spouse's nature.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 21.9% of samples (21.9% after cancellations)

## Jupiter aspects the 7th house

- **Id:** `marriage.h7.jupiter_aspect`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the 7th house is aspected by Guru
- **Text shown to users:** Jupiter casts its aspect on the 7th house. Jupiter's aspect is traditionally held to protect and strengthen the house it falls on.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 25.2% of samples (25.2% after cancellations)

## The Sun, Saturn, Rahu or Ketu occupies the 7th house

- **Id:** `marriage.h7.malefic_occupant`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -2
- **Condition:** the 7th house is occupied by Surya, Shani, Rahu or Ketu
- **Text shown to users:** The Sun, Saturn, Rahu or Ketu sits in the 7th house. Classical texts associate a malefic here with strain in married life, to be weighed together with the 7th lord and Venus rather than on its own. Mars in the 7th is judged separately as Chevvai dosha.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 27.3% of samples (27.3% after cancellations)

## Rahu or Ketu occupies the 7th house

- **Id:** `marriage.h7.node_occupant`
- **Status:** Approved
- **Tradition:** Kerala
- **Polarity:** +0
- **Condition:** the 7th house is occupied by Rahu or Ketu
- **Text shown to users:** Rahu or Ketu sits in the 7th house. South Indian practice, Kerala especially, treats this as calling for careful examination, including sarpa dosha considerations. Recorded for review, not scored; the malefic-occupant rule already scores the affliction.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 13.9% of samples (13.9% after cancellations)

## Saturn aspects the 7th house

- **Id:** `marriage.h7.saturn_aspect`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** the 7th house is aspected by Shani
- **Text shown to users:** Saturn casts its aspect on the 7th house. This is commonly read as a sign of delay or of a serious, duty-bound approach to marriage. Recorded as a timing indication, not scored.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 24.3% of samples (24.3% after cancellations)

## Saturn occupies the 7th house

- **Id:** `marriage.h7.saturn_occupant`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** Shani is in the 7th house
- **Text shown to users:** Saturn sits in the 7th house. Commonly read as delay in marriage. Recorded as a timing indication; the malefic-occupant rule already scores the affliction.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 8.2% of samples (8.2% after cancellations)

## Female native: Jupiter is debilitated

- **Id:** `marriage.jupiter.debilitated_female`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** all of: [the native is female; Guru is debilitated]
- **Text shown to users:** For a female native, classical texts also examine Jupiter as a significator of the husband. Here Jupiter is debilitated. Cannot be evaluated unless the native's sex is supplied.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 0.0% of samples (0.0% after cancellations); could not evaluate in 7.6%

## Female native: Jupiter is strong

- **Id:** `marriage.jupiter.strong_female`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** all of: [the native is female; Guru is exalted, in moolatrikona or in its own sign]
- **Text shown to users:** For a female native, classical texts also examine Jupiter as a significator of the husband. Here Jupiter is exalted, in moolatrikona or in its own sign. Cannot be evaluated unless the native's sex is supplied.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 0.0% of samples (0.0% after cancellations); could not evaluate in 25.9%

## Chevvai dosha cancelled: Mars in its own or exaltation sign

- **Id:** `marriage.kuja.cancel.dignity`
- **Status:** Approved
- **Tradition:** Tamil
- **Polarity:** +0
- **Condition:** Kuja is in Mesha, Vrischika or Makara
- **Cancels:** `marriage.kuja.from_lagna`, `marriage.kuja.from_moon`, `marriage.kuja.from_venus`
- **Text shown to users:** Mars is in its own sign (Mesha or Vrischika) or its exaltation sign (Makara), a commonly recognised cancellation of Chevvai dosha.
- **Source:** none supplied
- **Provenance note:** Chevvai (Kuja) dosha as commonly practised in Tamil Nadu, written from general knowledge. Reference points, houses and the cancellation list vary between practitioners (DESIGN.md variants R-2 to R-4); the reviewer must confirm each before approval.
- **Fires in:** 24.0% of samples (24.0% after cancellations)

## Chevvai dosha cancelled: Jupiter with or aspecting Mars

- **Id:** `marriage.kuja.cancel.jupiter`
- **Status:** Approved
- **Tradition:** Tamil
- **Polarity:** +0
- **Condition:** any of: [Kuja and Guru are in the same sign; Guru aspects Kuja]
- **Cancels:** `marriage.kuja.from_lagna`, `marriage.kuja.from_moon`, `marriage.kuja.from_venus`
- **Text shown to users:** Jupiter shares a sign with Mars or aspects it, a commonly recognised cancellation of Chevvai dosha.
- **Source:** none supplied
- **Provenance note:** Chevvai (Kuja) dosha as commonly practised in Tamil Nadu, written from general knowledge. Reference points, houses and the cancellation list vary between practitioners (DESIGN.md variants R-2 to R-4); the reviewer must confirm each before approval.
- **Fires in:** 29.1% of samples (29.1% after cancellations)

## Chevvai dosha cancelled by a house-specific sign exception, counted from the lagna

- **Id:** `marriage.kuja.cancel.sign_exception_lagna`
- **Status:** Approved
- **Tradition:** Tamil
- **Polarity:** +0
- **Condition:** any of: [all of: [Kuja is in the 2nd house counted from the lagna; Kuja is in Mithuna or Kanya]; all of: [Kuja is in the 4th house counted from the lagna; Kuja is in Mesha or Vrischika]; all of: [Kuja is in the 7th house counted from the lagna; Kuja is in Karka or Makara]; all of: [Kuja is in the 8th house counted from the lagna; Kuja is in Dhanus or Meena]; all of: [Kuja is in the 12th house counted from the lagna; Kuja is in Vrishabha or Tula]]
- **Cancels:** `marriage.kuja.from_lagna`
- **Text shown to users:** Mars occupies a dosha house counted from the lagna, but in a sign that commonly recognised exceptions exempt: the 2nd in Mithuna or Kanya, the 4th in Mesha or Vrischika, the 7th in Karka or Makara, the 8th in Dhanus or Meena, the 12th in Vrishabha or Tula.
- **Source:** none supplied
- **Provenance note:** Chevvai (Kuja) dosha as commonly practised in Tamil Nadu, written from general knowledge. Reference points, houses and the cancellation list vary between practitioners (DESIGN.md variants R-2 to R-4); the reviewer must confirm each before approval.
- **Fires in:** 8.1% of samples (8.1% after cancellations)

## Chevvai dosha cancelled by a house-specific sign exception, counted from the Moon

- **Id:** `marriage.kuja.cancel.sign_exception_moon`
- **Status:** Approved
- **Tradition:** Tamil
- **Polarity:** +0
- **Condition:** any of: [all of: [Kuja is in the 2nd house counted from Chandra; Kuja is in Mithuna or Kanya]; all of: [Kuja is in the 4th house counted from Chandra; Kuja is in Mesha or Vrischika]; all of: [Kuja is in the 7th house counted from Chandra; Kuja is in Karka or Makara]; all of: [Kuja is in the 8th house counted from Chandra; Kuja is in Dhanus or Meena]; all of: [Kuja is in the 12th house counted from Chandra; Kuja is in Vrishabha or Tula]]
- **Cancels:** `marriage.kuja.from_moon`
- **Text shown to users:** Mars occupies a dosha house counted from the Moon, but in a sign that commonly recognised exceptions exempt: the 2nd in Mithuna or Kanya, the 4th in Mesha or Vrischika, the 7th in Karka or Makara, the 8th in Dhanus or Meena, the 12th in Vrishabha or Tula.
- **Source:** none supplied
- **Provenance note:** Chevvai (Kuja) dosha as commonly practised in Tamil Nadu, written from general knowledge. Reference points, houses and the cancellation list vary between practitioners (DESIGN.md variants R-2 to R-4); the reviewer must confirm each before approval.
- **Fires in:** 6.6% of samples (6.6% after cancellations)

## Chevvai dosha cancelled by a house-specific sign exception, counted from Venus

- **Id:** `marriage.kuja.cancel.sign_exception_venus`
- **Status:** Rejected
- **Tradition:** Tamil
- **Polarity:** +0
- **Condition:** any of: [all of: [Kuja is in the 2nd house counted from Shukra; Kuja is in Mithuna or Kanya]; all of: [Kuja is in the 4th house counted from Shukra; Kuja is in Mesha or Vrischika]; all of: [Kuja is in the 7th house counted from Shukra; Kuja is in Karka or Makara]; all of: [Kuja is in the 8th house counted from Shukra; Kuja is in Dhanus or Meena]; all of: [Kuja is in the 12th house counted from Shukra; Kuja is in Vrishabha or Tula]]
- **Cancels:** `marriage.kuja.from_venus`
- **Text shown to users:** Mars occupies a dosha house counted from Venus, but in a sign that commonly recognised exceptions exempt: the 2nd in Mithuna or Kanya, the 4th in Mesha or Vrischika, the 7th in Karka or Makara, the 8th in Dhanus or Meena, the 12th in Vrishabha or Tula.
- **Source:** none supplied
- **Provenance note:** Chevvai (Kuja) dosha as commonly practised in Tamil Nadu, written from general knowledge. Reference points, houses and the cancellation list vary between practitioners (DESIGN.md variants R-2 to R-4); the reviewer must confirm each before approval.
- **Fires in:** 0.0% of samples (0.0% after cancellations)

## Chevvai dosha counted from the lagna

- **Id:** `marriage.kuja.from_lagna`
- **Status:** Approved
- **Tradition:** Tamil
- **Polarity:** -2
- **Condition:** Kuja is in the 2nd, 4th, 7th, 8th or 12th house counted from the lagna
- **Text shown to users:** Mars is in the 2nd, 4th, 7th, 8th or 12th house counted from the lagna: Chevvai (Kuja) dosha. Tamil practice gives this great weight in marriage matching, and it is cancelled in several recognised conditions.
- **Source:** none supplied
- **Provenance note:** Chevvai (Kuja) dosha as commonly practised in Tamil Nadu, written from general knowledge. Reference points, houses and the cancellation list vary between practitioners (DESIGN.md variants R-2 to R-4); the reviewer must confirm each before approval.
- **Fires in:** 44.7% of samples (20.0% after cancellations)

## Chevvai dosha counted from the Moon

- **Id:** `marriage.kuja.from_moon`
- **Status:** Approved
- **Tradition:** Tamil
- **Polarity:** -1
- **Condition:** Kuja is in the 2nd, 4th, 7th, 8th or 12th house counted from Chandra
- **Text shown to users:** Mars is in the 2nd, 4th, 7th, 8th or 12th house counted from the Moon: Chevvai (Kuja) dosha as judged from the Moon.
- **Source:** none supplied
- **Provenance note:** Chevvai (Kuja) dosha as commonly practised in Tamil Nadu, written from general knowledge. Reference points, houses and the cancellation list vary between practitioners (DESIGN.md variants R-2 to R-4); the reviewer must confirm each before approval.
- **Fires in:** 41.5% of samples (19.6% after cancellations)

## Chevvai dosha counted from Venus

- **Id:** `marriage.kuja.from_venus`
- **Status:** Rejected
- **Tradition:** Tamil
- **Polarity:** -2
- **Condition:** Kuja is in the 2nd, 4th, 7th, 8th or 12th house counted from Shukra
- **Text shown to users:** Mars is in the 2nd, 4th, 7th, 8th or 12th house counted from Venus. Not all practitioners use Venus as a reference point (variant R-2).
- **Source:** none supplied
- **Provenance note:** Chevvai (Kuja) dosha as commonly practised in Tamil Nadu, written from general knowledge. Reference points, houses and the cancellation list vary between practitioners (DESIGN.md variants R-2 to R-4); the reviewer must confirm each before approval.
- **Fires in:** 0.0% of samples (0.0% after cancellations)

## The 7th lord is combust

- **Id:** `marriage.l7.combust`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the lord of the 7th is combust
- **Text shown to users:** The lord of the 7th house is combust, too close to the Sun, which is traditionally held to weaken it.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 15.7% of samples (15.7% after cancellations)

## The 7th lord is debilitated

- **Id:** `marriage.l7.debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -2
- **Condition:** the lord of the 7th is debilitated
- **Text shown to users:** The lord of the 7th house is debilitated in the rasi chart. Classical texts treat a debilitated lord as struggling to deliver its house's results, unless cancelled.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 8.6% of samples (8.6% after cancellations)

## The 7th lord is in the 6th, 8th or 12th house

- **Id:** `marriage.l7.dusthana`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -2
- **Condition:** the lord of the 7th is in the 6th, 8th or 12th house
- **Text shown to users:** The lord of the 7th house sits in a dusthana (the 6th, 8th or 12th). Classical texts treat this as weakening what the 7th house promises.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 26.0% of samples (26.0% after cancellations)

## The 7th lord is in a kendra or trikona

- **Id:** `marriage.l7.kendra_trikona`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the lord of the 7th is in the 1st, 4th, 5th, 7th, 9th or 10th house
- **Text shown to users:** The lord of the 7th house sits in a kendra or trikona. Classical texts treat this as supporting what the 7th house promises.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 49.1% of samples (49.1% after cancellations)

## The 7th lord is debilitated in the navamsa

- **Id:** `marriage.l7.navamsa_debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the lord of the 7th is debilitated in the D-9 Navamsa
- **Text shown to users:** The lord of the rasi-chart 7th house is debilitated in the navamsa (D-9).
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 6.9% of samples (6.9% after cancellations)

## The 7th lord is strong in the navamsa

- **Id:** `marriage.l7.navamsa_strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the lord of the 7th is exalted, in moolatrikona or in its own sign in the D-9 Navamsa
- **Text shown to users:** The lord of the rasi-chart 7th house is exalted, in moolatrikona or in its own sign in the navamsa (D-9), the chart of marriage.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 22.2% of samples (22.2% after cancellations)

## The 7th lord is exalted, in moolatrikona or in its own sign

- **Id:** `marriage.l7.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the lord of the 7th is exalted, in moolatrikona or in its own sign
- **Text shown to users:** The lord of the 7th house is exalted, in moolatrikona or in its own sign in the rasi chart, and so able to give its results fully.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 23.4% of samples (23.4% after cancellations)

## The 7th house has 28 or more SAV bindus

- **Id:** `marriage.sav.h7_strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 7th house has at least 28 SAV bindus
- **Text shown to users:** The 7th house holds 28 or more Sarvashtakavarga bindus, at or above the average of 337 / 12 (about 28). Commonly read as a well-supported house. The threshold is for the reviewer to confirm.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 28.2% of samples (28.2% after cancellations)

## The 7th house has 24 or fewer SAV bindus

- **Id:** `marriage.sav.h7_weak`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the 7th house has at most 24 SAV bindus
- **Text shown to users:** The 7th house holds 24 or fewer Sarvashtakavarga bindus, well below the average of about 28. Commonly read as a house needing support. The threshold is for the reviewer to confirm.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 46.4% of samples (46.4% after cancellations)

## Periods examined for marriage

- **Id:** `marriage.timing.periods`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** always
- **Timing lords:** the lord of the 7th, Shukra, the lord of the 7th in the D-9 Navamsa
- **Text shown to users:** Dashas and bhuktis traditionally examined for the timing of marriage: those of the 7th lord, of Venus, and of the 7th lord of the navamsa.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 100.0% of samples (100.0% after cancellations)

## Venus is combust

- **Id:** `marriage.venus.combust`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** Shukra is combust
- **Text shown to users:** Venus, the karaka of marriage, is combust, too close to the Sun, which is traditionally held to weaken it.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 14.2% of samples (14.2% after cancellations)

## Venus is debilitated

- **Id:** `marriage.venus.debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -2
- **Condition:** Shukra is debilitated
- **Text shown to users:** Venus, the karaka of marriage, is debilitated in the rasi chart.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 7.2% of samples (7.2% after cancellations)

## Venus is in the 6th or 8th house

- **Id:** `marriage.venus.dusthana`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** Shukra is in the 6th or 8th house
- **Text shown to users:** Venus, the karaka of marriage, sits in the 6th or 8th house, which classical texts treat as weakening it. (Venus in the 12th, the house of bed comforts, is traditionally favourable and is not counted here.)
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 17.9% of samples (17.9% after cancellations)

## Venus, the karaka of marriage, occupies the 7th house

- **Id:** `marriage.venus.karaka_in_7`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** Shukra is in the 7th house
- **Text shown to users:** Venus, the karaka of marriage, sits in the 7th house itself. The classical dictum karako bhava nashaya holds that a karaka in its own bhava can harm that bhava's matters; how strongly it applies is debated, so this is noted rather than scored.
- **Source:** none supplied
- **Provenance note:** Added during review. The dictum is commonly stated; the reviewer did not verify a textual source.
- **Fires in:** 5.9% of samples (5.9% after cancellations)

## Venus is debilitated in the navamsa

- **Id:** `marriage.venus.navamsa_debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** Shukra is debilitated in the D-9 Navamsa
- **Text shown to users:** Venus is debilitated in the navamsa (D-9).
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 7.5% of samples (7.5% after cancellations)

## Venus is strong in the navamsa

- **Id:** `marriage.venus.navamsa_strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** Shukra is exalted, in moolatrikona or in its own sign in the D-9 Navamsa
- **Text shown to users:** Venus is exalted, in moolatrikona or in its own sign in the navamsa (D-9).
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 25.6% of samples (25.6% after cancellations)

## Venus is exalted, in moolatrikona or in its own sign

- **Id:** `marriage.venus.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** Shukra is exalted, in moolatrikona or in its own sign
- **Text shown to users:** Venus, the karaka (significator) of marriage, is exalted, in moolatrikona or in its own sign in the rasi chart.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 27.0% of samples (27.0% after cancellations)

## Venus shares a sign with Saturn, Rahu or Ketu

- **Id:** `marriage.venus.with_malefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** any of: [Shukra and Shani are in the same sign; Shukra and Rahu are in the same sign; Shukra and Ketu are in the same sign]
- **Text shown to users:** Venus, the karaka of marriage, shares a sign with Saturn, Rahu or Ketu, which classical texts treat as affliction to the karaka.
- **Source:** none supplied
- **Provenance note:** Written from general knowledge of a commonly stated principle. The reviewer must confirm the interpretation and supply the textual source before approval.
- **Fires in:** 22.0% of samples (22.0% after cancellations)

## Poruthams

The ten-porutham procedure is specified in `docs/phase3/DESIGN.md` section 8,
with its tables and variant choices (P-1 to P-8). Approve each porutham in
`corpus/marriage/porutham.review.json`. Match rates are over all 11,664
bride and groom pada combinations.

| Porutham | Status | Matching |
|---|---|---|
| Dina | Approved | 55.6% |
| Gana | Approved | 55.6% |
| Mahendra | Approved | 29.6% |
| Stree Deergha | Approved | 51.9% |
| Yoni | Approved | 92.9% |
| Rasi | Approved | 66.7% |
| Rasi Adhipati | Approved | 63.9% |
| Vasya | Rejected | blocked: table to be supplied |
| Rajju | Approved | 79.0% |
| Vedha | Approved | 95.9% |

