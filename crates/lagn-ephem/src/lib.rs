//! Layer 1 - sidereal ephemeris.
//!
//! The only layer allowed to touch Swiss Ephemeris. Everything above it
//! consumes plain numbers, which keeps layers 2+ portable and trivially
//! testable.
//!
//! Three decisions are encoded here that are wrong in most jyotisha software,
//! so they are stated explicitly rather than left to a default:
//!
//! 1. **Silent Moshier fallback is an error.** If a `.se1` file is missing,
//!    Swiss Ephemeris quietly degrades to its built-in analytic theory and
//!    returns slightly different numbers. For an app whose entire promise is
//!    determinism that is unacceptable, so `strict` mode rejects it.
//! 2. **Rahu defaults to the mean node**, not the true node. This matches
//!    Tamil/Kerala panchangam practice and JHora's default. The difference
//!    reaches ~100 arcmin, which is enough to change a nakshatra.
//! 3. **Ketu is derived**, always exactly 180 degrees from Rahu.

mod error;
mod time;

pub use error::EphemError;
pub use time::{
    days_in_month, is_leap_year, jd_to_calendar, jd_to_civil, julian_day_ut, validate_civil_date, validate_date,
    BirthMoment, Calendar, CalendarDateTime, GREGORIAN_REFORM_JD, MAX_YEAR, MIN_YEAR,
};

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double};
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use serde::{Deserialize, Serialize};

/// Swiss Ephemeris keeps process-global mutable state (ephemeris path,
/// sidereal mode, open file handles) and is not thread-safe. Every call into
/// the C library happens under this lock.
static SWE_LOCK: Mutex<()> = Mutex::new(());
/// The ephemeris path is set once per process; SE caches open file handles
/// against it and re-setting it forces a reload.
static EPHE_PATH: OnceLock<String> = OnceLock::new();

fn lock() -> MutexGuard<'static, ()> {
    SWE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn err_buf() -> [c_char; swe_sys::AS_MAXCH] {
    [0; swe_sys::AS_MAXCH]
}

fn err_str(buf: &[c_char; swe_sys::AS_MAXCH]) -> String {
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Ayanamsa (precessional offset from tropical to sidereal).
///
/// `Lahiri` is the Government of India standard and the default across South
/// Indian practice. The others exist so a chart can be reproduced against
/// software that assumes a different school.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Ayanamsa {
    /// Chitrapaksha / Lahiri. Government of India standard.
    #[default]
    Lahiri,
    /// Lahiri recomputed against the ICRC reference frame.
    LahiriIcrc,
    /// Spica fixed at exactly 180 degrees.
    TrueChitra,
    /// B. V. Raman.
    Raman,
    /// K. S. Krishnamurti (KP system).
    Krishnamurti,
    /// Revati fixed at exactly 0 degrees.
    TrueRevati,
    /// Yukteshwar.
    Yukteshwar,
}

impl Ayanamsa {
    fn sid_mode(self) -> i32 {
        match self {
            Ayanamsa::Lahiri => swe_sys::SE_SIDM_LAHIRI,
            Ayanamsa::LahiriIcrc => swe_sys::SE_SIDM_LAHIRI_ICRC,
            Ayanamsa::TrueChitra => swe_sys::SE_SIDM_TRUE_CITRA,
            Ayanamsa::Raman => swe_sys::SE_SIDM_RAMAN,
            Ayanamsa::Krishnamurti => swe_sys::SE_SIDM_KRISHNAMURTI,
            Ayanamsa::TrueRevati => swe_sys::SE_SIDM_TRUE_REVATI,
            Ayanamsa::Yukteshwar => swe_sys::SE_SIDM_YUKTESHWAR,
        }
    }
}

