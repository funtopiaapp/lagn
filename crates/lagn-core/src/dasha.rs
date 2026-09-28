//! Vimshottari dasha to three levels.
//!
//! # The year-length problem
//!
//! Vimshottari's 120 years are 120 *of something*, and the classical sources
//! do not say which. Software disagrees: some use the Julian year of 365.25
//! days, some the savana (civil) year of 360, some the sidereal solar year.
//! Over a 19-year Shani mahadasha the spread between conventions exceeds a
//! hundred days, which is more than enough to move a prediction across a
//! calendar year.
//!
//! There is no correct answer to hard-code, so [`YearLength`] is part of
//! [`crate::ChartSettings`] and is recorded on every chart. When our output
//! disagrees with someone's family astrologer, this is the first thing to
//! check - and it is usually the cause.

use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

use crate::chart::Chart;
use crate::nakshatra::Nakshatra;

/// The Vimshottari cycle: nine lords in fixed order, beginning at Ashwini.
pub const CYCLE: [Graha; 9] = [
    Graha::Ketu, Graha::Venus, Graha::Sun, Graha::Moon, Graha::Mars,
    Graha::Rahu, Graha::Jupiter, Graha::Saturn, Graha::Mercury,
];

/// Mahadasha length in Vimshottari years. Sums to exactly 120.
pub const fn dasha_years(g: Graha) -> f64 {
    match g {
        Graha::Ketu => 7.0,
        Graha::Venus => 20.0,
        Graha::Sun => 6.0,
        Graha::Moon => 10.0,
        Graha::Mars => 7.0,
        Graha::Rahu => 18.0,
        Graha::Jupiter => 16.0,
        Graha::Saturn => 19.0,
        Graha::Mercury => 17.0,
    }
}

/// Total cycle length in years.
pub const TOTAL_YEARS: f64 = 120.0;

/// How many days a "year" is worth when converting dasha years to dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum YearLength {
    /// 365.25 days. The most widely used convention and our default.
    #[default]
    Julian,
    /// 365.2425 days - the mean Gregorian year.
    Gregorian,
    /// 365.242190 days - the mean tropical year.
    Tropical,
    /// 365.256363 days - the sidereal solar year. Defensible on the grounds
    /// that jyotisha is a sidereal system throughout.
    SiderealSolar,
    /// 360 days - the savana or civil year of some traditional reckonings.
    Savana,
}

impl YearLength {
    pub const fn days(self) -> f64 {
        match self {
            YearLength::Julian => 365.25,
            YearLength::Gregorian => 365.2425,
            YearLength::Tropical => 365.242190,
            YearLength::SiderealSolar => 365.256363,
            YearLength::Savana => 360.0,
        }
    }
}

/// Position of a lord within the cycle.
#[inline]
fn cycle_index(g: Graha) -> usize {
    CYCLE.iter().position(|&c| c == g).expect("every graha except none is a dasha lord")
}

/// One dasha period at any level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashaPeriod {
    /// 1 = mahadasha, 2 = antardasha (bhukti), 3 = pratyantardasha.
    pub level: u8,
    pub lord: Graha,
    pub start_jd: f64,
    pub end_jd: f64,
    pub children: Vec<DashaPeriod>,
}

impl DashaPeriod {
    pub fn duration_days(&self) -> f64 {
        self.end_jd - self.start_jd
    }

    pub fn contains(&self, jd: f64) -> bool {
        jd >= self.start_jd && jd < self.end_jd
    }
}

/// The running dasha chain at an instant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DashaChain {
    pub maha: Graha,
    pub antar: Graha,
    pub pratyantar: Graha,
}

impl std::fmt::Display for DashaChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} / {} / {}", self.maha.name(), self.antar.name(), self.pratyantar.name())
    }
}

