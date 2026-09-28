//! Regression tests for defects found during Phase 1 QA.
//!
//! Each test names the defect, states what went wrong, and pins the corrected
//! behaviour. None of these were caught by the original 46 tests; all six were
//! found by exploratory probing of the assumptions the code documents about
//! itself.

use lagn_core::*;

fn ephe() {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe")
        .canonicalize()
        .unwrap();
    Ephemeris::set_ephemeris_path(d).unwrap();
}

fn chennai_1985() -> BirthData {
    BirthData {
        moment: BirthMoment {
            year: 1985, month: 6, day: 21,
            hour: 14, minute: 30, second: 0.0,
            utc_offset_hours: 5.5,
        },
        latitude: 13.0827,
        longitude: 80.2707,
        place_name: "Chennai".into(),
    }
}

// ---------------------------------------------------------------------------
// QA-1: norm360 violated its own [0, 360) contract
// ---------------------------------------------------------------------------
// `r + 360.0` rounds up to exactly 360.0 for any sufficiently small negative
// input. Sidereal longitudes come from subtracting the ayanamsa off a tropical
// longitude, so a value a few ulps below zero is reachable in normal operation.
// A longitude of 360.0 lands one cell past the end of the 12-, 27- and
// 108-fold grids that every downstream binning function uses.

#[test]
fn qa1_norm360_never_returns_360() {
    for v in [
        -1e-16, -1e-18, -1e-300, -f64::MIN_POSITIVE,
        -0.0, 0.0, 359.9999999999999, 360.0, -360.0, 720.0, -720.0,
    ] {
        let r = lagn_ephem::norm360(v);
        assert!(
            (0.0..360.0).contains(&r),
            "norm360({v:e}) = {r:.17}, outside [0, 360)"
        );
    }
}

#[test]
fn qa1_norm360_holds_across_a_dense_sweep_of_small_negatives() {
    // Walk the exponent range where the rounding bites.
    let mut mag = 1.0_f64;
    for _ in 0..320 {
        mag /= 2.0;
        let r = lagn_ephem::norm360(-mag);
        assert!(
            (0.0..360.0).contains(&r),
            "norm360(-{mag:e}) = {r:.17}"
        );
    }
}

#[test]
fn qa1_downstream_grids_stay_in_range_for_negative_epsilon() {
    // The consequence that made this worth fixing: a fraction_traversed of
    // 27.0 instead of 0.0 would produce a negative dasha balance.
    for v in [-1e-16, -1e-18, -f64::MIN_POSITIVE] {
        let np = NakshatraPosition::from_longitude(v);
        assert!(
            (0.0..1.0).contains(&np.fraction_traversed),
            "fraction_traversed {} for longitude {v:e}",
            np.fraction_traversed
        );
        assert!((1..=4).contains(&np.pada), "pada {} for {v:e}", np.pada);
        assert!(np.degrees_within >= 0.0 && np.degrees_within < 360.0 / 27.0);
    }
}

// ---------------------------------------------------------------------------
// QA-2: a non-finite Julian Day produced a complete, plausible, garbage chart
// ---------------------------------------------------------------------------
// swe_calc_ut(NaN, ..) returns SUCCESS with a NaN longitude. Every binning
// function then maps NaN to index 0 via a saturating float-to-int cast, so the
// result was a fully populated chart reading Mesha lagna, Ashwini pada 1 - with
// no error anywhere. This is precisely the silent-wrongness failure mode the
// whole project exists to avoid.

#[test]
fn qa2_non_finite_julian_day_is_rejected() {
    ephe();
    let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(e.position(bad, Graha::Sun).is_err(), "position accepted {bad}");
        assert!(e.positions(bad).is_err(), "positions accepted {bad}");
        assert!(e.ayanamsa_value(bad).is_err(), "ayanamsa accepted {bad}");
        assert!(
            e.angles(bad, 13.0, 80.0, HouseSystem::WholeSign).is_err(),
            "angles accepted {bad}"
        );
    }
}

#[test]
fn qa2_nan_would_have_silently_become_mesha() {
    // Documents why this mattered: the binning functions still do this, which
    // is fine only because NaN can no longer reach them.
    assert_eq!(Rasi::from_longitude(f64::NAN), Rasi::Mesha);
    assert_eq!(navamsa_sign(f64::NAN), Rasi::Mesha);
    assert_eq!(
        NakshatraPosition::from_longitude(f64::NAN).nakshatra,
        Nakshatra::Ashwini
    );
}

