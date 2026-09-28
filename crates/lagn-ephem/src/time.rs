//! Calendar and Julian Day handling.
//!
//! Deliberately has **no timezone database**. A birth time enters the system
//! as a wall-clock reading plus the UTC offset that was in force at that place
//! on that date, and resolving that offset is a separate concern with its own
//! failure modes (India used multiple local times before 1906, Bombay kept its
//! own until 1955, and IST itself has shifted). Making the offset an explicit
//! input keeps this layer deterministic and lets the offset be stored
//! alongside the raw input so a chart can be recomputed if tzdata changes.

use crate::EphemError;

/// Which calendar the date literal is expressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Calendar {
    /// Proleptic Gregorian.
    #[default]
    Gregorian,
    /// Julian - required for dates before the 1582 reform.
    Julian,
}

impl Calendar {
    /// The calendar actually in civil use on a given date.
    ///
    /// The Gregorian reform took effect 1582-10-15; 1582-10-05 through 10-14
    /// do not exist. Note that India did not adopt the Gregorian calendar
    /// until British administration, so for Indian historical dates the
    /// calendar of the *source document* matters more than this default.
    pub fn for_date(year: i32, month: u32, day: u32) -> Calendar {
        if (year, month, day) >= (1582, 10, 15) {
            Calendar::Gregorian
        } else {
            Calendar::Julian
        }
    }

    /// The calendar in civil use at an instant - the inverse of
    /// [`Calendar::for_date`], so that a date read in one calendar is printed
    /// back in the same one.
    ///
    /// JD 2299160.5 is 1582-10-15 00:00 UT, the first day of the Gregorian
    /// calendar.
    pub fn for_jd(jd_ut: f64) -> Calendar {
        if jd_ut >= GREGORIAN_REFORM_JD {
            Calendar::Gregorian
        } else {
            Calendar::Julian
        }
    }

    fn flag(self) -> std::os::raw::c_int {
        match self {
            Calendar::Gregorian => swe_sys::SE_GREG_CAL,
            Calendar::Julian => swe_sys::SE_JUL_CAL,
        }
    }
}

/// Julian Day of 1582-10-15 00:00 UT, the first day of the Gregorian calendar.
pub const GREGORIAN_REFORM_JD: f64 = 2299160.5;

/// Earliest year our `.se1` files cover, with a margin.
pub const MIN_YEAR: i32 = 1200;
/// Latest year our `.se1` files cover, with a margin.
pub const MAX_YEAR: i32 = 3000;

/// Is this a leap year under the given calendar?
///
/// The rules genuinely differ: the Julian calendar leaps every fourth year
/// without exception, which is why 1700, 1800 and 1900 are leap years in it
/// and not in the Gregorian.
pub const fn is_leap_year(year: i32, calendar: Calendar) -> bool {
    match calendar {
        Calendar::Julian => year.rem_euclid(4) == 0,
        Calendar::Gregorian => {
            year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
        }
    }
}

/// Number of days in a month under the given calendar.
pub const fn days_in_month(year: i32, month: u32, calendar: Calendar) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year, calendar) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Validate a calendar date.
///
/// Swiss Ephemeris accepts 30 February without complaint and returns the
/// Julian Day for 2 March, so a typo in a birth date silently yields a
/// complete, plausible, wrong chart. Catch it here instead.
pub fn validate_date(year: i32, month: u32, day: u32, calendar: Calendar) -> Result<(), EphemError> {
    if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
        return Err(EphemError::InvalidDate {
            year, month, day,
            reason: "year outside the range the bundled ephemeris files cover",
        });
    }
    if !(1..=12).contains(&month) {
        return Err(EphemError::InvalidDate { year, month, day, reason: "month must be 1-12" });
    }
    if day == 0 {
        return Err(EphemError::InvalidDate { year, month, day, reason: "day must be 1 or more" });
    }
    let max = days_in_month(year, month, calendar);
    if day > max {
        return Err(EphemError::InvalidDate {
            year, month, day,
            reason: if month == 2 { "February is shorter than that this year" } else { "that month is shorter than that" },
        });
    }
    // The ten days dropped by the Gregorian reform never existed anywhere the
    // reform was adopted in 1582. A date inside the gap is always a data error.
    if calendar == Calendar::Gregorian
        && year == 1582 && month == 10 && (5..=14).contains(&day)
    {
        return Err(EphemError::InvalidDate {
            year, month, day,
            reason: "inside the ten days dropped by the Gregorian reform",
        });
    }
    Ok(())
}

