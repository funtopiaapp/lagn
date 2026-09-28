//! Graha conditions: Baladi and Jagradadi avasthas, combustion, graha yuddha.
//!
//! Specification: `docs/phase2/DESIGN.md` section 6.

use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

use crate::dignity::Dignity;
use crate::varga::sign_and_degree;

/// Baladi avastha - age state by degrees within the sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaladiAvastha {
    /// Infant.
    Bala,
    /// Youth.
    Kumara,
    /// Adult - full results.
    Yuva,
    /// Old.
    Vriddha,
    /// Dead - no results.
    Mrita,
}

/// Baladi avastha at a longitude. Six-degree bands, `[a, b)` (V-9); the order
/// reverses in even signs.
pub fn baladi(longitude: f64) -> BaladiAvastha {
    use BaladiAvastha::*;
    const ORDER: [BaladiAvastha; 5] = [Bala, Kumara, Yuva, Vriddha, Mrita];
    let (sign, deg) = sign_and_degree(longitude);
    let band = ((deg / 6.0).floor() as usize).min(4);
    if sign.is_odd() { ORDER[band] } else { ORDER[4 - band] }
}

/// Jagradadi avastha - wakefulness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JagradadiAvastha {
    /// Awake - full results.
    Jagrat,
    /// Dreaming - middling results.
    Swapna,
    /// Deep sleep - no results.
    Sushupti,
}

/// Jagradadi from a nine-level dignity (V-6 default: compound relationship).
pub fn jagradadi(d: Dignity) -> JagradadiAvastha {
    use Dignity::*;
    match d {
        Exalted | Moolatrikona | OwnSign => JagradadiAvastha::Jagrat,
        GreatFriend | Friend | Neutral => JagradadiAvastha::Swapna,
        Enemy | GreatEnemy | Debilitated => JagradadiAvastha::Sushupti,
    }
}

/// Combustion orb in degrees, or `None` for grahas that are never combust
/// (the Sun itself, Rahu, Ketu). V-7.
pub fn combustion_orb(g: Graha, retrograde: bool) -> Option<f64> {
    Some(match g {
        Graha::Moon => 12.0,
        Graha::Mars => 17.0,
        Graha::Mercury => if retrograde { 12.0 } else { 14.0 },
        Graha::Jupiter => 11.0,
        Graha::Venus => if retrograde { 8.0 } else { 10.0 },
        Graha::Saturn => 15.0,
        Graha::Sun | Graha::Rahu | Graha::Ketu => return None,
    })
}

/// Shortest arc between two longitudes, `[0, 180]`.
pub fn arc(a: f64, b: f64) -> f64 {
    let d = lagn_ephem::norm360(a - b);
    if d > 180.0 { 360.0 - d } else { d }
}

/// Is a graha combust? Inclusive: at exactly the orb it is combust.
pub fn is_combust(g: Graha, longitude: f64, retrograde: bool, sun_longitude: f64) -> bool {
    combustion_orb(g, retrograde).is_some_and(|orb| arc(longitude, sun_longitude) <= orb)
}

/// The five grahas that can fight a planetary war.
pub const WAR_CAPABLE: [Graha; 5] =
    [Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Venus, Graha::Saturn];

/// Maximum separation for graha yuddha, degrees, inclusive.
pub const YUDDHA_ORB: f64 = 1.0;

#[cfg(test)]
mod tests {
    use super::*;
    use BaladiAvastha::*;

    #[test]
    fn baladi_bands_in_an_odd_sign() {
        // Mesha is odd.
        assert_eq!(baladi(0.0), Bala);
        assert_eq!(baladi(5.999), Bala);
        assert_eq!(baladi(6.0), Kumara);
        assert_eq!(baladi(12.0), Yuva);
        assert_eq!(baladi(18.0), Vriddha);
        assert_eq!(baladi(24.0), Mrita);
        assert_eq!(baladi(29.999), Mrita);
    }

    #[test]
    fn baladi_bands_reverse_in_an_even_sign() {
        // Vrishabha is even.
        assert_eq!(baladi(30.0), Mrita);
        assert_eq!(baladi(36.0), Vriddha);
        assert_eq!(baladi(42.0), Yuva);
        assert_eq!(baladi(48.0), Kumara);
        assert_eq!(baladi(54.0), Bala);
        assert_eq!(baladi(59.999), Bala);
    }

    #[test]
    fn jagradadi_covers_all_nine_dignities() {
        use Dignity::*;
        use JagradadiAvastha::*;
        for (d, e) in [
            (Exalted, Jagrat), (Moolatrikona, Jagrat), (OwnSign, Jagrat),
            (GreatFriend, Swapna), (Friend, Swapna), (Neutral, Swapna),
            (Enemy, Sushupti), (GreatEnemy, Sushupti), (Debilitated, Sushupti),
        ] {
            assert_eq!(jagradadi(d), e, "{d:?}");
        }
    }

    #[test]
    fn combustion_orbs_match_the_specification() {
        assert_eq!(combustion_orb(Graha::Moon, false), Some(12.0));
        assert_eq!(combustion_orb(Graha::Mars, false), Some(17.0));
        assert_eq!(combustion_orb(Graha::Mercury, false), Some(14.0));
        assert_eq!(combustion_orb(Graha::Mercury, true), Some(12.0));
        assert_eq!(combustion_orb(Graha::Jupiter, false), Some(11.0));
        assert_eq!(combustion_orb(Graha::Venus, false), Some(10.0));
        assert_eq!(combustion_orb(Graha::Venus, true), Some(8.0));
        assert_eq!(combustion_orb(Graha::Saturn, false), Some(15.0));
        for g in [Graha::Sun, Graha::Rahu, Graha::Ketu] {
            assert_eq!(combustion_orb(g, false), None);
        }
    }

    #[test]
    fn combustion_is_inclusive_and_wraps_across_zero() {
        assert!(is_combust(Graha::Jupiter, 11.0, false, 0.0));
        assert!(!is_combust(Graha::Jupiter, 11.0001, false, 0.0));
        // Across 0/360.
        assert!(is_combust(Graha::Saturn, 355.0, false, 5.0));
        assert!(!is_combust(Graha::Sun, 0.0, false, 0.0));
    }

    #[test]
    fn arc_is_the_shortest_separation() {
        assert_eq!(arc(10.0, 350.0), 20.0);
        assert_eq!(arc(350.0, 10.0), 20.0);
        assert_eq!(arc(0.0, 180.0), 180.0);
        assert_eq!(arc(90.0, 90.0), 0.0);
    }
}
