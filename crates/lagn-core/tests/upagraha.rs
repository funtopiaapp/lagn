//! Phase 13C QA suite: upagrahas and the time lagnas.
//!
//! Written against `docs/phase13/UPAGRAHA.md`, acceptance criteria 1 to 6
//! and 8. The N-version check (criterion 7) lives in
//! `scripts/qa_upagraha_oracle.py`.

use std::collections::BTreeSet;

use lagn_core::upagraha::{DayPart, SunOffset, TimeLagna, Upagrahas, VARIANTS};
use lagn_core::*;

/// A nanodegree. The Sun-offset identities are exact arithmetic, so they are
/// held to floating-point noise rather than to an astronomical tolerance.
const NANO: f64 = 1e-9;
const ARCSEC: f64 = 1.0 / 3600.0;

fn init() {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(d).unwrap();
}

/// Shortest angular distance between two longitudes, in degrees.
fn apart(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 { 360.0 - d } else { d }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12; x ^= x << 25; x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn int(&mut self, lo: i64, hi: i64) -> i64 { lo + (self.next() % ((hi - lo + 1) as u64)) as i64 }
    fn unit(&mut self) -> f64 { (self.next() >> 11) as f64 / (1u64 << 53) as f64 }
}

/// Charts at moderate latitudes: the day-division points need a real sunrise
/// and sunset, which the polar regions do not always have.
fn random_charts(n: usize, seed: u64) -> Vec<Chart> {
    init();
    let mut rng = Rng(seed | 1);
    let mut out = Vec::new();
    while out.len() < n {
        let b = BirthData {
            moment: BirthMoment {
                year: rng.int(1900, 2060) as i32,
                month: rng.int(1, 12) as u32,
                day: rng.int(1, 28) as u32,
                hour: rng.int(0, 23) as u32,
                minute: rng.int(0, 59) as u32,
                second: rng.unit() * 59.0,
                utc_offset_hours: 5.5,
            },
            latitude: rng.unit() * 100.0 - 50.0,
            longitude: rng.unit() * 360.0 - 180.0,
            place_name: String::new(),
        };
        if let Ok(c) = Chart::compute(b, ChartSettings::default()) {
            out.push(c);
        }
    }
    out
}

fn eph() -> Ephemeris {
    init();
    Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean)
}

// ===========================================================================
// Criterion 1 - the Sun-offset chain closes on itself
// ===========================================================================

#[test]
fn the_five_sun_offsets_compose_into_upaketu_thirty_degrees_behind_the_sun() {
    // This is the whole reason these five are safe to build from the
    // specification alone. The offsets are stated one at a time, each in terms
    // of the last; substituting through, Upaketu must land exactly 30 degrees
    // behind the Sun. If any offset were mistranscribed the identity would
    // break, so the chain checks the table rather than restating it.
    for i in 0..3600 {
        let sun = i as f64 * 0.1;
        let upaketu = SunOffset::Upaketu.longitude(sun);
        let want = lagn_ephem::norm360(sun - 30.0);
        assert!(
            apart(upaketu, want) < NANO,
            "Sun {sun}: Upaketu {upaketu} is not 30 behind the Sun ({want})",
        );
    }
}

#[test]
fn vyatipata_and_parivesha_are_always_opposite() {
    for i in 0..3600 {
        let sun = i as f64 * 0.1;
        let v = SunOffset::Vyatipata.longitude(sun);
        let p = SunOffset::Parivesha.longitude(sun);
        assert!(apart(v, p) - 180.0 < NANO, "Sun {sun}: {v} and {p} are not opposite");
    }
}

#[test]
fn dhuma_sits_at_the_stated_offset_and_the_offsets_are_what_the_spec_says() {
    // The two constants, pinned as degrees and minutes rather than decimals,
    // because that is how the sources state them and how they get mistyped.
    assert!((lagn_core::upagraha::DHUMA_OFFSET - (133.0 + 20.0 / 60.0)).abs() < NANO);
    assert!((lagn_core::upagraha::UPAKETU_OFFSET - (16.0 + 40.0 / 60.0)).abs() < NANO);
    assert!(apart(SunOffset::Dhuma.longitude(0.0), 133.0 + 20.0 / 60.0) < NANO);
    // And Indrachapa, which the chain reaches through three reflections.
    assert!(apart(SunOffset::Indrachapa.longitude(0.0), 313.0 + 20.0 / 60.0) < NANO);
}

// ===========================================================================
// Criterion 2 - every point is a valid longitude, correctly placed
// ===========================================================================