/// Validate a date as a *civil* date - the way a birth date is entered, with
/// the calendar implied by [`Calendar::for_date`].
///
/// Stricter than [`validate_date`]: it also rejects 1582-10-05..14. Those days
/// exist in the Julian calendar, and `for_date` labels them Julian, but in the
/// civil model used here the day after Julian 1582-10-04 is Gregorian
/// 1582-10-15. Accepting them silently moved a birth forward by ten days:
/// Julian 1582-10-10 is the same instant as Gregorian 1582-10-20.
///
/// Callers who genuinely mean the Julian calendar - a record kept in Britain
/// or British India before 1752, say - should call [`julian_day_ut`] with
/// `Calendar::Julian` explicitly instead.
pub fn validate_civil_date(year: i32, month: u32, day: u32) -> Result<(), EphemError> {
    validate_date(year, month, day, Calendar::for_date(year, month, day))?;
    if year == 1582 && month == 10 && (5..=14).contains(&day) {
        return Err(EphemError::InvalidDate {
            year, month, day,
            reason: "inside the ten days dropped by the Gregorian reform",
        });
    }
    Ok(())
}

/// Julian Day number in Universal Time.
///
/// `hour_ut` is a fractional hour in UT, e.g. 13.5 for 13:30.
pub fn julian_day_ut(
    year: i32,
    month: u32,
    day: u32,
    hour_ut: f64,
    calendar: Calendar,
) -> Result<f64, EphemError> {
    validate_date(year, month, day, calendar)?;
    if !hour_ut.is_finite() || !(0.0..24.0).contains(&hour_ut) {
        return Err(EphemError::InvalidTime {
            hour: if hour_ut.is_finite() { hour_ut as u32 } else { 99 },
            minute: 0,
            second: 0.0,
        });
    }
    Ok(unsafe {
        swe_sys::swe_julday(
            year,
            month as std::os::raw::c_int,
            day as std::os::raw::c_int,
            hour_ut,
            calendar.flag(),
        )
    })
}

/// A birth moment: wall-clock reading plus the offset in force at that place.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BirthMoment {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: f64,
    /// Hours east of UTC. India today is +5.5. For historical births take the
    /// offset from the birth record; the IANA tz database, for instance, gives
    /// Madras Mean Time as +5:21:10 (5.352778 h) for 1870-1906.
    pub utc_offset_hours: f64,
}

impl BirthMoment {
    pub fn to_jd_ut(&self) -> Result<f64, EphemError> {
        validate_civil_date(self.year, self.month, self.day)?;
        if self.hour > 23 || self.minute > 59 || !(0.0..60.0).contains(&self.second) {
            return Err(EphemError::InvalidTime {
                hour: self.hour,
                minute: self.minute,
                second: self.second,
            });
        }
        if !self.utc_offset_hours.is_finite()
            || !(-14.0..=14.0).contains(&self.utc_offset_hours)
        {
            return Err(EphemError::InvalidUtcOffset(self.utc_offset_hours));
        }

        let local_hour =
            self.hour as f64 + self.minute as f64 / 60.0 + self.second / 3600.0;

        // Build the JD at local midnight, then add the fractional day. Doing
        // it this way lets the offset push the instant across a date boundary
        // without any manual date arithmetic.
        let cal = Calendar::for_date(self.year, self.month, self.day);
        let jd_local_midnight = julian_day_ut(self.year, self.month, self.day, 0.0, cal)?;
        Ok(jd_local_midnight + (local_hour - self.utc_offset_hours) / 24.0)
    }
}

/// A calendar date/time reconstructed from a Julian Day.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CalendarDateTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: f64,
}

