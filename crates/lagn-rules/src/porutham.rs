//! The ten Poruthams (Tamil marriage matching).
//! Specification: `docs/phase3/DESIGN.md` section 8.
//!
//! Counts run from the bride's star or sign to the groom's, inclusive.
//! The result is informational: the procedure never recommends for or against
//! a marriage.

use lagn_core::relationship::{natural, NaturalRelation};
use lagn_core::{Nakshatra, Rasi};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PoruthamKind {
    Dina,
    Gana,
    Mahendra,
    StreeDeergha,
    Yoni,
    Rasi,
    RasiAdhipati,
    Vasya,
    Rajju,
    Vedha,
    /// The 12-porutham form, which Tamil and Kerala reports commonly print
    /// alongside the ten.
    Naadi,
    Varna,
}

impl PoruthamKind {
    pub const ALL: [PoruthamKind; 12] = [
        PoruthamKind::Dina, PoruthamKind::Gana, PoruthamKind::Mahendra,
        PoruthamKind::StreeDeergha, PoruthamKind::Yoni, PoruthamKind::Rasi,
        PoruthamKind::RasiAdhipati, PoruthamKind::Vasya, PoruthamKind::Rajju,
        PoruthamKind::Vedha, PoruthamKind::Naadi, PoruthamKind::Varna,
    ];

    pub fn name(self) -> &'static str {
        match self {
            PoruthamKind::Dina => "Dina",
            PoruthamKind::Gana => "Gana",
            PoruthamKind::Mahendra => "Mahendra",
            PoruthamKind::StreeDeergha => "Stree Deergha",
            PoruthamKind::Yoni => "Yoni",
            PoruthamKind::Rasi => "Rasi",
            PoruthamKind::RasiAdhipati => "Rasi Adhipati",
            PoruthamKind::Vasya => "Vasya",
            PoruthamKind::Rajju => "Rajju",
            PoruthamKind::Vedha => "Vedha",
            PoruthamKind::Naadi => "Naadi",
            PoruthamKind::Varna => "Varna",
        }
    }

    /// A failure of this porutham is reported as critical.
    pub fn is_critical(self) -> bool {
        // Naadi joins these two: the texts weigh it as heavily, and its
        // classical cancellations are not computed here, so a mismatch should
        // reach an astrologer rather than be averaged into a count.
        matches!(self, PoruthamKind::Rajju | PoruthamKind::Vedha | PoruthamKind::Naadi)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Matching,
    NotMatching,
    NotEvaluated,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoruthamResult {
    pub kind: PoruthamKind,
    pub verdict: Verdict,
    /// True only for a NotMatching Rajju or Vedha.
    pub critical: bool,
    pub detail: String,
}

/// The Moon's star and sign for one person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StarPos {
    pub nakshatra: Nakshatra,
    pub rasi: Rasi,
}

