# Review sheet: parents

For the reviewing astrologer. Each rule below is exactly what the engine evaluates:
the "Condition" line is generated from the rule itself, not written separately.

To approve a rule, set `review.status` to `approved` and fill in `reviewer` and
`date` in the corpus file. To reject, set `rejected`. Please correct anything wrong:
the condition, the polarity (-3 to +3), the cancellations, or the wording.

Firing rates come from 1000 sample births in South India, 1920-2025, evaluated in review mode.

| Rules | 14 |
|---|---|
| Approved | 14 |
| Draft | 0 |
| Rejected | 0 |

## A natural benefic occupies the 9th house

- **Id:** `parents.father.benefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 9th house is occupied by benefics
- **Text shown to users:** A natural benefic sits in the 9th house, the house of the father.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 22.0% of samples (22.0% after cancellations)

## Jupiter aspects the 9th house

- **Id:** `parents.father.jupiter_aspect`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 9th house is aspected by Guru
- **Text shown to users:** Jupiter aspects the 9th house, the house of the father, traditionally a protective influence.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 26.0% of samples (26.0% after cancellations)

## The Sun shares a sign with Saturn, Rahu or Ketu

- **Id:** `parents.father.karaka_afflicted`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** any of: [Surya and Shani are in the same sign; Surya and Rahu are in the same sign; Surya and Ketu are in the same sign]
- **Text shown to users:** The Sun, significator of the father, shares a sign with Saturn, Rahu or Ketu, traditionally read as strain the father may carry.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 22.3% of samples (22.3% after cancellations)

## The Sun is exalted, in moolatrikona or in its own sign

- **Id:** `parents.father.karaka_strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** Surya is exalted, in moolatrikona or in its own sign
- **Text shown to users:** The Sun, significator of the father, is dignified.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 15.8% of samples (15.8% after cancellations)

## The 9th lord is in the 6th, 8th or 12th house

- **Id:** `parents.father.lord_dusthana`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the lord of the 9th is in the 6th, 8th or 12th house
- **Text shown to users:** The lord of the 9th house, the house of the father, sits in a dusthana, traditionally a sign to support the father with care.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 26.1% of samples (26.1% after cancellations)

## The 9th lord is dignified or in a kendra or trikona

- **Id:** `parents.father.lord_supported`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** any of: [the lord of the 9th is exalted, in moolatrikona or in its own sign; the lord of the 9th is in the 1st, 4th, 5th, 7th, 9th or 10th house]
- **Text shown to users:** The lord of the 9th house, the house of the father, is dignified or well placed, traditionally a support for the father's welfare.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 57.7% of samples (57.7% after cancellations)

## Mars, Saturn, Rahu or Ketu occupies the 9th house

- **Id:** `parents.father.malefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the 9th house is occupied by Kuja, Shani, Rahu or Ketu
- **Text shown to users:** A natural malefic sits in the 9th house, the house of the father.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 30.0% of samples (30.0% after cancellations)

## A natural benefic occupies the 4th house

- **Id:** `parents.mother.benefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 4th house is occupied by benefics
- **Text shown to users:** A natural benefic sits in the 4th house, the house of the mother.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 28.3% of samples (28.3% after cancellations)

## Jupiter aspects the 4th house

- **Id:** `parents.mother.jupiter_aspect`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 4th house is aspected by Guru
- **Text shown to users:** Jupiter aspects the 4th house, the house of the mother, traditionally a protective influence.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 26.0% of samples (26.0% after cancellations)

## The Moon shares a sign with Saturn, Rahu or Ketu

- **Id:** `parents.mother.karaka_afflicted`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** any of: [Chandra and Shani are in the same sign; Chandra and Rahu are in the same sign; Chandra and Ketu are in the same sign]
- **Text shown to users:** The Moon, significator of the mother, shares a sign with Saturn, Rahu or Ketu, traditionally read as strain the mother may carry.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 23.0% of samples (23.0% after cancellations)

## The Moon is exalted, in moolatrikona or in its own sign

- **Id:** `parents.mother.karaka_strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** Chandra is exalted, in moolatrikona or in its own sign
- **Text shown to users:** The Moon, significator of the mother, is dignified.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 18.4% of samples (18.4% after cancellations)

## The 4th lord is in the 6th, 8th or 12th house

- **Id:** `parents.mother.lord_dusthana`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the lord of the 4th is in the 6th, 8th or 12th house
- **Text shown to users:** The lord of the 4th house, the house of the mother, sits in a dusthana, traditionally a sign to support the mother with care.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 22.1% of samples (22.1% after cancellations)

## The 4th lord is dignified or in a kendra or trikona

- **Id:** `parents.mother.lord_supported`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** any of: [the lord of the 4th is exalted, in moolatrikona or in its own sign; the lord of the 4th is in the 1st, 4th, 5th, 7th, 9th or 10th house]
- **Text shown to users:** The lord of the 4th house, the house of the mother, is dignified or well placed, traditionally a support for the mother's welfare.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 59.9% of samples (59.9% after cancellations)

## Mars, Saturn, Rahu or Ketu occupies the 4th house

- **Id:** `parents.mother.malefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the 4th house is occupied by Kuja, Shani, Rahu or Ketu
- **Text shown to users:** A natural malefic sits in the 4th house, the house of the mother.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 32.7% of samples (32.7% after cancellations)

