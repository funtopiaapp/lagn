//! Graha dignity: exaltation, moolatrikona, own sign, and relationship to the
//! sign lord.
//!
//! Specification: `docs/phase2/DESIGN.md` sections 4.1 and 4.2.
//! Rahu and Ketu have no dignity in Phase 2 (variant V-1).

use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

use crate::rasi::Rasi;
use crate::relationship::{compound_by_signs, CompoundRelation};
use crate::varga::sign_and_degree;

/// Nine-level dignity, best first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dignity {
    Exalted,
    Moolatrikona,
    OwnSign,
    GreatFriend,
    Friend,
    Neutral,
    Enemy,
    GreatEnemy,
    Debilitated,
}

impl Dignity {
    fn from_relation(r: CompoundRelation) -> Dignity {
        match r {
            CompoundRelation::GreatFriend => Dignity::GreatFriend,
            CompoundRelation::Friend => Dignity::Friend,
            CompoundRelation::Neutral => Dignity::Neutral,
            CompoundRelation::Enemy => Dignity::Enemy,
            CompoundRelation::GreatEnemy => Dignity::GreatEnemy,
        }
    }
}

/// Exaltation sign.
pub fn exaltation_sign(g: Graha) -> Option<Rasi> {
    Some(match g {
        Graha::Sun => Rasi::Mesha,
        Graha::Moon => Rasi::Vrishabha,
        Graha::Mars => Rasi::Makara,
        Graha::Mercury => Rasi::Kanya,
        Graha::Jupiter => Rasi::Karka,
        Graha::Venus => Rasi::Meena,
        Graha::Saturn => Rasi::Tula,
        Graha::Rahu | Graha::Ketu => return None,
    })
}

/// Debilitation sign: always the 7th from exaltation.
pub fn debilitation_sign(g: Graha) -> Option<Rasi> {
    exaltation_sign(g).map(|r| Rasi::from_index(r.index() as i32 + 6))
}

/// Sidereal longitude of the deepest exaltation point.
pub fn deep_exaltation_longitude(g: Graha) -> Option<f64> {
    let deg = match g {
        Graha::Sun => 10.0,
        Graha::Moon => 3.0,
        Graha::Mars => 28.0,
        Graha::Mercury => 15.0,
        Graha::Jupiter => 5.0,
        Graha::Venus => 27.0,
        Graha::Saturn => 20.0,
        Graha::Rahu | Graha::Ketu => return None,
    };
    Some(exaltation_sign(g)?.index() as f64 * 30.0 + deg)
}

/// Sidereal longitude of the deepest debilitation point.
pub fn deep_debilitation_longitude(g: Graha) -> Option<f64> {
    deep_exaltation_longitude(g).map(|l| lagn_ephem::norm360(l + 180.0))
}

/// Signs a graha rules. Empty for the nodes.
pub fn own_signs(g: Graha) -> &'static [Rasi] {
    match g {
        Graha::Sun => &[Rasi::Simha],
        Graha::Moon => &[Rasi::Karka],
        Graha::Mars => &[Rasi::Mesha, Rasi::Vrischika],
        Graha::Mercury => &[Rasi::Mithuna, Rasi::Kanya],
        Graha::Jupiter => &[Rasi::Dhanus, Rasi::Meena],
        Graha::Venus => &[Rasi::Vrishabha, Rasi::Tula],
        Graha::Saturn => &[Rasi::Makara, Rasi::Kumbha],
        Graha::Rahu | Graha::Ketu => &[],
    }
}

/// Moolatrikona sign and degree range `[from, to)` within it.
pub fn moolatrikona(g: Graha) -> Option<(Rasi, f64, f64)> {
    Some(match g {
        Graha::Sun => (Rasi::Simha, 0.0, 20.0),
        // V-8: BPHS (Santhanam) gives the rest of Vrishabha after the 3 degree
        // exaltation zone.
        Graha::Moon => (Rasi::Vrishabha, 3.0, 30.0),
        Graha::Mars => (Rasi::Mesha, 0.0, 12.0),
        Graha::Mercury => (Rasi::Kanya, 15.0, 20.0),
        Graha::Jupiter => (Rasi::Dhanus, 0.0, 10.0),
        Graha::Venus => (Rasi::Tula, 0.0, 15.0),
        Graha::Saturn => (Rasi::Kumbha, 0.0, 20.0),
        Graha::Rahu | Graha::Ketu => return None,
    })
}

/// Degree range `[0, to)` of the exaltation zone for the two grahas whose
/// exaltation sign is also their moolatrikona sign. Everyone else is exalted
/// throughout the exaltation sign.
fn exaltation_zone_end(g: Graha) -> f64 {
    match g {
        Graha::Moon => 3.0,
        Graha::Mercury => 15.0,
        _ => 30.0,
    }
}

