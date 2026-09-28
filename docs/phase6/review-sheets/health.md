# Review sheet: health

For the reviewing astrologer. Each rule below is exactly what the engine evaluates:
the "Condition" line is generated from the rule itself, not written separately.

To approve a rule, set `review.status` to `approved` and fill in `reviewer` and
`date` in the corpus file. To reject, set `rejected`. Please correct anything wrong:
the condition, the polarity (-3 to +3), the cancellations, or the wording.

Firing rates come from 1000 sample births in South India, 1920-2025, evaluated in review mode.

| Rules | 21 |
|---|---|
| Approved | 21 |
| Draft | 0 |
| Rejected | 0 |

## A natural benefic occupies the lagna

- **Id:** `health.h1.benefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 1st house is occupied by benefics
- **Text shown to users:** A natural benefic sits in the lagna, traditionally a support for health.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 26.4% of samples (26.4% after cancellations)

## Mars, Saturn, Rahu or Ketu occupies the lagna

- **Id:** `health.h1.malefic`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the 1st house is occupied by Kuja, Shani, Rahu or Ketu
- **Text shown to users:** A natural malefic sits in the lagna, traditionally read as a need to look after health more deliberately.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 27.7% of samples (27.7% after cancellations)

## A natural malefic occupies the 6th house

- **Id:** `health.h6.malefics`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 6th house is occupied by Surya, Kuja, Shani, Rahu or Ketu
- **Text shown to users:** A natural malefic sits in the 6th house, which classical texts read as the ability to overcome illness and opposition.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 35.4% of samples (35.4% after cancellations)

## Jupiter aspects the lagna

- **Id:** `health.jupiter_aspects_lagna`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the 1st house is aspected by Guru
- **Text shown to users:** Jupiter aspects the lagna, traditionally a strong protection of health.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 24.0% of samples (24.0% after cancellations)

## The lagna lord is combust

- **Id:** `health.l1.combust`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the lord of the 1st is combust
- **Text shown to users:** The lord of the lagna is combust, too close to the Sun, traditionally read as lower resilience.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 16.5% of samples (16.5% after cancellations)

## The 1st (lagna) lord is debilitated

- **Id:** `health.l1.debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -2
- **Condition:** the lord of the 1st is debilitated
- **Text shown to users:** The lord of the 1st (lagna) house is debilitated, which classical texts read as difficulty with health and vitality, unless the debilitation is cancelled.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 7.3% of samples (0.1% after cancellations)

## The 1st (lagna) lord is in the 6th, 8th or 12th house

- **Id:** `health.l1.dusthana`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -2
- **Condition:** the lord of the 1st is in the 6th, 8th or 12th house
- **Text shown to users:** The lord of the 1st (lagna) house sits in a dusthana (the 6th, 8th or 12th), which classical texts read as strain on health and vitality.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 24.1% of samples (24.1% after cancellations)

## The 1st (lagna) lord is in a kendra or trikona

- **Id:** `health.l1.kendra_trikona`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the lord of the 1st is in the 1st, 4th, 5th, 7th, 9th or 10th house
- **Text shown to users:** The lord of the 1st (lagna) house sits in a kendra or trikona, which classical texts read as support for health and vitality.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 52.6% of samples (52.6% after cancellations)

## The 1st (lagna) lord's debilitation is cancelled

- **Id:** `health.l1.neecha_bhanga`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** the lord of the 1st is debilitated with the debilitation cancelled (neecha bhanga)
- **Cancels:** `health.l1.debilitated`
- **Text shown to users:** The lord of the 1st (lagna) house is debilitated, but a recognised condition cancels the debilitation (neecha bhanga).
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 7.2% of samples (7.2% after cancellations)

## The 1st (lagna) lord is exalted, in moolatrikona or in its own sign

