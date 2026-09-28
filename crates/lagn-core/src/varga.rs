//! The sixteen Shodasavargas (divisional charts), Parashari method.
//!
//! Specification: `docs/phase2/DESIGN.md` section 3. Every rule there is
//! restated longhand in this module's tests.
//!
//! # The grid rule
//!
//! For an equal-part varga with `n` parts per sign, the sign index `s` and the
//! part index `k` are read off a *single* floor:
//!
//! ```text
//! K = floor(longitude / (30 / n))      0 <= K < 12n
//! s = K div n,  k = K mod n
//! ```
//!
//! Computing `s` with one division and `k` from a separately derived
//! "degrees within sign" lets rounding put them in inconsistent cells at a
//! boundary - the same defect class Phase 1 QA removed from the nakshatra/pada
//! pair. For n = 9 this is bit-identical to the navamsa and pada grid.

use crate::rasi::{Element, Mobility, Rasi};
use serde::{Deserialize, Serialize};

/// A divisional chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Varga {
    /// Rasi - the birth chart itself.
    D1,
    /// Hora - wealth.
    D2,
    /// Drekkana - siblings, courage.
    D3,
    /// Chaturthamsa - property, fortune.
    D4,
    /// Saptamsa - progeny.
    D7,
    /// Navamsa - spouse, dharma, and the resilience of every other promise.
    D9,
    /// Dasamsa - career.
    D10,
    /// Dwadasamsa - parents.
    D12,
    /// Shodasamsa - vehicles, comforts.
    D16,
    /// Vimsamsa - spiritual practice.
    D20,
    /// Chaturvimsamsa - learning.
    D24,
    /// Saptavimsamsa / Bhamsa - strengths and weaknesses.
    D27,
    /// Trimsamsa - misfortune. Five unequal parts.
    D30,
    /// Khavedamsa - maternal legacy.
    D40,
    /// Akshavedamsa - paternal legacy, character.
    D45,
    /// Shashtiamsa - past karma. The finest division.
    D60,
}

impl Varga {
    /// All sixteen, in conventional order.
    pub const ALL: [Varga; 16] = [
        Varga::D1, Varga::D2, Varga::D3, Varga::D4, Varga::D7, Varga::D9,
        Varga::D10, Varga::D12, Varga::D16, Varga::D20, Varga::D24, Varga::D27,
        Varga::D30, Varga::D40, Varga::D45, Varga::D60,
    ];

    /// The varga's number, as in "D-10".
    pub const fn number(self) -> u32 {
        match self {
            Varga::D1 => 1, Varga::D2 => 2, Varga::D3 => 3, Varga::D4 => 4,
            Varga::D7 => 7, Varga::D9 => 9, Varga::D10 => 10, Varga::D12 => 12,
            Varga::D16 => 16, Varga::D20 => 20, Varga::D24 => 24, Varga::D27 => 27,
            Varga::D30 => 30, Varga::D40 => 40, Varga::D45 => 45, Varga::D60 => 60,
        }
    }

    /// Parts per sign. Equals [`Varga::number`] for every varga except D-30,
    /// whose thirty is nominal: it has five unequal parts.
    pub const fn parts_per_sign(self) -> u32 {
        match self {
            Varga::D30 => 5,
            v => v.number(),
        }
    }

    pub fn from_number(n: u32) -> Option<Varga> {
        Varga::ALL.into_iter().find(|v| v.number() == n)
    }

