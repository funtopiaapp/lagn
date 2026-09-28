use crate::HouseSystem;

#[derive(Debug, thiserror::Error)]
pub enum EphemError {
    #[error("ephemeris directory not found: {0}")]
    EphemerisPathMissing(String),

    #[error(
        "ephemeris path already set to {existing}; cannot change to {requested} \
         (Swiss Ephemeris caches file handles against the first path set)"
    )]
    EphemerisPathConflict { existing: String, requested: String },

    #[error(
        "Swiss Ephemeris silently fell back to the Moshier theory for {body} at JD {jd_ut} - \
         the .se1 files are missing or do not cover this date. Results would not be reproducible. {message}"
    )]
    MoshierFallback { body: &'static str, jd_ut: f64, message: String },

    #[error("{call} failed: {message}")]
    Swiss { call: &'static str, message: String },

    #[error("latitude {0} out of range [-90, 90]")]
    InvalidLatitude(f64),

    #[error(
        "the ascendant is undefined at latitude {0} - at the geographic poles the \
         horizon coincides with the celestial equator and no point of the ecliptic rises"
    )]
    DegenerateAscendant(f64),

    #[error("Julian Day must be a finite number, got {0}")]
    NonFiniteJulianDay(f64),

    #[error("longitude {0} out of range [-180, 180]")]
    InvalidLongitude(f64),

    #[error("house system {system:?} is undefined at latitude {latitude}")]
    HouseSystemUndefined { system: HouseSystem, latitude: f64 },

    #[error("invalid calendar date: {year}-{month:02}-{day:02} ({reason})")]
    InvalidDate { year: i32, month: u32, day: u32, reason: &'static str },

    #[error("invalid clock time: {hour:02}:{minute:02}:{second:06.3}")]
    InvalidTime { hour: u32, minute: u32, second: f64 },

    #[error("UTC offset {0} hours is out of the plausible range [-14, +14]")]
    InvalidUtcOffset(f64),
}
