//! Upagrahas (shadowy points) and the time lagnas.
//!
//! Specification: `docs/phase13/UPAGRAHA.md`.
//!
//! Two unrelated kinds of thing live here because practice groups them:
//!
//!  - five points at fixed offsets from the Sun. Pure arithmetic, and the
//!    offsets compose into identities a test can check, which is why they are
//!    safe to state from the specification alone (section 3).
//!  - five points found by dividing the day into eight parts and taking the
//!    ascendant when a particular part begins. This reuses the division
//!    `day.rs` already makes for Rahu kalam, so it inherits a sunrise that is
//!    cross-validated against `swetest` rather than starting afresh.
//!
//! The three time lagnas are the same shape as the second group - reckoned
//! from sunrise - so they sit here too.

use std::borrow::Cow;

use crate::chart::Chart;
use crate::day::Vara;
use crate::jaimini::VariantChoice;
use crate::rasi::Rasi;
use lagn_ephem::{norm360, EphemError, Ephemeris, Graha, HouseSystem};
use serde::{Deserialize, Serialize};

/// Degrees, as a decimal, from degrees and arcminutes.
const fn dm(deg: f64, min: f64) -> f64 {
    deg + min / 60.0
}

/// Dhuma's offset from the Sun: 133 degrees 20 minutes.
pub const DHUMA_OFFSET: f64 = dm(133.0, 20.0);
/// Upaketu's offset from Indrachapa: 16 degrees 40 minutes.
pub const UPAKETU_OFFSET: f64 = dm(16.0, 40.0);

/// The five points at fixed offsets from the Sun.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SunOffset {
    Dhuma,
    Vyatipata,
    Parivesha,
    Indrachapa,
    Upaketu,
}

impl SunOffset {
    pub const ALL: [SunOffset; 5] = [
        SunOffset::Dhuma,
        SunOffset::Vyatipata,
        SunOffset::Parivesha,
        SunOffset::Indrachapa,
        SunOffset::Upaketu,
    ];

    pub fn name(self) -> &'static str {
        match self {
            SunOffset::Dhuma => "Dhuma",
            SunOffset::Vyatipata => "Vyatipata",
            SunOffset::Parivesha => "Parivesha",
            SunOffset::Indrachapa => "Indrachapa",
            SunOffset::Upaketu => "Upaketu",
        }
    }

    /// The longitude, given the Sun's sidereal longitude.
    ///
    /// Each is defined in terms of the one before it, exactly as the
    /// specification states them, rather than collapsed into a single offset.
    /// Writing them as a chain is what lets the test assert that the chain
    /// closes - collapsing them first would assume the very thing being
    /// checked.
    pub fn longitude(self, sun: f64) -> f64 {
        let dhuma = norm360(sun + DHUMA_OFFSET);
        let vyatipata = norm360(360.0 - dhuma);
        let parivesha = norm360(vyatipata + 180.0);
        let indrachapa = norm360(360.0 - parivesha);
        let upaketu = norm360(indrachapa + UPAKETU_OFFSET);
        match self {
            SunOffset::Dhuma => dhuma,
            SunOffset::Vyatipata => vyatipata,
            SunOffset::Parivesha => parivesha,
            SunOffset::Indrachapa => indrachapa,
            SunOffset::Upaketu => upaketu,
        }
    }
}

/// The five points found from the eight-part division of the day or night.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DayPart {
    Kaala,
    Mrityu,
    Ardhaprahara,
    Yamaghantaka,
    Gulika,
}

impl DayPart {
    pub const ALL: [DayPart; 5] = [
        DayPart::Kaala,
        DayPart::Mrityu,
        DayPart::Ardhaprahara,
        DayPart::Yamaghantaka,
        DayPart::Gulika,
    ];

    pub fn name(self) -> &'static str {
        match self {
            DayPart::Kaala => "Kaala",
            DayPart::Mrityu => "Mrityu",
            DayPart::Ardhaprahara => "Ardhaprahara",
            DayPart::Yamaghantaka => "Yamaghantaka",
            // The point South Indian practice calls Mandi (V-13-17).
            DayPart::Gulika => "Gulika",
        }
    }

    /// The graha whose part of the day gives this point.
    pub fn ruler(self) -> Graha {
        match self {
            DayPart::Kaala => Graha::Sun,
            DayPart::Mrityu => Graha::Mars,
            DayPart::Ardhaprahara => Graha::Mercury,
            DayPart::Yamaghantaka => Graha::Jupiter,
            DayPart::Gulika => Graha::Saturn,
        }
    }
}