- **Id:** `health.l1.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +2
- **Condition:** the lord of the 1st is exalted, in moolatrikona or in its own sign
- **Text shown to users:** The lord of the 1st (lagna) house is exalted, in moolatrikona or in its own sign, and able to give its results fully.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 23.7% of samples (23.7% after cancellations)

## The 6th lord is in the lagna

- **Id:** `health.l6_in_lagna`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the lord of the 6th is in the 1st house
- **Text shown to users:** The lord of the 6th house, the house of illness, sits in the lagna, traditionally a sign to be attentive to health.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 9.3% of samples (9.3% after cancellations)

## The 8th lord is in the lagna

- **Id:** `health.l8_in_lagna`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** the lord of the 8th is in the 1st house
- **Text shown to users:** The lord of the 8th house sits in the lagna, traditionally a sign to guard against strain and sudden setbacks to health.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 9.5% of samples (9.5% after cancellations)

## The Moon shares a sign with Saturn, Rahu or Ketu

- **Id:** `health.moon_afflicted`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** any of: [Chandra and Shani are in the same sign; Chandra and Rahu are in the same sign; Chandra and Ketu are in the same sign]
- **Text shown to users:** The Moon, significator of the mind, shares a sign with Saturn, Rahu or Ketu, traditionally read as a tendency to emotional strain; rest and routine help.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 23.0% of samples (23.0% after cancellations)

## The lagna has 28 or more SAV bindus

- **Id:** `health.sav.h1_strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** the 1st house has at least 28 SAV bindus
- **Text shown to users:** The lagna holds 28 or more Sarvashtakavarga bindus, a well-supported first house.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 63.7% of samples (63.7% after cancellations)

## The Sun is debilitated

- **Id:** `health.sun.debilitated`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** -1
- **Condition:** Surya is debilitated
- **Text shown to users:** The Sun, the karaka of vitality, is debilitated, unless the debilitation is cancelled.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 7.6% of samples (1.1% after cancellations)

## The Sun's debilitation is cancelled

- **Id:** `health.sun.neecha_bhanga`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** Surya is debilitated with the debilitation cancelled (neecha bhanga)
- **Cancels:** `health.sun.debilitated`
- **Text shown to users:** The Sun is debilitated, but a recognised condition cancels the debilitation (neecha bhanga).
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 6.5% of samples (6.5% after cancellations)

## The Sun is exalted, in moolatrikona or in its own sign

- **Id:** `health.sun.strong`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +1
- **Condition:** Surya is exalted, in moolatrikona or in its own sign
- **Text shown to users:** The Sun, the karaka of vitality, is exalted, in moolatrikona or in its own sign.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 15.8% of samples (15.8% after cancellations)

## Mars in the lagna: heat and minor injuries

- **Id:** `health.tendency.mars_lagna`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** Kuja is in the 1st house
- **Text shown to users:** Mars in the lagna is traditionally associated with heat, inflammation and minor cuts or injuries. Avoiding haste helps.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 8.3% of samples (8.3% after cancellations)

## Rahu or Ketu in the lagna: seek clear diagnosis

- **Id:** `health.tendency.nodes_lagna`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** the 1st house is occupied by Rahu or Ketu
- **Text shown to users:** Rahu or Ketu in the lagna is traditionally associated with complaints that are hard to pin down. A proper medical diagnosis matters more than guesswork.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 13.9% of samples (13.9% after cancellations)

## Saturn in the lagna: joints and posture

- **Id:** `health.tendency.saturn_lagna`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** Shani is in the 1st house
- **Text shown to users:** Saturn in the lagna is traditionally associated with stiffness in the joints and bones. Regular movement and good posture are worth attention.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 8.3% of samples (8.3% after cancellations)

## The Sun in the 6th, 8th or 12th: energy and eyes

- **Id:** `health.tendency.sun_dusthana`
- **Status:** Approved
- **Tradition:** Parashari
- **Polarity:** +0
- **Condition:** Surya is in the 6th, 8th or 12th house
- **Text shown to users:** The Sun in a dusthana is traditionally associated with fluctuating energy and attention to the eyes and heart. Regular check-ups are a sensible habit.
- **Source:** none supplied
- **Provenance note:** Authored from general knowledge of a commonly taught Parashari principle and reviewed by AI at the product owner's direction. No textual citation has been verified; an astrologer may re-review.
- **Fires in:** 23.7% of samples (23.7% after cancellations)