/// A computed Vimshottari tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vimshottari {
    pub birth_jd: f64,
    pub year_length: YearLength,
    pub janma_nakshatra: Nakshatra,
    /// Lord of the janma nakshatra - the mahadasha running at birth.
    pub birth_lord: Graha,
    /// Years of the birth mahadasha still unelapsed at the moment of birth.
    pub balance_years: f64,
    /// Mahadashas in sequence. The first begins *before* birth: its opening
    /// portion had already elapsed when the native was born, and keeping that
    /// portion is what makes the antardasha boundaries inside it correct.
    pub mahadashas: Vec<DashaPeriod>,
}

/// How many mahadashas to generate. Nine completes the 120-year cycle; a tenth
/// covers the tail, because the first one is only partly post-birth.
const MAHADASHA_COUNT: usize = 10;

impl Vimshottari {
    pub fn compute(chart: &Chart) -> Vimshottari {
        let nak = chart.janma_nakshatra();
        Self::from_parts(
            chart.jd_ut,
            nak.nakshatra,
            nak.fraction_traversed,
            chart.settings.year_length,
        )
    }

    /// Build from raw parts. Split out from [`Vimshottari::compute`] so the
    /// dasha arithmetic can be tested without an ephemeris.
    pub fn from_parts(
        birth_jd: f64,
        janma_nakshatra: Nakshatra,
        fraction_traversed: f64,
        year_length: YearLength,
    ) -> Vimshottari {
        let dpy = year_length.days();
        let birth_lord = janma_nakshatra.lord();
        let first_years = dasha_years(birth_lord);
        let balance_years = (1.0 - fraction_traversed) * first_years;

        // Wind back to where the first mahadasha actually began.
        let mut cursor = birth_jd - fraction_traversed * first_years * dpy;
        let start_idx = cycle_index(birth_lord);

        let mut mahadashas = Vec::with_capacity(MAHADASHA_COUNT);
        for step in 0..MAHADASHA_COUNT {
            let lord = CYCLE[(start_idx + step) % 9];
            let span = dasha_years(lord) * dpy;
            let start = cursor;
            let end = start + span;
            mahadashas.push(DashaPeriod {
                level: 1,
                lord,
                start_jd: start,
                end_jd: end,
                children: subdivide(lord, start, end, 2),
            });
            cursor = end;
        }

        Vimshottari {
            birth_jd,
            year_length,
            janma_nakshatra,
            birth_lord,
            balance_years,
            mahadashas,
        }
    }

    /// The dasha chain running at an instant, or `None` outside the tree.
    pub fn at(&self, jd: f64) -> Option<DashaChain> {
        let maha = self.mahadashas.iter().find(|p| p.contains(jd))?;
        let antar = maha.children.iter().find(|p| p.contains(jd))?;
        let pratyantar = antar.children.iter().find(|p| p.contains(jd))?;
        Some(DashaChain {
            maha: maha.lord,
            antar: antar.lord,
            pratyantar: pratyantar.lord,
        })
    }

    /// The chain running at birth.
    pub fn at_birth(&self) -> Option<DashaChain> {
        self.at(self.birth_jd)
    }

    /// Mahadashas clipped to begin no earlier than birth - what a reading
    /// actually shows, since the pre-birth portion of the first one is not the
    /// native's to live through.
    pub fn mahadashas_from_birth(&self) -> Vec<&DashaPeriod> {
        self.mahadashas
            .iter()
            .filter(|p| p.end_jd > self.birth_jd)
            .collect()
    }
}