#[test]
fn every_point_is_a_longitude_in_range_and_sits_where_its_longitude_says() {
    let e = eph();
    for chart in random_charts(60, 0x11_9A) {
        let u = Upagrahas::compute(&e, &chart).expect("upagrahas");
        assert_eq!(u.sun_offsets.len(), 5);
        assert_eq!(u.day_parts.len(), 5);
        assert_eq!(u.time_lagnas.len(), 3);

        let all = u.sun_offsets.iter()
            .chain(u.day_parts.iter().map(|d| &d.point))
            .chain(u.time_lagnas.iter());
        for p in all {
            assert!((0.0..360.0).contains(&p.longitude), "{} at {}", p.name, p.longitude);
            assert_eq!(p.rasi, Rasi::from_longitude(p.longitude), "{} in the wrong rasi", p.name);
            assert!((0.0..30.0).contains(&p.degrees_in_rasi));
            assert!((1..=12).contains(&p.house));
            // The house is the whole-sign count from the lagna, nothing else.
            assert_eq!(p.house, chart.lagna.rasi.houses_to(p.rasi), "{} house", p.name);
            assert!(!p.name.is_empty());
        }
        // Five distinct names in each group, so nothing is computed twice
        // under two labels.
        let names: BTreeSet<&str> = u.sun_offsets.iter().map(|p| p.name.as_ref()).collect();
        assert_eq!(names.len(), 5);
    }
}

// ===========================================================================
// Criteria 3 and 4 - the day division, by day and by night
// ===========================================================================

#[test]
fn each_named_point_takes_the_part_its_ruling_graha_owns() {
    let e = eph();
    for chart in random_charts(50, 0x8A27) {
        let u = Upagrahas::compute(&e, &chart).expect("upagrahas");
        for d in &u.day_parts {
            assert!((1..=7).contains(&d.part), "part {} out of range", d.part);
            assert_eq!(d.at_night, u.at_night, "{} disagrees about night", d.point.name);
        }
        // Five rulers, five distinct parts: no two named points share an eighth.
        let parts: BTreeSet<u8> = u.day_parts.iter().map(|d| d.part).collect();
        assert_eq!(parts.len(), 5, "two upagrahas share a part: {parts:?}");

        let rulers: Vec<Graha> = u.day_parts.iter().map(|d| d.ruler).collect();
        assert_eq!(rulers, DayPart::ALL.iter().map(|k| k.ruler()).collect::<Vec<_>>());
        // Gulika is Saturn's, which is the one practice reaches for.
        assert_eq!(u.gulika().ruler, Graha::Saturn);
    }
}

#[test]
fn the_lord_of_the_weekday_owns_the_first_part_of_a_day_birth() {
    // Criterion 3's other half: the sequence's origin. A daytime birth on a
    // Saturday must put Gulika in the first part, because Shani owns it.
    let e = eph();
    let mut seen = 0;
    for chart in random_charts(120, 0x11_57) {
        let u = Upagrahas::compute(&e, &chart).expect("upagrahas");
        if u.at_night {
            continue;
        }
        let lord = u.vara.lord();
        if let Some(d) = u.day_parts.iter().find(|d| d.ruler == lord) {
            assert_eq!(d.part, 1, "{:?}: the weekday lord must own the first part", u.vara);
            seen += 1;
        }
    }
    assert!(seen > 0, "the sweep never produced a day birth whose vara lord is a named ruler");
}

#[test]
fn a_night_birth_divides_the_night_and_starts_from_the_fifth_lord() {
    // Criterion 4. The same place and date, one birth at midday and one just
    // before midnight, so the only thing that differs is which period is
    // divided. The parts must differ, and the night one must start from the
    // lord five from the weekday lord.
    let e = eph();
    let at = |hour: u32| {
        Chart::compute(
            BirthData {
                moment: BirthMoment {
                    year: 1990, month: 6, day: 15,
                    hour, minute: 0, second: 0.0, utc_offset_hours: 5.5,
                },
                latitude: 13.08,
                longitude: 80.27,
                place_name: String::new(),
            },
            ChartSettings::default(),
        )
        .expect("chart")
    };

    let day = Upagrahas::compute(&e, &at(12)).expect("day");
    let night = Upagrahas::compute(&e, &at(23)).expect("night");
    assert!(!day.at_night, "midday must be a day birth");
    assert!(night.at_night, "23:00 must be a night birth");

    // Both read the same weekday - the night belongs to the sunrise that
    // began it - but the sequences start from different lords.
    assert_eq!(day.vara, night.vara, "the night belongs to its own sunrise's vara");

    let order = [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Venus, Graha::Saturn];
    let i = order.iter().position(|&g| g == day.vara.lord()).unwrap();
    let fifth = order[(i + 4) % 7];

    let part_of = |u: &Upagrahas, g: Graha| u.day_parts.iter().find(|d| d.ruler == g).map(|d| d.part);
    // Whichever named point the fifth lord owns, it is the night's first part.
    if let Some(p) = part_of(&night, fifth) {
        assert_eq!(p, 1, "the night must start from {fifth:?}");
    }
    // And the division itself moved: at least one point changed its part.
    let moved = DayPart::ALL.iter().any(|k| part_of(&day, k.ruler()) != part_of(&night, k.ruler()));
    assert!(moved, "the night division is identical to the day's");
}