/// The special dignity (exalted, moolatrikona, own, debilitated) of a graha at
/// a rasi-chart longitude, or `None` if it holds none and dignity falls to the
/// relationship with the sign lord.
pub fn rasi_zone(g: Graha, longitude: f64) -> Option<Dignity> {
    let (sign, deg) = sign_and_degree(longitude);
    if exaltation_sign(g)? == sign && deg < exaltation_zone_end(g) {
        return Some(Dignity::Exalted);
    }
    if debilitation_sign(g)? == sign {
        return Some(Dignity::Debilitated);
    }
    if let Some((mt_sign, from, to)) = moolatrikona(g) {
        if mt_sign == sign && deg >= from && deg < to {
            return Some(Dignity::Moolatrikona);
        }
    }
    if own_signs(g).contains(&sign) {
        return Some(Dignity::OwnSign);
    }
    None
}

/// How moolatrikona is treated in sign-only (divisional) charts. Variant V-4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VargaMoolatrikona {
    /// A graha in its moolatrikona sign is Moolatrikona.
    #[default]
    AsMoolatrikona,
    /// A graha in its moolatrikona sign is merely in its own sign.
    AsOwnSign,
}

/// The special dignity of a graha in a sign-only chart, or `None`.
pub fn varga_zone(g: Graha, sign: Rasi, mt: VargaMoolatrikona) -> Option<Dignity> {
    if exaltation_sign(g)? == sign {
        return Some(Dignity::Exalted);
    }
    if debilitation_sign(g)? == sign {
        return Some(Dignity::Debilitated);
    }
    if moolatrikona(g).map(|(s, _, _)| s) == Some(sign) {
        return Some(match mt {
            VargaMoolatrikona::AsMoolatrikona => Dignity::Moolatrikona,
            VargaMoolatrikona::AsOwnSign => Dignity::OwnSign,
        });
    }
    if own_signs(g).contains(&sign) {
        return Some(Dignity::OwnSign);
    }
    None
}

