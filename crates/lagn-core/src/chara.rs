//! Chara dasha: the Jaimini rasi dasha.
//!
//! Specification: `docs/phase13/CHARA-DASHA.md`.
//!
//! Unlike Vimshottari this is a dasha of *signs*, not of grahas, and it
//! starts at birth rather than part-way through a period. A rasi dasha is
//! reckoned from the lagna, so there is no fraction of a nakshatra already
//! traversed to carry forward.
//!
//! Two directions are in play and they are not the same one, which is the
//! single easiest thing to get wrong here. The order the signs are visited in
//! comes from the *lagna's* parity (section 3). The direction used to measure
//! how long a given sign's dasha lasts comes from *that sign's* parity
//! (section 4). V-13-9 and V-13-10 record both, because a practitioner
//! comparing against other software will meet software that chooses
//! differently.

use std::borrow::Cow;

use crate::chart::Chart;
use crate::dasha::YearLength;
use crate::jaimini::VariantChoice;
use crate::rasi::Rasi;
use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

/// Which way round the zodiac a sequence or a count runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Zodiacal: Mesha to Vrishabha to Mithuna.
    Direct,
    /// Anti-zodiacal.
    Reverse,
}

impl Direction {
    /// The direction a sign's parity implies: odd signs forward, even back.
    pub fn of(rasi: Rasi) -> Direction {
        if rasi.is_odd() { Direction::Direct } else { Direction::Reverse }
    }

    /// The `n`th sign from `start` in this direction, counting `start` as 1.
    pub fn nth_from(self, start: Rasi, n: u8) -> Rasi {
        let step = n as i32 - 1;
        match self {
            Direction::Direct => Rasi::from_index(start.index() as i32 + step),
            Direction::Reverse => Rasi::from_index(start.index() as i32 - step),
        }
    }

    /// Signs from `start` to `target` in this direction, counting `start` as
    /// 1, so the result is 1 to 12.
    pub fn count(self, start: Rasi, target: Rasi) -> u8 {
        let diff = match self {
            Direction::Direct => target.index() as i32 - start.index() as i32,
            Direction::Reverse => start.index() as i32 - target.index() as i32,
        };
        diff.rem_euclid(12) as u8 + 1
    }
}

/// The years a sign's dasha runs for, and the working behind it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CharaLength {
    pub rasi: Rasi,
    pub lord: Graha,
    /// The sign the lord occupies.
    pub lord_rasi: Rasi,
    /// Which way the count ran - this sign's own parity (V-13-10).
    pub direction: Direction,
    /// 1 to 12, counting the sign itself as 1.
    pub count: u8,
    pub years: f64,
    /// True when the lord is at home and the period is 12 rather than 0
    /// (V-13-13).
    pub lord_at_home: bool,
}

/// How long a sign's dasha lasts. Section 4.
pub fn length(chart: &Chart, rasi: Rasi) -> CharaLength {
    let lord = rasi.lord();
    let lord_rasi = chart.placement(lord).rasi;
    let direction = Direction::of(rasi);
    let count = direction.count(rasi, lord_rasi);
    let lord_at_home = count == 1;
    // A sign whose lord sits in it would otherwise get nothing, and a
    // zero-length period cannot stay in a sequence.
    let years = if lord_at_home { 12.0 } else { count as f64 - 1.0 };
    CharaLength { rasi, lord, lord_rasi, direction, count, years, lord_at_home }
}

/// One Chara period, maha or antar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharaPeriod {
    /// 1 = mahadasha, 2 = antardasha.
    pub level: u8,
    pub rasi: Rasi,
    pub start_jd: f64,
    pub end_jd: f64,
    /// Which cycle through the twelve signs this belongs to, from 1.
    pub cycle: u32,
    pub children: Vec<CharaPeriod>,
}

impl CharaPeriod {
    pub fn duration_days(&self) -> f64 {
        self.end_jd - self.start_jd
    }

    pub fn contains(&self, jd: f64) -> bool {
        jd >= self.start_jd && jd < self.end_jd
    }
}

/// The running Chara chain at an instant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CharaChain {
    pub maha: Rasi,
    pub antar: Rasi,
}

/// The horizon the engine fills, in years from birth. Matches Vimshottari's
/// 120, because one Chara cycle can be as short as twelve years and a reading
/// has to reach well past it.
pub const HORIZON_YEARS: f64 = 120.0;

