//! The annual (solar return) chart, Muntha, and kaksha transit.
//!
//! Specification: `docs/phase13/VARSHA-KAKSHA.md`.
//!
//! Two small groups that share one property: both are derivations rather than
//! tables, so both can be checked. The solar return is checked against
//! itself, since the Sun at the answer must equal the natal Sun; and the
//! kaksha verdict is read straight off the `prastara` bitmaps that
//! `ashtakavarga.rs` already computes and cross-validates.

use std::borrow::Cow;

use crate::ashtakavarga::{Ashtakavarga, Contributor};
use crate::chart::Chart;
use crate::jaimini::VariantChoice;
use crate::rasi::Rasi;
use lagn_ephem::{norm360, EphemError, Ephemeris, Graha};
use serde::{Deserialize, Serialize};

/// Width of one kaksha: 3 degrees 45 minutes, an eighth of a sign.
pub const KAKSHA_WIDTH: f64 = 30.0 / 8.0;

/// The eight kaksha owners, in order from the start of a sign (V-13-25).
pub const KAKSHA_ORDER: [Contributor; 8] = [
    Contributor::Saturn,
    Contributor::Jupiter,
    Contributor::Mars,
    Contributor::Sun,
    Contributor::Venus,
    Contributor::Mercury,
    Contributor::Moon,
    Contributor::Lagna,
];

/// Which kaksha a longitude stands in, 1 to 8, and who owns it.
pub fn kaksha_of(longitude: f64) -> (u8, Contributor) {
    let within = Rasi::degrees_within(norm360(longitude));
    // The last kaksha absorbs anything a rounding step past its top.
    let i = ((within / KAKSHA_WIDTH).floor() as usize).min(7);
    (i as u8 + 1, KAKSHA_ORDER[i])
}

/// A transiting graha's kaksha, and whether its Ashtakavarga supports it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KakshaTransit {
    pub graha: Graha,
    pub longitude: f64,
    pub rasi: Rasi,
    /// 1 to 8, from the start of the sign.
    pub kaksha: u8,
    pub owner: Contributor,
    /// True when the owner gave a bindu in this graha's own BAV for this sign.
    pub supported: bool,
    /// The graha's BAV bindus in this sign, for context.
    pub bindus: u8,
}

/// Where a transiting graha stands, and whether its own Ashtakavarga backs it.
///
/// `None` for the nodes, which have no Bhinnashtakavarga.
pub fn kaksha_transit(av: &Ashtakavarga, graha: Graha, longitude: f64) -> Option<KakshaTransit> {
    let bav = av.bav_of(graha)?;
    let lon = norm360(longitude);
    let rasi = Rasi::from_longitude(lon);
    let (kaksha, owner) = kaksha_of(lon);

    // `contributors` reads the prastara bitmap, which records exactly which
    // contributor gave each bindu - so the verdict is read, not inferred.
    let gave = av.contributors(graha, rasi)?;
    Some(KakshaTransit {
        graha,
        longitude: lon,
        rasi,
        kaksha,
        owner,
        supported: gave.contains(&owner),
        bindus: bav[rasi.index() as usize],
    })
}

/// A milliarcsecond, in degrees. The solar return is solved, so it is held to
/// the precision of the solve rather than to an astronomical tolerance.
const MAS: f64 = 1.0 / 3_600_000.0;

/// Signed difference between two longitudes, wrapped to `[-180, 180)`.
///
/// Wrapped so the crossing of 360 is not a discontinuity the bisection would
/// mistake for a root.
fn delta(a: f64, b: f64) -> f64 {
    let d = norm360(a - b);
    if d >= 180.0 { d - 360.0 } else { d }
}

