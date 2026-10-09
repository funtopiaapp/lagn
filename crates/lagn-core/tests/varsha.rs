//! Phase 13F/13G QA suite: kaksha transit and the annual chart.
//!
//! Written against `docs/phase13/VARSHA-KAKSHA.md`, criteria 1 to 5 and 7.
//! The N-version check (criterion 6) lives in `scripts/qa_varsha_oracle.py`.

use std::collections::HashSet;

use lagn_core::ashtakavarga::{Ashtakavarga, Contributor};
use lagn_core::varsha::{kaksha_of, kaksha_transit, Varshaphala, KAKSHA_ORDER, KAKSHA_WIDTH, VARIANTS};
use lagn_core::*;

/// A milliarcsecond, in degrees. The solar return is solved, so it is held to
/// the precision of the solve.
const MAS: f64 = 1.0 / 3_600_000.0;

fn init() {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(d).unwrap();
}

fn eph() -> Ephemeris {
    init();
    Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean)
}

fn natal() -> Chart {
    init();
    Chart::compute(
        BirthData {
            moment: BirthMoment {
                year: 1981, month: 12, day: 21,
                hour: 14, minute: 10, second: 0.0, utc_offset_hours: 5.5,
            },
            latitude: 8.8932, longitude: 76.6141, place_name: "Kollam".into(),
        },
        ChartSettings::default(),
    )
    .expect("chart")
}

fn apart(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 { 360.0 - d } else { d }
}

// ===========================================================================
// Criteria 1 and 2 - the solar return
// ===========================================================================

#[test]
fn the_sun_at_every_annual_chart_equals_the_natal_sun() {
    // The property that makes this safe to build: the answer is checked
    // against itself. A solver that converged on the wrong root, or a bracket
    // that missed the crossing, fails here rather than producing a chart that
    // looks plausible.
    let e = eph();
    let n = natal();
    let natal_sun = n.placement(Graha::Sun).longitude;

    for age in 0..=90u32 {
        let v = Varshaphala::compute(&e, &n, age).expect("annual chart");
        assert_eq!(v.age, age);
        assert!(
            v.sun_error < MAS,
            "age {age}: the Sun is {} degrees off the natal Sun", v.sun_error,
        );
        assert!(
            apart(v.chart.placement(Graha::Sun).longitude, natal_sun) < MAS,
            "age {age}: the chart's own Sun does not match either",
        );
    }
}

#[test]
fn each_annual_chart_falls_on_or_after_its_birthday_and_inside_the_year() {
    // Criterion 2: the *right* return. The Sun regains its longitude once a
    // year, so picking the wrong crossing would give a chart a year out -
    // internally consistent and completely wrong.
    let e = eph();
    let n = natal();
    let mut previous = n.jd_ut;

    for age in 1..=60u32 {
        let v = Varshaphala::compute(&e, &n, age).expect("annual chart");
        let years = (v.return_jd - n.jd_ut) / 365.2425;
        assert!(
            (years - age as f64).abs() < 0.01,
            "age {age}: the return is {years:.3} years after birth",
        );
        // Strictly increasing: no year is ever revisited or skipped.
        assert!(v.return_jd > previous, "age {age} goes backwards");
        previous = v.return_jd;
    }
}

#[test]
fn the_annual_chart_keeps_the_natal_charts_conventions_and_place() {
    let e = eph();
    let n = natal();
    let v = Varshaphala::compute(&e, &n, 45).expect("annual chart");
    // V-13-22: cast for the birth place.
    assert_eq!(v.chart.birth.latitude, n.birth.latitude);
    assert_eq!(v.chart.birth.longitude, n.birth.longitude);
    assert_eq!(v.chart.birth.place_name, n.birth.place_name);
    // And with the same ayanamsa and node type, so the two charts cannot
    // disagree about a convention.
    assert_eq!(v.chart.settings, n.settings);
    // The exact moment survives: it is not rounded through a civil second.
    assert_eq!(v.chart.jd_ut.to_bits(), v.return_jd.to_bits());
}

// ===========================================================================
// Criterion 3 - Muntha
// ===========================================================================

#[test]
fn muntha_advances_one_sign_a_year_and_returns_after_twelve() {
    let e = eph();
    let n = natal();
    let lagna = n.lagna.rasi;

    for age in 0..=24u32 {
        let v = Varshaphala::compute(&e, &n, age).expect("annual chart");
        let want = Rasi::from_index(lagna.index() as i32 + age as i32);
        assert_eq!(v.muntha, want, "age {age}: Muntha in the wrong sign");
        // The house it occupies is counted in the annual chart, not the natal.
        assert_eq!(v.muntha_house, v.chart.lagna.rasi.houses_to(v.muntha));
    }
    // At birth it is the lagna itself (V-13-24), and twelve years later again.
    assert_eq!(Varshaphala::compute(&e, &n, 0).unwrap().muntha, lagna);
    assert_eq!(Varshaphala::compute(&e, &n, 12).unwrap().muntha, lagna);
    assert_eq!(Varshaphala::compute(&e, &n, 24).unwrap().muntha, lagna);
}