/// A computed Chara dasha.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharaDasha {
    pub birth_jd: f64,
    pub year_length: YearLength,
    pub lagna: Rasi,
    /// The order the signs are visited in, from the lagna's parity (V-13-9).
    pub direction: Direction,
    /// The twelve signs in sequence, with each one's length and its working.
    pub lengths: Vec<CharaLength>,
    /// Mahadashas from birth, enough of them to cover [`HORIZON_YEARS`].
    pub periods: Vec<CharaPeriod>,
    pub variants: Vec<VariantChoice>,
}

/// The variant defaults in force for Chara dasha. Section 6 of the spec.
pub const VARIANTS: [VariantChoice; 5] = [
    VariantChoice {
        id: Cow::Borrowed("V-13-9"),
        question: Cow::Borrowed("direction of the sequence"),
        chosen: Cow::Borrowed("by the lagna's odd/even parity"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-10"),
        question: Cow::Borrowed("direction used to count a sign's length"),
        chosen: Cow::Borrowed("the parity of that sign"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-11"),
        question: Cow::Borrowed("antardasha direction"),
        chosen: Cow::Borrowed("the main sequence's direction"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-12"),
        question: Cow::Borrowed("lord of a dual-lorded sign"),
        chosen: Cow::Borrowed("sole traditional lord (Kuja, Shani)"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-13"),
        question: Cow::Borrowed("period when the lord is in its own sign"),
        chosen: Cow::Borrowed("12 years"),
    },
];

impl CharaDasha {
    pub fn compute(chart: &Chart) -> CharaDasha {
        let lagna = chart.lagna.rasi;
        let direction = Direction::of(lagna);
        let year_days = chart.settings.year_length.days();
        let birth_jd = chart.jd_ut;

        // The twelve signs in the order they are visited, each with its own
        // length. The order is the lagna's direction; the lengths are each
        // sign's own. Keeping these two apart is the whole of section 4.
        let sequence: Vec<Rasi> = (1..=12).map(|n| direction.nth_from(lagna, n)).collect();
        let lengths: Vec<CharaLength> = sequence.iter().map(|&r| length(chart, r)).collect();

        let mut periods = Vec::new();
        // Elapsed time is accumulated in *years*, not in Julian days, and every
        // boundary is computed from birth as `birth + elapsed * year_days`.
        //
        // Two reasons, both found by the oracle. Accumulating days drifts: after
        // two dozen periods the absolute dates are an ulp or so out, and when a
        // cycle length happens to divide the horizon exactly the drift decided
        // whether a whole extra cycle was emitted. Accumulating years is exact -
        // the lengths are small integers - and consecutive boundaries come from
        // the identical expression, so a period's end and the next one's start
        // are bitwise equal rather than merely close.
        let mut elapsed = 0.0f64;
        let mut cycle = 1u32;
        // A cycle can be as short as twelve years, so repeat until the horizon
        // is covered rather than assuming one pass is enough.
        while elapsed < HORIZON_YEARS {
            for l in &lengths {
                let start = birth_jd + elapsed * year_days;
                let end = birth_jd + (elapsed + l.years) * year_days;
                // Twelve equal antardashas, beginning with the sign itself and
                // running in the sequence's direction (V-13-11).
                let children = (1..=12u8)
                    .map(|n| {
                        let f = (n as f64 - 1.0) / 12.0;
                        CharaPeriod {
                            level: 2,
                            rasi: direction.nth_from(l.rasi, n),
                            start_jd: birth_jd + (elapsed + l.years * f) * year_days,
                            // The last child ends exactly where the parent does:
                            // accumulating twelfths would leave a rounding crumb
                            // on the boundary.
                            end_jd: if n == 12 {
                                end
                            } else {
                                birth_jd + (elapsed + l.years * (n as f64 / 12.0)) * year_days
                            },
                            cycle,
                            children: Vec::new(),
                        }
                    })
                    .collect();
                periods.push(CharaPeriod {
                    level: 1,
                    rasi: l.rasi,
                    start_jd: start,
                    end_jd: end,
                    cycle,
                    children,
                });
                elapsed += l.years;
            }
            cycle += 1;
        }

        CharaDasha {
            birth_jd,
            year_length: chart.settings.year_length,
            lagna,
            direction,
            lengths,
            periods,
            variants: VARIANTS.to_vec(),
        }
    }

    /// Total years in one pass through the twelve signs.
    pub fn cycle_years(&self) -> f64 {
        self.lengths.iter().map(|l| l.years).sum()
    }

    /// The maha and antar signs running at an instant, if it is in range.
    pub fn at(&self, jd: f64) -> Option<CharaChain> {
        let maha = self.periods.iter().find(|p| p.contains(jd))?;
        let antar = maha.children.iter().find(|c| c.contains(jd))?;
        Some(CharaChain { maha: maha.rasi, antar: antar.rasi })
    }
}
