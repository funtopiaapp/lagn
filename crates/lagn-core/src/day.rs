//! A day's own timings: the panchanga and the inauspicious parts of the day.
//!
//! This is the other time scale the engine works at. A dasha stretch is years
//! long; these are the hour-and-a-half segments a South Indian household
//! actually consults before stepping out, and they are measured from sunrise,
//! not from midnight.
//!
//! Everything here is arithmetic on a sunrise and a sunset. Nothing is
//! interpreted and nothing is scored: a panchangam states what the day is, and
//! what it is, is computed.

use serde::{Deserialize, Serialize};

use crate::nakshatra::NakshatraPosition;

/// The weekday, counted as the tradition counts it: from sunrise, so the hours
/// after midnight still belong to the day that began at the previous sunrise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vara {
    Ravi,
    Soma,
    Mangala,
    Budha,
    Guru,
    Shukra,
    Shani,
}

impl Vara {
    /// Sunday first, matching the classical order of the week.
    pub const ALL: [Vara; 7] =
        [Vara::Ravi, Vara::Soma, Vara::Mangala, Vara::Budha, Vara::Guru, Vara::Shukra, Vara::Shani];

    /// Sanskrit name, as a panchangam prints it.
    pub fn name(self) -> &'static str {
        match self {
            Vara::Ravi => "Ravivara",
            Vara::Soma => "Somavara",
            Vara::Mangala => "Mangalavara",
            Vara::Budha => "Budhavara",
            Vara::Guru => "Guruvara",
            Vara::Shukra => "Shukravara",
            Vara::Shani => "Shanivara",
        }
    }

    /// Tamil name, as a Tamil panchangam prints it.
    pub fn tamil_name(self) -> &'static str {
        match self {
            Vara::Ravi => "Nyayiru",
            Vara::Soma => "Thingal",
            Vara::Mangala => "Sevvai",
            Vara::Budha => "Budhan",
            Vara::Guru => "Viyazhan",
            Vara::Shukra => "Velli",
            Vara::Shani => "Sani",
        }
    }

    /// English name, for a reader who knows neither.
    pub fn english(self) -> &'static str {
        match self {
            Vara::Ravi => "Sunday",
            Vara::Soma => "Monday",
            Vara::Mangala => "Tuesday",
            Vara::Budha => "Wednesday",
            Vara::Guru => "Thursday",
            Vara::Shukra => "Friday",
            Vara::Shani => "Saturday",
        }
    }

    /// The graha that lords the day.
    pub fn lord(self) -> crate::Graha {
        use crate::Graha::*;
        match self {
            Vara::Ravi => Sun,
            Vara::Soma => Moon,
            Vara::Mangala => Mars,
            Vara::Budha => Mercury,
            Vara::Guru => Jupiter,
            Vara::Shukra => Venus,
            Vara::Shani => Saturn,
        }
    }

    /// The weekday containing a sunrise. JD 0.0 was a Monday noon, so the
    /// integer part of `jd + 0.5` counts days from a Monday.
    pub fn from_sunrise_jd(sunrise_jd: f64) -> Vara {
        // (jd + 1.5) mod 7 == 0 on a Sunday, which is the classical first day.
        let idx = ((sunrise_jd + 1.5).floor() as i64).rem_euclid(7) as usize;
        Vara::ALL[idx]
    }
}

/// One named segment of the daylight, with the part of the day it occupies.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DaySegment {
    pub start_jd: f64,
    pub end_jd: f64,
    /// 1..=8 for the eighth-parts, 1..=15 for Abhijit's fifteenth.
    pub part: u8,
}

/// The parts of a day that tradition marks out.
///
/// Rahu kalam, Yamagandam and Kuligai each take one eighth of the daylight,
/// and which eighth depends on the weekday. Abhijit muhurta is the eighth
/// fifteenth - the middle of the day - and is the one that is auspicious.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DayTimings {
    pub sunrise_jd: f64,
    pub sunset_jd: f64,
    pub vara: Vara,
    pub rahu_kalam: DaySegment,
    pub yamagandam: DaySegment,
    pub kuligai: DaySegment,
    pub abhijit: DaySegment,
}

/// Which eighth of the daylight each segment takes, by weekday.
///
/// These tables are the ones Tamil and Kerala panchangams print. They are
/// fixed data, not a derivation: there is no formula behind them to get wrong,
/// only a table to copy correctly.
const RAHU: [u8; 7] = [8, 2, 7, 5, 6, 4, 3];
const YAMA: [u8; 7] = [5, 4, 3, 2, 1, 7, 6];
const KULI: [u8; 7] = [7, 6, 5, 4, 3, 2, 1];

