//! Raw FFI bindings to the vendored Swiss Ephemeris C library.
//!
//! Hand-written rather than bindgen-generated: the surface we need is small
//! and stable, and avoiding a libclang build dependency keeps cross-compiling
//! to `wasm32`, iOS and Android simple later on.
//!
//! Everything here is `unsafe` and does no validation. Use `lagn-ephem`.
//!
//! # Thread safety
//! Swiss Ephemeris keeps global mutable state (ephemeris path, sidereal mode,
//! open file handles). It is **not** thread-safe. `lagn-ephem` serialises all
//! access behind a mutex; do not call these directly from multiple threads.
#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_double, c_int};

/// Buffer size Swiss Ephemeris expects for error strings (`AS_MAXCH`).
pub const AS_MAXCH: usize = 256;

// ---------------------------------------------------------------------------
// Bodies
// ---------------------------------------------------------------------------
pub const SE_ECL_NUT: c_int = -1;
pub const SE_SUN: c_int = 0;
pub const SE_MOON: c_int = 1;
pub const SE_MERCURY: c_int = 2;
pub const SE_VENUS: c_int = 3;
pub const SE_MARS: c_int = 4;
pub const SE_JUPITER: c_int = 5;
pub const SE_SATURN: c_int = 6;
pub const SE_URANUS: c_int = 7;
pub const SE_NEPTUNE: c_int = 8;
pub const SE_PLUTO: c_int = 9;
pub const SE_MEAN_NODE: c_int = 10;
pub const SE_TRUE_NODE: c_int = 11;
pub const SE_MEAN_APOG: c_int = 12;
pub const SE_OSCU_APOG: c_int = 13;
pub const SE_EARTH: c_int = 14;
pub const SE_CHIRON: c_int = 15;

// ---------------------------------------------------------------------------
// Calculation flags
// ---------------------------------------------------------------------------
pub const SEFLG_JPLEPH: i32 = 1;
pub const SEFLG_SWIEPH: i32 = 2;
pub const SEFLG_MOSEPH: i32 = 4;
pub const SEFLG_HELCTR: i32 = 8;
pub const SEFLG_TRUEPOS: i32 = 16;
pub const SEFLG_J2000: i32 = 32;
pub const SEFLG_NONUT: i32 = 64;
pub const SEFLG_SPEED: i32 = 256;
pub const SEFLG_NOGDEFL: i32 = 512;
pub const SEFLG_NOABERR: i32 = 1024;
pub const SEFLG_EQUATORIAL: i32 = 2 * 1024;
pub const SEFLG_XYZ: i32 = 4 * 1024;
pub const SEFLG_RADIANS: i32 = 8 * 1024;
pub const SEFLG_BARYCTR: i32 = 16 * 1024;
pub const SEFLG_TOPOCTR: i32 = 32 * 1024;
pub const SEFLG_SIDEREAL: i32 = 64 * 1024;
pub const SEFLG_ICRS: i32 = 128 * 1024;

// ---------------------------------------------------------------------------
// Calendar
// ---------------------------------------------------------------------------
pub const SE_JUL_CAL: c_int = 0;
pub const SE_GREG_CAL: c_int = 1;

// ---------------------------------------------------------------------------
// Sidereal modes (ayanamsa). Full list in swephexp.h; these are the ones
// that matter for Indian astrology.
// ---------------------------------------------------------------------------
pub const SE_SIDM_FAGAN_BRADLEY: i32 = 0;
pub const SE_SIDM_LAHIRI: i32 = 1;
pub const SE_SIDM_RAMAN: i32 = 3;
pub const SE_SIDM_KRISHNAMURTI: i32 = 5;
pub const SE_SIDM_YUKTESHWAR: i32 = 7;
pub const SE_SIDM_JN_BHASIN: i32 = 8;
pub const SE_SIDM_TRUE_CITRA: i32 = 27;
pub const SE_SIDM_TRUE_REVATI: i32 = 28;
pub const SE_SIDM_TRUE_PUSHYA: i32 = 29;
pub const SE_SIDM_LAHIRI_1940: i32 = 43;
pub const SE_SIDM_LAHIRI_VP285: i32 = 44;
pub const SE_SIDM_KRISHNAMURTI_VP291: i32 = 45;
pub const SE_SIDM_LAHIRI_ICRC: i32 = 46;