/// The weekday lords in the order the parts follow.
const PART_ORDER: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// Which eighth of the period a graha rules, 1-based, or None for the eighth
/// part, which is unruled.
///
/// The first part belongs to `first`, and the rest follow `PART_ORDER`
/// wrapping round. Seven parts, seven lords; the eighth is nobody's.
fn part_of(first: Graha, graha: Graha) -> Option<u8> {
    let start = PART_ORDER.iter().position(|&g| g == first)?;
    let want = PART_ORDER.iter().position(|&g| g == graha)?;
    let n = (want as i32 - start as i32).rem_euclid(7) as u8;
    Some(n + 1)
}

/// The lord the night's parts start from: the fifth from the weekday lord,
/// counting the weekday lord as the first (V-13-15).
fn night_first(vara_lord: Graha) -> Graha {
    let i = PART_ORDER.iter().position(|&g| g == vara_lord).expect("a vara lord is one of the seven");
    PART_ORDER[(i + 4) % 7]
}

/// The three lagnas that advance from sunrise at a fixed rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeLagna {
    Bhava,
    Hora,
    Ghati,
}

impl TimeLagna {
    pub const ALL: [TimeLagna; 3] = [TimeLagna::Bhava, TimeLagna::Hora, TimeLagna::Ghati];

    pub fn name(self) -> &'static str {
        match self {
            TimeLagna::Bhava => "Bhava lagna",
            TimeLagna::Hora => "Hora lagna",
            TimeLagna::Ghati => "Ghati lagna",
        }
    }

    /// Degrees advanced per hour since sunrise.
    pub const fn degrees_per_hour(self) -> f64 {
        match self {
            // One sign per two hours, per hour, per 24 minutes.
            TimeLagna::Bhava => 15.0,
            TimeLagna::Hora => 30.0,
            TimeLagna::Ghati => 75.0,
        }
    }

    /// The lagna `hours` after a sunrise at which the Sun stood at `sunrise_sun`.
    pub fn longitude(self, sunrise_sun: f64, hours: f64) -> f64 {
        norm360(sunrise_sun + self.degrees_per_hour() * hours)
    }
}

/// One computed point, with where it falls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// Cow rather than &'static str so the type round-trips through JSON like
    /// every other chart type, as `VariantChoice` does.
    pub name: Cow<'static, str>,
    pub longitude: f64,
    pub rasi: Rasi,
    pub degrees_in_rasi: f64,
    /// Whole-sign house from the lagna.
    pub house: u8,
}

fn point(name: &'static str, longitude: f64, lagna: Rasi) -> Point {
    let longitude = norm360(longitude);
    let rasi = Rasi::from_longitude(longitude);
    Point {
        name: Cow::Borrowed(name),
        longitude,
        rasi,
        degrees_in_rasi: Rasi::degrees_within(longitude),
        house: lagna.houses_to(rasi),
    }
}

/// A day-division point, with the working shown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DayPartPoint {
    pub point: Point,
    pub ruler: Graha,
    /// Which eighth of the period, 1 to 7.
    pub part: u8,
    /// True when the birth fell at night, so the night was divided.
    pub at_night: bool,
}

/// Everything 13C computes for a chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Upagrahas {
    /// True when the birth fell between sunset and the next sunrise.
    pub at_night: bool,
    pub vara: Vara,
    /// Hours from the governing sunrise to the moment of birth.
    pub hours_since_sunrise: f64,
    /// The Sun's sidereal longitude at that sunrise.
    pub sunrise_sun: f64,
    pub sun_offsets: Vec<Point>,
    pub day_parts: Vec<DayPartPoint>,
    pub time_lagnas: Vec<Point>,
    pub variants: Vec<VariantChoice>,
}