/// The moment the Sun next regains `natal_sun`, on or after `from_jd`.
///
/// Bracket and bisect, as `Chart::lagna_window` does, rather than estimate a
/// rate: the Sun's daily motion varies by about three per cent over the year,
/// and a rate estimate would be wrong by minutes at the solstices.
pub fn solar_return(eph: &Ephemeris, natal_sun: f64, from_jd: f64) -> Result<f64, EphemError> {
    // The Sun advances just under a degree a day and never reverses, so the
    // difference from the natal longitude rises monotonically through zero
    // once a year. Step forward in days until it does.
    let mut lo = from_jd;
    let mut d_lo = delta(eph.position(lo, Graha::Sun)?.longitude, natal_sun);
    // Already there, to the precision anyone cares about.
    if d_lo.abs() < MAS {
        return Ok(lo);
    }
    // Walk to the first day where the difference has become non-negative,
    // which is the crossing.
    let mut hi = lo;
    for _ in 0..400 {
        hi = lo + 1.0;
        let d_hi = delta(eph.position(hi, Graha::Sun)?.longitude, natal_sun);
        if d_lo < 0.0 && d_hi >= 0.0 {
            break;
        }
        lo = hi;
        d_lo = d_hi;
    }

    // Bisect. Sixty halvings of a day is far below the precision of the
    // ephemeris, so this converges long before it runs out.
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        let d = delta(eph.position(mid, Graha::Sun)?.longitude, natal_sun);
        if d.abs() < MAS {
            return Ok(mid);
        }
        if d < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok((lo + hi) / 2.0)
}

/// An annual chart, with Muntha.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Varshaphala {
    /// Completed years at the start of this year.
    pub age: u32,
    /// The moment the Sun regained its natal longitude.
    pub return_jd: f64,
    /// How far the Sun at that moment differs from the natal Sun, in degrees.
    /// Reported so a reader can see the solve closed rather than trust it.
    pub sun_error: f64,
    pub chart: Chart,
    /// Muntha's sign, and the house it occupies in the annual chart.
    pub muntha: Rasi,
    pub muntha_house: u8,
    /// Where each graha of the annual chart stands in the natal
    /// Ashtakavarga's kakshas. The nodes are absent: they have no
    /// Bhinnashtakavarga to be supported by.
    pub kaksha: Vec<KakshaTransit>,
    pub variants: Vec<VariantChoice>,
}

pub const VARIANTS: [VariantChoice; 4] = [
    VariantChoice {
        id: Cow::Borrowed("V-13-22"),
        question: Cow::Borrowed("place for the annual chart"),
        chosen: Cow::Borrowed("the birth place"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-23"),
        question: Cow::Borrowed("which solar return begins the year"),
        chosen: Cow::Borrowed("the first on or after the birthday"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-24"),
        question: Cow::Borrowed("Muntha counted from"),
        chosen: Cow::Borrowed("completed years"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-25"),
        question: Cow::Borrowed("kaksha owner order"),
        chosen: Cow::Borrowed("Shani, Guru, Kuja, Surya, Shukra, Budha, Chandra, Lagna"),
    },
];

/// Mean days in a year, for stepping to the nominal anniversary. Only a
/// starting point for the bracket: the solve decides the answer.
const YEAR: f64 = 365.2425;

impl Varshaphala {
    /// The annual chart for the year in which the native completes `age`.
    ///
    /// The kaksha readings compare the *annual* chart's positions against the
    /// *natal* Ashtakavarga, which is what makes them a transit rather than a
    /// second chart's own Ashtakavarga.
    pub fn compute(eph: &Ephemeris, natal: &Chart, age: u32) -> Result<Varshaphala, EphemError> {
        let natal_sun = natal.placement(Graha::Sun).longitude;
        // Start the search a few days before the nominal anniversary, so the
        // first crossing found is this year's and not the next (V-13-23).
        let from = natal.jd_ut + YEAR * age as f64 - 5.0;
        let return_jd = solar_return(eph, natal_sun, from)?;

        // Cast the annual chart for that moment, at the birth place
        // (V-13-22), with the natal chart's own settings so the two agree
        // about ayanamsa and node type.
        let chart = Chart::compute_at(
            return_jd,
            natal.birth.latitude,
            natal.birth.longitude,
            natal.birth.place_name.clone(),
            natal.birth.moment.utc_offset_hours,
            natal.settings,
        )?;

        let sun_error = delta(chart.placement(Graha::Sun).longitude, natal_sun).abs();

        // Muntha: the natal lagna, advanced one sign per completed year.
        let muntha = Rasi::from_index(natal.lagna.rasi.index() as i32 + age as i32);
        let muntha_house = chart.lagna.rasi.houses_to(muntha);

        let av = Ashtakavarga::compute(natal);
        let kaksha = chart
            .placements
            .iter()
            .filter_map(|p| kaksha_transit(&av, p.graha, p.longitude))
            .collect();

        Ok(Varshaphala {
            age,
            return_jd,
            sun_error,
            chart,
            muntha,
            muntha_house,
            kaksha,
            variants: VARIANTS.to_vec(),
        })
    }
}
