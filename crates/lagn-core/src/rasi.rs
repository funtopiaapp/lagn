//! The twelve rasis and their classical attributes.

use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

/// A rasi (zodiacal sign), 30 degrees of sidereal longitude.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Rasi {
    Mesha = 0,
    Vrishabha = 1,
    Mithuna = 2,
    Karka = 3,
    Simha = 4,
    Kanya = 5,
    Tula = 6,
    Vrischika = 7,
    Dhanus = 8,
    Makara = 9,
    Kumbha = 10,
    Meena = 11,
}

/// Chara / sthira / dvisvabhava - governs, among much else, which sign a
/// navamsa series starts from and how quickly a bhava's results manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mobility {
    /// Movable / cardinal.
    Chara,
    /// Fixed.
    Sthira,
    /// Dual / mutable.
    Dvisvabhava,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Element {
    Agni,
    Prithvi,
    Vayu,
    Jala,
}

impl Rasi {
    pub const ALL: [Rasi; 12] = [
        Rasi::Mesha, Rasi::Vrishabha, Rasi::Mithuna, Rasi::Karka,
        Rasi::Simha, Rasi::Kanya, Rasi::Tula, Rasi::Vrischika,
        Rasi::Dhanus, Rasi::Makara, Rasi::Kumbha, Rasi::Meena,
    ];

    /// From a 0-based index, wrapping. Negative inputs wrap correctly, which
    /// matters because house arithmetic routinely goes below zero.
    #[inline]
    pub fn from_index(i: i32) -> Rasi {
        Rasi::ALL[i.rem_euclid(12) as usize]
    }

    #[inline]
    pub fn index(self) -> u8 {
        self as u8
    }

    /// The rasi containing a sidereal longitude.
    #[inline]
    pub fn from_longitude(lon: f64) -> Rasi {
        Rasi::from_index((lagn_ephem::norm360(lon) / 30.0).floor() as i32)
    }

    /// Degrees traversed within this rasi, `[0, 30)`.
    #[inline]
    pub fn degrees_within(lon: f64) -> f64 {
        lagn_ephem::norm360(lon) % 30.0
    }

    /// Rasi lord (adhipati). Uses the classical seven-lord scheme - no outer
    /// planets, so Vrischika is ruled by Kuja and Kumbha by Shani.
    pub fn lord(self) -> Graha {
        match self {
            Rasi::Mesha | Rasi::Vrischika => Graha::Mars,
            Rasi::Vrishabha | Rasi::Tula => Graha::Venus,
            Rasi::Mithuna | Rasi::Kanya => Graha::Mercury,
            Rasi::Karka => Graha::Moon,
            Rasi::Simha => Graha::Sun,
            Rasi::Dhanus | Rasi::Meena => Graha::Jupiter,
            Rasi::Makara | Rasi::Kumbha => Graha::Saturn,
        }
    }

    pub fn mobility(self) -> Mobility {
        match self.index() % 3 {
            0 => Mobility::Chara,
            1 => Mobility::Sthira,
            _ => Mobility::Dvisvabhava,
        }
    }

    pub fn element(self) -> Element {
        match self.index() % 4 {
            0 => Element::Agni,
            1 => Element::Prithvi,
            2 => Element::Vayu,
            _ => Element::Jala,
        }
    }

    /// Odd (vishama) signs are male/cruel; even (sama) signs female/mild.
    #[inline]
    pub fn is_odd(self) -> bool {
        self.index() % 2 == 0
    }

    /// Sanskrit name.
    pub fn name(self) -> &'static str {
        match self {
            Rasi::Mesha => "Mesha", Rasi::Vrishabha => "Vrishabha",
            Rasi::Mithuna => "Mithuna", Rasi::Karka => "Karka",
            Rasi::Simha => "Simha", Rasi::Kanya => "Kanya",
            Rasi::Tula => "Tula", Rasi::Vrischika => "Vrischika",
            Rasi::Dhanus => "Dhanus", Rasi::Makara => "Makara",
            Rasi::Kumbha => "Kumbha", Rasi::Meena => "Meena",
        }
    }

    /// Tamil name, as printed on a South Indian jathagam.
    pub fn tamil_name(self) -> &'static str {
        match self {
            Rasi::Mesha => "Mesham", Rasi::Vrishabha => "Rishabam",
            Rasi::Mithuna => "Mithunam", Rasi::Karka => "Kadagam",
            Rasi::Simha => "Simmam", Rasi::Kanya => "Kanni",
            Rasi::Tula => "Thulam", Rasi::Vrischika => "Viruchigam",
            Rasi::Dhanus => "Dhanusu", Rasi::Makara => "Magaram",
            Rasi::Kumbha => "Kumbam", Rasi::Meena => "Meenam",
        }
    }

    /// Signed distance in signs from `self` to `other`, 1-based and inclusive,
    /// i.e. the "nth house from" count used throughout jyotisha.
    #[inline]
    pub fn houses_to(self, other: Rasi) -> u8 {
        (other.index() as i32 - self.index() as i32).rem_euclid(12) as u8 + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longitude_maps_to_rasi() {
        assert_eq!(Rasi::from_longitude(0.0), Rasi::Mesha);
        assert_eq!(Rasi::from_longitude(29.999), Rasi::Mesha);
        assert_eq!(Rasi::from_longitude(30.0), Rasi::Vrishabha);
        assert_eq!(Rasi::from_longitude(359.999), Rasi::Meena);
        assert_eq!(Rasi::from_longitude(360.0), Rasi::Mesha);
        assert_eq!(Rasi::from_longitude(-1.0), Rasi::Meena);
    }

    #[test]
    fn each_of_the_seven_grahas_rules_the_expected_count() {
        // Sun and Moon rule one sign each; the other five rule two.
        for (g, n) in [
            (Graha::Sun, 1), (Graha::Moon, 1), (Graha::Mars, 2),
            (Graha::Mercury, 2), (Graha::Jupiter, 2), (Graha::Venus, 2),
            (Graha::Saturn, 2),
        ] {
            let count = Rasi::ALL.iter().filter(|r| r.lord() == g).count();
            assert_eq!(count, n, "{} rules {count} signs, expected {n}", g.name());
        }
    }

    #[test]
    fn mobility_and_element_cycle_correctly() {
        assert_eq!(Rasi::Mesha.mobility(), Mobility::Chara);
        assert_eq!(Rasi::Vrishabha.mobility(), Mobility::Sthira);
        assert_eq!(Rasi::Mithuna.mobility(), Mobility::Dvisvabhava);
        assert_eq!(Rasi::Karka.mobility(), Mobility::Chara);
        assert_eq!(Rasi::Mesha.element(), Element::Agni);
        assert_eq!(Rasi::Simha.element(), Element::Agni);
        assert_eq!(Rasi::Dhanus.element(), Element::Agni);
    }

    #[test]
    fn house_counting_is_one_based_and_inclusive() {
        assert_eq!(Rasi::Mesha.houses_to(Rasi::Mesha), 1);
        assert_eq!(Rasi::Mesha.houses_to(Rasi::Vrishabha), 2);
        assert_eq!(Rasi::Mesha.houses_to(Rasi::Meena), 12);
        assert_eq!(Rasi::Meena.houses_to(Rasi::Mesha), 2);
        // The 7th from anything is always its opposite.
        for r in Rasi::ALL {
            assert_eq!(r.houses_to(Rasi::from_index(r.index() as i32 + 6)), 7);
        }
    }
}