pub const VARIANTS: [VariantChoice; 4] = [
    VariantChoice {
        id: Cow::Borrowed("V-13-14"),
        question: Cow::Borrowed("which moment of its part gives an upagraha"),
        chosen: Cow::Borrowed("the start of the part"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-15"),
        question: Cow::Borrowed("night-birth part sequence"),
        chosen: Cow::Borrowed("starts from the lord 5th from the weekday lord"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-16"),
        question: Cow::Borrowed("Sun longitude the time lagnas start from"),
        chosen: Cow::Borrowed("the Sun at that day's sunrise"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-17"),
        question: Cow::Borrowed("Gulika and Mandi"),
        chosen: Cow::Borrowed("the same point"),
    },
];

/// Hours in a day, for turning a Julian Day difference into hours.
const HOURS: f64 = 24.0;

impl Upagrahas {
    /// Compute every point for a chart.
    ///
    /// Needs the ephemeris because the day-division points are ascendants at
    /// moments other than birth, and the time lagnas start from the Sun at
    /// sunrise rather than at birth (V-13-16).
    pub fn compute(eph: &Ephemeris, chart: &Chart) -> Result<Upagrahas, EphemError> {
        let (lat, lon) = (chart.birth.latitude, chart.birth.longitude);
        let birth = chart.jd_ut;
        let lagna = chart.lagna.rasi;

        // The sunrise that governs this moment, and the sunset after it. A
        // birth before that day's sunrise belongs to the previous day's night,
        // so the search starts half a day back and the pair is chosen by where
        // the birth actually falls.
        let today = eph.sun_day(birth - 0.5, lat, lon)?;
        let (at_night, period_start, period_end, sunrise_jd) = if birth < today.sunrise_jd {
            // Before sunrise: the night that began at the previous sunset.
            let prev = eph.sun_day(birth - 1.5, lat, lon)?;
            (true, prev.sunset_jd, today.sunrise_jd, prev.sunrise_jd)
        } else if birth >= today.sunset_jd {
            // After sunset: the night running to tomorrow's sunrise.
            let next = eph.sun_day(today.sunset_jd + 0.1, lat, lon)?;
            (true, today.sunset_jd, next.sunrise_jd, today.sunrise_jd)
        } else {
            (false, today.sunrise_jd, today.sunset_jd, today.sunrise_jd)
        };

        let vara = Vara::from_sunrise_jd(sunrise_jd);
        let vara_lord = vara.lord();
        let first = if at_night { night_first(vara_lord) } else { vara_lord };

        // The Sun at that sunrise, which is where the time lagnas start.
        let sunrise_sun = eph.position(sunrise_jd, Graha::Sun)?.longitude;
        let hours_since_sunrise = (birth - sunrise_jd) * HOURS;

        let sun_offsets = SunOffset::ALL
            .iter()
            .map(|&o| point(o.name(), o.longitude(chart.placement(Graha::Sun).longitude), lagna))
            .collect();

        let eighth = (period_end - period_start) / 8.0;
        let mut day_parts = Vec::with_capacity(5);
        for kind in DayPart::ALL {
            let part = part_of(first, kind.ruler()).expect("every ruler is one of the seven");
            // The ascendant when the part begins (V-13-14).
            let at = period_start + eighth * (part as f64 - 1.0);
            let (angles, _) = eph.angles(at, lat, lon, HouseSystem::WholeSign)?;
            day_parts.push(DayPartPoint {
                point: point(kind.name(), angles.ascendant, lagna),
                ruler: kind.ruler(),
                part,
                at_night,
            });
        }

        let time_lagnas = TimeLagna::ALL
            .iter()
            .map(|&l| point(l.name(), l.longitude(sunrise_sun, hours_since_sunrise), lagna))
            .collect();

        Ok(Upagrahas {
            at_night,
            vara,
            hours_since_sunrise,
            sunrise_sun,
            sun_offsets,
            day_parts,
            time_lagnas,
            variants: VARIANTS.to_vec(),
        })
    }

    /// Gulika, which practice reaches for far more often than the rest.
    pub fn gulika(&self) -> &DayPartPoint {
        self.day_parts
            .iter()
            .find(|p| p.ruler == Graha::Saturn)
            .expect("Gulika is always computed")
    }
}