impl StarPos {
    /// All 108 valid (nakshatra, rasi) combinations, one per pada. Absolute
    /// pada `i` lies in nakshatra `i / 4` (four padas per star) and in rasi
    /// `i / 9` (nine padas per sign).
    pub fn all_padas() -> Vec<StarPos> {
        (0..108)
            .map(|i| StarPos { nakshatra: Nakshatra::from_index(i / 4), rasi: Rasi::from_index(i / 9) })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tables - DESIGN.md section 8.2
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gana {
    Deva,
    Manushya,
    Rakshasa,
}

pub fn gana(n: Nakshatra) -> Gana {
    use Nakshatra::*;
    match n {
        Ashwini | Mrigashira | Punarvasu | Pushya | Hasta | Swati | Anuradha | Shravana | Revati => Gana::Deva,
        Bharani | Rohini | Ardra | PurvaPhalguni | UttaraPhalguni | PurvaAshadha | UttaraAshadha
        | PurvaBhadrapada | UttaraBhadrapada => Gana::Manushya,
        Krittika | Ashlesha | Magha | Chitra | Vishakha | Jyeshtha | Mula | Dhanishta | Shatabhisha => {
            Gana::Rakshasa
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Yoni {
    Horse, Elephant, Sheep, Serpent, Dog, Cat, Rat, Cow, Buffalo, Tiger, Deer, Monkey, Mongoose, Lion,
}

pub fn yoni(n: Nakshatra) -> Yoni {
    use Nakshatra::*;
    match n {
        Ashwini | Shatabhisha => Yoni::Horse,
        Bharani | Revati => Yoni::Elephant,
        Krittika | Pushya => Yoni::Sheep,
        Rohini | Mrigashira => Yoni::Serpent,
        Ardra | Mula => Yoni::Dog,
        Punarvasu | Ashlesha => Yoni::Cat,
        Magha | PurvaPhalguni => Yoni::Rat,
        UttaraPhalguni | UttaraBhadrapada => Yoni::Cow,
        Hasta | Swati => Yoni::Buffalo,
        Chitra | Vishakha => Yoni::Tiger,
        Anuradha | Jyeshtha => Yoni::Deer,
        PurvaAshadha | Shravana => Yoni::Monkey,
        UttaraAshadha => Yoni::Mongoose,
        Dhanishta | PurvaBhadrapada => Yoni::Lion,
    }
}

pub fn yoni_enemies(a: Yoni, b: Yoni) -> bool {
    use Yoni::*;
    let pair = |x, y| (a == x && b == y) || (a == y && b == x);
    pair(Horse, Buffalo) || pair(Elephant, Lion) || pair(Sheep, Monkey) || pair(Serpent, Mongoose)
        || pair(Dog, Deer) || pair(Cat, Rat) || pair(Cow, Tiger)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rajju {
    Pada,
    Kati,
    Nabhi,
    Kanta,
    Siro,
}

pub fn rajju(n: Nakshatra) -> Rajju {
    use Nakshatra::*;
    match n {
        Ashwini | Ashlesha | Magha | Jyeshtha | Mula | Revati => Rajju::Pada,
        Bharani | Pushya | PurvaPhalguni | Anuradha | PurvaAshadha | UttaraBhadrapada => Rajju::Kati,
        Krittika | Punarvasu | UttaraPhalguni | Vishakha | UttaraAshadha | PurvaBhadrapada => Rajju::Nabhi,
        Rohini | Ardra | Hasta | Swati | Shravana | Shatabhisha => Rajju::Kanta,
        Mrigashira | Chitra | Dhanishta => Rajju::Siro,
    }
}

/// The signs each rasi holds sway over, for Vasya porutham.
///
/// Phase 3 rejected this table rather than guess it: printed Tamil tables
/// agree on eight rows and disagree on four - Tula, Vrischika, Makara and
/// Kumbha. It is built now at the product owner's direction, using the
/// most commonly printed reading of each disputed row, and
/// [`vasya_disputed`] marks the four so a verdict that rests on one can say
/// so instead of sounding as settled as the other six.
///
/// Rows the sources agree on:
///   Mesha -> Simha, Vrischika        Vrishabha -> Karka, Tula
///   Mithuna -> Kanya                 Karka -> Vrischika, Dhanus
///   Simha -> Tula                    Kanya -> Mithuna, Meena
///   Dhanus -> Meena                  Meena -> Makara
///
/// Rows taken on the majority reading, with the variant noted:
///   Tula -> Makara, Kanya   (some print Makara alone)
///   Vrischika -> Kanya, Karka   (some print Karka alone)
///   Makara -> Mesha, Kumbha   (some print Kumbha alone)
///   Kumbha -> Mesha   (some add Makara)
pub fn vasya_signs(r: Rasi) -> &'static [Rasi] {
    use Rasi::*;
    match r {
        Mesha => &[Simha, Vrischika],
        Vrishabha => &[Karka, Tula],
        Mithuna => &[Kanya],
        Karka => &[Vrischika, Dhanus],
        Simha => &[Tula],
        Kanya => &[Mithuna, Meena],
        Tula => &[Makara, Kanya],
        Vrischika => &[Kanya, Karka],
        Dhanus => &[Meena],
        Makara => &[Mesha, Kumbha],
        Kumbha => &[Mesha],
        Meena => &[Makara],
    }
}

/// True for the four rows printed tables disagree on. A verdict that turns on
/// one of these is reported as resting on a disputed row.
pub fn vasya_disputed(r: Rasi) -> bool {
    matches!(r, Rasi::Tula | Rasi::Vrischika | Rasi::Makara | Rasi::Kumbha)
}

/// Vasya porutham: the bride's rasi falls under the sway of the groom's.
///
/// The direction is the one Tamil sources most often print - the groom's sign
/// holds sway - rather than a mutual test.
pub fn vasya(bride: Rasi, groom: Rasi) -> bool {
    vasya_signs(groom).contains(&bride)
}

/// The three naadis, by the humour each nakshatra is assigned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Naadi {
    Adi,
    Madhya,
    Antya,
}

/// Which naadi a nakshatra belongs to.
///
/// The assignment runs 1-2-3-3-2-1 along the twenty-seven, which is why it is
/// quoted as a zig-zag rather than a repeating triple.
pub fn naadi(n: Nakshatra) -> Naadi {
    use Nakshatra::*;
    match n {
        Ashwini | Ardra | Punarvasu | UttaraPhalguni | Hasta | Jyeshtha | Mula | Shatabhisha
        | PurvaBhadrapada => Naadi::Adi,
        Bharani | Mrigashira | Pushya | PurvaPhalguni | Chitra | Anuradha | PurvaAshadha
        | Dhanishta | UttaraBhadrapada => Naadi::Madhya,
        Krittika | Rohini | Ashlesha | Magha | Swati | Vishakha | UttaraAshadha | Shravana
        | Revati => Naadi::Antya,
    }
}

/// The four varnas, by the element of the moon-sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Varna {
    Shudra,
    Vaishya,
    Kshatriya,
    Brahmin,
}

impl Varna {
    pub fn name(self) -> &'static str {
        match self {
            Varna::Shudra => "Shudra",
            Varna::Vaishya => "Vaishya",
            Varna::Kshatriya => "Kshatriya",
            Varna::Brahmin => "Brahmin",
        }
    }
}

/// The varna of a moon-sign: water is Brahmin, fire Kshatriya, earth Vaishya
/// and air Shudra.
pub fn varna(r: Rasi) -> Varna {
    use Rasi::*;
    match r {
        Karka | Vrischika | Meena => Varna::Brahmin,
        Mesha | Simha | Dhanus => Varna::Kshatriya,
        Vrishabha | Kanya | Makara => Varna::Vaishya,
        Mithuna | Tula | Kumbha => Varna::Shudra,
    }
}

pub fn vedha(a: Nakshatra, b: Nakshatra) -> bool {
    use Nakshatra::*;
    const PAIRS: [(Nakshatra, Nakshatra); 15] = [
        (Ashwini, Jyeshtha), (Bharani, Anuradha), (Krittika, Vishakha), (Rohini, Swati),
        (Ardra, Shravana), (Punarvasu, UttaraAshadha), (Pushya, PurvaAshadha), (Ashlesha, Mula),
        (Magha, Revati), (PurvaPhalguni, UttaraBhadrapada), (UttaraPhalguni, PurvaBhadrapada),
        (Hasta, Shatabhisha), (Mrigashira, Chitra), (Mrigashira, Dhanishta), (Chitra, Dhanishta),
    ];
    PAIRS.iter().any(|&(x, y)| (a == x && b == y) || (a == y && b == x))
}

// ---------------------------------------------------------------------------
// The procedure
// ---------------------------------------------------------------------------

/// Inclusive count from the bride's nakshatra to the groom's, 1..=27.
pub fn star_count(bride: Nakshatra, groom: Nakshatra) -> u8 {
    (groom.index() as i32 - bride.index() as i32).rem_euclid(27) as u8 + 1
}

/// All ten poruthams, in fixed order.
pub fn match_stars(bride: StarPos, groom: StarPos) -> Vec<PoruthamResult> {
    let n = star_count(bride.nakshatra, groom.nakshatra);
    let r = bride.rasi.houses_to(groom.rasi);
    let mk = |kind: PoruthamKind, ok: Option<bool>, detail: String| {
        let verdict = match ok {
            Some(true) => Verdict::Matching,
            Some(false) => Verdict::NotMatching,
            None => Verdict::NotEvaluated,
        };
        PoruthamResult { kind, verdict, critical: kind.is_critical() && verdict == Verdict::NotMatching, detail }
    };

    let (gb, gg) = (gana(bride.nakshatra), gana(groom.nakshatra));
    let gana_ok = gb == gg
        || matches!((gb, gg), (Gana::Deva, Gana::Manushya) | (Gana::Manushya, Gana::Deva));
    let (yb, yg) = (yoni(bride.nakshatra), yoni(groom.nakshatra));
    let (lb, lg) = (bride.rasi.lord(), groom.rasi.lord());
    let adhipati_ok = lb == lg
        || (natural(lb, lg) != Some(NaturalRelation::Enemy)
            && natural(lg, lb) != Some(NaturalRelation::Enemy));
    let (rb, rg) = (rajju(bride.nakshatra), rajju(groom.nakshatra));

    vec![
        mk(PoruthamKind::Dina, Some(matches!(n % 9, 0 | 2 | 4 | 6 | 8)), format!("star count {n}")),
        mk(PoruthamKind::Gana, Some(gana_ok), format!("{gb:?} and {gg:?}")),
        mk(PoruthamKind::Mahendra, Some(matches!(n, 4 | 7 | 10 | 13 | 16 | 19 | 22 | 25)), format!("star count {n}")),
        mk(PoruthamKind::StreeDeergha, Some(n > 13), format!("star count {n}")),
        mk(PoruthamKind::Yoni, Some(!yoni_enemies(yb, yg)), format!("{yb:?} and {yg:?}")),
        mk(PoruthamKind::Rasi, Some(!matches!(r, 2 | 6 | 8 | 12)), format!("sign count {r}")),
        mk(PoruthamKind::RasiAdhipati, Some(adhipati_ok), format!("lords {} and {}", lb.name(), lg.name())),
        mk(PoruthamKind::Vasya, Some(vasya(bride.rasi, groom.rasi)), {
            let note = if vasya_disputed(groom.rasi) {
                " - printed tables disagree on this row, so this one verdict is less settled than the rest"
            } else {
                ""
            };
            format!("{:?} under {:?}{note}", bride.rasi, groom.rasi)
        }),
        mk(PoruthamKind::Rajju, Some(rb != rg), format!("{rb:?} and {rg:?}")),
        mk(PoruthamKind::Vedha, Some(!vedha(bride.nakshatra, groom.nakshatra)), format!(
            "{} and {}", bride.nakshatra.name(), groom.nakshatra.name()
        )),
        // Sharing a naadi is what the texts object to, as with Rajju.
        mk(PoruthamKind::Naadi, Some(naadi(bride.nakshatra) != naadi(groom.nakshatra)), format!(
            "{:?} and {:?}", naadi(bride.nakshatra), naadi(groom.nakshatra)
        )),
        // The groom's varna should not stand below the bride's.
        mk(PoruthamKind::Varna, Some(varna(groom.rasi) >= varna(bride.rasi)), format!(
            "{} and {}", varna(bride.rasi).name(), varna(groom.rasi).name()
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gana_groups_have_nine_each() {
        for g in [Gana::Deva, Gana::Manushya, Gana::Rakshasa] {
            assert_eq!(Nakshatra::ALL.iter().filter(|&&n| gana(n) == g).count(), 9, "{g:?}");
        }
    }

    #[test]
    fn rajju_follows_the_zigzag() {
        const ORDER: [Rajju; 5] = [Rajju::Pada, Rajju::Kati, Rajju::Nabhi, Rajju::Kanta, Rajju::Siro];
        const ZIG: [usize; 9] = [0, 1, 2, 3, 4, 3, 2, 1, 0];
        for (i, n) in Nakshatra::ALL.iter().enumerate() {
            assert_eq!(rajju(*n), ORDER[ZIG[i % 9]], "{}", n.name());
        }
    }

    #[test]
    fn vedha_is_symmetric_irreflexive_and_covers_every_star() {
        for a in Nakshatra::ALL {
            assert!(!vedha(a, a));
            assert!(Nakshatra::ALL.iter().any(|&b| vedha(a, b)), "{} has no vedha partner", a.name());
            for b in Nakshatra::ALL {
                assert_eq!(vedha(a, b), vedha(b, a));
            }
        }
    }

    #[test]
    fn yoni_enemies_are_symmetric_and_never_self() {
        use Yoni::*;
        let all = [Horse, Elephant, Sheep, Serpent, Dog, Cat, Rat, Cow, Buffalo, Tiger, Deer, Monkey, Mongoose, Lion];
        for a in all {
            assert!(!yoni_enemies(a, a));
            for b in all {
                assert_eq!(yoni_enemies(a, b), yoni_enemies(b, a));
            }
        }
        // Every animal except none has exactly one enemy.
        for a in all {
            assert_eq!(all.iter().filter(|&&b| yoni_enemies(a, b)).count(), 1, "{a:?}");
        }
    }

    #[test]
    fn all_padas_are_consistent() {
        let padas = StarPos::all_padas();
        assert_eq!(padas.len(), 108);
        for (i, p) in padas.iter().enumerate() {
            // Midpoint of the pada must reproduce both fields.
            let lon = (i as f64 + 0.5) * (360.0 / 108.0);
            assert_eq!(p.nakshatra, Nakshatra::from_longitude(lon));
            assert_eq!(p.rasi, Rasi::from_longitude(lon));
        }
    }

    #[test]
    fn star_count_is_inclusive() {
        assert_eq!(star_count(Nakshatra::Ashwini, Nakshatra::Ashwini), 1);
        assert_eq!(star_count(Nakshatra::Ashwini, Nakshatra::Bharani), 2);
        assert_eq!(star_count(Nakshatra::Bharani, Nakshatra::Ashwini), 27);
    }

    #[test]
    fn every_porutham_reaches_a_verdict_and_only_rajju_and_vedha_are_critical() {
        for b in StarPos::all_padas().iter().step_by(7) {
            for g in StarPos::all_padas().iter().step_by(5) {
                for res in match_stars(*b, *g) {
                    // Vasya was blocked until its table was supplied; all ten
                    // are evaluated now, so none may come back undecided.
                    assert_ne!(res.verdict, Verdict::NotEvaluated, "{:?} is undecided", res.kind);
                    if res.critical {
                        assert!(res.kind.is_critical() && res.verdict == Verdict::NotMatching);
                    }
                }
            }
        }
    }

    #[test]
    fn the_vasya_table_is_total_and_marks_what_is_disputed() {
        use lagn_core::Rasi::*;

        // Every sign has a row, and no row is empty: a missing row would make
        // Vasya silently unmatchable for that sign.
        for r in [Mesha, Vrishabha, Mithuna, Karka, Simha, Kanya, Tula, Vrischika, Dhanus, Makara, Kumbha, Meena] {
            assert!(!vasya_signs(r).is_empty(), "{r:?} has no vasya row");
            // No sign holds sway over itself; the table is about one sign over
            // another.
            assert!(!vasya_signs(r).contains(&r), "{r:?} is vasya to itself");
        }

        // Exactly the four rows Phase 3 found the sources disagreeing on.
        let disputed: Vec<_> = [Mesha, Vrishabha, Mithuna, Karka, Simha, Kanya, Tula, Vrischika, Dhanus, Makara, Kumbha, Meena]
            .into_iter().filter(|r| vasya_disputed(*r)).collect();
        assert_eq!(disputed, vec![Tula, Vrischika, Makara, Kumbha]);

        // The eight agreed rows, as printed.
        assert_eq!(vasya_signs(Mesha), &[Simha, Vrischika]);
        assert_eq!(vasya_signs(Vrishabha), &[Karka, Tula]);
        assert_eq!(vasya_signs(Mithuna), &[Kanya]);
        assert_eq!(vasya_signs(Karka), &[Vrischika, Dhanus]);
        assert_eq!(vasya_signs(Simha), &[Tula]);
        assert_eq!(vasya_signs(Kanya), &[Mithuna, Meena]);
        assert_eq!(vasya_signs(Dhanus), &[Meena]);
        assert_eq!(vasya_signs(Meena), &[Makara]);
    }

    #[test]
    fn vasya_is_directional_not_mutual() {
        use lagn_core::Rasi::*;
        // Simha falls under Mesha's sway, so a Simha bride with a Mesha groom
        // matches - and the reverse does not, because the test is not mutual.
        assert!(vasya(Simha, Mesha));
        assert!(!vasya(Mesha, Simha));
        // Meena under Dhanus matches; Dhanus under Meena does not.
        assert!(vasya(Meena, Dhanus));
        assert!(!vasya(Dhanus, Meena));
        // Some pairs are vasya both ways, and that is the table, not a bug:
        // Mithuna and Kanya each appear in the other's row.
        assert!(vasya(Mithuna, Kanya) && vasya(Kanya, Mithuna));
    }
}

// ---------------------------------------------------------------------------
// Papasamyam and dasa sandhi
//
// Both need whole charts rather than the two stars the ten poruthams work
// from, and both are printed in commercial South Indian matching reports
// alongside the poruthams.
// ---------------------------------------------------------------------------

use lagn_core::{Chart, Graha};

/// The houses a malefic is counted in, from each reference point.
const PAPA_HOUSES: [u8; 6] = [1, 2, 4, 7, 8, 12];

/// The grahas counted as papa. Classical lists vary at the edges - some count
/// a waning Moon, some drop Ketu - so the five everyone agrees on are counted
/// and the variance is stated rather than hidden.
const PAPA_GRAHAS: [Graha; 5] = [Graha::Sun, Graha::Mars, Graha::Saturn, Graha::Rahu, Graha::Ketu];

/// One reference point's count, and which grahas made it up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PapaFrom {
    /// "the lagna", "the Moon" or "Venus". Owned, because the report it goes
    /// into is deserialised as well as serialised.
    pub from: String,
    pub count: u8,
    /// The grahas that fell in a counted house, with the house.
    pub placements: Vec<(Graha, u8)>,
}

/// Papasamyam for one chart: the papa count from the lagna, the Moon and
/// Venus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Papa {
    pub from: Vec<PapaFrom>,
    pub total: u8,
}

/// Count the papa placements in a chart.
///
/// Weighted variants exist - half a point for some placements, a point for
/// others, and the weights differ between sources - so this counts placements
/// rather than inventing a weighting. The comparison below is of like with
/// like, which is what the check is for.
pub fn papasamyam(chart: &Chart) -> Papa {
    let lagna = chart.lagna.rasi;
    let houses_from = |origin: Rasi| -> Vec<(Graha, u8)> {
        let mut out: Vec<(Graha, u8)> = PAPA_GRAHAS
            .into_iter()
            .filter_map(|g| {
                let h = origin.houses_to(chart.placement(g).rasi);
                PAPA_HOUSES.contains(&h).then_some((g, h))
            })
            .collect();
        out.sort_by_key(|(g, _)| *g);
        out
    };

    let mut from = Vec::new();
    for (label, origin) in [
        ("the lagna", lagna),
        ("the Moon", chart.placement(Graha::Moon).rasi),
        ("Venus", chart.placement(Graha::Venus).rasi),
    ] {
        let placements = houses_from(origin);
        from.push(PapaFrom { from: label.to_string(), count: placements.len() as u8, placements });
    }
    let total = from.iter().map(|f| f.count).sum();
    Papa { from, total }
}

/// Papasamyam compared between the two charts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PapaComparison {
    pub bride: Papa,
    pub groom: Papa,
    /// True when the groom's count is not below the bride's, which is what the
    /// check asks for.
    pub balanced: bool,
    pub difference: i16,
}

pub fn compare_papa(bride: &Chart, groom: &Chart) -> PapaComparison {
    let (b, g) = (papasamyam(bride), papasamyam(groom));
    let difference = g.total as i16 - b.total as i16;
    PapaComparison { balanced: difference >= 0, difference, bride: b, groom: g }
}

/// One coincidence of mahadasha changes between the two charts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DasaSandhi {
    /// Julian Day of the bride's change, and the lords either side of it.
    pub bride_jd: f64,
    pub bride_from: Graha,
    pub bride_to: Graha,
    pub groom_jd: f64,
    pub groom_from: Graha,
    pub groom_to: Graha,
    /// How far apart the two changes fall, in months.
    pub months_apart: f64,
}

/// Mahadasha changes in the two charts that fall within `within_months` of
/// each other.
///
/// The check asks whether both people change major period at about the same
/// time: the texts read two simultaneous turnings as a strain on a new
/// household. Eighteen months is the window Tamil reports commonly use.
pub fn dasa_sandhi(bride: &Chart, groom: &Chart, within_months: f64) -> Vec<DasaSandhi> {
    use lagn_core::Vimshottari;
    let changes = |c: &Chart| -> Vec<(f64, Graha, Graha)> {
        let v = Vimshottari::compute(c);
        v.mahadashas
            .windows(2)
            .map(|w| (w[1].start_jd, w[0].lord, w[1].lord))
            .collect()
    };
    let (bc, gc) = (changes(bride), changes(groom));
    // A month of the same length the dasha uses, so the window is consistent
    // with everything else the engine counts.
    let month = 365.25 / 12.0;
    let mut out = Vec::new();
    for (bjd, bf, bt) in &bc {
        for (gjd, gf, gt) in &gc {
            let months = (bjd - gjd).abs() / month;
            if months <= within_months {
                out.push(DasaSandhi {
                    bride_jd: *bjd, bride_from: *bf, bride_to: *bt,
                    groom_jd: *gjd, groom_from: *gf, groom_to: *gt,
                    months_apart: months,
                });
            }
        }
    }
    out.sort_by(|a, b| a.bride_jd.partial_cmp(&b.bride_jd).expect("finite"));
    out
}