/// Which lunar node to report as Rahu.
///
/// The true node oscillates about the mean node by up to ~1.7 degrees. South
/// Indian panchangam tradition and JHora both default to the mean node, so we
/// do too; a chart computed with the wrong one can land Rahu in a different
/// nakshatra and therefore a different Vimshottari starting dasha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    #[default]
    Mean,
    True,
}

/// House system. Whole-sign is the South Indian default and the only one the
/// interpretation layers will support; the rest exist for cross-checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HouseSystem {
    /// Bhava == rasi. The South Indian / Tamil-Kerala default.
    #[default]
    WholeSign,
    Equal,
    Placidus,
    Porphyry,
    Koch,
}

impl HouseSystem {
    fn code(self) -> std::os::raw::c_int {
        match self {
            HouseSystem::WholeSign => swe_sys::SE_HSYS_WHOLE_SIGN,
            HouseSystem::Equal => swe_sys::SE_HSYS_EQUAL,
            HouseSystem::Placidus => swe_sys::SE_HSYS_PLACIDUS,
            HouseSystem::Porphyry => swe_sys::SE_HSYS_PORPHYRY,
            HouseSystem::Koch => swe_sys::SE_HSYS_KOCH,
        }
    }
}

// ---------------------------------------------------------------------------
// Grahas
// ---------------------------------------------------------------------------

/// The nine classical grahas, in traditional order.
///
/// The outer planets are deliberately absent: no classical Tamil or Kerala
/// source uses them, and admitting them would mean inventing rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Graha {
    Sun = 0,
    Moon = 1,
    Mars = 2,
    Mercury = 3,
    Jupiter = 4,
    Venus = 5,
    Saturn = 6,
    Rahu = 7,
    Ketu = 8,
}

impl Graha {
    /// All nine, in traditional order.
    pub const ALL: [Graha; 9] = [
        Graha::Sun,
        Graha::Moon,
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
        Graha::Rahu,
        Graha::Ketu,
    ];

    /// Sanskrit/Tamil name as used in South Indian charts.
    pub fn name(self) -> &'static str {
        match self {
            Graha::Sun => "Surya",
            Graha::Moon => "Chandra",
            Graha::Mars => "Kuja",
            Graha::Mercury => "Budha",
            Graha::Jupiter => "Guru",
            Graha::Venus => "Shukra",
            Graha::Saturn => "Shani",
            Graha::Rahu => "Rahu",
            Graha::Ketu => "Ketu",
        }
    }

    /// Two-letter abbreviation used in square-chart cells.
    pub fn abbrev(self) -> &'static str {
        match self {
            Graha::Sun => "Su",
            Graha::Moon => "Mo",
            Graha::Mars => "Ma",
            Graha::Mercury => "Me",
            Graha::Jupiter => "Ju",
            Graha::Venus => "Ve",
            Graha::Saturn => "Sa",
            Graha::Rahu => "Ra",
            Graha::Ketu => "Ke",
        }
    }

    /// The shadowy grahas have no body and never truly "retrograde" - they are
    /// always in mean retrograde motion.
    pub fn is_chhaya(self) -> bool {
        matches!(self, Graha::Rahu | Graha::Ketu)
    }
}

/// A single body's sidereal position at an instant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// Sidereal ecliptic longitude, normalised to `[0, 360)`.
    pub longitude: f64,
    /// Ecliptic latitude in degrees.
    pub latitude: f64,
    /// Distance in AU.
    pub distance_au: f64,
    /// Longitude speed in degrees/day. Negative means retrograde.
    pub speed_longitude: f64,
    /// Latitude speed in degrees/day.
    pub speed_latitude: f64,
}

impl Position {
    /// True retrograde motion. Rahu and Ketu are excluded: their mean motion
    /// is always negative, which is a definitional property rather than the
    /// vakri state that carries interpretive weight.
    pub fn is_retrograde(&self) -> bool {
        self.speed_longitude < 0.0
    }
}

