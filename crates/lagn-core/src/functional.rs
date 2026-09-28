//! Functional nature of each graha for a lagna, neecha bhanga, and sambandha.
//! Specification: `docs/phase6/DESIGN.md` sections 3.1-3.3.

use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

use crate::analysis::DerivationSettings;
use crate::chart::Chart;
use crate::dignity::{self, Dignity};
use crate::drishti;
use crate::rasi::Rasi;
use crate::varga::Varga;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionalNature {
    /// Rules a kendra other than the 1st and a trikona.
    Yogakaraka,
    /// Rules a trikona (the lagna lord always).
    Benefic,
    /// Rules 3, 6, 8, 11 or 12 and no trikona.
    Malefic,
    /// Anything else: in practice, kendra-only lords.
    Neutral,
}

/// Houses (1..=12) a graha rules for a lagna, ascending. Empty for the nodes.
pub fn houses_ruled(lagna: Rasi, g: Graha) -> Vec<u8> {
    (1..=12u8)
        .filter(|&h| Rasi::from_index(lagna.index() as i32 + h as i32 - 1).lord() == g)
        .collect()
}

/// Functional nature for a lagna. `None` for Rahu and Ketu. Variant F-1.
pub fn functional_nature(lagna: Rasi, g: Graha) -> Option<FunctionalNature> {
    if g.is_chhaya() {
        return None;
    }
    let hs = houses_ruled(lagna, g);
    let has = |set: &[u8]| hs.iter().any(|h| set.contains(h));
    Some(if has(&[4, 7, 10]) && has(&[5, 9]) {
        FunctionalNature::Yogakaraka
    } else if has(&[1, 5, 9]) {
        FunctionalNature::Benefic
    } else if has(&[3, 6, 8, 11, 12]) {
        FunctionalNature::Malefic
    } else {
        FunctionalNature::Neutral
    })
}

/// How two grahas are related.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SambandhaKind {
    Conjunct,
    MutualAspect,
    Exchange,
}

impl Chart {
    /// Functional nature of a graha in this chart. `None` for the nodes.
    pub fn functional_nature(&self, g: Graha) -> Option<FunctionalNature> {
        functional_nature(self.lagna.rasi, g)
    }

    /// Every way two distinct grahas are related in D-1. Empty if none, or if
    /// `a == b`.
    pub fn sambandha(&self, a: Graha, b: Graha, s: &DerivationSettings) -> Vec<SambandhaKind> {
        if a == b {
            return Vec::new();
        }
        let (sa, sb) = (self.placement(a).rasi, self.placement(b).rasi);
        let mut out = Vec::new();
        if sa == sb {
            out.push(SambandhaKind::Conjunct);
        }
        if drishti::graha_aspects(a, sa, sb, s.node_aspects) && drishti::graha_aspects(b, sb, sa, s.node_aspects) {
            out.push(SambandhaKind::MutualAspect);
        }
        // Parivartana: each in a sign the other rules. The nodes rule nothing.
        if !a.is_chhaya() && !b.is_chhaya() && sa.lord() == b && sb.lord() == a {
            out.push(SambandhaKind::Exchange);
        }
        out
    }

    /// Neecha bhanga (variant N-1). `None` for the nodes; `Some(false)` when
    /// the graha isn't debilitated or no cancellation holds.
    pub fn neecha_bhanga(&self, g: Graha, s: &DerivationSettings) -> Option<bool> {
        let deb = dignity::debilitation_sign(g)?;
        let p = self.placement(g);
        if self.dignity(g, Varga::D1, s) != Some(Dignity::Debilitated) {
            return Some(false);
        }
        let moon = self.placement(Graha::Moon).rasi;
        let in_kendra = |x: Graha| {
            let r = self.placement(x).rasi;
            [1, 4, 7, 10].contains(&self.lagna.rasi.houses_to(r)) || [1, 4, 7, 10].contains(&moon.houses_to(r))
        };
        let deb_lord = deb.lord();
        let exalt_lord = dignity::exaltation_sign(g)?.lord();
        // 1. debilitation-sign lord in a kendra from the lagna or the Moon.
        let c1 = in_kendra(deb_lord);
        // 2. exaltation-sign lord in a kendra from the lagna or the Moon.
        let c2 = in_kendra(exalt_lord);
        // 3. conjunct or aspected by the debilitation-sign lord.
        let lord_sign = self.placement(deb_lord).rasi;
        let c3 = deb_lord != g
            && (lord_sign == p.rasi || drishti::graha_aspects(deb_lord, lord_sign, p.rasi, s.node_aspects));
        // 4. exalted in the navamsa.
        let c4 = dignity::exaltation_sign(g) == Some(p.navamsa);
        Some(c1 || c2 || c3 || c4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use FunctionalNature::*;

    #[test]
    fn the_classical_yogakarakas() {
        for (lagna, g) in [
            (Rasi::Vrishabha, Graha::Saturn), (Rasi::Tula, Graha::Saturn),
            (Rasi::Karka, Graha::Mars), (Rasi::Simha, Graha::Mars),
            (Rasi::Makara, Graha::Venus), (Rasi::Kumbha, Graha::Venus),
        ] {
            assert_eq!(functional_nature(lagna, g), Some(Yogakaraka), "{:?} lagna, {}", lagna, g.name());
        }
    }

    #[test]
    fn standard_cases_for_mesha_and_vrishabha() {
        let m = Rasi::Mesha;
        assert_eq!(functional_nature(m, Graha::Mars), Some(Benefic), "lagna lord, though it rules the 8th");
        assert_eq!(functional_nature(m, Graha::Saturn), Some(Malefic), "10 and 11");
        assert_eq!(functional_nature(m, Graha::Jupiter), Some(Benefic), "9 and 12");
        assert_eq!(functional_nature(m, Graha::Mercury), Some(Malefic), "3 and 6");
        assert_eq!(functional_nature(m, Graha::Venus), Some(Neutral), "2 and 7");
        assert_eq!(functional_nature(Rasi::Vrishabha, Graha::Jupiter), Some(Malefic), "8 and 11");
        assert_eq!(functional_nature(m, Graha::Rahu), None);
    }

    #[test]
    fn every_graha_rules_the_right_number_of_houses() {
        for l in Rasi::ALL {
            let total: usize = Graha::ALL.iter().map(|&g| houses_ruled(l, g).len()).sum();
            assert_eq!(total, 12);
            assert_eq!(houses_ruled(l, Graha::Sun).len(), 1);
            assert!(houses_ruled(l, Graha::Rahu).is_empty());
        }
    }
}