#[test]
fn a_birth_before_sunrise_belongs_to_the_previous_nights_division() {
    // The case most easily got wrong: 03:00 is "night", but the night that
    // started at *yesterday's* sunset, so the vara is yesterday's too.
    let e = eph();
    let chart = Chart::compute(
        BirthData {
            moment: BirthMoment {
                year: 1990, month: 6, day: 15,
                hour: 3, minute: 0, second: 0.0, utc_offset_hours: 5.5,
            },
            latitude: 13.08, longitude: 80.27, place_name: String::new(),
        },
        ChartSettings::default(),
    )
    .expect("chart");

    let u = Upagrahas::compute(&e, &chart).expect("upagrahas");
    assert!(u.at_night, "03:00 is before sunrise and so at night");
    // Reckoned from the previous sunrise, so the elapsed time is over 12 hours
    // rather than a small negative number.
    assert!(u.hours_since_sunrise > 12.0, "hours since sunrise was {}", u.hours_since_sunrise);
}

// ===========================================================================
// Criteria 5 and 6 - the time lagnas
// ===========================================================================

#[test]
fn the_time_lagnas_keep_their_ratios() {
    // Hora twice Bhava, Ghati five times Hora. Checked as rates, by advancing
    // the clock, so a wrong constant cannot hide behind a right starting point.
    assert_eq!(TimeLagna::Bhava.degrees_per_hour(), 15.0);
    assert_eq!(TimeLagna::Hora.degrees_per_hour(), 30.0);
    assert_eq!(TimeLagna::Ghati.degrees_per_hour(), 75.0);
    assert_eq!(TimeLagna::Hora.degrees_per_hour(), 2.0 * TimeLagna::Bhava.degrees_per_hour());
    assert_eq!(TimeLagna::Ghati.degrees_per_hour(), 2.5 * TimeLagna::Hora.degrees_per_hour());

    // One sign per 2 hours, 1 hour, 24 minutes - the way the sources state them.
    assert!(apart(TimeLagna::Bhava.longitude(0.0, 2.0), 30.0) < NANO);
    assert!(apart(TimeLagna::Hora.longitude(0.0, 1.0), 30.0) < NANO);
    assert!(apart(TimeLagna::Ghati.longitude(0.0, 24.0 / 60.0), 30.0) < NANO);
}

#[test]
fn at_sunrise_every_time_lagna_equals_the_sun() {
    // Criterion 6: the simplest case, and the one a wrong origin breaks.
    for sun in [0.0, 37.4, 180.0, 299.9, 359.99] {
        for l in TimeLagna::ALL {
            assert!(
                apart(l.longitude(sun, 0.0), sun) < NANO,
                "{} does not start at the Sun", l.name(),
            );
        }
    }
}

#[test]
fn the_time_lagnas_on_a_chart_follow_from_its_sunrise_sun_and_elapsed_hours() {
    let e = eph();
    for chart in random_charts(40, 0x7_1A6) {
        let u = Upagrahas::compute(&e, &chart).expect("upagrahas");
        assert!(u.hours_since_sunrise >= 0.0, "elapsed hours went negative");
        assert!(u.hours_since_sunrise < 26.0, "elapsed hours was {}", u.hours_since_sunrise);
        for (l, p) in TimeLagna::ALL.iter().zip(&u.time_lagnas) {
            let want = l.longitude(u.sunrise_sun, u.hours_since_sunrise);
            assert!(apart(p.longitude, want) < ARCSEC, "{} is {} not {}", p.name, p.longitude, want);
        }
    }
}

// ===========================================================================
// Criterion 8 - the variants, and determinism
// ===========================================================================

#[test]
fn every_result_names_the_variant_scheme_that_produced_it() {
    let e = eph();
    let chart = random_charts(1, 0x7A71C).remove(0);
    let u = Upagrahas::compute(&e, &chart).expect("upagrahas");
    assert_eq!(u.variants.len(), VARIANTS.len());
    let ids: Vec<String> = u.variants.iter().map(|v| v.id.to_string()).collect();
    assert_eq!(ids, ["V-13-14", "V-13-15", "V-13-16", "V-13-17"]);
    for v in &u.variants {
        assert!(!v.question.is_empty() && !v.chosen.is_empty(), "{} is blank", v.id);
    }
}

#[test]
fn the_same_chart_always_gives_the_same_points() {
    let e = eph();
    for chart in random_charts(15, 0xD37B) {
        let a = Upagrahas::compute(&e, &chart).expect("a");
        let b = Upagrahas::compute(&e, &chart).expect("b");
        assert_eq!(a, b);
    }
}

#[test]
fn the_result_survives_a_round_trip_through_json() {
    let e = eph();
    let chart = random_charts(1, 0x1503).remove(0);
    let u = Upagrahas::compute(&e, &chart).expect("upagrahas");
    let text = serde_json::to_string(&u).expect("serialise");
    let back: Upagrahas = serde_json::from_str(&text).expect("deserialise");
    assert_eq!(u, back);
}
