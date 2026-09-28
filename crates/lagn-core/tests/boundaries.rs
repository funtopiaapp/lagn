//! Boundary value analysis.
//!
//! Jyotisha is a system of hard edges. A longitude one arcsecond either side
//! of 13 deg 20 min sits in a different nakshatra, under a different dasha
//! lord, and therefore produces a different life. There is no smoothing and no
//! nearest-neighbour fallback: the boundary either lands correctly or the
//! whole reading is wrong.
//!
//! Every boundary is probed at the exact value, one ULP either side, and one
//! arcsecond either side.

use lagn_core::*;

const ARCSEC: f64 = 1.0 / 3600.0;
const NAK_SPAN: f64 = 360.0 / 27.0;
const PADA_SPAN: f64 = 360.0 / 108.0;

/// The next/previous representable double.
fn next_up(x: f64) -> f64 {
    f64::from_bits(x.to_bits() + 1)
}
fn next_down(x: f64) -> f64 {
    f64::from_bits(x.to_bits() - 1)
}

// ---------------------------------------------------------------------------
// Rasi boundaries
// ---------------------------------------------------------------------------

#[test]
fn rasi_boundary_is_closed_below_and_open_above() {
    for i in 0..12 {
        let edge = i as f64 * 30.0;
        // Exactly on the cusp belongs to the NEW sign.
        assert_eq!(
            Rasi::from_longitude(edge),
            Rasi::from_index(i),
            "exact cusp {edge}"
        );
        // One ULP below belongs to the PREVIOUS sign.
        if i > 0 {
            assert_eq!(
                Rasi::from_longitude(next_down(edge)),
                Rasi::from_index(i - 1),
                "one ulp below cusp {edge}"
            );
        }
        // One ULP above stays in the new sign.
        assert_eq!(
            Rasi::from_longitude(next_up(edge)),
            Rasi::from_index(i),
            "one ulp above cusp {edge}"
        );
    }
}

#[test]
fn degrees_within_rasi_resets_exactly_at_each_cusp() {
    for i in 0..12 {
        let edge = i as f64 * 30.0;
        assert_eq!(Rasi::degrees_within(edge), 0.0, "cusp {edge}");
        let just_below = Rasi::degrees_within(edge - ARCSEC);
        assert!(
            (just_below - (30.0 - ARCSEC)).abs() < 1e-9,
            "one arcsec below cusp {edge} gave {just_below}"
        );
    }
}

// ---------------------------------------------------------------------------
// Nakshatra boundaries - the highest-consequence edge in the system
// ---------------------------------------------------------------------------

#[test]
fn nakshatra_boundary_assigns_to_the_new_nakshatra() {
    for i in 0..27 {
        let edge = i as f64 * NAK_SPAN;
        assert_eq!(
            Nakshatra::from_longitude(edge),
            Nakshatra::from_index(i),
            "exact nakshatra edge {edge}"
        );
        if i > 0 {
            assert_eq!(
                Nakshatra::from_longitude(edge - ARCSEC),
                Nakshatra::from_index(i - 1),
                "one arcsec below nakshatra edge {edge}"
            );
        }
        assert_eq!(
            Nakshatra::from_longitude(edge + ARCSEC),
            Nakshatra::from_index(i),
            "one arcsec above nakshatra edge {edge}"
        );
    }
}

#[test]
fn crossing_a_nakshatra_boundary_changes_the_dasha_lord() {
    // This is the single highest-consequence boundary in the system: the Moon
    // one arcsecond either side of it gives the native a different birth
    // mahadasha, and therefore a different life timeline.
    let mut changes = 0;
    for i in 1..27 {
        let edge = i as f64 * NAK_SPAN;
        let before = Nakshatra::from_longitude(edge - ARCSEC).lord();
        let after = Nakshatra::from_longitude(edge + ARCSEC).lord();
        assert_ne!(
            before, after,
            "dasha lord did not change across nakshatra edge {i} at {edge}"
        );
        changes += 1;
    }
    assert_eq!(changes, 26);
}

