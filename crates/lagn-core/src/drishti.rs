//! Drishti (aspects): Parashari graha drishti and Jaimini rasi drishti.
//!
//! Specification: `docs/phase2/DESIGN.md` section 5. Both are sign-based.
//! Degree-based sphuta drishti belongs to Shadbala (2C).

use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

use crate::rasi::{Mobility, Rasi};

/// Graha drishti cast by Rahu and Ketu. Variant V-3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeAspects {
    /// The nodes cast no graha drishti.
    #[default]
    None,
    /// The 7th only, like the other grahas.
    Seventh,
    /// 5th, 7th and 9th, like Jupiter.
    FiveSevenNine,
}

/// Houses a graha aspects, counted inclusively from its own sign.
pub fn aspect_houses(g: Graha, nodes: NodeAspects) -> &'static [u8] {
    match g {
        Graha::Sun | Graha::Moon | Graha::Mercury | Graha::Venus => &[7],
        Graha::Mars => &[4, 7, 8],
        Graha::Jupiter => &[5, 7, 9],
        Graha::Saturn => &[3, 7, 10],
        Graha::Rahu | Graha::Ketu => match nodes {
            NodeAspects::None => &[],
            NodeAspects::Seventh => &[7],
            NodeAspects::FiveSevenNine => &[5, 7, 9],
        },
    }
}

/// Does a graha in `from` cast graha drishti on the sign `target`?
///
/// A graha never aspects its own sign: conjunction is not an aspect.
pub fn graha_aspects(g: Graha, from: Rasi, target: Rasi, nodes: NodeAspects) -> bool {
    aspect_houses(g, nodes).contains(&from.houses_to(target))
}

/// Signs aspected by a graha in `from`.
pub fn signs_aspected(g: Graha, from: Rasi, nodes: NodeAspects) -> Vec<Rasi> {
    aspect_houses(g, nodes)
        .iter()
        .map(|&h| Rasi::from_index(from.index() as i32 + h as i32 - 1))
        .collect()
}

/// Jaimini rasi drishti between two signs.
///
/// - A chara sign aspects every sthira sign except the next sign.
/// - A sthira sign aspects every chara sign except the previous sign.
/// - A dvisvabhava sign aspects the other three dvisvabhava signs.
pub fn rasi_aspects(from: Rasi, target: Rasi) -> bool {
    if from == target {
        return false;
    }
    let next = Rasi::from_index(from.index() as i32 + 1);
    let prev = Rasi::from_index(from.index() as i32 - 1);
    match (from.mobility(), target.mobility()) {
        (Mobility::Chara, Mobility::Sthira) => target != next,
        (Mobility::Sthira, Mobility::Chara) => target != prev,
        (Mobility::Dvisvabhava, Mobility::Dvisvabhava) => true,
        _ => false,
    }
}

/// The three signs a sign aspects by rasi drishti.
pub fn rasi_aspected_by(from: Rasi) -> Vec<Rasi> {
    Rasi::ALL.into_iter().filter(|&t| rasi_aspects(from, t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aspect_house_table_matches_the_specification() {
        let n = NodeAspects::None;
        assert_eq!(aspect_houses(Graha::Sun, n), &[7]);
        assert_eq!(aspect_houses(Graha::Moon, n), &[7]);
        assert_eq!(aspect_houses(Graha::Mercury, n), &[7]);
        assert_eq!(aspect_houses(Graha::Venus, n), &[7]);
        assert_eq!(aspect_houses(Graha::Mars, n), &[4, 7, 8]);
        assert_eq!(aspect_houses(Graha::Jupiter, n), &[5, 7, 9]);
        assert_eq!(aspect_houses(Graha::Saturn, n), &[3, 7, 10]);
        assert!(aspect_houses(Graha::Rahu, n).is_empty());
        assert_eq!(aspect_houses(Graha::Ketu, NodeAspects::Seventh), &[7]);
        assert_eq!(aspect_houses(Graha::Rahu, NodeAspects::FiveSevenNine), &[5, 7, 9]);
    }

    #[test]
    fn every_graha_that_aspects_at_all_aspects_the_seventh() {
        for g in Graha::ALL {
            let h = aspect_houses(g, NodeAspects::Seventh);
            assert!(h.contains(&7), "{} misses the 7th", g.name());
        }
    }

    #[test]
    fn no_graha_aspects_its_own_sign() {
        for g in Graha::ALL {
            for r in Rasi::ALL {
                assert!(!graha_aspects(g, r, r, NodeAspects::FiveSevenNine));
            }
        }
    }

    #[test]
    fn saturn_in_mesha_aspects_mithuna_tula_makara() {
        let got = signs_aspected(Graha::Saturn, Rasi::Mesha, NodeAspects::None);
        assert_eq!(got, vec![Rasi::Mithuna, Rasi::Tula, Rasi::Makara]);
    }

    #[test]
    fn rasi_drishti_matches_the_worked_cases() {
        // Mesha (chara): sthira signs except Vrishabha.
        assert_eq!(rasi_aspected_by(Rasi::Mesha), vec![Rasi::Simha, Rasi::Vrischika, Rasi::Kumbha]);
        // Vrishabha (sthira): chara signs except Mesha.
        assert_eq!(rasi_aspected_by(Rasi::Vrishabha), vec![Rasi::Karka, Rasi::Tula, Rasi::Makara]);
        // Mithuna (dual): the other duals.
        assert_eq!(rasi_aspected_by(Rasi::Mithuna), vec![Rasi::Kanya, Rasi::Dhanus, Rasi::Meena]);
    }

    #[test]
    fn rasi_drishti_is_symmetric_and_always_three() {
        for a in Rasi::ALL {
            assert_eq!(rasi_aspected_by(a).len(), 3, "{:?}", a);
            for b in Rasi::ALL {
                assert_eq!(rasi_aspects(a, b), rasi_aspects(b, a), "{:?} {:?}", a, b);
            }
        }
    }
}