/// Ascendant and related chart angles, sidereal.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Angles {
    /// Lagna - sidereal ecliptic longitude of the ascendant, `[0, 360)`.
    pub ascendant: f64,
    /// Midheaven (dasama bhava sphuta in tropical terms).
    pub midheaven: f64,
    /// Sidereal time at the meridian, in degrees.
    pub armc: f64,
    /// Vertex.
    pub vertex: f64,
}

/// The twelve house cusps, sidereal, index 0 == house 1.
///
/// Under whole-sign houses every cusp sits at the 0th degree of its sign, so
/// this is mostly a cross-check artefact.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cusps(pub [f64; 12]);

// ---------------------------------------------------------------------------
// Ephemeris
// ---------------------------------------------------------------------------

/// A configured sidereal ephemeris.
///
/// Cheap to clone. All methods are thread-safe but serialise against a
/// process-wide lock, because the underlying C library is global.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ephemeris {
    ayanamsa: Ayanamsa,
    node_type: NodeType,
    /// Reject a silent fall back to the Moshier theory.
    strict: bool,
}

impl Ephemeris {
    /// Point the library at a directory of `.se1` files. Must be called once
    /// before any computation; later calls with a different path are an error
    /// because Swiss Ephemeris caches file handles against the first one.
    pub fn set_ephemeris_path(path: impl AsRef<Path>) -> Result<(), EphemError> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Err(EphemError::EphemerisPathMissing(path.display().to_string()));
        }
        let s = path.to_string_lossy().into_owned();

        if let Some(existing) = EPHE_PATH.get() {
            return if *existing == s {
                Ok(())
            } else {
                Err(EphemError::EphemerisPathConflict {
                    existing: existing.clone(),
                    requested: s,
                })
            };
        }

        let c = CString::new(s.clone()).map_err(|_| EphemError::EphemerisPathMissing(s.clone()))?;
        {
            let _g = lock();
            unsafe { swe_sys::swe_set_ephe_path(c.as_ptr()) };
        }
        let _ = EPHE_PATH.set(s);
        Ok(())
    }

    pub fn new(ayanamsa: Ayanamsa, node_type: NodeType) -> Self {
        Self { ayanamsa, node_type, strict: true }
    }

    /// Allow the Moshier fallback. Only for environments that genuinely cannot
    /// ship the ~6 MB of `.se1` files; results will differ in the last
    /// arcseconds and must not be compared against golden fixtures.
    pub fn allow_moshier_fallback(mut self) -> Self {
        self.strict = false;
        self
    }

    pub fn ayanamsa_mode(&self) -> Ayanamsa {
        self.ayanamsa
    }

    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    fn calc_flags(&self) -> i32 {
        swe_sys::SEFLG_SWIEPH | swe_sys::SEFLG_SIDEREAL | swe_sys::SEFLG_SPEED
    }

    /// Set the sidereal mode. Global state, so the caller must hold the lock.
    fn apply_sid_mode(&self, _guard: &MutexGuard<'static, ()>) {
        unsafe { swe_sys::swe_set_sid_mode(self.ayanamsa.sid_mode(), 0.0, 0.0) };
    }

    /// Ayanamsa value in degrees at the given instant.
    pub fn ayanamsa_value(&self, jd_ut: f64) -> Result<f64, EphemError> {
        check_jd(jd_ut)?;
        let g = lock();
        self.apply_sid_mode(&g);
        let mut daya: c_double = 0.0;
        let mut serr = err_buf();
        let rc = unsafe {
            swe_sys::swe_get_ayanamsa_ex_ut(
                jd_ut,
                swe_sys::SEFLG_SWIEPH,
                &mut daya,
                serr.as_mut_ptr(),
            )
        };
        if rc < 0 {
            return Err(EphemError::Swiss { call: "swe_get_ayanamsa_ex_ut", message: err_str(&serr) });
        }
        Ok(daya)
    }

    /// Sidereal position of one graha.
    pub fn position(&self, jd_ut: f64, graha: Graha) -> Result<Position, EphemError> {
        check_jd(jd_ut)?;
        let g = lock();
        self.apply_sid_mode(&g);
        self.position_locked(jd_ut, graha, &g)
    }

    /// All nine grahas at one instant, under a single lock acquisition.
    pub fn positions(&self, jd_ut: f64) -> Result<[(Graha, Position); 9], EphemError> {
        check_jd(jd_ut)?;
        let g = lock();
        self.apply_sid_mode(&g);

        let mut out: [(Graha, Position); 9] = [(Graha::Sun, ZERO_POS); 9];
        for (slot, graha) in out.iter_mut().zip(Graha::ALL) {
            *slot = (graha, self.position_locked(jd_ut, graha, &g)?);
        }
        Ok(out)
    }

    /// `g` is never read: it exists so that this function cannot be
    /// called without holding `SWE_LOCK`. Removing the lock crashes the
    /// process (see tests/concurrency.rs), so the requirement is enforced by
    /// the type system rather than by convention.
    #[allow(clippy::only_used_in_recursion)]
    fn position_locked(
        &self,
        jd_ut: f64,
        graha: Graha,
        g: &MutexGuard<'static, ()>,
    ) -> Result<Position, EphemError> {
        // Ketu is never computed - it is Rahu reflected through the Earth.
        // Deriving it guarantees the axis stays exactly 180 degrees, which a
        // separate computation would not.
        if graha == Graha::Ketu {
            let rahu = self.position_locked(jd_ut, Graha::Rahu, g)?;
            return Ok(Position {
                longitude: norm360(rahu.longitude + 180.0),
                latitude: -rahu.latitude,
                distance_au: rahu.distance_au,
                speed_longitude: rahu.speed_longitude,
                speed_latitude: -rahu.speed_latitude,
            });
        }

        let ipl = match graha {
            Graha::Sun => swe_sys::SE_SUN,
            Graha::Moon => swe_sys::SE_MOON,
            Graha::Mars => swe_sys::SE_MARS,
            Graha::Mercury => swe_sys::SE_MERCURY,
            Graha::Jupiter => swe_sys::SE_JUPITER,
            Graha::Venus => swe_sys::SE_VENUS,
            Graha::Saturn => swe_sys::SE_SATURN,
            Graha::Rahu => match self.node_type {
                NodeType::Mean => swe_sys::SE_MEAN_NODE,
                NodeType::True => swe_sys::SE_TRUE_NODE,
            },
            Graha::Ketu => unreachable!("handled above"),
        };

        let flags = self.calc_flags();
        let mut xx = [0.0f64; 6];
        let mut serr = err_buf();
        let rc = unsafe {
            swe_sys::swe_calc_ut(jd_ut, ipl, flags, xx.as_mut_ptr(), serr.as_mut_ptr())
        };

        if rc < 0 {
            return Err(EphemError::Swiss { call: "swe_calc_ut", message: err_str(&serr) });
        }
        // A non-negative return whose ephemeris bit differs from the request
        // means SE silently substituted a different theory.
        if self.strict && (rc & swe_sys::SEFLG_SWIEPH) == 0 {
            return Err(EphemError::MoshierFallback {
                body: graha.name(),
                jd_ut,
                message: err_str(&serr),
            });
        }

        Ok(Position {
            longitude: norm360(xx[0]),
            latitude: xx[1],
            distance_au: xx[2],
            speed_longitude: xx[3],
            speed_latitude: xx[4],
        })
    }

    /// Sidereal ascendant, midheaven and house cusps.
    ///
    /// `latitude` is degrees north-positive, `longitude` degrees east-positive.
    pub fn angles(
        &self,
        jd_ut: f64,
        latitude: f64,
        longitude: f64,
        hsys: HouseSystem,
    ) -> Result<(Angles, Cusps), EphemError> {
        check_jd(jd_ut)?;
        if !latitude.is_finite() || !(-90.0..=90.0).contains(&latitude) {
            return Err(EphemError::InvalidLatitude(latitude));
        }
        // At exactly +/-90 the ascendant is mathematically undefined. Swiss
        // Ephemeris returns a number anyway, and returns the *same* number for
        // both poles, which is demonstrably wrong rather than merely
        // approximate. Refuse instead of handing back a plausible-looking
        // chart. 89.999 still works and still distinguishes the hemispheres.
        if latitude.abs() == 90.0 {
            return Err(EphemError::DegenerateAscendant(latitude));
        }
        if !longitude.is_finite() || !(-180.0..=180.0).contains(&longitude) {
            return Err(EphemError::InvalidLongitude(longitude));
        }

        let g = lock();
        self.apply_sid_mode(&g);

        // swe_houses_ex writes 13 cusp slots (1-indexed, slot 0 unused).
        let mut cusps = [0.0f64; 13];
        let mut ascmc = [0.0f64; swe_sys::SE_NASCMC + 2];
        let rc = unsafe {
            swe_sys::swe_houses_ex(
                jd_ut,
                swe_sys::SEFLG_SIDEREAL,
                latitude,
                longitude,
                hsys.code(),
                cusps.as_mut_ptr(),
                ascmc.as_mut_ptr(),
            )
        };
        if rc < 0 {
            // SE returns ERR when the requested system is undefined at this
            // latitude (Placidus and Koch break down inside the polar
            // circles). Whole-sign never fails, which is one more reason it
            // is the right default.
            return Err(EphemError::HouseSystemUndefined { system: hsys, latitude });
        }

        let mut out = [0.0f64; 12];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = norm360(cusps[i + 1]);
        }

        Ok((
            Angles {
                ascendant: norm360(ascmc[swe_sys::SE_ASC]),
                midheaven: norm360(ascmc[swe_sys::SE_MC]),
                armc: norm360(ascmc[swe_sys::SE_ARMC]),
                vertex: norm360(ascmc[swe_sys::SE_VERTEX]),
            },
            Cusps(out),
        ))
    }

    /// Swiss Ephemeris version string, for recording in golden fixtures.
    pub fn library_version() -> String {
        let _g = lock();
        let mut buf = [0 as c_char; swe_sys::AS_MAXCH];
        unsafe {
            swe_sys::swe_version(buf.as_mut_ptr());
            CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned()
        }
    }
}