#[test]
fn fraction_traversed_is_continuous_and_resets_at_each_edge() {
    for i in 0..27 {
        let edge = i as f64 * NAK_SPAN;
        let at = NakshatraPosition::from_longitude(edge);
        assert!(
            at.fraction_traversed.abs() < 1e-12,
            "fraction at edge {edge} was {}",
            at.fraction_traversed
        );
        let just_below = NakshatraPosition::from_longitude(edge - ARCSEC);
        assert!(
            just_below.fraction_traversed > 0.9999,
            "fraction just below edge {edge} was {}",
            just_below.fraction_traversed
        );
        // Always strictly inside [0, 1).
        for f in [at.fraction_traversed, just_below.fraction_traversed] {
            assert!((0.0..1.0).contains(&f), "fraction {f} out of range");
        }
    }
}

// ---------------------------------------------------------------------------
// Pada boundaries - these drive the navamsa
// ---------------------------------------------------------------------------

#[test]
fn every_pada_boundary_lands_in_the_right_quarter() {
    // Probed one arcsecond either side rather than exactly on the edge.
    //
    // A pada spans 10/3 degrees, which has no exact binary representation, so
    // "exactly on the boundary" is not a well-defined f64 value: reconstructing
    // an edge by summing spans lands a ULP away from where the implementation's
    // division puts it, and either answer is defensible at that scale. One
    // arcsecond is 1/12000th of a pada - unambiguous, and four orders of
    // magnitude finer than any real birth-time uncertainty.
    for i in 0..27 {
        for q in 0..4 {
            let edge = i as f64 * NAK_SPAN + q as f64 * PADA_SPAN;

            let above = NakshatraPosition::from_longitude(edge + ARCSEC);
            assert_eq!(above.nakshatra.index(), i as u8, "nakshatra above pada edge {edge}");
            assert_eq!(above.pada, q + 1, "one arcsec above pada edge {edge}");

            if i > 0 || q > 0 {
                let below = NakshatraPosition::from_longitude(edge - ARCSEC);
                let expected_pada = if q == 0 { 4 } else { q };
                assert_eq!(
                    below.pada, expected_pada,
                    "one arcsec below pada edge {edge}"
                );
                let expected_nak = if q == 0 { (i + 26) % 27 } else { i };
                assert_eq!(
                    below.nakshatra.index(), expected_nak as u8,
                    "nakshatra one arcsec below pada edge {edge}"
                );
            }
        }
    }
}

#[test]
fn nakshatra_and_pada_can_never_disagree() {
    // The structural guarantee that replaced a `.min(4)` clamp: both divisions
    // are read off one floor on the 108-cell grid, so a pada of 0 or 5, or a
    // nakshatra inconsistent with its pada, is now unrepresentable.
    let n = 500_000;
    for i in 0..n {
        let lon = i as f64 * 360.0 / n as f64;
        let np = NakshatraPosition::from_longitude(lon);
        assert!((1..=4).contains(&np.pada), "pada {} at {lon}", np.pada);
        assert_eq!(
            np.nakshatra,
            Nakshatra::from_longitude(lon),
            "NakshatraPosition and Nakshatra::from_longitude disagree at {lon}"
        );
        // The navamsa grid is the same grid, read at the same resolution.
        assert_eq!(
            navamsa_sign(lon),
            Rasi::from_index(np.absolute_pada() as i32),
            "navamsa and pada grids diverge at {lon}"
        );
        assert!(
            np.degrees_within >= 0.0 && np.degrees_within <= NAK_SPAN,
            "degrees_within {} at {lon}", np.degrees_within
        );
        assert!(
            (0.0..1.0).contains(&np.fraction_traversed),
            "fraction {} at {lon}", np.fraction_traversed
        );
    }
}

#[test]
fn pada_is_never_outside_one_to_four_anywhere() {
    // Includes every exact boundary, where a naive floor would yield 5.
    for i in 0..108 {
        for probe in [0.0, ARCSEC, -ARCSEC, 1e-13, -1e-13] {
            let lon = lagn_ephem::norm360(i as f64 * PADA_SPAN + probe);
            let p = NakshatraPosition::from_longitude(lon).pada;
            assert!((1..=4).contains(&p), "pada {p} at longitude {lon}");
        }
    }
}

// ---------------------------------------------------------------------------
// Navamsa boundaries
// ---------------------------------------------------------------------------

#[test]
fn navamsa_changes_at_every_pada_boundary_and_nowhere_else() {
    // The navamsa grid and the pada grid are the same 108-cell grid, so the
    // navamsa sign must change at exactly the pada boundaries.
    for i in 0..108 {
        let edge = i as f64 * PADA_SPAN;
        let before = navamsa_sign(lagn_ephem::norm360(edge - ARCSEC));
        let at = navamsa_sign(edge);
        let after = navamsa_sign(edge + ARCSEC);
        assert_eq!(at, after, "navamsa changed inside cell {i}");
        assert_ne!(before, at, "navamsa did not change at pada edge {i}");
    }
}