/// Recursively subdivide a period into the nine sub-lords, starting from the
/// parent's own lord.
///
/// Sub-period lengths are proportional: within a mahadasha of L, the
/// antardasha of M runs for `years(L) * years(M) / 120` years. The same
/// proportion applies one level down.
///
/// Boundaries accumulate forward and the final child's end is snapped exactly
/// to the parent's end, so no floating-point residue can leave a gap that
/// `contains` would fall through.
fn subdivide(lord: Graha, start: f64, end: f64, level: u8) -> Vec<DashaPeriod> {
    if level > 3 {
        return Vec::new();
    }
    let total = end - start;
    let base = cycle_index(lord);
    let mut out = Vec::with_capacity(9);
    let mut cursor = start;

    for step in 0..9 {
        let sub = CYCLE[(base + step) % 9];
        let span = total * dasha_years(sub) / TOTAL_YEARS;
        let sub_start = cursor;
        // Snap the last boundary rather than letting rounding decide it.
        let sub_end = if step == 8 { end } else { cursor + span };
        out.push(DashaPeriod {
            level,
            lord: sub,
            start_jd: sub_start,
            end_jd: sub_end,
            children: subdivide(sub, sub_start, sub_end, level + 1),
        });
        cursor = sub_end;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const JD: f64 = 2451545.0;

    #[test]
    fn dasha_years_sum_to_120() {
        let total: f64 = CYCLE.iter().map(|&g| dasha_years(g)).sum();
        assert_eq!(total, TOTAL_YEARS);
    }

    #[test]
    fn birth_at_nakshatra_start_gives_full_balance() {
        let v = Vimshottari::from_parts(JD, Nakshatra::Ashwini, 0.0, YearLength::Julian);
        assert_eq!(v.birth_lord, Graha::Ketu);
        assert!((v.balance_years - 7.0).abs() < 1e-12);
        // Nothing elapsed, so the mahadasha starts exactly at birth.
        assert!((v.mahadashas[0].start_jd - JD).abs() < 1e-9);
    }

    #[test]
    fn half_traversed_nakshatra_halves_the_balance() {
        let v = Vimshottari::from_parts(JD, Nakshatra::Rohini, 0.5, YearLength::Julian);
        assert_eq!(v.birth_lord, Graha::Moon);
        assert!((v.balance_years - 5.0).abs() < 1e-12);
        // The mahadasha began five Julian years before birth.
        let expected_start = JD - 5.0 * 365.25;
        assert!((v.mahadashas[0].start_jd - expected_start).abs() < 1e-9);
    }

    #[test]
    fn mahadashas_are_contiguous_and_correctly_ordered() {
        let v = Vimshottari::from_parts(JD, Nakshatra::Bharani, 0.3, YearLength::Julian);
        assert_eq!(v.birth_lord, Graha::Venus);
        // Venus -> Sun -> Moon -> Mars -> Rahu -> Jupiter -> Saturn -> Mercury -> Ketu -> Venus
        let expected = [
            Graha::Venus, Graha::Sun, Graha::Moon, Graha::Mars, Graha::Rahu,
            Graha::Jupiter, Graha::Saturn, Graha::Mercury, Graha::Ketu, Graha::Venus,
        ];
        for (p, e) in v.mahadashas.iter().zip(expected) {
            assert_eq!(p.lord, e);
        }
        for w in v.mahadashas.windows(2) {
            assert_eq!(w[0].end_jd, w[1].start_jd, "gap between mahadashas");
        }
    }

    #[test]
    fn one_full_cycle_is_exactly_120_years() {
        let v = Vimshottari::from_parts(JD, Nakshatra::Ashwini, 0.0, YearLength::Julian);
        let nine = &v.mahadashas[..9];
        let span = nine[8].end_jd - nine[0].start_jd;
        assert!(
            (span - 120.0 * 365.25).abs() < 1e-6,
            "cycle spanned {span} days"
        );
    }

    #[test]
    fn antardashas_start_with_the_mahadasha_lord_and_tile_it() {
        let v = Vimshottari::from_parts(JD, Nakshatra::Ashwini, 0.0, YearLength::Julian);
        for maha in &v.mahadashas {
            assert_eq!(maha.children.len(), 9);
            assert_eq!(maha.children[0].lord, maha.lord, "antardasha must open with its own lord");
            assert_eq!(maha.children[0].start_jd, maha.start_jd);
            assert_eq!(maha.children[8].end_jd, maha.end_jd);
            for w in maha.children.windows(2) {
                assert_eq!(w[0].end_jd, w[1].start_jd);
            }
        }
    }

    #[test]
    fn antardasha_lengths_follow_the_proportional_rule() {
        let v = Vimshottari::from_parts(JD, Nakshatra::Ashwini, 0.0, YearLength::Julian);
        // Ketu mahadasha = 7 years. Venus antardasha within it:
        // 7 * 20 / 120 = 1.16667 years.
        let ketu = &v.mahadashas[0];
        let venus = ketu.children.iter().find(|c| c.lord == Graha::Venus).unwrap();
        let expected = 7.0 * 20.0 / 120.0 * 365.25;
        assert!(
            (venus.duration_days() - expected).abs() < 1e-6,
            "got {} days, expected {expected}",
            venus.duration_days()
        );
    }

    #[test]
    fn pratyantardashas_tile_their_antardasha() {
        let v = Vimshottari::from_parts(JD, Nakshatra::Ashwini, 0.0, YearLength::Julian);
        for maha in &v.mahadashas {
            for antar in &maha.children {
                assert_eq!(antar.children.len(), 9);
                assert_eq!(antar.children[0].lord, antar.lord);
                assert_eq!(antar.children[0].start_jd, antar.start_jd);
                assert_eq!(antar.children[8].end_jd, antar.end_jd);
                // Depth stops at three levels.
                assert!(antar.children[0].children.is_empty());
            }
        }
    }

    #[test]
    fn no_instant_falls_through_a_boundary_crack() {
        let v = Vimshottari::from_parts(JD, Nakshatra::Ashlesha, 0.618, YearLength::Julian);
        let first = v.mahadashas[0].start_jd;
        let last = v.mahadashas[9].end_jd;
        // Walk the whole tree in ~5-day steps; every instant must resolve.
        let mut jd = first;
        while jd < last {
            assert!(v.at(jd).is_some(), "no chain at JD {jd}");
            jd += 5.0;
        }
        // And just inside each boundary.
        for maha in &v.mahadashas {
            for antar in &maha.children {
                for prat in &antar.children {
                    assert!(v.at(prat.start_jd).is_some(), "gap at {}", prat.start_jd);
                    assert!(v.at(prat.end_jd - 1e-6).is_some());
                }
            }
        }
    }

    #[test]
    fn birth_chain_opens_on_the_janma_lord() {
        let v = Vimshottari::from_parts(JD, Nakshatra::Magha, 0.25, YearLength::Julian);
        let chain = v.at_birth().unwrap();
        assert_eq!(chain.maha, Graha::Ketu, "Magha is ruled by Ketu");
    }

    #[test]
    fn year_length_changes_dates_but_not_structure() {
        let a = Vimshottari::from_parts(JD, Nakshatra::Rohini, 0.4, YearLength::Julian);
        let b = Vimshottari::from_parts(JD, Nakshatra::Rohini, 0.4, YearLength::Savana);
        let lords_a: Vec<_> = a.mahadashas.iter().map(|p| p.lord).collect();
        let lords_b: Vec<_> = b.mahadashas.iter().map(|p| p.lord).collect();
        assert_eq!(lords_a, lords_b, "lord sequence must not depend on year length");
        assert!((a.balance_years - b.balance_years).abs() < 1e-12);
        // But the wall-clock dates must differ, and by a predictable amount.
        // Rohini (Moon, 10y) 40% elapsed, then ten mahadashas totalling 130
        // years, is 126 dasha-years from the start of the tree to its end.
        // Julian minus savana is 5.25 days per year, so 126 * 5.25 = 661.5.
        let drift = (a.mahadashas[9].end_jd - b.mahadashas[9].end_jd).abs();
        assert!(
            (drift - 661.5).abs() < 1e-6,
            "expected 661.5 days of drift, got {drift}"
        );
        // Nearly two years of divergence on the same birth data is exactly why
        // this convention is a recorded setting and not a hard-coded constant.
        assert!(drift / 365.25 > 1.8);
    }

    #[test]
    fn is_deterministic() {
        let a = Vimshottari::from_parts(JD, Nakshatra::Jyeshtha, 0.777, YearLength::Julian);
        for _ in 0..20 {
            assert_eq!(
                Vimshottari::from_parts(JD, Nakshatra::Jyeshtha, 0.777, YearLength::Julian),
                a
            );
        }
    }
}
