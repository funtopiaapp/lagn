# Review sheet: wealth

For the reviewing astrologer. Each rule below is exactly what the engine evaluates:
the "Condition" line is generated from the rule itself, not written separately.

To approve a rule, set `review.status` to `approved` and fill in `reviewer` and
`date` in the corpus file. To reject, set `rejected`. Please correct anything wrong:
the condition, the polarity (-3 to +3), the cancellations, or the wording.

Firing rates come from 1000 sample births in South India, 1920-2025, evaluated in review mode.

| Rules | 16 |
|---|---|
| Approved | 16 |
| Draft | 0 |
| Rejected | 0 |

## Dhana yoga: the lords of wealth houses are related

- **Id:** `wealth.dhana_yoga`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** any of: [the lord of the 2nd and the lord of the 11th are related (conjunction, mutual aspect or exchange); the lord of the 2nd and the lord of the 9th are related (conjunction, mutual aspect or exchange); the lord of the 2nd and the lord of the 5th are related (conjunction, mutual aspect or exchange); the lord of the 5th and the lord of the 11th are related (conjunction, mutual aspect or exchange); the lord of the 9th and the lord of the 11th are related (conjunction, mutual aspect or exchange); the lord of the 1st and the lord of the 2nd are related (conjunction, mutual aspect or exchange); the lord of the 1st and the lord of the 11th are related (conjunction, mutual aspect or exchange); the lord of the 5th and the lord of the 9th are related (conjunction, mutual aspect or exchange)]
- **Text shown to users:** Lords of the wealth-giving houses (1, 2, 5, 9, 11) are related by conjunction, mutual aspect or exchange: a dhana yoga, classically a sign of prosperity.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 79.5% of samples (79.5% after cancellations)

## A graha occupies the 11th house

- **Id:** `wealth.h11.occupied`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 11th house is occupied by any graha
- **Text shown to users:** At least one graha sits in the 11th house. Classical texts hold that every graha gives gains from the 11th.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 48.3% of samples (48.3% after cancellations)

## A natural benefic occupies the 2nd house

- **Id:** `wealth.h2.benefic_occupant`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 2nd house is occupied by benefics
- **Text shown to users:** A natural benefic sits in the 2nd house, the house of wealth and family.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 25.0% of samples (25.0% after cancellations)

## Jupiter is debilitated

- **Id:** `wealth.jupiter.debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** Guru is debilitated
- **Text shown to users:** Jupiter, the karaka of wealth and wisdom, is debilitated, unless the debilitation is cancelled.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 7.6% of samples (0.0% after cancellations)

## Jupiter's debilitation is cancelled

- **Id:** `wealth.jupiter.neecha_bhanga`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** Guru is debilitated with the debilitation cancelled (neecha bhanga)
- **Cancels:** `wealth.jupiter.debilitated`
- **Text shown to users:** Jupiter is debilitated, but a recognised condition cancels the debilitation (neecha bhanga).
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 7.6% of samples (7.6% after cancellations)

## Jupiter is exalted, in moolatrikona or in its own sign

- **Id:** `wealth.jupiter.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** Guru is exalted, in moolatrikona or in its own sign
- **Text shown to users:** Jupiter, the karaka of wealth and wisdom, is exalted, in moolatrikona or in its own sign.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 25.9% of samples (25.9% after cancellations)

## The 11th lord is in the 6th, 8th or 12th house

- **Id:** `wealth.l11.dusthana`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the lord of the 11th is in the 6th, 8th or 12th house
- **Text shown to users:** The lord of the 11th house, the house of gains, sits in a dusthana, which classical texts read as gains that come with effort or are spent.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 26.1% of samples (26.1% after cancellations)

## The 11th lord is exalted, in moolatrikona or in its own sign

- **Id:** `wealth.l11.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the lord of the 11th is exalted, in moolatrikona or in its own sign
- **Text shown to users:** The lord of the 11th house, the house of gains, is exalted, in moolatrikona or in its own sign.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 23.9% of samples (23.9% after cancellations)

## The 2nd lord is debilitated

- **Id:** `wealth.l2.debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -2
- **Condition:** the lord of the 2nd is debilitated
- **Text shown to users:** The lord of the 2nd house is debilitated, which classical texts read as difficulty with wealth and family resources, unless the debilitation is cancelled.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 8.0% of samples (0.5% after cancellations)

## The 2nd lord is in the 6th, 8th or 12th house

- **Id:** `wealth.l2.dusthana`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -2
- **Condition:** the lord of the 2nd is in the 6th, 8th or 12th house
- **Text shown to users:** The lord of the 2nd house sits in a dusthana (the 6th, 8th or 12th), which classical texts read as strain on wealth and family resources.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 25.7% of samples (25.7% after cancellations)

## The 2nd lord is in a kendra or trikona

- **Id:** `wealth.l2.kendra_trikona`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the lord of the 2nd is in the 1st, 4th, 5th, 7th, 9th or 10th house
- **Text shown to users:** The lord of the 2nd house sits in a kendra or trikona, which classical texts read as support for wealth and family resources.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 47.4% of samples (47.4% after cancellations)

## The 2nd lord's debilitation is cancelled

- **Id:** `wealth.l2.neecha_bhanga`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** the lord of the 2nd is debilitated with the debilitation cancelled (neecha bhanga)
- **Cancels:** `wealth.l2.debilitated`
- **Text shown to users:** The lord of the 2nd house is debilitated, but a recognised condition cancels the debilitation (neecha bhanga).
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 7.5% of samples (7.5% after cancellations)

## The 2nd lord is exalted, in moolatrikona or in its own sign

- **Id:** `wealth.l2.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the lord of the 2nd is exalted, in moolatrikona or in its own sign
- **Text shown to users:** The lord of the 2nd house is exalted, in moolatrikona or in its own sign, and able to give its results fully.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 21.3% of samples (21.3% after cancellations)

## The 9th lord is strong and in a kendra or trikona

- **Id:** `wealth.lakshmi`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** all of: [the lord of the 9th is in the 1st, 4th, 5th, 7th, 9th or 10th house; the lord of the 9th is exalted, in moolatrikona or in its own sign]
- **Text shown to users:** The lord of the 9th house, the house of fortune, is dignified and well placed: a strong indication of fortune (akin to Lakshmi yoga).
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 14.0% of samples (14.0% after cancellations)

## The 11th house has 28 or more SAV bindus

- **Id:** `wealth.sav.h11_strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 11th house has at least 28 SAV bindus
- **Text shown to users:** The 11th house holds 28 or more Sarvashtakavarga bindus: a well-supported house of gains.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 82.5% of samples (82.5% after cancellations)

## Periods examined for wealth

- **Id:** `wealth.timing.periods`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** always
- **Timing lords:** the lord of the 2nd, the lord of the 11th, the lord of the 9th
- **Text shown to users:** Dashas and bhuktis traditionally examined for financial gains: those of the 2nd, 11th and 9th lords.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 100.0% of samples (100.0% after cancellations)