/// The day's segments from its sunrise and sunset.
///
/// Both are Julian Day in UT; the segments come back the same way, so no
/// calendar arithmetic happens here or anywhere above this.
pub fn day_timings(sunrise_jd: f64, sunset_jd: f64) -> DayTimings {
    let vara = Vara::from_sunrise_jd(sunrise_jd);
    let i = Vara::ALL.iter().position(|v| *v == vara).expect("every vara is in ALL");
    let daylight = sunset_jd - sunrise_jd;

    // The nth eighth of the daylight, n being 1-based as the tables are.
    let eighth = |n: u8| {
        let w = daylight / 8.0;
        DaySegment {
            start_jd: sunrise_jd + w * (n as f64 - 1.0),
            end_jd: sunrise_jd + w * n as f64,
            part: n,
        }
    };

    // Abhijit is the eighth of fifteen parts: the middle of the day, and the
    // one segment here that is auspicious rather than avoided.
    let fifteenth = daylight / 15.0;
    let abhijit = DaySegment {
        start_jd: sunrise_jd + fifteenth * 7.0,
        end_jd: sunrise_jd + fifteenth * 8.0,
        part: 8,
    };

    DayTimings {
        sunrise_jd,
        sunset_jd,
        vara,
        rahu_kalam: eighth(RAHU[i]),
        yamagandam: eighth(YAMA[i]),
        kuligai: eighth(KULI[i]),
        abhijit,
    }
}

// ---------------------------------------------------------------------------
// Panchanga
// ---------------------------------------------------------------------------

/// The five limbs of the day. Each is a function of the Sun's and Moon's
/// sidereal longitudes, so each is computed, never looked up.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Panchanga {
    /// 1..=30. 1-15 are the waxing fortnight, 16-30 the waning.
    pub tithi: u8,
    /// The Moon's nakshatra and pada.
    pub nakshatra: NakshatraPosition,
    /// 1..=27.
    pub yoga: u8,
    /// 1..=60.
    pub karana: u8,
    pub vara: Vara,
}

/// Name of a tithi, 1..=30.
pub fn tithi_name(tithi: u8) -> &'static str {
    const NAMES: [&str; 15] = [
        "Prathamai", "Dvitiyai", "Tritiyai", "Chaturthi", "Panchami", "Shashthi", "Saptami",
        "Ashtami", "Navami", "Dashami", "Ekadashi", "Dvadashi", "Trayodashi", "Chaturdashi",
        "Purnima",
    ];
    match tithi {
        15 => "Purnima",
        30 => "Amavasai",
        t if (1..=14).contains(&t) => NAMES[(t - 1) as usize],
        t if (16..=29).contains(&t) => NAMES[(t - 16) as usize],
        _ => "unknown",
    }
}

/// Whether a tithi falls in the waxing or waning fortnight.
pub fn paksha(tithi: u8) -> &'static str {
    if tithi <= 15 { "Shukla paksha (waxing)" } else { "Krishna paksha (waning)" }
}

const YOGA_NAMES: [&str; 27] = [
    "Vishkambha", "Preeti", "Ayushman", "Saubhagya", "Shobhana", "Atiganda", "Sukarma", "Dhriti",
    "Shoola", "Ganda", "Vriddhi", "Dhruva", "Vyaghata", "Harshana", "Vajra", "Siddhi",
    "Vyatipata", "Variyana", "Parigha", "Shiva", "Siddha", "Sadhya", "Shubha", "Shukla",
    "Brahma", "Indra", "Vaidhriti",
];

/// Name of a yoga, 1..=27.
pub fn yoga_name(yoga: u8) -> &'static str {
    YOGA_NAMES.get((yoga.max(1) - 1) as usize).copied().unwrap_or("unknown")
}

/// Name of a karana, 1..=60.
///
/// The first karana of the lunar month stands alone, then seven repeat eight
/// times, and the last three stand alone at the end.
pub fn karana_name(karana: u8) -> &'static str {
    const MOVABLE: [&str; 7] =
        ["Bava", "Balava", "Kaulava", "Taitila", "Garaja", "Vanija", "Vishti"];
    match karana {
        1 => "Kimstughna",
        58 => "Shakuni",
        59 => "Chatushpada",
        60 => "Naga",
        k if (2..=57).contains(&k) => MOVABLE[((k - 2) % 7) as usize],
        _ => "unknown",
    }
}

/// The panchanga at an instant, from the two sidereal longitudes that define
/// it and the sunrise whose day it belongs to.
///
/// `sun` and `moon` are sidereal ecliptic longitudes in degrees. The elongation
/// of the Moon from the Sun gives the tithi and the karana; their sum gives the
/// yoga; the Moon alone gives the nakshatra.
pub fn panchanga(sun: f64, moon: f64, sunrise_jd: f64) -> Panchanga {
    let norm = |d: f64| d.rem_euclid(360.0);
    let elongation = norm(moon - sun);
    // A tithi is 12 degrees of elongation, a karana half of one.
    let tithi = (elongation / 12.0).floor() as u8 + 1;
    let karana = (elongation / 6.0).floor() as u8 + 1;
    // A yoga is 13 degrees 20 minutes of the two longitudes summed.
    let yoga = (norm(sun + moon) / (360.0 / 27.0)).floor() as u8 + 1;
    Panchanga {
        tithi: tithi.min(30),
        nakshatra: NakshatraPosition::from_longitude(moon),
        yoga: yoga.min(27),
        karana: karana.min(60),
        vara: Vara::from_sunrise_jd(sunrise_jd),
    }
}
