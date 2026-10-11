//! Yogini dasha: the 36-year nakshatra cycle of eight yoginis.
//!
//! Specification: `docs/phase13/YOGINI.md`.
//!
//! The reason this one of 13D's dozen conditional dashas could be built from
//! a specification alone is its structure. The eight periods are the
//! consecutive integers 1 to 8, and they sum to 36, which is the cycle length
//! by definition. A mistranscribed period breaks the run and the total at
//! once, so the table checks itself. Ashtottari and the rest have irregular
//! periods, where a wrong one produces a wrong total that looks exactly as
//! plausible as the right one.

use std::borrow::Cow;

use crate::chart::Chart;
use crate::dasha::YearLength;
use crate::jaimini::VariantChoice;
use crate::nakshatra::Nakshatra;
use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

/// The eight yoginis, in sequence.
///
/// Ordered, because the sequence is part of what a yogini is: the derived
/// order is the order the dasha runs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Yogini {
    Mangala,
    Pingala,
    Dhanya,
    Bhramari,
    Bhadrika,
    Ulka,
    Siddha,
    Sankata,
}

impl Yogini {
    /// All eight, in the order the dasha runs.
    pub const ALL: [Yogini; 8] = [
        Yogini::Mangala,
        Yogini::Pingala,
        Yogini::Dhanya,
        Yogini::Bhramari,
        Yogini::Bhadrika,
        Yogini::Ulka,
        Yogini::Siddha,
        Yogini::Sankata,
    ];

    /// 1 to 8, as the expositions number them.
    pub const fn number(self) -> u8 {
        match self {
            Yogini::Mangala => 1,
            Yogini::Pingala => 2,
            Yogini::Dhanya => 3,
            Yogini::Bhramari => 4,
            Yogini::Bhadrika => 5,
            Yogini::Ulka => 6,
            Yogini::Siddha => 7,
            Yogini::Sankata => 8,
        }
    }

    /// Period in years.
    ///
    /// Equal to the yogini's own number, which is why the eight sum to 36 and
    /// why a wrong value cannot hide: it would break the run of consecutive
    /// integers as well as the total.
    pub const fn years(self) -> f64 {
        self.number() as f64
    }

    /// The graha that lords the yogini. Sankata's is Rahu (V-13-36).
    pub const fn lord(self) -> Graha {
        match self {
            Yogini::Mangala => Graha::Moon,
            Yogini::Pingala => Graha::Sun,
            Yogini::Dhanya => Graha::Jupiter,
            Yogini::Bhramari => Graha::Mars,
            Yogini::Bhadrika => Graha::Mercury,
            Yogini::Ulka => Graha::Saturn,
            Yogini::Siddha => Graha::Venus,
            Yogini::Sankata => Graha::Rahu,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Yogini::Mangala => "Mangala",
            Yogini::Pingala => "Pingala",
            Yogini::Dhanya => "Dhanya",
            Yogini::Bhramari => "Bhramari",
            Yogini::Bhadrika => "Bhadrika",
            Yogini::Ulka => "Ulka",
            Yogini::Siddha => "Siddha",
            Yogini::Sankata => "Sankata",
        }
    }

    /// The `n`th yogini from this one, counting this one as 1.
    pub fn nth_from(self, n: u8) -> Yogini {
        let i = (self.number() as usize - 1 + n as usize - 1) % 8;
        Yogini::ALL[i]
    }
}

/// Total years in one pass through the eight. Thirty-six by construction.
pub const CYCLE_YEARS: f64 = 36.0;

/// The yogini a janma nakshatra starts the dasha in (V-13-33).
///
/// `(nakshatra number + 3) mod 8`, with a remainder of 0 meaning the eighth.
/// The nakshatra number is Ashwini = 1.
pub fn starting_yogini(nakshatra: Nakshatra) -> Yogini {
    let serial = nakshatra.index() as u32 + 1;
    let r = (serial + 3) % 8;
    Yogini::ALL[(if r == 0 { 8 } else { r }) as usize - 1]
}

/// One Yogini period, maha or antar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YoginiPeriod {
    /// 1 = mahadasha, 2 = antardasha.
    pub level: u8,
    pub yogini: Yogini,
    pub lord: Graha,
    pub start_jd: f64,
    pub end_jd: f64,
    /// Which pass through the eight this belongs to, from 1.
    pub cycle: u32,
    pub children: Vec<YoginiPeriod>,
}

impl YoginiPeriod {
    pub fn duration_days(&self) -> f64 {
        self.end_jd - self.start_jd
    }

    pub fn contains(&self, jd: f64) -> bool {
        jd >= self.start_jd && jd < self.end_jd
    }
}