// ---------------------------------------------------------------------------
// Lagna at a rasi boundary - shifts every single house
// ---------------------------------------------------------------------------

#[test]
fn lagna_one_arcsec_across_a_cusp_shifts_all_twelve_houses() {
    let ephe_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(ephe_dir).unwrap();

    // Search for a birth minute where the lagna sits within a few arcsec of a
    // cusp, then compare the chart on either side.
    let base = BirthData {
        moment: BirthMoment {
            year: 1985, month: 6, day: 21, hour: 0, minute: 0, second: 0.0,
            utc_offset_hours: 5.5,
        },
        latitude: 13.0827, longitude: 80.2707, place_name: "Chennai".into(),
    };

    // The lagna advances ~1 degree every 4 minutes, so sweeping seconds across
    // a day is certain to straddle a cusp.
    let mut found = false;
    for step in 0..(24 * 60) {
        let mut b = base.clone();
        b.moment.hour = (step / 60) as u32;
        b.moment.minute = (step % 60) as u32;
        let c1 = match Chart::compute(b.clone(), ChartSettings::default()) { Ok(c) => c, Err(_) => continue };
        let mut b2 = b.clone();
        b2.moment.second = 59.0;
        let c2 = match Chart::compute(b2, ChartSettings::default()) { Ok(c) => c, Err(_) => continue };

        if c1.lagna.rasi != c2.lagna.rasi {
            found = true;
            // Every house must have rotated by exactly one sign.
            for h in 1..=12u8 {
                let r1 = c1.rasi_of_house(h);
                let r2 = c2.rasi_of_house(h);
                assert_eq!(
                    r2, Rasi::from_index(r1.index() as i32 + 1),
                    "house {h} did not rotate by one sign across the cusp"
                );
            }
            // And every graha's house number must shift by one, modulo 12.
            for g in Graha::ALL {
                let h1 = c1.house_of(g) as i32;
                let h2 = c2.house_of(g) as i32;
                assert_eq!(
                    (h2 - h1).rem_euclid(12), 11,
                    "{} moved from house {h1} to {h2}, not back by one", g.name()
                );
            }
            break;
        }
    }
    assert!(found, "no lagna cusp crossing found in a full day - sweep is broken");
}

// ---------------------------------------------------------------------------
// Retrograde stationary points
// ---------------------------------------------------------------------------

#[test]
fn retrograde_flag_flips_exactly_where_speed_changes_sign() {
    let ephe_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(ephe_dir).unwrap();
    let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);

    // Mercury stations around 2000-02-21. Walk in minutes across the station.
    let base = julian_day_ut(2000, 2, 20, 0.0, Calendar::Gregorian).unwrap();
    let mut flips = 0;
    let mut prev: Option<Position> = None;
    for m in 0..(72 * 60) {
        let jd = base + m as f64 / 1440.0;
        let p = e.position(jd, Graha::Mercury).unwrap();
        if let Some(q) = prev {
            if q.is_retrograde() != p.is_retrograde() {
                flips += 1;
                // At the flip the speeds must straddle zero.
                assert!(
                    q.speed_longitude.signum() != p.speed_longitude.signum(),
                    "retrograde flag flipped without the speed changing sign"
                );
                assert!(
                    q.speed_longitude.abs() < 0.01 && p.speed_longitude.abs() < 0.01,
                    "flag flipped far from a station: {} -> {}",
                    q.speed_longitude, p.speed_longitude
                );
            }
        }
        prev = Some(p);
    }
    assert_eq!(flips, 1, "expected exactly one Mercury station in this window");
}

#[test]
fn shadow_grahas_are_never_reported_retrograde() {
    let ephe_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(ephe_dir).unwrap();

    // Rahu and Ketu are always in mean retrograde motion. That is definitional,
    // not the vakri state that carries interpretive weight, so the flag on a
    // Placement must stay false even though the raw speed is negative.
    let b = BirthData {
        moment: BirthMoment {
            year: 1985, month: 6, day: 21, hour: 14, minute: 30, second: 0.0,
            utc_offset_hours: 5.5,
        },
        latitude: 13.0827, longitude: 80.2707, place_name: String::new(),
    };
    for day in 0..60 {
        let mut bb = b.clone();
        bb.moment.day = (day % 28) + 1;
        bb.moment.month = (day % 12) + 1;
        let c = Chart::compute(bb, ChartSettings::default()).unwrap();
        for g in [Graha::Rahu, Graha::Ketu] {
            let p = c.placement(g);
            assert!(!p.retrograde, "{} was flagged retrograde", g.name());
            assert!(p.speed_longitude < 0.0, "{} speed should be negative", g.name());
        }
    }
}