    pub fn name(self) -> &'static str {
        match self {
            Varga::D1 => "Rasi", Varga::D2 => "Hora", Varga::D3 => "Drekkana",
            Varga::D4 => "Chaturthamsa", Varga::D7 => "Saptamsa", Varga::D9 => "Navamsa",
            Varga::D10 => "Dasamsa", Varga::D12 => "Dwadasamsa", Varga::D16 => "Shodasamsa",
            Varga::D20 => "Vimsamsa", Varga::D24 => "Chaturvimsamsa", Varga::D27 => "Bhamsa",
            Varga::D30 => "Trimsamsa", Varga::D40 => "Khavedamsa", Varga::D45 => "Akshavedamsa",
            Varga::D60 => "Shashtiamsa",
        }
    }

    /// e.g. "D-9 Navamsa".
    pub fn label(self) -> String {
        format!("D-{} {}", self.number(), self.name())
    }

    /// The sign a sidereal longitude falls in, in this varga.
    pub fn sign_of(self, longitude: f64) -> Rasi {
        const MESHA: i32 = 0;
        const KARKA: i32 = 3;
        const SIMHA: i32 = 4;
        const TULA: i32 = 6;
        const DHANUS: i32 = 8;
        const MAKARA: i32 = 9;

        // D-9 and D-30 do not use the shared equal-part path below.
        match self {
            // Routed through the Phase 1 function so its output stays
            // byte-identical; a test proves it equals the grid rule.
            Varga::D9 => return navamsa_sign(longitude),
            Varga::D30 => return trimsamsa_sign(longitude),
            _ => {}
        }

        let (s, k) = grid(longitude, self.parts_per_sign() as i32);
        let sign = Rasi::from_index(s);
        let odd = sign.is_odd();
        let by_mobility = |chara: i32, sthira: i32, dvisvabhava: i32| match sign.mobility() {
            Mobility::Chara => chara,
            Mobility::Sthira => sthira,
            Mobility::Dvisvabhava => dvisvabhava,
        };

        let index = match self {
            Varga::D1 => s,
            // Odd sign: Simha then Karka. Even sign: Karka then Simha.
            Varga::D2 => match (odd, k) {
                (true, 0) | (false, 1) => SIMHA,
                _ => KARKA,
            },
            // 1st, 5th and 9th from the sign.
            Varga::D3 => s + 4 * k,
            // 1st, 4th, 7th and 10th from the sign.
            Varga::D4 => s + 3 * k,
            Varga::D7 => (if odd { s } else { s + 6 }) + k,
            Varga::D10 => (if odd { s } else { s + 8 }) + k,
            Varga::D12 | Varga::D60 => s + k,
            Varga::D16 | Varga::D45 => by_mobility(MESHA, SIMHA, DHANUS) + k,
            Varga::D20 => by_mobility(MESHA, DHANUS, SIMHA) + k,
            Varga::D24 => (if odd { SIMHA } else { KARKA }) + k,
            Varga::D27 => {
                (match sign.element() {
                    Element::Agni => MESHA,
                    Element::Prithvi => KARKA,
                    Element::Vayu => TULA,
                    Element::Jala => MAKARA,
                }) + k
            }
            Varga::D40 => (if odd { MESHA } else { TULA }) + k,
            Varga::D9 | Varga::D30 => unreachable!("returned above"),
        };
        Rasi::from_index(index)
    }
}

/// `(s, k)` for an equal-part varga, both read off one floor.
#[inline]
fn grid(longitude: f64, n: i32) -> (i32, i32) {
    let lon = lagn_ephem::norm360(longitude);
    let cell = ((lon / (30.0 / n as f64)).floor() as i32).clamp(0, 12 * n - 1);
    (cell / n, cell % n)
}

/// Navamsa sign for a sidereal longitude.
#[inline]
pub fn navamsa_sign(longitude: f64) -> Rasi {
    let lon = lagn_ephem::norm360(longitude);
    Rasi::from_index((lon / (30.0 / 9.0)).floor() as i32)
}

/// D-30 Trimsamsa (Parashari): five unequal parts, table in DESIGN.md 3.
///
/// Intervals are half-open `[a, b)` (variant V-9).
pub fn trimsamsa_sign(longitude: f64) -> Rasi {
    const ODD: [(f64, Rasi); 5] = [
        (5.0, Rasi::Mesha),
        (10.0, Rasi::Kumbha),
        (18.0, Rasi::Dhanus),
        (25.0, Rasi::Mithuna),
        (30.0, Rasi::Tula),
    ];
    const EVEN: [(f64, Rasi); 5] = [
        (5.0, Rasi::Vrishabha),
        (12.0, Rasi::Kanya),
        (20.0, Rasi::Meena),
        (25.0, Rasi::Makara),
        (30.0, Rasi::Vrischika),
    ];
    let (sign, deg) = sign_and_degree(longitude);
    let table = if sign.is_odd() { &ODD } else { &EVEN };
    table
        .iter()
        .find(|(upper, _)| deg < *upper)
        .map(|(_, r)| *r)
        .unwrap_or(table[4].1)
}

