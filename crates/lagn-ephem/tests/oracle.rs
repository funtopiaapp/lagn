//! Layer 1 verification against `swetest`, the reference CLI that ships with
//! Swiss Ephemeris.
//!
//! Every expected value below was produced by:
//!
//! ```text
//! swetest -b1.1.2000 -ut12:00 -p0123456mt -sid1 -fPlsj -eswe -edir<ephe>
//! swetest -b1.1.2000 -ut12:00 -house80.2707,13.0827,W -sid1 -eswe -edir<ephe>
//! ```
//!
//! with Swiss Ephemeris 2.10.03. If these ever drift, either our flags changed
//! or the vendored library was upgraded - both are things we want to be told
//! about loudly.

use lagn_ephem::*;

/// Matches the precision `swetest` prints (1e-7 deg is ~0.4 milliarcsec).
const TOL: f64 = 1e-6;

fn ephe() -> Ephemeris {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe")
        .canonicalize()
        .expect("ephe directory must exist at repo root");
    Ephemeris::set_ephemeris_path(dir).expect("set ephemeris path");
    Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean)
}

/// J2000.0 exactly: JD 2451545.0 UT.
const J2000: f64 = 2451545.0;

fn assert_deg(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() < TOL,
        "{what}: got {actual:.7}, expected {expected:.7} (diff {:.3e})",
        (actual - expected).abs()
    );
}

#[test]
fn lahiri_ayanamsa_matches_reference() {
    // swetest prints 23 deg 51' 11.6009" (Lahiri) at J2000.
    let expected = 23.0 + 51.0 / 60.0 + 11.6009 / 3600.0;
    let got = ephe().ayanamsa_value(J2000).unwrap();
    assert!(
        (got - expected).abs() < 1e-7,
        "ayanamsa: got {got:.9}, expected {expected:.9}"
    );
}

#[test]
fn sidereal_longitudes_match_reference() {
    let e = ephe();
    let cases = [
        (Graha::Sun, 256.5156962),
        (Graha::Moon, 199.4705290),
        (Graha::Mercury, 248.0360546),
        (Graha::Venus, 217.7125659),
        (Graha::Mars, 304.1100800),
        (Graha::Jupiter, 1.3998647),
        (Graha::Saturn, 16.5424410),
        // Mean node - our default Rahu.
        (Graha::Rahu, 101.1874236),
    ];
    for (graha, expected) in cases {
        let p = e.position(J2000, graha).unwrap();
        assert_deg(p.longitude, expected, graha.name());
    }
}

#[test]
fn true_node_is_selectable_and_differs_from_mean() {
    let mean = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
    let tru = Ephemeris::new(Ayanamsa::Lahiri, NodeType::True);
    let _ = ephe(); // ensure path is set

    assert_deg(
        mean.position(J2000, Graha::Rahu).unwrap().longitude,
        101.1874236,
        "mean node",
    );
    assert_deg(
        tru.position(J2000, Graha::Rahu).unwrap().longitude,
        100.1008004,
        "true node",
    );
}

#[test]
fn ketu_is_exactly_opposite_rahu() {
    let e = ephe();
    // Sample a whole nodal cycle (~18.6 years) so this cannot pass by luck.
    for i in 0..200 {
        let jd = J2000 + (i as f64) * 34.0;
        let rahu = e.position(jd, Graha::Rahu).unwrap();
        let ketu = e.position(jd, Graha::Ketu).unwrap();
        let sep = norm360(ketu.longitude - rahu.longitude);
        assert!(
            (sep - 180.0).abs() < 1e-9,
            "Rahu/Ketu separation at JD {jd} was {sep}, not 180"
        );
        assert_eq!(ketu.latitude, -rahu.latitude);
    }
}

#[test]
fn whole_sign_ascendant_matches_reference() {
    let e = ephe();
    // Chennai: 13.0827 N, 80.2707 E.
    let (angles, cusps) = e
        .angles(J2000, 13.0827, 80.2707, HouseSystem::WholeSign)
        .unwrap();

    assert_deg(angles.ascendant, 72.0895865, "ascendant");
    assert_deg(angles.midheaven, 336.9399878, "midheaven");
    assert_deg(angles.armc, 0.7277724, "armc");

    // Under whole-sign the 1st cusp is the 0th degree of the lagna's sign.
    // 72.09 deg falls in Mithuna, which begins at 60 deg.
    assert_deg(cusps.0[0], 60.0, "cusp 1");
    for (i, c) in cusps.0.iter().enumerate() {
        assert_deg(*c, (60.0 + 30.0 * i as f64) % 360.0, &format!("cusp {}", i + 1));
    }
}

#[test]
fn whole_sign_houses_work_at_polar_latitudes() {
    let e = ephe();
    // Placidus and Koch are undefined inside the polar circles; whole-sign is
    // always defined. This is one of the edge cases in the Phase 1 criteria.
    for lat in [70.0, 78.0, 85.0, 89.9] {
        let r = e.angles(J2000, lat, 15.0, HouseSystem::WholeSign);
        assert!(r.is_ok(), "whole-sign failed at latitude {lat}: {r:?}");
    }
}

#[test]
fn retrograde_detection() {
    let e = ephe();
    // Mercury was retrograde from 2000-02-21 to 2000-03-14. 2000-03-01 12:00 UT.
    let jd = julian_day_ut(2000, 3, 1, 12.0, Calendar::Gregorian).unwrap();
    let merc = e.position(jd, Graha::Mercury).unwrap();
    assert!(
        merc.is_retrograde(),
        "Mercury should be vakri on 2000-03-01, speed was {}",
        merc.speed_longitude
    );

    // The Sun and Moon never retrograde.
    assert!(!e.position(jd, Graha::Sun).unwrap().is_retrograde());
    assert!(!e.position(jd, Graha::Moon).unwrap().is_retrograde());
}

#[test]
fn positions_batch_agrees_with_individual_calls() {
    let e = ephe();
    let jd = julian_day_ut(1947, 8, 14, 18.5, Calendar::Gregorian).unwrap();
    let batch = e.positions(jd).unwrap();
    assert_eq!(batch.len(), 9);
    for (graha, pos) in batch {
        let single = e.position(jd, graha).unwrap();
        assert_eq!(pos, single, "batch/single mismatch for {}", graha.name());
    }
}

#[test]
fn identical_input_gives_bit_identical_output() {
    let e = ephe();
    let jd = julian_day_ut(1985, 6, 21, 3.75, Calendar::Gregorian).unwrap();
    let first = e.positions(jd).unwrap();
    for _ in 0..50 {
        assert_eq!(e.positions(jd).unwrap(), first, "determinism violated");
    }
}

#[test]
fn invalid_coordinates_are_rejected() {
    let e = ephe();
    assert!(e.angles(J2000, 91.0, 0.0, HouseSystem::WholeSign).is_err());
    assert!(e.angles(J2000, 0.0, 181.0, HouseSystem::WholeSign).is_err());
}
