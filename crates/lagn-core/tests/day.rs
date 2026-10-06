//! A day's timings and panchanga. Specification: the printed Tamil and Kerala
//! panchangam tables, which are fixed data rather than a derivation.

use lagn_core::day::{
    day_timings, karana_name, paksha, panchanga, tithi_name, yoga_name, Vara,
};
use lagn_core::{julian_day_ut, Calendar, Ephemeris};
use std::path::Path;

fn init() {
    let d = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(d).unwrap();
}

/// Noon UT on a date, as a starting point for a sunrise search.
fn noon(y: i32, m: u32, d: u32) -> f64 {
    julian_day_ut(y, m, d, 12.0, Calendar::Gregorian).unwrap()
}

#[test]
fn the_weekday_of_a_sunrise_is_the_classical_one() {
    // 2026-10-04 was a Sunday; 2026-10-05 a Monday. The sunrise is what counts,
    // so a JD in the morning of each must give that day.
    let sunday = julian_day_ut(2026, 10, 4, 1.0, Calendar::Gregorian).unwrap();
    assert_eq!(Vara::from_sunrise_jd(sunday), Vara::Ravi);
    assert_eq!(Vara::from_sunrise_jd(sunday).english(), "Sunday");
    assert_eq!(Vara::from_sunrise_jd(sunday + 1.0), Vara::Soma);

    // All seven, in order, and each one's lord.
    let names: Vec<&str> = (0..7).map(|i| Vara::from_sunrise_jd(sunday + i as f64).english()).collect();
    assert_eq!(names, ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]);
    assert_eq!(Vara::Mangala.lord(), lagn_core::Graha::Mars);
    assert_eq!(Vara::Shani.tamil_name(), "Sani");
}

#[test]
fn the_eight_parts_tile_the_daylight_exactly() {
    init();
    let eph = Ephemeris::new(lagn_core::Ayanamsa::Lahiri, lagn_core::NodeType::Mean);
    // Madurai, a date in each season, so the day length varies.
    for (y, m, d) in [(2026, 1, 15), (2026, 4, 15), (2026, 7, 15), (2026, 10, 15)] {
        let sun = eph.sun_day(noon(y, m, d) - 0.5, 9.9252, 78.1198).expect("sunrise");
        let t = day_timings(sun.sunrise_jd, sun.sunset_jd);

        assert!(t.sunset_jd > t.sunrise_jd, "{y}-{m}: sunset before sunrise");
        let eighth = (t.sunset_jd - t.sunrise_jd) / 8.0;

        // Each avoided segment is exactly one eighth, and inside the daylight.
        for (name, seg) in [("rahu", t.rahu_kalam), ("yama", t.yamagandam), ("kuligai", t.kuligai)] {
            let len = seg.end_jd - seg.start_jd;
            assert!((len - eighth).abs() < 1e-9, "{name} is {len} not {eighth}");
            assert!(seg.start_jd >= t.sunrise_jd - 1e-9 && seg.end_jd <= t.sunset_jd + 1e-9, "{name} outside daylight");
            assert!((1..=8).contains(&seg.part));
        }
        // The three never land on the same eighth as each other.
        let parts = [t.rahu_kalam.part, t.yamagandam.part, t.kuligai.part];
        assert_eq!(
            parts.iter().collect::<std::collections::BTreeSet<_>>().len(),
            3,
            "two segments share an eighth on {y}-{m}-{d}: {parts:?}"
        );

        // Abhijit is the middle fifteenth, so it straddles the midpoint.
        let midday = t.sunrise_jd + (t.sunset_jd - t.sunrise_jd) / 2.0;
        assert!(t.abhijit.start_jd < midday && midday < t.abhijit.end_jd, "abhijit misses midday");
    }
}

#[test]
fn rahu_kalam_takes_the_eighth_each_weekday_table_gives() {
    init();
    let eph = Ephemeris::new(lagn_core::Ayanamsa::Lahiri, lagn_core::NodeType::Mean);
    // 2026-10-04 is a Sunday. The printed table: Sunday the 8th eighth,
    // Monday the 2nd, Tuesday the 7th, Wednesday the 5th, Thursday the 6th,
    // Friday the 4th, Saturday the 3rd.
    let want = [(Vara::Ravi, 8), (Vara::Soma, 2), (Vara::Mangala, 7), (Vara::Budha, 5),
                (Vara::Guru, 6), (Vara::Shukra, 4), (Vara::Shani, 3)];
    let start = noon(2026, 10, 4) - 0.5;
    for (i, (vara, part)) in want.iter().enumerate() {
        let sun = eph.sun_day(start + i as f64, 9.9252, 78.1198).expect("sunrise");
        let t = day_timings(sun.sunrise_jd, sun.sunset_jd);
        assert_eq!(t.vara, *vara, "day {i} is the wrong vara");
        assert_eq!(t.rahu_kalam.part, *part, "{} rahu kalam", vara.english());
    }
}

