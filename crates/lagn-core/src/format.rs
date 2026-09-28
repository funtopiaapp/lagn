//! Display formatting shared by every front end, so the CLI, the server and
//! the web app can never disagree about how a value is shown.

/// Degrees as `DD°MM'SS.ss"`, **truncated** to hundredths of a second.
///
/// Truncation, not rounding: a graha at 29°59'59.999" is still in the old
/// sign, and rounding would print 30°00'00.00" - a degree that belongs to the
/// next sign. Integer arithmetic makes 60 seconds or 60 minutes impossible.
///
/// Non-finite or negative input is shown as `--`, never as a plausible angle.
pub fn dms(deg: f64) -> String {
    if !deg.is_finite() || deg < 0.0 {
        return "--".to_string();
    }
    let centisec = (deg * 360_000.0).floor() as u64;
    let d = centisec / 360_000;
    let m = (centisec / 6_000) % 60;
    let cs = centisec % 6_000;
    format!("{:02}°{:02}'{:02}.{:02}\"", d, m, cs / 100, cs % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_known_values() {
        assert_eq!(dms(0.0), "00°00'00.00\"");
        assert_eq!(dms(10.5), "10°30'00.00\"");
        assert_eq!(dms(6.280181301367762), "06°16'48.65\"");
    }

    #[test]
    fn truncates_instead_of_rounding_up_into_the_next_degree() {
        assert_eq!(dms(29.999_999_999), "29°59'59.99\"");
        assert_eq!(dms(29.999_999), "29°59'59.99\"");
        assert_eq!(dms(59.999_999_9), "59°59'59.99\"");
    }

    #[test]
    fn never_shows_sixty_minutes_or_seconds() {
        let mut x = 0.0;
        while x < 360.0 {
            let s = dms(x);
            let m: u32 = s[s.find('°').unwrap() + '°'.len_utf8()..s.find('\'').unwrap()].parse().unwrap();
            let sec: f64 = s[s.find('\'').unwrap() + 1..s.len() - 1].parse().unwrap();
            assert!(m < 60 && sec < 60.0, "{x} -> {s}");
            x += 0.000_137;
        }
    }

    #[test]
    fn rejects_non_finite_and_negative() {
        assert_eq!(dms(f64::NAN), "--");
        assert_eq!(dms(f64::INFINITY), "--");
        assert_eq!(dms(-1.0), "--");
    }
}