/// The running chain at an instant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct YoginiChain {
    pub maha: Yogini,
    pub antar: Yogini,
}

/// Years from birth the engine fills. Matches Vimshottari's horizon, because
/// a 36-year cycle would otherwise run out well inside a long life.
pub const HORIZON_YEARS: f64 = 120.0;

/// A computed Yogini dasha.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YoginiDasha {
    pub birth_jd: f64,
    pub year_length: YearLength,
    pub janma_nakshatra: Nakshatra,
    /// The yogini running at birth.
    pub birth_yogini: Yogini,
    /// Years of that yogini still unelapsed at the moment of birth.
    pub balance_years: f64,
    /// Mahadashas in sequence. The first begins *before* birth: its opening
    /// portion had already elapsed, and keeping it is what makes the
    /// antardasha boundaries inside it correct.
    pub periods: Vec<YoginiPeriod>,
    pub variants: Vec<VariantChoice>,
}

pub const VARIANTS: [VariantChoice; 5] = [
    VariantChoice {
        id: Cow::Borrowed("V-13-33"),
        question: Cow::Borrowed("the starting rule"),
        chosen: Cow::Borrowed("(nakshatra + 3) mod 8, 0 meaning the 8th"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-34"),
        question: Cow::Borrowed("balance at birth"),
        chosen: Cow::Borrowed("proportional to the untraversed part of the janma nakshatra"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-35"),
        question: Cow::Borrowed("antardasha order"),
        chosen: Cow::Borrowed("the eight from the mahadasha's own yogini, proportional"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-36"),
        question: Cow::Borrowed("Sankata's lord"),
        chosen: Cow::Borrowed("Rahu"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-37"),
        question: Cow::Borrowed("when Yogini applies"),
        chosen: Cow::Borrowed("always shown, as an alternative to Vimshottari"),
    },
];

impl YoginiDasha {
    pub fn compute(chart: &Chart) -> YoginiDasha {
        let pos = chart.janma_nakshatra();
        let janma_nakshatra = pos.nakshatra;
        let birth_yogini = starting_yogini(janma_nakshatra);
        let year_days = chart.settings.year_length.days();
        let birth_jd = chart.jd_ut;

        // What remains of the running yogini at birth (V-13-34).
        let balance_years = birth_yogini.years() * (1.0 - pos.fraction_traversed);

        // The first period began before birth, by the elapsed part. Elapsed
        // time is accumulated in *years* and every boundary computed from
        // that origin, for the reason CHARA-DASHA.md section 9 records:
        // accumulating days drifts, and consecutive boundaries from the same
        // expression are bitwise equal rather than merely close.
        let origin_years = -(birth_yogini.years() - balance_years);

        let mut periods = Vec::new();
        let mut elapsed = origin_years;
        let mut cycle = 1u32;
        let mut yogini = birth_yogini;

        while elapsed < HORIZON_YEARS {
            let years = yogini.years();
            let start = birth_jd + elapsed * year_days;
            let end = birth_jd + (elapsed + years) * year_days;

            // Eight antardashas, opening on the mahadasha's own yogini and
            // proportional to their years out of 36 (V-13-35). They sum to
            // the mahadasha because the eight periods sum to 36.
            let mut children = Vec::with_capacity(8);
            let mut inner = elapsed;
            for n in 1..=8u8 {
                let sub = yogini.nth_from(n);
                let span = years * sub.years() / CYCLE_YEARS;
                children.push(YoginiPeriod {
                    level: 2,
                    yogini: sub,
                    lord: sub.lord(),
                    start_jd: birth_jd + inner * year_days,
                    // The last child closes exactly on the parent.
                    end_jd: if n == 8 { end } else { birth_jd + (inner + span) * year_days },
                    cycle,
                    children: Vec::new(),
                });
                inner += span;
            }

            periods.push(YoginiPeriod {
                level: 1,
                yogini,
                lord: yogini.lord(),
                start_jd: start,
                end_jd: end,
                cycle,
                children,
            });

            elapsed += years;
            yogini = yogini.nth_from(2);
            if yogini == birth_yogini {
                cycle += 1;
            }
        }

        YoginiDasha {
            birth_jd,
            year_length: chart.settings.year_length,
            janma_nakshatra,
            birth_yogini,
            balance_years,
            periods,
            variants: VARIANTS.to_vec(),
        }
    }

    /// The maha and antar yoginis running at an instant, if it is in range.
    pub fn at(&self, jd: f64) -> Option<YoginiChain> {
        let maha = self.periods.iter().find(|p| p.contains(jd))?;
        let antar = maha.children.iter().find(|c| c.contains(jd))?;
        Some(YoginiChain { maha: maha.yogini, antar: antar.yogini })
    }
}