// ---------------------------------------------------------------------------
// `ascmc` array indices returned by swe_houses_ex
// ---------------------------------------------------------------------------
pub const SE_ASC: usize = 0;
pub const SE_MC: usize = 1;
pub const SE_ARMC: usize = 2;
pub const SE_VERTEX: usize = 3;
pub const SE_EQUASC: usize = 4;
pub const SE_NASCMC: usize = 8;

// ---------------------------------------------------------------------------
// House systems (passed as the ASCII code of the letter)
// ---------------------------------------------------------------------------
/// Whole-sign houses. The South Indian / Tamil-Kerala default: bhava == rasi.
pub const SE_HSYS_WHOLE_SIGN: c_int = b'W' as c_int;
/// Equal houses from the ascendant.
pub const SE_HSYS_EQUAL: c_int = b'A' as c_int;
/// Placidus. Present only so we can diff against Western software.
pub const SE_HSYS_PLACIDUS: c_int = b'P' as c_int;
/// Porphyry - the basis of the Sripati bhava scheme used in parts of N. India.
pub const SE_HSYS_PORPHYRY: c_int = b'O' as c_int;
/// Koch.
pub const SE_HSYS_KOCH: c_int = b'K' as c_int;

extern "C" {
    // --- setup / teardown ---
    pub fn swe_set_ephe_path(path: *const c_char);
    pub fn swe_close();
    pub fn swe_version(svers: *mut c_char) -> *mut c_char;
    pub fn swe_get_library_path(spath: *mut c_char) -> *mut c_char;
    pub fn swe_set_topo(geolon: c_double, geolat: c_double, geoalt: c_double);

    // --- bodies ---
    pub fn swe_calc_ut(
        tjd_ut: c_double,
        ipl: i32,
        iflag: i32,
        xx: *mut c_double,
        serr: *mut c_char,
    ) -> i32;
    pub fn swe_calc(
        tjd_et: c_double,
        ipl: c_int,
        iflag: i32,
        xx: *mut c_double,
        serr: *mut c_char,
    ) -> i32;
    pub fn swe_get_planet_name(ipl: c_int, spname: *mut c_char) -> *mut c_char;

    // --- sidereal / ayanamsa ---
    pub fn swe_set_sid_mode(sid_mode: i32, t0: c_double, ayan_t0: c_double);
    pub fn swe_get_ayanamsa_ex_ut(
        tjd_ut: c_double,
        iflag: i32,
        daya: *mut c_double,
        serr: *mut c_char,
    ) -> i32;
    pub fn swe_get_ayanamsa_name(isidmode: i32) -> *const c_char;

    // --- houses / ascendant ---
    pub fn swe_houses_ex(
        tjd_ut: c_double,
        iflag: i32,
        geolat: c_double,
        geolon: c_double,
        hsys: c_int,
        cusps: *mut c_double,
        ascmc: *mut c_double,
    ) -> c_int;
    pub fn swe_house_name(hsys: c_int) -> *const c_char;

    // --- time ---
    pub fn swe_julday(
        year: c_int,
        month: c_int,
        day: c_int,
        hour: c_double,
        gregflag: c_int,
    ) -> c_double;
    pub fn swe_revjul(
        jd: c_double,
        gregflag: c_int,
        jyear: *mut c_int,
        jmon: *mut c_int,
        jday: *mut c_int,
        jut: *mut c_double,
    );
    pub fn swe_deltat_ex(tjd: c_double, iflag: i32, serr: *mut c_char) -> c_double;
    /// Local Mean Time -> Local Apparent Time. Needed for pre-standard-time births.
    pub fn swe_lmt_to_lat(
        tjd_lmt: c_double,
        geolon: c_double,
        tjd_lat: *mut c_double,
        serr: *mut c_char,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    /// Proves the C library actually compiled and linked.
    #[test]
    fn links_and_reports_version() {
        let mut buf = [0 as c_char; AS_MAXCH];
        let v = unsafe {
            swe_version(buf.as_mut_ptr());
            CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned()
        };
        assert!(!v.is_empty(), "swe_version returned empty");
        println!("Swiss Ephemeris version: {v}");
    }

    /// Julian Day for 2000-01-01 12:00 UT is exactly 2451545.0 by definition.
    #[test]
    fn julday_j2000_epoch() {
        let jd = unsafe { swe_julday(2000, 1, 1, 12.0, SE_GREG_CAL) };
        assert_eq!(jd, 2451545.0);
    }
}