#[test]
fn sunrise_is_where_an_almanac_puts_it() {
    init();
    let eph = Ephemeris::new(lagn_core::Ayanamsa::Lahiri, lagn_core::NodeType::Mean);
    // Madurai, 2026-10-04. Sunrise is about 06:10 IST and sunset about 18:03,
    // so the day is a little under twelve hours this close to the equinox.
    let sun = eph.sun_day(noon(2026, 10, 4) - 0.5, 9.9252, 78.1198).expect("sunrise");
    let ist = |jd: f64| {
        let c = lagn_core::jd_to_civil(jd, 5.5);
        c.hour as f64 + c.minute as f64 / 60.0
    };
    let (rise, set) = (ist(sun.sunrise_jd), ist(sun.sunset_jd));
    assert!((5.9..=6.4).contains(&rise), "sunrise at {rise} IST");
    assert!((17.8..=18.3).contains(&set), "sunset at {set} IST");
    // Near the equinox the daylight is close to twelve hours.
    let hours = sun.day_length() * 24.0;
    assert!((11.5..=12.5).contains(&hours), "daylight {hours} hours");
}

#[test]
fn a_polar_day_with_no_sunrise_is_refused_rather_than_guessed() {
    init();
    let eph = Ephemeris::new(lagn_core::Ayanamsa::Lahiri, lagn_core::NodeType::Mean);
    // Longyearbyen in December: the Sun does not rise at all.
    let r = eph.sun_day(noon(2026, 12, 21) - 0.5, 78.22, 15.63);
    assert!(r.is_err(), "a day with no sunrise must not return one");
}

#[test]
fn the_panchanga_limbs_are_computed_from_the_two_longitudes() {
    // New moon: the Moon is with the Sun, so elongation 0 means the first
    // tithi of the waxing fortnight.
    let p = panchanga(100.0, 100.0, 0.0);
    assert_eq!(p.tithi, 1);
    assert_eq!(paksha(p.tithi), "Shukla paksha (waxing)");

    // Opposition: full moon ends the fifteenth tithi.
    let p = panchanga(0.0, 179.9, 0.0);
    assert_eq!(p.tithi, 15);
    assert_eq!(tithi_name(15), "Purnima");

    // Just past opposition begins the waning fortnight.
    let p = panchanga(0.0, 181.0, 0.0);
    assert_eq!(p.tithi, 16);
    assert_eq!(paksha(p.tithi), "Krishna paksha (waning)");
    assert_eq!(tithi_name(16), "Prathamai");
    assert_eq!(tithi_name(30), "Amavasai");

    // Every tithi, yoga and karana in range has a name, and none is "unknown".
    for t in 1..=30u8 {
        assert_ne!(tithi_name(t), "unknown", "tithi {t}");
    }
    for y in 1..=27u8 {
        assert_ne!(yoga_name(y), "unknown", "yoga {y}");
    }
    for k in 1..=60u8 {
        assert_ne!(karana_name(k), "unknown", "karana {k}");
    }
    // A karana is half a tithi, so it advances twice as fast.
    assert_eq!(panchanga(0.0, 5.0, 0.0).karana, 1);
    assert_eq!(panchanga(0.0, 7.0, 0.0).karana, 2);
}

#[test]
fn the_panchanga_is_bounded_over_a_whole_year_of_real_positions() {
    init();
    let eph = Ephemeris::new(lagn_core::Ayanamsa::Lahiri, lagn_core::NodeType::Mean);
    let mut seen_tithis = std::collections::BTreeSet::new();
    for d in 0..365 {
        let jd = noon(2026, 1, 1) + d as f64;
        let sun = eph.position(jd, lagn_core::Graha::Sun).expect("sun");
        let moon = eph.position(jd, lagn_core::Graha::Moon).expect("moon");
        let p = panchanga(sun.longitude, moon.longitude, jd);
        assert!((1..=30).contains(&p.tithi), "tithi {} out of range", p.tithi);
        assert!((1..=27).contains(&p.yoga), "yoga {} out of range", p.yoga);
        assert!((1..=60).contains(&p.karana), "karana {} out of range", p.karana);
        seen_tithis.insert(p.tithi);
    }
    // A year passes through every tithi many times over.
    assert_eq!(seen_tithis.len(), 30, "only saw {} distinct tithis", seen_tithis.len());
}
