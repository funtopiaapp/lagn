# Review sheet: progeny

For the reviewing astrologer. Each rule below is exactly what the engine evaluates:
the "Condition" line is generated from the rule itself, not written separately.

To approve a rule, set `review.status` to `approved` and fill in `reviewer` and
`date` in the corpus file. To reject, set `rejected`. Please correct anything wrong:
the condition, the polarity (-3 to +3), the cancellations, or the wording.

Firing rates come from 1000 sample births in South India, 1920-2025, evaluated in review mode.

| Rules | 15 |
|---|---|
| Approved | 15 |
| Draft | 0 |
| Rejected | 0 |

## Jupiter is strong in the saptamsa (D-7)

- **Id:** `progeny.d7.jupiter_strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** Guru is exalted, in moolatrikona or in its own sign in the D-7 Saptamsa
- **Text shown to users:** Jupiter is exalted, in moolatrikona or in its own sign in the saptamsa (D-7), the chart of progeny.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 24.1% of samples (24.1% after cancellations)

## A natural benefic occupies the 5th house

- **Id:** `progeny.h5.benefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 5th house is occupied by benefics
- **Text shown to users:** A natural benefic sits in the 5th house of children.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 25.8% of samples (25.8% after cancellations)

## Rahu or Ketu occupies the 5th house

- **Id:** `progeny.h5.nodes`
- **Status:** Approved
- **Tradition:** Kerala
- **Polarity:** -1
- **Condition:** the 5th house is occupied by Rahu or Ketu
- **Text shown to users:** Rahu or Ketu sits in the 5th house. South Indian practice, Kerala especially, associates this with sarpa dosha affecting progeny, and recommends traditional remedies.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 16.8% of samples (16.8% after cancellations)

## Saturn occupies the 5th house

- **Id:** `progeny.h5.saturn`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** Shani is in the 5th house
- **Text shown to users:** Saturn in the 5th house is traditionally read as delay rather than denial in matters of children. Noted, not scored.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 8.6% of samples (8.6% after cancellations)

## Jupiter is debilitated

- **Id:** `progeny.jupiter.debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** Guru is debilitated
- **Text shown to users:** Jupiter, the karaka of children (putrakaraka), is debilitated, unless the debilitation is cancelled.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 7.6% of samples (0.0% after cancellations)

## Jupiter is in the 6th, 8th or 12th house

- **Id:** `progeny.jupiter.dusthana`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** Guru is in the 6th, 8th or 12th house
- **Text shown to users:** Jupiter, the putrakaraka, sits in a dusthana, traditionally a sign that matters of children may call for patience.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 24.3% of samples (24.3% after cancellations)

## Jupiter's debilitation is cancelled

- **Id:** `progeny.jupiter.neecha_bhanga`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** Guru is debilitated with the debilitation cancelled (neecha bhanga)
- **Cancels:** `progeny.jupiter.debilitated`
- **Text shown to users:** Jupiter is debilitated, but a recognised condition cancels the debilitation (neecha bhanga).
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 7.6% of samples (7.6% after cancellations)

## Jupiter is exalted, in moolatrikona or in its own sign

- **Id:** `progeny.jupiter.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** Guru is exalted, in moolatrikona or in its own sign
- **Text shown to users:** Jupiter, the karaka of children (putrakaraka), is exalted, in moolatrikona or in its own sign.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 25.9% of samples (25.9% after cancellations)

## Jupiter aspects the 5th house

- **Id:** `progeny.jupiter_aspects_5`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the 5th house is aspected by Guru
- **Text shown to users:** Jupiter, the putrakaraka, aspects the 5th house of children, traditionally a strong blessing for progeny.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 24.1% of samples (24.1% after cancellations)

## The 5th lord is debilitated

- **Id:** `progeny.l5.debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -2
- **Condition:** the lord of the 5th is debilitated
- **Text shown to users:** The lord of the 5th house is debilitated, which classical texts read as difficulty with children, unless the debilitation is cancelled.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 6.3% of samples (0.6% after cancellations)

## The 5th lord is in the 6th, 8th or 12th house

- **Id:** `progeny.l5.dusthana`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -2
- **Condition:** the lord of the 5th is in the 6th, 8th or 12th house
- **Text shown to users:** The lord of the 5th house sits in a dusthana (the 6th, 8th or 12th), which classical texts read as strain on children.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 24.1% of samples (24.1% after cancellations)

## The 5th lord is in a kendra or trikona

- **Id:** `progeny.l5.kendra_trikona`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the lord of the 5th is in the 1st, 4th, 5th, 7th, 9th or 10th house
- **Text shown to users:** The lord of the 5th house sits in a kendra or trikona, which classical texts read as support for children.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 50.7% of samples (50.7% after cancellations)

## The 5th lord's debilitation is cancelled

- **Id:** `progeny.l5.neecha_bhanga`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** the lord of the 5th is debilitated with the debilitation cancelled (neecha bhanga)
- **Cancels:** `progeny.l5.debilitated`
- **Text shown to users:** The lord of the 5th house is debilitated, but a recognised condition cancels the debilitation (neecha bhanga).
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 5.7% of samples (5.7% after cancellations)

## The 5th lord is exalted, in moolatrikona or in its own sign

- **Id:** `progeny.l5.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the lord of the 5th is exalted, in moolatrikona or in its own sign
- **Text shown to users:** The lord of the 5th house is exalted, in moolatrikona or in its own sign, and able to give its results fully.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 25.3% of samples (25.3% after cancellations)

## Periods examined for children

- **Id:** `progeny.timing.periods`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** always
- **Timing lords:** the lord of the 5th, Guru, the lord of the 5th in the D-7 Saptamsa
- **Text shown to users:** Dashas and bhuktis traditionally examined for the birth of children: those of the 5th lord, Jupiter and the 5th lord of the saptamsa.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 100.0% of samples (100.0% after cancellations)