// ---------------------------------------------------------------------------
// QA-3: both geographic poles returned the same ascendant
// ---------------------------------------------------------------------------
// At exactly +/-90 the horizon coincides with the celestial equator and no
// point of the ecliptic rises. Swiss Ephemeris returned a number anyway, and
// the *same* number for north and south - not an approximation, just wrong.

#[test]
fn qa3_exact_poles_are_refused() {
    ephe();
    let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
    for lat in [90.0, -90.0] {
        let r = e.angles(2451545.0, lat, 0.0, HouseSystem::WholeSign);
        assert!(matches!(r, Err(EphemError::DegenerateAscendant(_))), "lat {lat}: {r:?}");
    }
}

#[test]
fn qa3_near_poles_still_work_and_distinguish_hemispheres() {
    ephe();
    let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
    let (n, _) = e.angles(2451545.0, 89.999, 0.0, HouseSystem::WholeSign).unwrap();
    let (s, _) = e.angles(2451545.0, -89.999, 0.0, HouseSystem::WholeSign).unwrap();
    assert_ne!(
        n.ascendant, s.ascendant,
        "north and south near-pole ascendants must differ"
    );
}

// ---------------------------------------------------------------------------
// QA-4: impossible calendar dates produced charts
// ---------------------------------------------------------------------------
// Swiss Ephemeris accepts 30 February and returns the Julian Day for 2 March.
// A single mistyped digit in a birth date therefore yielded a complete,
// plausible, wrong chart with no indication anything was amiss.

#[test]
fn qa4_impossible_dates_are_rejected() {
    for (y, m, d) in [
        (1985, 2, 30), (1985, 2, 31), (1985, 4, 31), (1985, 6, 31),
        (1985, 9, 31), (1985, 11, 31), (1985, 13, 1), (1985, 0, 1),
        (1985, 6, 0), (1985, 6, 32), (1900, 2, 29),
    ] {
        let cal = Calendar::for_date(y, m, d);
        assert!(
            julian_day_ut(y, m, d, 12.0, cal).is_err(),
            "{y}-{m:02}-{d:02} was accepted"
        );
    }
}

#[test]
fn qa4_impossible_dates_cannot_reach_a_chart() {
    ephe();
    let mut b = chennai_1985();
    b.moment.month = 2;
    b.moment.day = 30;
    assert!(
        Chart::compute(b, ChartSettings::default()).is_err(),
        "30 February produced a chart"
    );
}

#[test]
fn qa4_leap_rules_differ_between_calendars() {
    // 1900 is a leap year in the Julian calendar and not in the Gregorian.
    assert!(is_leap_year(1900, Calendar::Julian));
    assert!(!is_leap_year(1900, Calendar::Gregorian));
    assert!(is_leap_year(2000, Calendar::Gregorian));
    assert!(is_leap_year(1996, Calendar::Gregorian));
    assert!(!is_leap_year(1997, Calendar::Gregorian));

    assert_eq!(days_in_month(1900, 2, Calendar::Julian), 29);
    assert_eq!(days_in_month(1900, 2, Calendar::Gregorian), 28);
    assert_eq!(days_in_month(2000, 2, Calendar::Gregorian), 29);
}

#[test]
fn qa4_valid_leap_days_are_still_accepted() {
    for (y, m, d) in [(2000, 2, 29), (1996, 2, 29), (2024, 2, 29), (1600, 2, 29)] {
        let cal = Calendar::for_date(y, m, d);
        assert!(
            julian_day_ut(y, m, d, 12.0, cal).is_ok(),
            "{y}-{m:02}-{d:02} was wrongly rejected"
        );
    }
}

#[test]
fn qa4_the_gregorian_reform_gap_does_not_exist() {
    // 1582-10-05 through 10-14 were skipped by the reform.
    for d in 5..=14 {
        assert!(
            julian_day_ut(1582, 10, d, 12.0, Calendar::Gregorian).is_err(),
            "1582-10-{d:02} should not exist in the Gregorian calendar"
        );
    }
    assert!(julian_day_ut(1582, 10, 4, 12.0, Calendar::Julian).is_ok());
    assert!(julian_day_ut(1582, 10, 15, 12.0, Calendar::Gregorian).is_ok());
}