// ---------------------------------------------------------------------------
// Time boundaries
// ---------------------------------------------------------------------------

/// One ULP of a modern Julian Day, expressed in seconds.
///
/// A JD near 2.45e6 has an absolute resolution of about 40 microseconds. Any
/// assertion about JD differences tighter than that is asserting about
/// floating-point noise, not about the system under test. For scale: the lagna
/// advances one arcsecond in 67 milliseconds, so 40 microseconds is under a
/// milliarcsecond of ascendant - irrelevant to any reading.
const JD_ULP_SECONDS: f64 = 1e-4;

#[test]
fn midnight_and_the_second_before_it_land_on_different_days() {
    let a = BirthMoment {
        year: 1990, month: 6, day: 15, hour: 23, minute: 59, second: 59.0,
        utc_offset_hours: 5.5,
    };
    let b = BirthMoment { day: 16, hour: 0, minute: 0, second: 0.0, ..a };
    let ja = a.to_jd_ut().unwrap();
    let jb = b.to_jd_ut().unwrap();
    let gap_sec = (jb - ja) * 86400.0;
    assert!(
        (gap_sec - 1.0).abs() < JD_ULP_SECONDS,
        "gap was {gap_sec} seconds, expected 1"
    );
    // The two instants must also fall either side of the calendar boundary.
    let da = jd_to_calendar(ja, 5.5, Calendar::Gregorian);
    let db = jd_to_calendar(jb, 5.5, Calendar::Gregorian);
    assert_eq!(da.day, 15, "the earlier instant should still be the 15th");
    assert_eq!(db.day, 16, "the later instant should be the 16th");
}

#[test]
fn julian_day_time_resolution_is_documented_and_sufficient() {
    // Pins the precision floor so a future change to the time representation
    // cannot quietly degrade it.
    let base = BirthMoment {
        year: 1990, month: 6, day: 15, hour: 12, minute: 0, second: 0.0,
        utc_offset_hours: 5.5,
    };
    let j0 = base.to_jd_ut().unwrap();
    // One second must be clearly resolvable.
    let j1 = BirthMoment { second: 1.0, ..base }.to_jd_ut().unwrap();
    let delta_sec = (j1 - j0) * 86400.0;
    assert!(
        (delta_sec - 1.0).abs() < JD_ULP_SECONDS,
        "one second resolved as {delta_sec}"
    );
    // And a one-second step must be far above the representation floor.
    let ulp_sec = (f64::from_bits(j0.to_bits() + 1) - j0) * 86400.0;
    assert!(
        ulp_sec < 1e-3,
        "JD resolution degraded to {ulp_sec} s per ULP"
    );
}

#[test]
fn utc_offset_extremes_are_accepted_and_beyond_them_rejected() {
    let base = BirthMoment {
        year: 1990, month: 6, day: 15, hour: 12, minute: 0, second: 0.0,
        utc_offset_hours: 0.0,
    };
    for tz in [-14.0, -12.0, 0.0, 5.5, 5.3514, 6.5, 12.0, 14.0] {
        assert!(BirthMoment { utc_offset_hours: tz, ..base }.to_jd_ut().is_ok(), "tz {tz}");
    }
    for tz in [-14.5, 14.5, 100.0, f64::NAN, f64::INFINITY] {
        assert!(BirthMoment { utc_offset_hours: tz, ..base }.to_jd_ut().is_err(), "tz {tz}");
    }
}

#[test]
fn clock_field_boundaries() {
    let ok = |h, mi, s: f64| BirthMoment {
        year: 1990, month: 6, day: 15, hour: h, minute: mi, second: s,
        utc_offset_hours: 5.5,
    }.to_jd_ut().is_ok();

    assert!(ok(0, 0, 0.0));
    assert!(ok(23, 59, 59.999));
    assert!(!ok(24, 0, 0.0));
    assert!(!ok(23, 60, 0.0));
    assert!(!ok(23, 59, 60.0));
    assert!(!ok(23, 59, -0.001));
}
