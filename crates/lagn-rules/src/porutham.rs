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
}

impl PoruthamKind {
    pub const ALL: [PoruthamKind; 10] = [
        PoruthamKind::Dina, PoruthamKind::Gana, PoruthamKind::Mahendra,
        PoruthamKind::StreeDeergha, PoruthamKind::Yoni, PoruthamKind::Rasi,
        PoruthamKind::RasiAdhipati, PoruthamKind::Vasya, PoruthamKind::Rajju,
        PoruthamKind::Vedha,
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
        }
    }

    /// A failure of this porutham is reported as critical.
    pub fn is_critical(self) -> bool {
        matches!(self, PoruthamKind::Rajju | PoruthamKind::Vedha)
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
        mk(PoruthamKind::Vasya, None, "table not yet supplied by the reviewer (variant P-7)".into()),
        mk(PoruthamKind::Rajju, Some(rb != rg), format!("{rb:?} and {rg:?}")),
        mk(PoruthamKind::Vedha, Some(!vedha(bride.nakshatra, groom.nakshatra)), format!(
            "{} and {}", bride.nakshatra.name(), groom.nakshatra.name()
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
    fn vasya_is_always_not_evaluated_and_criticality_is_only_rajju_vedha() {
        for b in StarPos::all_padas().iter().step_by(7) {
            for g in StarPos::all_padas().iter().step_by(5) {
                for res in match_stars(*b, *g) {
                    if res.kind == PoruthamKind::Vasya {
                        assert_eq!(res.verdict, Verdict::NotEvaluated);
                    }
                    if res.critical {
                        assert!(res.kind.is_critical() && res.verdict == Verdict::NotMatching);
                    }
                }
            }
        }
    }
}
