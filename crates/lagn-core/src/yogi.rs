//! Yogi, Avayogi and the Yoga sphuta.
//!
//! Specification: `docs/phase13/YOGI.md`.
//!
//! Three quantities held together by two identities. The sphuta's offset of
//! 93 degrees 20 minutes is exactly seven nakshatras, so the Yogi's lord sits
//! seven places along the nine-lord Vimshottari cycle from the lord of the
//! nakshatra `Sun + Moon` falls in; and the Avayogi, being the sixth
//! nakshatra from the Yogi's, always sits five places past the Yogi.
//!
//! Neither identity is a separate rule to remember. Both follow from the
//! offsets, which is what makes them checkable and this group safe to build
//! without reference output.

use std::borrow::Cow;

use crate::chart::Chart;
use crate::jaimini::VariantChoice;
use crate::nakshatra::{Nakshatra, NakshatraPosition, NAKSHATRA_SPAN};
use crate::rasi::Rasi;
use lagn_ephem::{norm360, Graha};
use serde::{Deserialize, Serialize};

/// The Yoga sphuta's offset from `Sun + Moon`: 93 degrees 20 minutes.
///
/// Written as seven nakshatra spans rather than as 93.3333, because that is
/// what it is - and a test asserts the two agree, so a mistyped decimal
/// cannot shift every chart quietly.
pub const SPHUTA_OFFSET: f64 = 7.0 * NAKSHATRA_SPAN;

/// Nakshatras from the Yogi's to the Avayogi's, counting the Yogi's as 1.
pub const AVAYOGI_COUNT: u8 = 6;

/// A derived point and where it falls.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct YogaSphuta {
    pub longitude: f64,
    pub rasi: Rasi,
    pub degrees_in_rasi: f64,
    pub nakshatra: Nakshatra,
    pub pada: u8,
}

/// Yogi, Avayogi and the point they come from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Yogi {
    pub sphuta: YogaSphuta,
    /// Lord of the nakshatra the sphuta falls in.
    pub yogi: Graha,
    /// The sixth nakshatra from the Yogi's.
    pub avayogi_nakshatra: Nakshatra,
    pub avayogi: Graha,
    pub variants: Vec<VariantChoice>,
}

pub const VARIANTS: [VariantChoice; 3] = [
    VariantChoice {
        id: Cow::Borrowed("V-13-38"),
        question: Cow::Borrowed("the Yoga sphuta offset"),
        chosen: Cow::Borrowed("93 degrees 20 minutes, which is seven nakshatras exactly"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-39"),
        question: Cow::Borrowed("counting to the Avayogi"),
        chosen: Cow::Borrowed("the 6th nakshatra from the Yogi's, counting the Yogi's as 1"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-40"),
        question: Cow::Borrowed("Duplicate Yogi"),
        chosen: Cow::Borrowed("not computed; the accounts disagree and no identity settles it"),
    },
];

/// The Yoga sphuta from the Sun's and Moon's sidereal longitudes.
pub fn sphuta(sun: f64, moon: f64) -> YogaSphuta {
    let lon = norm360(sun + moon + SPHUTA_OFFSET);
    let pos = NakshatraPosition::from_longitude(lon);
    YogaSphuta {
        longitude: lon,
        rasi: Rasi::from_longitude(lon),
        degrees_in_rasi: Rasi::degrees_within(lon),
        nakshatra: pos.nakshatra,
        pada: pos.pada,
    }
}

impl Yogi {
    pub fn compute(chart: &Chart) -> Yogi {
        let sun = chart.placement(Graha::Sun).longitude;
        let moon = chart.placement(Graha::Moon).longitude;
        let sphuta = sphuta(sun, moon);

        let yogi = sphuta.nakshatra.lord();
        // The sixth from the Yogi's, counting the Yogi's as the first
        // (V-13-39), so five nakshatras on.
        let avayogi_nakshatra =
            Nakshatra::from_index(sphuta.nakshatra.index() as i32 + AVAYOGI_COUNT as i32 - 1);

        Yogi {
            sphuta,
            yogi,
            avayogi_nakshatra,
            avayogi: avayogi_nakshatra.lord(),
            variants: VARIANTS.to_vec(),
        }
    }
}