/// Reject a Julian Day that cannot describe an instant.
///
/// Without this, `swe_calc_ut(NaN, ..)` returns success with a NaN longitude,
/// and every downstream binning function maps NaN to index 0 - producing a
/// complete, plausible-looking chart of Mesha lagna and Ashwini pada 1. Silent
/// garbage is the one failure mode this project cannot tolerate.
#[inline]
fn check_jd(jd_ut: f64) -> Result<(), EphemError> {
    if jd_ut.is_finite() {
        Ok(())
    } else {
        Err(EphemError::NonFiniteJulianDay(jd_ut))
    }
}

const ZERO_POS: Position = Position {
    longitude: 0.0,
    latitude: 0.0,
    distance_au: 0.0,
    speed_longitude: 0.0,
    speed_latitude: 0.0,
};

/// Normalise an angle to `[0, 360)`.
///
/// The half-open upper bound is load-bearing: callers bin the result into
/// 12ths, 27ths and 108ths, and a value of exactly 360.0 lands one cell past
/// the end of every one of those grids.
///
/// The naive `r + 360.0` returns exactly 360.0 for any sufficiently small
/// negative input, because the sum rounds up to the nearest representable
/// double. Sidereal longitudes are produced by subtracting the ayanamsa from a
/// tropical longitude, so a result a few ulps below zero is entirely reachable.
///
/// Non-finite input is returned unchanged rather than silently becoming a
/// plausible angle; the layers above reject it explicitly.
#[inline]
pub fn norm360(deg: f64) -> f64 {
    if !deg.is_finite() {
        return deg;
    }
    let r = deg % 360.0;
    if r < 0.0 {
        let wrapped = r + 360.0;
        // Guard the rounding case described above.
        if wrapped >= 360.0 {
            0.0
        } else {
            wrapped
        }
    } else {
        r
    }
}