#[test]
fn qa4_years_outside_ephemeris_coverage_are_rejected_early() {
    for y in [0, -4000, 1199, 3001, 1_000_000, i32::MAX, i32::MIN] {
        assert!(
            julian_day_ut(y, 6, 15, 12.0, Calendar::Gregorian).is_err(),
            "year {y} was accepted"
        );
    }
    assert!(julian_day_ut(MIN_YEAR, 6, 15, 12.0, Calendar::Gregorian).is_ok());
    assert!(julian_day_ut(MAX_YEAR, 6, 15, 12.0, Calendar::Gregorian).is_ok());
}

// ---------------------------------------------------------------------------
// QA-5: JSON serialisation lost up to 1 ULP
// ---------------------------------------------------------------------------
// serde_json's default float parser is documented as lossy by at most one ULP.
// For a kernel whose entire contract is bit-identical reproducibility, and
// whose golden fixtures and public API both travel as JSON, that broke the
// contract at the boundary. Fixed by enabling serde_json's `float_roundtrip`.

#[test]
fn qa5_chart_survives_a_json_round_trip_bit_for_bit() {
    ephe();
    let c = Chart::compute(chennai_1985(), ChartSettings::default()).unwrap();
    let json = serde_json::to_string(&c).unwrap();
    let back: Chart = serde_json::from_str(&json).unwrap();
    assert_eq!(back, c, "chart changed across a JSON round trip");

    // And every float individually, so a failure names the field.
    for (a, b) in c.placements.iter().zip(back.placements.iter()) {
        assert_eq!(a.longitude.to_bits(), b.longitude.to_bits(), "{:?} longitude", a.graha);
        assert_eq!(a.speed_longitude.to_bits(), b.speed_longitude.to_bits(), "{:?} speed", a.graha);
        assert_eq!(
            a.nakshatra.fraction_traversed.to_bits(),
            b.nakshatra.fraction_traversed.to_bits(),
            "{:?} fraction_traversed", a.graha
        );
    }
    assert_eq!(c.jd_ut.to_bits(), back.jd_ut.to_bits());
    assert_eq!(c.ayanamsa_value.to_bits(), back.ayanamsa_value.to_bits());
}

#[test]
fn qa5_dasha_tree_survives_a_json_round_trip_bit_for_bit() {
    ephe();
    let c = Chart::compute(chennai_1985(), ChartSettings::default()).unwrap();
    let v = Vimshottari::compute(&c);
    let back: Vimshottari = serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap();
    assert_eq!(back, v, "dasha tree changed across a JSON round trip");

    for (a, b) in v.mahadashas.iter().zip(back.mahadashas.iter()) {
        assert_eq!(a.start_jd.to_bits(), b.start_jd.to_bits(), "{:?} start", a.lord);
        assert_eq!(a.end_jd.to_bits(), b.end_jd.to_bits(), "{:?} end", a.lord);
    }
}

#[test]
fn qa5_round_trip_holds_across_many_charts() {
    ephe();
    // A ULP-level defect is intermittent - it depends on the bit pattern - so
    // one chart is not evidence. Sweep enough to be sure.
    for day in 0..120 {
        let mut b = chennai_1985();
        b.moment.day = (day % 28) + 1;
        b.moment.month = (day % 12) + 1;
        b.moment.minute = day * 7 % 60;
        let c = Chart::compute(b, ChartSettings::default()).unwrap();
        let back: Chart = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back, c, "round trip failed on iteration {day}");
    }
}

// ---------------------------------------------------------------------------
// QA-6: the strict-mode Moshier guard had never been exercised
// ---------------------------------------------------------------------------
// The error type existed and was documented, but nothing proved it ever fires.
// An untested guard is not a guard.

#[test]
fn qa6_strict_mode_rejects_the_silent_moshier_fallback() {
    ephe();
    let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
    // JD for roughly year 1100 - before the earliest bundled .se1 file.
    let jd = 2122999.0;
    let r = e.position(jd, Graha::Sun);
    assert!(
        matches!(r, Err(EphemError::MoshierFallback { .. })),
        "expected a MoshierFallback error, got {r:?}"
    );
}

#[test]
fn qa6_relaxed_mode_permits_the_fallback_deliberately() {
    ephe();
    let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean).allow_moshier_fallback();
    let jd = 2122999.0;
    assert!(
        e.position(jd, Graha::Sun).is_ok(),
        "relaxed mode should allow the Moshier fallback"
    );
}

