# Review sheet: vitality

For the reviewing astrologer. Each rule below is exactly what the engine evaluates:
the "Condition" line is generated from the rule itself, not written separately.

To approve a rule, set `review.status` to `approved` and fill in `reviewer` and
`date` in the corpus file. To reject, set `rejected`. Please correct anything wrong:
the condition, the polarity (-3 to +3), the cancellations, or the wording.

Firing rates come from 1000 sample births in South India, 1920-2025, evaluated in review mode.

| Rules | 10 |
|---|---|
| Approved | 10 |
| Draft | 0 |
| Rejected | 0 |

## Periods traditionally marked for extra care

- **Id:** `vitality.care.periods`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** always
- **Timing lords:** the lord of the 6th, the lord of the 8th
- **Text shown to users:** Dashas and bhuktis of the 6th and 8th lords are traditionally times to look after health with extra care: rest, routine and regular check-ups.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 100.0% of samples (100.0% after cancellations)

## A natural benefic occupies the 8th house

- **Id:** `vitality.h8.benefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 8th house is occupied by benefics
- **Text shown to users:** A natural benefic sits in the 8th house, traditionally a support for vitality.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 24.7% of samples (24.7% after cancellations)

## Mars, Saturn, Rahu or Ketu occupies the 8th house

- **Id:** `vitality.h8.malefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the 8th house is occupied by Kuja, Shani, Rahu or Ketu
- **Text shown to users:** A natural malefic sits in the 8th house, traditionally a sign to take particular care in periods ruled by that graha.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 30.4% of samples (30.4% after cancellations)

## Jupiter aspects the lagna or the 8th house

- **Id:** `vitality.jupiter_aspect`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** any of: [the 1st house is aspected by Guru; the 8th house is aspected by Guru]
- **Text shown to users:** Jupiter aspects the lagna or the 8th house, traditionally a protective influence on vitality.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 51.5% of samples (51.5% after cancellations)

## The lagna lord is exalted, in moolatrikona or in its own sign

- **Id:** `vitality.l1.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the lord of the 1st is exalted, in moolatrikona or in its own sign
- **Text shown to users:** The lord of the lagna is dignified, traditionally a sign of strong constitution and resilience.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 23.7% of samples (23.7% after cancellations)

## The lagna lord is debilitated or in the 6th, 8th or 12th house

- **Id:** `vitality.l1.weak`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** any of: [the lord of the 1st is debilitated; the lord of the 1st is in the 6th, 8th or 12th house]
- **Text shown to users:** The lord of the lagna is debilitated or in a dusthana, traditionally a sign to build and protect one's constitution deliberately.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 30.6% of samples (30.6% after cancellations)

## The 8th lord is dignified or in a kendra or trikona

- **Id:** `vitality.l8.well_placed`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** any of: [the lord of the 8th is exalted, in moolatrikona or in its own sign; the lord of the 8th is in the 1st, 4th, 5th, 7th, 9th or 10th house]
- **Text shown to users:** The lord of the 8th house is dignified or well placed, traditionally a sign of sustained vitality.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 65.7% of samples (65.7% after cancellations)

## Saturn is debilitated

- **Id:** `vitality.saturn.debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** Shani is debilitated
- **Text shown to users:** Saturn, the karaka of longevity (ayushkaraka), is debilitated, unless the debilitation is cancelled.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 5.9% of samples (0.6% after cancellations)

## Saturn's debilitation is cancelled

- **Id:** `vitality.saturn.neecha_bhanga`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** Shani is debilitated with the debilitation cancelled (neecha bhanga)
- **Cancels:** `vitality.saturn.debilitated`
- **Text shown to users:** Saturn is debilitated, but a recognised condition cancels the debilitation (neecha bhanga).
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 5.3% of samples (5.3% after cancellations)

## Saturn is exalted, in moolatrikona or in its own sign

- **Id:** `vitality.saturn.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** Shani is exalted, in moolatrikona or in its own sign
- **Text shown to users:** Saturn, the karaka of longevity (ayushkaraka), is exalted, in moolatrikona or in its own sign.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 31.2% of samples (31.2% after cancellations)