/// A Julian Day as a civil date: Julian calendar before the 1582 reform,
/// Gregorian from it onward, matching how [`BirthMoment`] reads its input.
///
/// Prefer this over [`jd_to_calendar`] with a hard-coded calendar. Formatting
/// every date as Gregorian shifts pre-1582 dates by 9-10 days, so a birth
/// entered as 1400-06-15 would be printed back as 1400-06-24.
pub fn jd_to_civil(jd_ut: f64, offset_hours: f64) -> CalendarDateTime {
    let cal = Calendar::for_jd(jd_ut + offset_hours / 24.0);
    jd_to_calendar(jd_ut, offset_hours, cal)
}

/// Inverse of [`julian_day_ut`]. Returns UT unless `offset_hours` shifts it.
pub fn jd_to_calendar(jd_ut: f64, offset_hours: f64, calendar: Calendar) -> CalendarDateTime {
    let jd = jd_ut + offset_hours / 24.0;
    let (mut y, mut mo, mut d) = (0i32, 0i32, 0i32);
    let mut ut = 0f64;
    unsafe { swe_sys::swe_revjul(jd, calendar.flag(), &mut y, &mut mo, &mut d, &mut ut) };

    // Guard against 23:59:59.9995 rounding up to a bogus 60th second.
    let total = (ut * 3600.0).round() / 3600.0;
    let hour = total.floor();
    let minute = ((total - hour) * 60.0).floor();
    let second = ((total - hour) * 60.0 - minute) * 60.0;

    CalendarDateTime {
        year: y,
        month: mo as u32,
        day: d as u32,
        hour: hour as u32,
        minute: minute as u32,
        second: if second.abs() < 1e-6 { 0.0 } else { second },
    }
}

#[cfg(test)]
mod roundtrip_tests {
    use super::*;

    #[test]
    fn jd_roundtrips_through_the_calendar() {
        for (y, m, d, h) in [
            (1947, 8, 15, 6.25),
            (1899, 12, 31, 23.5),
            (2000, 1, 1, 12.0),
            (2099, 6, 30, 0.0),
            (1582, 10, 15, 8.0),
        ] {
            let cal = Calendar::for_date(y, m, d);
            let jd = julian_day_ut(y, m, d, h, cal).unwrap();
            let back = jd_to_calendar(jd, 0.0, cal);
            assert_eq!((back.year, back.month, back.day), (y, m, d), "date {y}-{m}-{d}");
            let hh = back.hour as f64 + back.minute as f64 / 60.0 + back.second / 3600.0;
            assert!((hh - h).abs() < 1e-6, "hour {hh} vs {h}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_epoch_is_exact() {
        let jd = julian_day_ut(2000, 1, 1, 12.0, Calendar::Gregorian).unwrap();
        assert_eq!(jd, 2451545.0);
    }

    #[test]
    fn calendar_switches_at_the_gregorian_reform() {
        assert_eq!(Calendar::for_date(1582, 10, 14), Calendar::Julian);
        assert_eq!(Calendar::for_date(1582, 10, 15), Calendar::Gregorian);
    }

    #[test]
    fn ist_offset_is_subtracted() {
        // 1947-08-15 00:00 IST (+5:30) == 1947-08-14 18:30 UT.
        let m = BirthMoment {
            year: 1947, month: 8, day: 15,
            hour: 0, minute: 0, second: 0.0,
            utc_offset_hours: 5.5,
        };
        let jd = m.to_jd_ut().unwrap();
        let expected =
            julian_day_ut(1947, 8, 14, 18.5, Calendar::Gregorian).unwrap();
        assert!((jd - expected).abs() < 1e-9, "{jd} vs {expected}");
    }

    #[test]
    fn offset_can_cross_a_date_boundary_backwards() {
        // 1947-08-15 02:00 IST == 1947-08-14 20:30 UT - previous day.
        let m = BirthMoment {
            year: 1947, month: 8, day: 15,
            hour: 2, minute: 0, second: 0.0,
            utc_offset_hours: 5.5,
        };
        let jd = m.to_jd_ut().unwrap();
        let expected =
            julian_day_ut(1947, 8, 14, 20.5, Calendar::Gregorian).unwrap();
        assert!((jd - expected).abs() < 1e-9);
    }

    #[test]
    fn rejects_impossible_clock_times() {
        let m = BirthMoment {
            year: 1990, month: 1, day: 1,
            hour: 25, minute: 0, second: 0.0,
            utc_offset_hours: 5.5,
        };
        assert!(m.to_jd_ut().is_err());
    }
}