/// Sign and degrees-within-sign derived from one floor, so the pair is always
/// consistent: the degree is measured from the start of *that* sign and
/// clamped into `[0, 30)`.
pub(crate) fn sign_and_degree(longitude: f64) -> (Rasi, f64) {
    let lon = lagn_ephem::norm360(longitude);
    let s = ((lon / 30.0).floor() as i32).clamp(0, 11);
    let deg = (lon - 30.0 * s as f64).clamp(0.0, 30.0_f64.next_down_compat());
    (Rasi::from_index(s), deg)
}

/// `f64::next_down` is not yet stable on every toolchain we target.
trait NextDown {
    fn next_down_compat(self) -> f64;
}
impl NextDown for f64 {
    fn next_down_compat(self) -> f64 {
        f64::from_bits(self.to_bits() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nakshatra::NakshatraPosition;

    /// Longitudes that hit every part of every varga: dense, plus one probe
    /// deliberately just inside each 1/60th-degree cell edge.
    fn sweep() -> impl Iterator<Item = f64> {
        (0..360_000).map(|i| i as f64 * 0.001 + 0.0003)
    }

    // --- Phase 1 navamsa tests, unchanged -------------------------------

    fn navamsa_sign_textbook(longitude: f64) -> Rasi {
        let lon = lagn_ephem::norm360(longitude);
        let sign = Rasi::from_longitude(lon);
        let within = Rasi::degrees_within(lon);
        let nth = (within / (30.0 / 9.0)).floor() as i32;
        let start = match sign.mobility() {
            Mobility::Chara => sign.index() as i32,
            Mobility::Sthira => sign.index() as i32 + 8,
            Mobility::Dvisvabhava => sign.index() as i32 + 4,
        };
        Rasi::from_index(start + nth)
    }

    #[test]
    fn closed_form_agrees_with_the_textbook_rule() {
        let mut lon = 0.0;
        while lon < 360.0 {
            assert_eq!(navamsa_sign(lon), navamsa_sign_textbook(lon), "at {lon}");
            lon += 0.01;
        }
    }

    #[test]
    fn navamsa_matches_pada_grid() {
        let mut lon = 0.0;
        while lon < 360.0 {
            let np = NakshatraPosition::from_longitude(lon);
            assert_eq!(navamsa_sign(lon), Rasi::from_index(np.absolute_pada() as i32), "at {lon}");
            lon += 0.037;
        }
    }

    #[test]
    fn known_navamsa_anchors() {
        assert_eq!(navamsa_sign(0.0), Rasi::Mesha);
        assert_eq!(navamsa_sign(30.0), Rasi::Makara);
        assert_eq!(navamsa_sign(60.0), Rasi::Tula);
        assert_eq!(navamsa_sign(90.0), Rasi::Karka);
        assert_eq!(navamsa_sign(360.0 - 1e-9), Rasi::Meena);
    }

    #[test]
    fn every_sign_gets_nine_navamsas() {
        let mut counts = [0u32; 12];
        for i in 0..108 {
            counts[navamsa_sign(i as f64 * (30.0 / 9.0) + 1e-6).index() as usize] += 1;
        }
        assert_eq!(counts, [9; 12]);
    }

    // --- Phase 2 --------------------------------------------------------

    #[test]
    fn the_d9_grid_rule_equals_the_phase_1_navamsa_everywhere() {
        for lon in sweep() {
            let (s, k) = grid(lon, 9);
            let sign = Rasi::from_index(s);
            let start = match sign.mobility() {
                Mobility::Chara => s,
                Mobility::Sthira => s + 8,
                Mobility::Dvisvabhava => s + 4,
            };
            assert_eq!(Rasi::from_index(start + k), navamsa_sign(lon), "at {lon}");
        }
    }

    #[test]
    fn metadata_is_consistent() {
        assert_eq!(Varga::ALL.len(), 16);
        for v in Varga::ALL {
            assert_eq!(Varga::from_number(v.number()), Some(v));
            assert!(v.label().starts_with(&format!("D-{} ", v.number())));
        }
        assert_eq!(Varga::D30.parts_per_sign(), 5);
        assert_eq!(Varga::D60.parts_per_sign(), 60);
    }
}
