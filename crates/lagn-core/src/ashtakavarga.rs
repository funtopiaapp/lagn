//! Ashtakavarga: Bhinnashtakavarga (BAV), Sarvashtakavarga (SAV) and prastara.
//!
//! Specification: `docs/phase2/DESIGN.md` section 7 (BPHS ch. 66).
//!
//! The table is self-checking: every graha's BAV total and the SAV total (337)
//! are fixed regardless of the chart, so a single mistranscribed house number
//! breaks an invariant test.

use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

use crate::chart::Chart;
use crate::rasi::Rasi;
use crate::relationship::SEVEN;

/// The eight sources of bindus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Contributor {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Lagna,
}

impl Contributor {
    pub const ALL: [Contributor; 8] = [
        Contributor::Sun, Contributor::Moon, Contributor::Mars, Contributor::Mercury,
        Contributor::Jupiter, Contributor::Venus, Contributor::Saturn, Contributor::Lagna,
    ];

    fn sign_in(self, chart: &Chart) -> Rasi {
        match self {
            Contributor::Sun => chart.placement(Graha::Sun).rasi,
            Contributor::Moon => chart.placement(Graha::Moon).rasi,
            Contributor::Mars => chart.placement(Graha::Mars).rasi,
            Contributor::Mercury => chart.placement(Graha::Mercury).rasi,
            Contributor::Jupiter => chart.placement(Graha::Jupiter).rasi,
            Contributor::Venus => chart.placement(Graha::Venus).rasi,
            Contributor::Saturn => chart.placement(Graha::Saturn).rasi,
            Contributor::Lagna => chart.lagna.rasi,
        }
    }
}

/// `TABLE[p][c]`: houses from contributor `c` that give a bindu in graha `p`'s
/// BAV. Rows and columns are in the order Sun Moon Mars Mercury Jupiter Venus
/// Saturn (Lagna last for columns).
const TABLE: [[&[u8]; 8]; 7] = [
    // Sun's BAV (48)
    [&[1, 2, 4, 7, 8, 9, 10, 11], &[3, 6, 10, 11], &[1, 2, 4, 7, 8, 9, 10, 11],
     &[3, 5, 6, 9, 10, 11, 12], &[5, 6, 9, 11], &[6, 7, 12],
     &[1, 2, 4, 7, 8, 9, 10, 11], &[3, 4, 6, 10, 11, 12]],
    // Moon's BAV (49)
    [&[3, 6, 7, 8, 10, 11], &[1, 3, 6, 7, 10, 11], &[2, 3, 5, 6, 9, 10, 11],
     &[1, 3, 4, 5, 7, 8, 10, 11], &[1, 4, 7, 8, 10, 11, 12], &[3, 4, 5, 7, 9, 10, 11],
     &[3, 5, 6, 11], &[3, 6, 10, 11]],
    // Mars's BAV (39)
    [&[3, 5, 6, 10, 11], &[3, 6, 11], &[1, 2, 4, 7, 8, 10, 11],
     &[3, 5, 6, 11], &[6, 10, 11, 12], &[6, 8, 11, 12],
     &[1, 4, 7, 8, 9, 10, 11], &[1, 3, 6, 10, 11]],
    // Mercury's BAV (54)
    [&[5, 6, 9, 11, 12], &[2, 4, 6, 8, 10, 11], &[1, 2, 4, 7, 8, 9, 10, 11],
     &[1, 3, 5, 6, 9, 10, 11, 12], &[6, 8, 11, 12], &[1, 2, 3, 4, 5, 8, 9, 11],
     &[1, 2, 4, 7, 8, 9, 10, 11], &[1, 2, 4, 6, 8, 10, 11]],
    // Jupiter's BAV (56)
    [&[1, 2, 3, 4, 7, 8, 9, 10, 11], &[2, 5, 7, 9, 11], &[1, 2, 4, 7, 8, 10, 11],
     &[1, 2, 4, 5, 6, 9, 10, 11], &[1, 2, 3, 4, 7, 8, 10, 11], &[2, 5, 6, 9, 10, 11],
     &[3, 5, 6, 12], &[1, 2, 4, 5, 6, 7, 9, 10, 11]],
    // Venus's BAV (52)
    [&[8, 11, 12], &[1, 2, 3, 4, 5, 8, 9, 11, 12], &[3, 5, 6, 9, 11, 12],
     &[3, 5, 6, 9, 11], &[5, 8, 9, 10, 11], &[1, 2, 3, 4, 5, 8, 9, 10, 11],
     &[3, 4, 5, 8, 9, 10, 11], &[1, 2, 3, 4, 5, 8, 9, 11]],
    // Saturn's BAV (39)
    [&[1, 2, 4, 7, 8, 10, 11], &[3, 6, 11], &[3, 5, 6, 10, 11, 12],
     &[6, 8, 9, 10, 11, 12], &[5, 6, 11, 12], &[6, 11, 12],
     &[3, 5, 6, 11], &[1, 3, 4, 6, 10, 11]],
];

