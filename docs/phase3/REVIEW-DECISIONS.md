# Review decisions: marriage corpus

**Reviewer:** Claude (AI), at the product owner's direction, 2026-09-21.
**Status:** applied to the corpus on 2026-09-21, after the product owner granted permission for AI-reviewed content.
**Not** a practising astrologer's review. The product owner decided an AI
review would stand in for one. This record exists so an astrologer can
re-review from it later: every decision says what changed and why.

**Standard applied.** Approve a rule only when it states a principle that is
widely taught in Parashari and Tamil practice, in terms the engine evaluates
faithfully. Change a rule when its logic double counts, is miscalibrated, or
contradicts a well-known exception. Reject it when the principle is too
disputed or too weak to show users. No textual citations were added:
`source.reference` stays null for every rule, because none could be checked
against a text.

## Rules

| Rule | Decision | Change | Reasoning |
|---|---|---|---|
| `h7.benefic_occupant` +2 | Approve | none | Benefics in the kalatra bhava as support is a core principle. |
| `h7.malefic_occupant` -2 | **Approve, changed** | set becomes Sun, Saturn, Rahu, Ketu | With the generic "malefics" set, Mars in the 7th scored -2 here *and* -2 as Chevvai dosha: one placement counted twice. Mars in the 7th is now judged only by the Chevvai rules, with their cancellations. The waning Moon drops out, a minor loss. |
| `h7.jupiter_aspect` +2 | Approve | none | Jupiter's aspect protecting a house is universally taught. |
| `h7.saturn_aspect` 0 | Approve | none | Delay indication, correctly left unscored. |
| `h7.saturn_occupant` 0 | Approve | none | As above. |
| `h7.node_occupant` 0 | Approve | none | Correctly unscored and flagged for examination. |
| `h2.malefic_occupant` -1 | **Reject** | none | Fires in 40% of charts, so it adds noise rather than signal. The 2nd house is secondary for marriage, and Mars there already counts under Chevvai dosha. |
| `l7.dusthana` -2 | Approve | none | Core principle. |
| `l7.kendra_trikona` +2 | Approve | none | Core principle. |
| `l7.strong` +2 | Approve | none | Core principle. |
| `l7.debilitated` -2 | Approve | none | Core principle. Neecha bhanga (cancelled debilitation) is not modelled yet, so this can overstate an afflicted lord whose debilitation is cancelled. Recorded as a known limitation. |
| `l7.combust` -1 | Approve | none | A lord within its combustion orb is classically weakened (asta). Half weight, as a lesser affliction than debilitation. |
| `l7.navamsa_strong` +1, `l7.navamsa_debilitated` -1 | Approve | none | D-9 confirmation of the lord at half weight is standard practice. |
| `venus.strong` +2, `venus.debilitated` -2, `venus.combust` -1 | Approve | none | Kalatra karaka dignity and combustion: core. Combustion at half weight, as for the 7th lord. |
| `venus.dusthana` -1 | **Approve, changed** | houses 6, 8 (12 removed) | Venus in the 12th, the bhava of bed comforts, is a well-known exception, traditionally held favourable. Penalising it contradicted that. |
| `venus.navamsa_*` ±1 | Approve | none | As for the 7th lord. |
| `venus.with_malefic` -1 | **Approve, changed** | Mars removed: Saturn, Rahu, Ketu | Venus with Mars is classically read as passion and intensity, not simple affliction. Venus with Saturn or the nodes is the affliction the rule means. |
| `jupiter.strong_female` +1, `jupiter.debilitated_female` -1 | Approve | none | Jupiter as husband's significator in a female chart is standard. The rules correctly evaluate to "unknown" without the native's sex. |
| `sav.h7_strong` +1 (28 or more) | Approve | none | The commonly taught "above 28 is good". |
| `sav.h7_weak` -1 (24 or fewer) | Approve | none | The commonly taught "below 25 is weak". Calibration note: this fires in 46% of sample charts, because the 7th house structurally gets fewer bindus than average. An astrologer may prefer a lower threshold. |
| `timing.periods` 0 | Approve | none | 7th lord, Venus and navamsa 7th lord periods: standard timing. |
| **new** `venus.karaka_in_7` 0 | **Add, approve** | new rule | *Karako bhava nashaya*: a karaka in its own bhava can harm it. Venus in the 7th scored +2 as a benefic occupant with no counterweight. Added as noted and unscored, because how strongly the dictum applies is debated. |

### Chevvai (Kuja) dosha

| Rule | Decision | Change | Reasoning |
|---|---|---|---|
| `kuja.from_lagna` -2 | Approve | none | Houses 2, 4, 7, 8, 12, the standard Tamil list (R-3). |
| `kuja.from_moon` | **Approve, changed** | -2 to -1 | Dosha from the Moon is recognised but weighted below the lagna in Tamil practice. At -2 each, one Mars could score -4 from two viewpoints. |
| `kuja.from_venus` | **Reject** | none | Venus is not a universal reference point (R-2), and adding a third viewpoint over-counts one placement. |
| `kuja.cancel.dignity` | Approve | none | Own sign or exaltation is a universally printed cancellation. |
| `kuja.cancel.jupiter` | Approve | none | Jupiter's conjunction or aspect is a universally printed cancellation. |
| `kuja.cancel.sign_exception_lagna`, `_moon` | Approve | none | The house-specific sign exceptions (2nd Mithuna/Kanya, 4th Mesha/Vrischika, 7th Karka/Makara, 8th Dhanus/Meena, 12th Vrishabha/Tula) as commonly printed. |
| `kuja.cancel.sign_exception_venus` | **Reject** | none | It cancels a rejected rule. |

Not added, and recommended for a later review: the Tamil practice that
Chevvai dosha in *both* partners cancels (dosha samyam), which belongs in
porutham matching. Neecha bhanga for the 7th lord and Venus also needs an
engine rule first.

## Poruthams

| Porutham | Decision | Reasoning |
|---|---|---|
| Dina, Gana, Mahendra, Stree Deergha, Yoni, Rasi, Rasi Adhipati, Rajju, Vedha | Approve | The tables and rules in DESIGN.md section 8 match the widely printed Tamil ten-porutham method, with the defaults in P-1 to P-6 and P-8. Rajju and Vedha are correctly reported as critical. |
| **Vasya** | **Reject** | I am not confident of the sign-to-sign Vasya table. Printed Tamil tables disagree on several entries (Thulam, Viruchigam, Makaram, Kumbam). A wrong table produces confident wrong verdicts, which is worse than none. Supply a table from the panchangam you trust and it can be added. |

## Variant decisions

| ID | Decision |
|---|---|
| V-1 to V-9 (Phase 2) | Keep all defaults. Each is the conservative or majority choice, and V-3 (no node aspects) keeps disputed Rahu/Ketu aspects out of scoring. |
| P-1 to P-6, P-8 | Keep defaults, as above. |
| P-7 (Vasya) | Rejected, as above. |
| R-1 (benefic class) | Keep: Mercury always benefic. The association rule is disputed, and with the Sun as a malefic it would make Mercury malefic in most charts. |
| R-2 | **Lagna and Moon**; Venus rejected. |
| R-3 | Houses 2, 4, 7, 8, 12. |
| R-4 | The cancellation list above. |
