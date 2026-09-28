# Review sheet: later_life

For the reviewing astrologer. Each rule below is exactly what the engine evaluates:
the "Condition" line is generated from the rule itself, not written separately.

To approve a rule, set `review.status` to `approved` and fill in `reviewer` and
`date` in the corpus file. To reject, set `rejected`. Please correct anything wrong:
the condition, the polarity (-3 to +3), the cancellations, or the wording.

Firing rates come from 1000 sample births in South India, 1920-2025, evaluated in review mode.

| Rules | 9 |
|---|---|
| Approved | 9 |
| Draft | 0 |
| Rejected | 0 |

## The 11th lord is dignified or in a kendra or trikona

- **Id:** `later_life.continuing_income`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** any of: [the lord of the 11th is exalted, in moolatrikona or in its own sign; the lord of the 11th is in the 1st, 4th, 5th, 7th, 9th or 10th house]
- **Text shown to users:** The lord of the 11th house, gains, is dignified or well placed: traditionally a sign of continuing income.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 64.6% of samples (64.6% after cancellations)

## The Sun, Mars, Saturn or Rahu occupies the 12th house

- **Id:** `later_life.h12.expenses`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the 12th house is occupied by Surya, Kuja, Shani or Rahu
- **Text shown to users:** A natural malefic occupies the 12th house, traditionally a sign of expenses to plan for. (Ketu in the 12th is classically favourable for spiritual life and is not counted.)
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 30.2% of samples (30.2% after cancellations)

## A benefic occupies the 12th, or Jupiter aspects it

- **Id:** `later_life.h12.peaceful`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** any of: [the 12th house is occupied by benefics; the 12th house is aspected by Guru]
- **Text shown to users:** A benefic occupies or Jupiter aspects the 12th house, the house of rest and spiritual life: traditionally a peaceful retirement.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 45.9% of samples (45.9% after cancellations)

## The 4th lord is dignified or in a kendra or trikona

- **Id:** `later_life.home_comforts`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** any of: [the lord of the 4th is exalted, in moolatrikona or in its own sign; the lord of the 4th is in the 1st, 4th, 5th, 7th, 9th or 10th house]
- **Text shown to users:** The lord of the 4th house, home and comforts, is dignified or well placed: traditionally a sign of a settled home in later life.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 59.9% of samples (59.9% after cancellations)

## Jupiter is exalted, in moolatrikona or in its own sign

- **Id:** `later_life.jupiter.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** Guru is exalted, in moolatrikona or in its own sign
- **Text shown to users:** Jupiter is dignified, traditionally a sign of wisdom, respect and contentment in later years.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 25.9% of samples (25.9% after cancellations)

## Saturn is debilitated

- **Id:** `later_life.saturn.debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** Shani is debilitated
- **Text shown to users:** Saturn, which classically governs old age, is debilitated, unless the debilitation is cancelled: planning and routine help in later years.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 5.9% of samples (0.6% after cancellations)

## Saturn's debilitation is cancelled

- **Id:** `later_life.saturn.neecha_bhanga`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** Shani is debilitated with the debilitation cancelled (neecha bhanga)
- **Cancels:** `later_life.saturn.debilitated`
- **Text shown to users:** Saturn is debilitated, but a recognised condition cancels the debilitation (neecha bhanga).
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 5.3% of samples (5.3% after cancellations)

## Saturn is exalted, in moolatrikona or in its own sign

- **Id:** `later_life.saturn.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** Shani is exalted, in moolatrikona or in its own sign
- **Text shown to users:** Saturn, which classically governs old age, is dignified: traditionally a sign of steady, disciplined later years.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 31.2% of samples (31.2% after cancellations)

## Periods examined for later life

- **Id:** `later_life.timing.periods`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** always
- **Timing lords:** the lord of the 11th, the lord of the 4th, Shani, Guru
- **Text shown to users:** Dashas and bhuktis examined for later life: those of the 11th and 4th lords, Saturn and Jupiter.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 100.0% of samples (100.0% after cancellations)