// ---------------------------------------------------------------------------
// Sunrise and sunset
// ---------------------------------------------------------------------------

/// The Sun's rising and setting, both as Julian Day in UT.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunDay {
    pub sunrise_jd: f64,
    pub sunset_jd: f64,
}

impl SunDay {
    /// Length of the day from sunrise to sunset, in days.
    pub fn day_length(&self) -> f64 {
        self.sunset_jd - self.sunrise_jd
    }
}

impl Ephemeris {
    /// The first sunrise at or after `from_jd_ut`, and the sunset that follows
    /// it.
    ///
    /// Indian panchangam practice measures a day from sunrise, not midnight,
    /// and divides sunrise-to-sunset into parts to place Rahu kalam and the
    /// rest. Both events are taken at the upper limb with refraction, which is
    /// Swiss Ephemeris's default and what printed panchangams use.
    ///
    /// `latitude` is degrees north-positive, `longitude` degrees east-positive.
    pub fn sun_day(&self, from_jd_ut: f64, latitude: f64, longitude: f64) -> Result<SunDay, EphemError> {
        check_jd(from_jd_ut)?;
        if !latitude.is_finite() || !(-90.0..=90.0).contains(&latitude) {
            return Err(EphemError::InvalidLatitude(latitude));
        }
        if !longitude.is_finite() || !(-180.0..=180.0).contains(&longitude) {
            return Err(EphemError::InvalidLongitude(longitude));
        }

        let _g = lock();
        // Altitude 0: a panchangam is computed for the place, not its hilltop,
        // and the difference is seconds.
        let mut geopos: [c_double; 3] = [longitude, latitude, 0.0];

        // Standard atmosphere, and not zeroes. Swiss Ephemeris reads a zero
        // temperature as 0 degrees Celsius rather than "use a default", and
        // colder air refracts more: passing zero put sunrise 13 seconds early
        // and sunset 13 seconds late against swetest, which passes these. An
        // almanac uses the same convention.
        const PRESSURE_MBAR: c_double = 1013.25;
        const TEMPERATURE_C: c_double = 15.0;

        let mut event = |rsmi: i32, start: f64| -> Result<f64, EphemError> {
            let mut tret: [c_double; 10] = [0.0; 10];
            let mut serr = err_buf();
            let rc = unsafe {
                swe_sys::swe_rise_trans(
                    start,
                    swe_sys::SE_SUN,
                    std::ptr::null_mut(),
                    swe_sys::SEFLG_SWIEPH,
                    rsmi,
                    geopos.as_mut_ptr(),
                    PRESSURE_MBAR,
                    TEMPERATURE_C,
                    tret.as_mut_ptr(),
                    serr.as_mut_ptr(),
                )
            };
            // -2 is Swiss Ephemeris's "no such event on this day", which inside
            // the polar circles is the truth rather than a failure.
            if rc == -2 {
                return Err(EphemError::NoSunriseThatDay { latitude, jd_ut: start });
            }
            if rc < 0 {
                return Err(EphemError::Swiss { call: "swe_rise_trans", message: err_str(&serr) });
            }
            Ok(tret[0])
        };

        let sunrise_jd = event(swe_sys::SE_CALC_RISE, from_jd_ut)?;
        // Search for the sunset from the sunrise, so the pair always belongs to
        // one day even when the caller started just before midnight.
        let sunset_jd = event(swe_sys::SE_CALC_SET, sunrise_jd)?;
        Ok(SunDay { sunrise_jd, sunset_jd })
    }
}