// ===========================================================================
// Criteria 4 and 5 - kaksha
// ===========================================================================

#[test]
fn eight_kakshas_of_three_degrees_forty_five_tile_a_sign() {
    assert!((KAKSHA_WIDTH - 3.75).abs() < 1e-12, "a kaksha is 3 deg 45 min");
    assert!((KAKSHA_WIDTH * 8.0 - 30.0).abs() < 1e-12, "eight kakshas tile a sign");

    // The order itself, pinned literally against section 4's table.
    //
    // This is a transcription check and it has to be written out. The sweep
    // below compares `kaksha_of` against `KAKSHA_ORDER`, which it satisfies by
    // construction - so swapping two entries in the constant passed the whole
    // suite while the oracle caught it. A test that reads the constant it is
    // testing proves only that the code is self-consistent.
    assert_eq!(
        KAKSHA_ORDER,
        [
            Contributor::Saturn,
            Contributor::Jupiter,
            Contributor::Mars,
            Contributor::Sun,
            Contributor::Venus,
            Contributor::Mercury,
            Contributor::Moon,
            Contributor::Lagna,
        ],
        "the kaksha owners are not in the order section 4 gives",
    );

    // The eight owners are exactly the eight Ashtakavarga contributors - no
    // repeat, none missing.
    assert_eq!(KAKSHA_ORDER.len(), 8);
    assert_eq!(
        KAKSHA_ORDER.iter().collect::<HashSet<_>>(),
        Contributor::ALL.iter().collect::<HashSet<_>>(),
    );

    // And they run in order from the start of every sign.
    for sign in 0..12 {
        let base = 30.0 * sign as f64;
        for (i, &owner) in KAKSHA_ORDER.iter().enumerate() {
            let mid = base + KAKSHA_WIDTH * (i as f64 + 0.5);
            let (k, got) = kaksha_of(mid);
            assert_eq!(k, i as u8 + 1, "at {mid}");
            assert_eq!(got, owner, "at {mid}");
        }
    }
}

#[test]
fn a_kaksha_verdict_is_exactly_what_the_prastara_says() {
    // Criterion 5. The verdict must be a reading of the Ashtakavarga, not a
    // second opinion about it: supported is true precisely when the kaksha's
    // owner is among the contributors that gave a bindu there.
    let n = natal();
    let av = Ashtakavarga::compute(&n);
    let step = 0.37;
    let mut checked = 0usize;
    let mut supported = 0usize;

    for g in lagn_core::relationship::SEVEN {
        let mut lon = 0.0;
        while lon < 360.0 {
            let t = kaksha_transit(&av, g, lon).expect("a graha with a BAV");
            let gave = av.contributors(g, t.rasi).expect("contributors");
            assert_eq!(t.supported, gave.contains(&t.owner), "{g:?} at {lon}");
            assert_eq!(t.bindus, av.bindus(g, t.rasi).unwrap());
            assert_eq!(t.kaksha, kaksha_of(lon).0);
            // A sign's bindus can never exceed the eight contributors.
            assert!(t.bindus <= 8);
            checked += 1;
            supported += t.supported as usize;
            lon += step;
        }
    }
    assert!(checked > 6000, "only {checked} positions checked");
    // Both verdicts occur, or one branch is untested rather than correct.
    assert!(supported > 0 && supported < checked, "only one verdict ever seen");
}

#[test]
fn the_nodes_have_no_kaksha_transit_because_they_have_no_bhinnashtakavarga() {
    let av = Ashtakavarga::compute(&natal());
    for g in [Graha::Rahu, Graha::Ketu] {
        assert!(kaksha_transit(&av, g, 100.0).is_none(), "{g:?} should have no BAV");
    }
}

// ===========================================================================
// Criterion 7 - the variants, and determinism
// ===========================================================================

#[test]
fn every_result_names_the_variant_scheme_that_produced_it() {
    let e = eph();
    let v = Varshaphala::compute(&e, &natal(), 30).expect("annual chart");
    assert_eq!(v.variants.len(), VARIANTS.len());
    let ids: Vec<String> = v.variants.iter().map(|x| x.id.to_string()).collect();
    assert_eq!(ids, ["V-13-22", "V-13-23", "V-13-24", "V-13-25"]);
}

#[test]
fn the_same_inputs_always_give_the_same_annual_chart_and_it_round_trips() {
    let e = eph();
    let n = natal();
    let a = Varshaphala::compute(&e, &n, 33).expect("a");
    assert_eq!(a, Varshaphala::compute(&e, &n, 33).expect("b"));
    let text = serde_json::to_string(&a).expect("serialise");
    let back: Varshaphala = serde_json::from_str(&text).expect("deserialise");
    assert_eq!(a, back);
}