#[test]
fn qa6_dates_inside_coverage_never_trip_the_guard() {
    ephe();
    let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
    for y in [1200, 1400, 1600, 1800, 1801, 2000, 2399, 2401, 2900] {
        let jd = julian_day_ut(y, 6, 15, 12.0, Calendar::for_date(y, 6, 15)).unwrap();
        assert!(
            e.positions(jd).is_ok(),
            "year {y} is inside coverage but tripped the guard"
        );
    }
}

// ---------------------------------------------------------------------------
// QA-7: pre-1582 dates were printed back in the wrong calendar
// ---------------------------------------------------------------------------
// Input dates before the Gregorian reform are read as Julian, which is right.
// But dasha boundaries were always formatted as proleptic Gregorian, so a
// birth entered as 1400-06-15 was listed as starting on 1400-06-24 - the
// native's own birth date visibly shifted by nine days.

#[test]
fn qa7_civil_dates_round_trip_in_the_calendar_they_were_read_in() {
    // Every day across the reform window and a spread of eras.
    let mut checked = 0;
    for y in [1200, 1400, 1500, 1581, 1582, 1583, 1700, 1900, 2000, 2400] {
        for m in 1..=12u32 {
            for d in [1u32, 10, 15, 28] {
                // Only dates that are valid as civil input can be expected to
                // print back unchanged; the reform gap is not one of them.
                if validate_civil_date(y, m, d).is_err() { continue; }
                let cal = Calendar::for_date(y, m, d);
                let jd = julian_day_ut(y, m, d, 12.0, cal).unwrap();
                let back = jd_to_civil(jd, 0.0);
                assert_eq!(
                    (back.year, back.month, back.day), (y, m, d),
                    "{y}-{m:02}-{d:02} ({cal:?}) printed back as {}-{:02}-{:02}",
                    back.year, back.month, back.day
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 450, "only {checked} dates exercised");
}

#[test]
fn qa7_the_reform_boundary_is_seamless() {
    // 1582-10-04 (Julian) is immediately followed by 1582-10-15 (Gregorian).
    let last_julian = julian_day_ut(1582, 10, 4, 12.0, Calendar::Julian).unwrap();
    let first_greg = julian_day_ut(1582, 10, 15, 12.0, Calendar::Gregorian).unwrap();
    assert_eq!(first_greg - last_julian, 1.0, "the reform should skip no instants");
    let a = jd_to_civil(last_julian, 0.0);
    let b = jd_to_civil(first_greg, 0.0);
    assert_eq!((a.year, a.month, a.day), (1582, 10, 4));
    assert_eq!((b.year, b.month, b.day), (1582, 10, 15));
    assert_eq!(Calendar::for_jd(GREGORIAN_REFORM_JD), Calendar::Gregorian);
    assert_eq!(Calendar::for_jd(GREGORIAN_REFORM_JD - 1e-6), Calendar::Julian);
}

#[test]
fn qa7_a_pre_reform_birth_prints_its_own_birth_date() {
    ephe();
    let b = BirthData {
        moment: BirthMoment {
            year: 1400, month: 6, day: 15, hour: 12, minute: 0, second: 0.0,
            utc_offset_hours: 5.3514,
        },
        latitude: 13.0827, longitude: 80.2707, place_name: String::new(),
    };
    let c = Chart::compute(b, ChartSettings::default()).unwrap();
    let d = jd_to_civil(c.jd_ut, 5.3514);
    assert_eq!((d.year, d.month, d.day), (1400, 6, 15));
    assert_eq!((d.hour, d.minute), (12, 0));
}

#[test]
fn qa7_the_reform_gap_is_rejected_on_the_civil_input_path() {
    // Found by qa7_civil_dates_round_trip: these were read as Julian and
    // silently landed ten days later.
    for d in 5..=14 {
        let m = BirthMoment {
            year: 1582, month: 10, day: d, hour: 12, minute: 0, second: 0.0,
            utc_offset_hours: 0.0,
        };
        assert!(m.to_jd_ut().is_err(), "civil 1582-10-{d:02} was accepted");
    }
    // An explicitly Julian date in that range is still expressible.
    assert!(julian_day_ut(1582, 10, 10, 12.0, Calendar::Julian).is_ok());
    // The days either side of the gap remain valid civil dates.
    for d in [4, 15] {
        assert!(validate_civil_date(1582, 10, d).is_ok(), "1582-10-{d:02}");
    }
}