/// Full dignity given a special zone (if any) and the data needed to fall
/// back on the compound relationship with the lord of the occupied sign.
///
/// `occupied` is the sign the graha is in within the chart being judged.
/// `temp_self` and `temp_lord` are the signs the graha and the lord occupy in
/// whichever chart supplies the temporary relationship (variant V-5).
pub fn resolve(
    g: Graha,
    zone: Option<Dignity>,
    occupied: Rasi,
    temp_self: Rasi,
    temp_lord: Rasi,
) -> Option<Dignity> {
    if g.is_chhaya() {
        return None;
    }
    if let Some(z) = zone {
        return Some(z);
    }
    let lord = occupied.lord();
    // A graha in a sign it rules always has a zone, so lord != g here.
    compound_by_signs(g, lord, temp_self, temp_lord).map(Dignity::from_relation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relationship::SEVEN;

    #[test]
    fn debilitation_is_always_seventh_from_exaltation() {
        for g in SEVEN {
            let e = exaltation_sign(g).unwrap();
            assert_eq!(e.houses_to(debilitation_sign(g).unwrap()), 7, "{}", g.name());
        }
    }

    #[test]
    fn exaltation_and_debilitation_signs_match_the_specification() {
        use Rasi::*;
        let spec = [
            (Graha::Sun, Mesha, Tula), (Graha::Moon, Vrishabha, Vrischika),
            (Graha::Mars, Makara, Karka), (Graha::Mercury, Kanya, Meena),
            (Graha::Jupiter, Karka, Makara), (Graha::Venus, Meena, Kanya),
            (Graha::Saturn, Tula, Mesha),
        ];
        for (g, ex, de) in spec {
            assert_eq!(exaltation_sign(g), Some(ex), "{}", g.name());
            assert_eq!(debilitation_sign(g), Some(de), "{}", g.name());
        }
    }

    #[test]
    fn deep_points_are_exactly_opposite() {
        for g in SEVEN {
            let e = deep_exaltation_longitude(g).unwrap();
            let d = deep_debilitation_longitude(g).unwrap();
            assert!((lagn_ephem::norm360(d - e) - 180.0).abs() < 1e-12);
        }
        assert_eq!(deep_exaltation_longitude(Graha::Sun), Some(10.0));
        assert_eq!(deep_exaltation_longitude(Graha::Saturn), Some(200.0));
        assert_eq!(deep_exaltation_longitude(Graha::Venus), Some(357.0));
    }

    #[test]
    fn own_signs_agree_with_rasi_lordship() {
        for g in SEVEN {
            for r in Rasi::ALL {
                assert_eq!(own_signs(g).contains(&r), r.lord() == g, "{} {:?}", g.name(), r);
            }
        }
    }

    #[test]
    fn nodes_have_no_dignity() {
        for g in [Graha::Rahu, Graha::Ketu] {
            assert_eq!(exaltation_sign(g), None);
            assert!(own_signs(g).is_empty());
            assert_eq!(rasi_zone(g, 45.0), None);
            assert_eq!(resolve(g, None, Rasi::Mesha, Rasi::Mesha, Rasi::Mesha), None);
        }
    }

    #[test]
    fn moon_and_mercury_split_their_exaltation_sign_by_degree() {
        // Moon: Vrishabha 0-3 exalted, 3-30 moolatrikona.
        assert_eq!(rasi_zone(Graha::Moon, 30.0 + 2.999), Some(Dignity::Exalted));
        assert_eq!(rasi_zone(Graha::Moon, 30.0 + 3.0), Some(Dignity::Moolatrikona));
        assert_eq!(rasi_zone(Graha::Moon, 30.0 + 29.9), Some(Dignity::Moolatrikona));
        // Mercury: Kanya 0-15 exalted, 15-20 MT, 20-30 own.
        assert_eq!(rasi_zone(Graha::Mercury, 150.0 + 14.9), Some(Dignity::Exalted));
        assert_eq!(rasi_zone(Graha::Mercury, 150.0 + 15.0), Some(Dignity::Moolatrikona));
        assert_eq!(rasi_zone(Graha::Mercury, 150.0 + 19.9), Some(Dignity::Moolatrikona));
        assert_eq!(rasi_zone(Graha::Mercury, 150.0 + 20.0), Some(Dignity::OwnSign));
    }

    #[test]
    fn other_grahas_are_exalted_throughout_their_exaltation_sign() {
        for g in [Graha::Sun, Graha::Mars, Graha::Jupiter, Graha::Venus, Graha::Saturn] {
            let base = exaltation_sign(g).unwrap().index() as f64 * 30.0;
            for d in [0.0, 14.9, 29.99] {
                assert_eq!(rasi_zone(g, base + d), Some(Dignity::Exalted), "{} at {d}", g.name());
            }
        }
    }

    #[test]
    fn moolatrikona_and_own_split_the_moolatrikona_sign() {
        // (graha, sign start, mt_end): MT is [start, mt_end), own after.
        for (g, base, end) in [
            (Graha::Sun, 120.0, 20.0), (Graha::Mars, 0.0, 12.0),
            (Graha::Jupiter, 240.0, 10.0), (Graha::Venus, 180.0, 15.0),
            (Graha::Saturn, 300.0, 20.0),
        ] {
            assert_eq!(rasi_zone(g, base), Some(Dignity::Moolatrikona), "{}", g.name());
            assert_eq!(rasi_zone(g, base + end - 0.001), Some(Dignity::Moolatrikona), "{}", g.name());
            assert_eq!(rasi_zone(g, base + end), Some(Dignity::OwnSign), "{} at boundary", g.name());
            assert_eq!(rasi_zone(g, base + 29.99), Some(Dignity::OwnSign), "{}", g.name());
        }
    }

    #[test]
    fn varga_zone_is_sign_level() {
        assert_eq!(varga_zone(Graha::Moon, Rasi::Vrishabha, VargaMoolatrikona::AsMoolatrikona), Some(Dignity::Exalted));
        assert_eq!(varga_zone(Graha::Mercury, Rasi::Kanya, VargaMoolatrikona::AsMoolatrikona), Some(Dignity::Exalted));
        assert_eq!(varga_zone(Graha::Sun, Rasi::Simha, VargaMoolatrikona::AsMoolatrikona), Some(Dignity::Moolatrikona));
        assert_eq!(varga_zone(Graha::Sun, Rasi::Simha, VargaMoolatrikona::AsOwnSign), Some(Dignity::OwnSign));
        assert_eq!(varga_zone(Graha::Mars, Rasi::Vrischika, VargaMoolatrikona::AsMoolatrikona), Some(Dignity::OwnSign));
        assert_eq!(varga_zone(Graha::Saturn, Rasi::Mesha, VargaMoolatrikona::AsMoolatrikona), Some(Dignity::Debilitated));
        assert_eq!(varga_zone(Graha::Saturn, Rasi::Simha, VargaMoolatrikona::AsMoolatrikona), None);
    }

    #[test]
    fn resolve_falls_back_to_the_compound_relation_with_the_sign_lord() {
        // Sun in Vrishabha: lord Venus, a natural enemy of the Sun.
        // Venus in the 2nd from the Sun -> temporary friend -> Neutral.
        let d = resolve(Graha::Sun, None, Rasi::Vrishabha, Rasi::Vrishabha, Rasi::Mithuna);
        assert_eq!(d, Some(Dignity::Neutral));
        // Venus in the same sign as the Sun -> temporary enemy -> Great enemy.
        let d = resolve(Graha::Sun, None, Rasi::Vrishabha, Rasi::Vrishabha, Rasi::Vrishabha);
        assert_eq!(d, Some(Dignity::GreatEnemy));
        // A zone always wins over the relationship.
        let d = resolve(Graha::Sun, Some(Dignity::Exalted), Rasi::Mesha, Rasi::Mesha, Rasi::Tula);
        assert_eq!(d, Some(Dignity::Exalted));
    }
}
