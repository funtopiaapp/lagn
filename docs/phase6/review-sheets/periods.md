# Review sheet: periods

For the reviewing astrologer. Each rule below is exactly what the engine evaluates:
the "Condition" line is generated from the rule itself, not written separately.

To approve a rule, set `review.status` to `approved` and fill in `reviewer` and
`date` in the corpus file. To reject, set `rejected`. Please correct anything wrong:
the condition, the polarity (-3 to +3), the cancellations, or the wording.

Firing rates come from 1000 sample births in South India, 1920-2025, evaluated in review mode, each under all 81 mahadasha/antardasha lord pairs.

| Rules | 14 |
|---|---|
| Approved | 14 |
| Draft | 0 |
| Rejected | 0 |

## The antardasha lord is combust

- **Id:** `periods.antar.combust`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the antardasha lord is combust
- **Text shown to users:** The sub-period lord is combust, so its significations may be subdued.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 10.6% of samples (10.6% after cancellations)

## The antardasha lord is debilitated

- **Id:** `periods.antar.debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the antardasha lord is debilitated
- **Text shown to users:** The sub-period lord is debilitated, so its results may come with obstacles, unless the debilitation is cancelled.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 6.1% of samples (0.5% after cancellations); could not evaluate in 22.2%

## The antardasha lord sits in the 6th, 8th or 12th house

- **Id:** `periods.antar.dusthana`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the antardasha lord is in the 6th, 8th or 12th house
- **Text shown to users:** The sub-period lord sits in a dusthana, so health, debts, disputes, sudden changes or expenses can come forward.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 25.2% of samples (25.2% after cancellations)

## The antardasha lord is a functional benefic

- **Id:** `periods.antar.functional_benefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the antardasha lord is a functional benefic for this lagna
- **Text shown to users:** The sub-period lord rules a trikona for this lagna, classically supportive.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 27.8% of samples (27.8% after cancellations); could not evaluate in 22.2%

## The antardasha lord is a functional malefic for this lagna

- **Id:** `periods.antar.functional_malefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the antardasha lord is a functional malefic for this lagna
- **Text shown to users:** The sub-period lord rules difficult houses for this lagna (3, 6, 8, 11 or 12 without a trikona), so its matters tend to demand effort.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 31.7% of samples (31.7% after cancellations); could not evaluate in 22.2%

## The antardasha lord sits in a kendra or trikona

- **Id:** `periods.antar.kendra_trikona`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the antardasha lord is in the 1st, 4th, 5th, 7th, 9th or 10th house
- **Text shown to users:** The sub-period lord sits in a kendra or trikona, classically favourable.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 50.1% of samples (50.1% after cancellations)

## The antardasha lord's debilitation is cancelled

- **Id:** `periods.antar.neecha_bhanga`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** the antardasha lord is debilitated with the debilitation cancelled (neecha bhanga)
- **Cancels:** `periods.antar.debilitated`
- **Text shown to users:** The sub-period lord is debilitated, but a recognised condition cancels the debilitation.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 5.6% of samples (5.6% after cancellations); could not evaluate in 22.2%

## The antardasha lord is exalted, in moolatrikona or in its own sign

- **Id:** `periods.antar.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the antardasha lord is exalted, in moolatrikona or in its own sign
- **Text shown to users:** The sub-period lord is dignified and able to give its results fully.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 17.6% of samples (17.6% after cancellations); could not evaluate in 22.2%

## The antardasha lord is the yogakaraka

- **Id:** `periods.antar.yogakaraka`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the antardasha lord is the yogakaraka for this lagna
- **Text shown to users:** The sub-period lord is the yogakaraka for this lagna, classically among the most rewarding sub-periods.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 5.5% of samples (5.5% after cancellations); could not evaluate in 22.2%

## The antardasha lord is in a kendra or trikona from the mahadasha lord

- **Id:** `periods.harmony`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the antardasha lord is in the 1st, 4th, 5th, 7th, 9th or 10th house counted from the mahadasha lord
- **Text shown to users:** The two period lords stand in kendra or trikona from each other, a classically harmonious combination.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 55.3% of samples (55.3% after cancellations)

## Jupiter aspects the antardasha lord

- **Id:** `periods.jupiter_aspect`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** Guru aspects the antardasha lord
- **Text shown to users:** Jupiter aspects the sub-period lord, traditionally a protective influence through the period.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 19.7% of samples (19.7% after cancellations)

## The mahadasha lord is a functional malefic for this lagna

- **Id:** `periods.maha.functional_malefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the mahadasha lord is a functional malefic for this lagna
- **Text shown to users:** The major-period lord rules difficult houses for this lagna, colouring the whole period with effort and adjustment.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 31.7% of samples (31.7% after cancellations); could not evaluate in 22.2%

## The mahadasha lord is the yogakaraka

- **Id:** `periods.maha.yogakaraka`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the mahadasha lord is the yogakaraka for this lagna
- **Text shown to users:** The major-period lord is the yogakaraka for this lagna, supporting the whole period.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 5.5% of samples (5.5% after cancellations); could not evaluate in 22.2%

## The antardasha lord is 6th or 8th from the mahadasha lord

- **Id:** `periods.shashtashtaka`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the antardasha lord is in the 6th or 8th house counted from the mahadasha lord
- **Text shown to users:** The two period lords stand 6th and 8th from each other (shashtashtaka), a classically uneasy combination calling for patience.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 12.1% of samples (12.1% after cancellations)