/// Fixed BAV totals, Sun..Saturn. Chart-independent.
pub const BAV_TOTALS: [u32; 7] = [48, 49, 39, 54, 56, 52, 39];
/// Fixed SAV total. Chart-independent.
pub const SAV_TOTAL: u32 = 337;

/// Benefic houses from contributor `c` in graha `p`'s BAV. `None` for nodes.
pub fn benefic_houses(p: Graha, c: Contributor) -> Option<&'static [u8]> {
    let row = SEVEN.iter().position(|&g| g == p)?;
    let col = Contributor::ALL.iter().position(|&x| x == c).expect("all contributors");
    Some(TABLE[row][col])
}

/// A computed Ashtakavarga.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ashtakavarga {
    /// `bav[p][sign]`, p in Sun..Saturn order, sign index Mesha = 0.
    pub bav: [[u8; 12]; 7],
    /// `sav[sign]`.
    pub sav: [u8; 12],
    /// `prastara[p][sign]`: bit `i` set when `Contributor::ALL[i]` gave the
    /// bindu. `bav[p][s]` is the popcount of `prastara[p][s]`.
    pub prastara: [[u8; 12]; 7],
    /// Lagna sign, so house-indexed views need no chart.
    pub lagna: Rasi,
}

impl Ashtakavarga {
    pub fn compute(chart: &Chart) -> Ashtakavarga {
        let mut prastara = [[0u8; 12]; 7];
        for (p, row) in TABLE.iter().enumerate() {
            for (ci, houses) in row.iter().enumerate() {
                let from = Contributor::ALL[ci].sign_in(chart);
                for &h in houses.iter() {
                    let sign = Rasi::from_index(from.index() as i32 + h as i32 - 1);
                    prastara[p][sign.index() as usize] |= 1 << ci;
                }
            }
        }
        let mut bav = [[0u8; 12]; 7];
        let mut sav = [0u8; 12];
        for p in 0..7 {
            for s in 0..12 {
                bav[p][s] = prastara[p][s].count_ones() as u8;
                sav[s] += bav[p][s];
            }
        }
        Ashtakavarga { bav, sav, prastara, lagna: chart.lagna.rasi }
    }

    /// A graha's BAV, indexed by sign. `None` for the nodes.
    pub fn bav_of(&self, g: Graha) -> Option<[u8; 12]> {
        SEVEN.iter().position(|&x| x == g).map(|i| self.bav[i])
    }

    /// Bindus in one sign of one graha's BAV.
    pub fn bindus(&self, g: Graha, sign: Rasi) -> Option<u8> {
        self.bav_of(g).map(|b| b[sign.index() as usize])
    }

    /// Contributors that gave a bindu in one sign of one graha's BAV.
    pub fn contributors(&self, g: Graha, sign: Rasi) -> Option<Vec<Contributor>> {
        let i = SEVEN.iter().position(|&x| x == g)?;
        let mask = self.prastara[i][sign.index() as usize];
        Some(
            Contributor::ALL
                .into_iter()
                .enumerate()
                .filter(|(bit, _)| mask & (1 << bit) != 0)
                .map(|(_, c)| c)
                .collect(),
        )
    }

    /// SAV of a house counted from the lagna, 1..=12.
    pub fn sav_in_house(&self, house: u8) -> u8 {
        self.sav[Rasi::from_index(self.lagna.index() as i32 + house as i32 - 1).index() as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_rows_sum_to_the_fixed_bav_totals() {
        for (p, row) in TABLE.iter().enumerate() {
            let total: u32 = row.iter().map(|h| h.len() as u32).sum();
            assert_eq!(total, BAV_TOTALS[p], "{}'s BAV row", SEVEN[p].name());
        }
        assert_eq!(BAV_TOTALS.iter().sum::<u32>(), SAV_TOTAL);
    }

    #[test]
    fn every_table_entry_is_a_valid_house_listed_once_in_order() {
        for row in TABLE {
            for houses in row {
                assert!(houses.windows(2).all(|w| w[0] < w[1]), "unsorted or duplicate: {houses:?}");
                assert!(houses.iter().all(|&h| (1..=12).contains(&h)), "bad house: {houses:?}");
            }
        }
    }

    #[test]
    fn benefic_houses_lookup_matches_the_specification_samples() {
        assert_eq!(benefic_houses(Graha::Sun, Contributor::Venus), Some(&[6u8, 7, 12][..]));
        assert_eq!(benefic_houses(Graha::Jupiter, Contributor::Saturn), Some(&[3u8, 5, 6, 12][..]));
        assert_eq!(benefic_houses(Graha::Venus, Contributor::Sun), Some(&[8u8, 11, 12][..]));
        assert_eq!(benefic_houses(Graha::Saturn, Contributor::Lagna), Some(&[1u8, 3, 4, 6, 10, 11][..]));
        assert_eq!(benefic_houses(Graha::Rahu, Contributor::Sun), None);
    }
}
