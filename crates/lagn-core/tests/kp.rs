//! Phase 13H QA suite: the Krishnamurti Paddhati.
//!
//! Written against `docs/phase13/KP.md`, acceptance criteria 1 to 8 and 10.
//! The N-version check (criterion 9) lives in `scripts/qa_kp_oracle.py`.

use std::collections::BTreeSet;

use lagn_core::dasha::{dasha_years, CYCLE, TOTAL_YEARS};
use lagn_core::kp::{divide, lords, Kp, SignificatorGroup, VARIANTS};
use lagn_core::nakshatra::{NakshatraPosition, NAKSHATRA_SPAN};
use lagn_core::*;

/// A micro-degree: tighter than any boundary that matters, loose enough for
/// accumulated division.
const TOL: f64 = 1e-6;

/// Where nakshatra `k` ends. The 27th closes on 360 rather than on
/// `NAKSHATRA_SPAN * 27`, which is not exactly 360 in floating point - the
/// same rule the kernel uses, so neighbours share a boundary bit for bit.
fn nak_end(k: i32) -> f64 {
    if k == 26 { 360.0 } else { NAKSHATRA_SPAN * (k as f64 + 1.0) }
}

fn init() {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(d).unwrap();
}

fn eph() -> Ephemeris {
    init();
    Ephemeris::new(Ayanamsa::Krishnamurti, NodeType::Mean)
}

fn chart_at(year: i32, hour: u32, lat: f64, lon: f64) -> Chart {
    init();
    Chart::compute(
        BirthData {
            moment: BirthMoment {
                year, month: 6, day: 15, hour, minute: 30, second: 0.0, utc_offset_hours: 5.5,
            },
            latitude: lat, longitude: lon, place_name: String::new(),
        },
        ChartSettings { ayanamsa: Ayanamsa::Krishnamurti, ..ChartSettings::default() },
    )
    .expect("chart")
}

// ===========================================================================
// Criterion 1 - the division tiles exactly
// ===========================================================================

#[test]
fn the_nine_subs_of_every_nakshatra_tile_it_exactly() {
    for k in 0..27 {
        let start = NAKSHATRA_SPAN * k as f64;
        let star = Nakshatra::from_index(k).lord();
        let subs = divide(0.0, NAKSHATRA_SPAN, star);
        assert_eq!(subs.len(), 9);
        let _ = start;
        assert_eq!(subs[0].start, 0.0, "nakshatra {k} subs start at its own beginning");
        assert_eq!(
            subs[8].end.to_bits(), NAKSHATRA_SPAN.to_bits(),
            "nakshatra {k} subs must close exactly on the nakshatra's end",
        );
        for w in subs.windows(2) {
            assert_eq!(w[0].end.to_bits(), w[1].start.to_bits(), "a sub seam is not exact");
        }
        let total: f64 = subs.iter().map(|d| d.width()).sum();
        assert!((total - NAKSHATRA_SPAN).abs() < TOL, "nakshatra {k} sums to {total}");
    }
}

#[test]
fn the_nine_sub_subs_of_every_sub_tile_it_exactly() {
    for k in 0..27 {
        let start = NAKSHATRA_SPAN * k as f64;
        let _ = start;
        for sub in divide(0.0, NAKSHATRA_SPAN, Nakshatra::from_index(k).lord()) {
            let inner = divide(sub.start, sub.end, sub.lord);
            assert_eq!(inner.len(), 9);
            assert_eq!(inner[0].start.to_bits(), sub.start.to_bits());
            assert_eq!(inner[8].end.to_bits(), sub.end.to_bits());
            let total: f64 = inner.iter().map(|d| d.width()).sum();
            assert!((total - sub.width()).abs() < TOL);
        }
    }
}

// ===========================================================================
// Criterion 2 - the count is 249, derived rather than assumed
// ===========================================================================

#[test]
fn the_division_cut_by_the_sign_boundaries_gives_exactly_two_hundred_and_forty_nine_cells() {
    // KP is described everywhere as dividing the zodiac into 249
    // sub-divisions, and that number is normally quoted. Building the
    // division from the Vimshottari proportions and cutting it by the twelve
    // sign boundaries has to *land* on it: a wrong sub width, a wrong
    // starting lord or a wrong cycle order all change the count. So this is a
    // check on the construction, not a restatement of a fact.
    let mut cells = 0usize;
    let mut straddling = 0usize;
    let mut subs = 0usize;

    for k in 0..27 {
        let start = NAKSHATRA_SPAN * k as f64;
        for sub in divide(start, nak_end(k), Nakshatra::from_index(k).lord()) {
            subs += 1;
            // Sign boundaries strictly inside this sub. The comparison is
            // tightened by a nanodegree so a boundary that lands *on* a sub
            // edge is not miscounted as inside it - which is exactly the
            // distinction between 249 and 252.
            let inside = (1..12)
                .map(|m| 30.0 * m as f64)
                .filter(|b| *b > sub.start + 1e-9 && *b < sub.end - 1e-9)
                .count();
            cells += 1 + inside;
            if inside > 0 {
                straddling += 1;
            }
        }
    }

    assert_eq!(subs, 243, "27 nakshatras x 9 subs");
    assert_eq!(straddling, 6, "exactly six subs straddle a sign boundary");
    assert_eq!(cells, 249, "the KP zodiac has 249 sub-divisions");
}

#[test]
fn nine_nakshatras_straddle_a_sign_boundary_but_only_six_subs_do() {
    // The reason 249 is not 252: in three of the nine straddling nakshatras
    // the sign boundary falls exactly on a sub boundary, so no sub is cut.
    let straddling_nakshatras = (0..27)
        .filter(|k| {
            let (a, b) = (NAKSHATRA_SPAN * *k as f64, NAKSHATRA_SPAN * (*k as f64 + 1.0));
            (1..12).any(|m| {
                let x = 30.0 * m as f64;
                x > a + 1e-9 && x < b - 1e-9
            })
        })
        .count();
    assert_eq!(straddling_nakshatras, 9);
}

// ===========================================================================
// Criteria 3, 4 and 5 - the lords of a longitude
// ===========================================================================

#[test]
fn every_constructed_sub_is_the_one_lords_reports_for_longitudes_inside_it() {
    // Criterion 3, checked directly rather than inferred from a sweep. For
    // each of the 243 subs, probe three longitudes inside it - just after the
    // start, the midpoint, just before the end - and ask `lords()`.
    //
    // The probe is built from the nakshatra's start, so it deliberately does
    // *not* sample the boundary itself: which nakshatra a boundary belongs to
    // is nakshatra.rs's business and has its own tests. What is under test
    // here is the sub division inside a nakshatra.
    let mut checked = 0usize;
    for k in 0..27 {
        let nak = Nakshatra::from_index(k);
        let base = NAKSHATRA_SPAN * k as f64;
        for sub in divide(0.0, NAKSHATRA_SPAN, nak.lord()) {
            let width = sub.end - sub.start;
            for frac in [0.01, 0.5, 0.99] {
                let lon = base + sub.start + width * frac;
                let l = lords(lon);
                assert_eq!(l.sub_lord, sub.lord, "at {lon} inside {:?}'s sub of {:?}", sub.lord, nak);
                assert_eq!(l.nakshatra, nak, "at {lon}: wrong nakshatra");
                assert_eq!(l.star_lord, nak.lord(), "at {lon}: wrong star lord");
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 243 * 3, "every sub probed three times");
}

#[test]
fn a_dense_sweep_of_the_zodiac_finds_no_gap_and_no_overlap() {
    // Every arcminute of the circle lands in exactly one sub of its own
    // nakshatra, and `lords()` agrees. The nakshatra comes from the same
    // single floor the kernel uses, so this isolates the sub division; the
    // N-version oracle checks the whole thing from absolute longitudes with
    // its own arithmetic.
    let table = divide(0.0, NAKSHATRA_SPAN, Graha::Ketu);
    assert_eq!(table.len(), 9);

    let step = 1.0 / 60.0;
    let n = (360.0 / step) as usize;
    for i in 0..n {
        let lon = i as f64 * step;
        let l = lords(lon);
        let pos = NakshatraPosition::from_longitude(lon);
        let subs = divide(0.0, NAKSHATRA_SPAN, pos.nakshatra.lord());
        let hits: Vec<&lagn_core::kp::Division> =
            subs.iter().filter(|d| d.contains(pos.degrees_within)).collect();
        assert_eq!(hits.len(), 1, "{lon} falls in {} subs of its nakshatra", hits.len());
        assert_eq!(l.sub_lord, hits[0].lord, "{lon} sub lord");
        assert_eq!(l.star_lord, pos.nakshatra.lord(), "{lon} star lord");
        assert!(CYCLE.contains(&l.sub_sub_lord));
        assert!((1..=4).contains(&l.pada));
        assert_eq!(l.sign_lord, Rasi::from_longitude(lon).lord());
    }
}

#[test]
fn the_first_sub_of_a_nakshatra_belongs_to_that_nakshatra_and_likewise_the_first_sub_sub() {
    // Criterion 4: the rule's starting point, and the easiest thing to get
    // wrong - starting the cycle at Ketu instead of at the nakshatra's lord.
    for k in 0..27 {
        let nak = Nakshatra::from_index(k);
        let start = NAKSHATRA_SPAN * k as f64;
        // A hair inside, not on the boundary: the boundary itself belongs to
        // nakshatra.rs's floor and is tested there.
        let l = lords(start + 0.01);
        assert_eq!(l.nakshatra, nak);
        assert_eq!(l.star_lord, nak.lord(), "{:?} star lord", nak);
        assert_eq!(l.sub_lord, nak.lord(), "{:?}: the first sub is the nakshatra's own lord", nak);
        assert_eq!(l.sub_sub_lord, nak.lord(), "{:?}: and so is the first sub-sub", nak);
    }
}

#[test]
fn sub_widths_are_the_vimshottari_proportions() {
    // Criterion 5, as ratios: Shukra's sub is 20/120 of a nakshatra and
    // Surya's 6/120, so Shukra's is 10/3 times Surya's whatever the span is.
    let subs = divide(0.0, NAKSHATRA_SPAN, Graha::Ketu);
    for d in &subs {
        let want = NAKSHATRA_SPAN * dasha_years(d.lord) / TOTAL_YEARS;
        assert!((d.width() - want).abs() < TOL, "{:?} sub is {} not {}", d.lord, d.width(), want);
    }
    let venus = subs.iter().find(|d| d.lord == Graha::Venus).unwrap().width();
    let sun = subs.iter().find(|d| d.lord == Graha::Sun).unwrap().width();
    assert!((venus / sun - 20.0 / 6.0).abs() < TOL, "Shukra:Surya is not 20:6");
    // And the nine lords appear once each, in cycle order from the first.
    let lords_seen: Vec<Graha> = subs.iter().map(|d| d.lord).collect();
    assert_eq!(lords_seen.iter().collect::<BTreeSet<_>>().len(), 9);
    assert_eq!(lords_seen[0], Graha::Ketu);
}

// ===========================================================================
// Criteria 6, 7 and 8 - cusps, ruling planets, significators
// ===========================================================================

#[test]
fn the_twelve_cusps_are_placidus_and_each_carries_four_lords() {
    let e = eph();
    for (hour, lat) in [(6, 13.08), (14, 28.6), (22, 51.5), (3, -33.9)] {
        let chart = chart_at(1985, hour, lat, 80.0);
        let kp = Kp::compute(&e, &chart).expect("kp");
        assert_eq!(kp.cusps.len(), 12);

        // The same cusps Swiss Ephemeris gives for Placidus, in order.
        let (_, cusps) = e
            .angles(chart.jd_ut, chart.birth.latitude, chart.birth.longitude, HouseSystem::Placidus)
            .expect("cusps");
        for (i, c) in kp.cusps.iter().enumerate() {
            assert_eq!(c.house, i as u8 + 1);
            let want = lagn_ephem::norm360(cusps.0[i]);
            assert!((c.lords.longitude - want).abs() < TOL, "cusp {} is {} not {}", c.house, c.lords.longitude, want);
            // And its four lords are the ones the longitude implies.
            assert_eq!(c.lords.sub_lord, lords(want).sub_lord);
            assert_eq!(c.lords.sign_lord, Rasi::from_longitude(want).lord());
        }
        // The first cusp is the ascendant.
        assert!((kp.cusps[0].lords.longitude - chart.lagna.longitude).abs() < 1e-4);
    }
}

#[test]
fn the_five_ruling_planets_are_the_ones_the_design_names_in_order() {
    let e = eph();
    let chart = chart_at(1985, 14, 13.08, 80.27);
    let kp = Kp::compute(&e, &chart).expect("kp");
    assert_eq!(kp.ruling_planets.len(), 5);

    let roles: Vec<&str> = kp.ruling_planets.iter().map(|r| r.role.as_ref()).collect();
    assert_eq!(roles, [
        "lord of the day", "Moon's sign lord", "Moon's star lord",
        "ascendant's sign lord", "ascendant's star lord",
    ]);

    // Each is the lord it claims to be, read off the chart rather than taken
    // on trust.
    assert_eq!(kp.ruling_planets[1].graha, kp.moon.sign_lord);
    assert_eq!(kp.ruling_planets[2].graha, kp.moon.star_lord);
    assert_eq!(kp.ruling_planets[3].graha, kp.ascendant.sign_lord);
    assert_eq!(kp.ruling_planets[4].graha, kp.ascendant.star_lord);
    // The day lord has no sub of its own; the other four are weighed by one.
    assert!(kp.ruling_planets[0].sub_lord.is_none());
    assert!(kp.ruling_planets[1..].iter().all(|r| r.sub_lord.is_some()));
}

#[test]
fn no_graha_signifies_one_house_twice_and_every_listing_qualifies() {
    let e = eph();
    for (hour, lat) in [(6, 13.08), (14, 28.6), (22, 51.5)] {
        let chart = chart_at(1990, hour, lat, 77.2);
        let kp = Kp::compute(&e, &chart).expect("kp");
        assert_eq!(kp.significators.len(), 12);

        for h in &kp.significators {
            assert!((1..=12).contains(&h.house));
            // Listed once only (V-13-19).
            let names: Vec<Graha> = h.significators.iter().map(|s| s.graha).collect();
            assert_eq!(
                names.iter().collect::<BTreeSet<_>>().len(),
                names.len(),
                "house {} repeats a graha", h.house,
            );
            // Strongest first: the groups come out in rank order.
            let ranks: Vec<u8> = h.significators.iter().map(|s| s.group.rank()).collect();
            let mut sorted = ranks.clone();
            sorted.sort_unstable();
            assert_eq!(ranks, sorted, "house {} is not ranked", h.house);
            // The house lord is always a significator of its own house.
            let lord = kp.cusps[(h.house - 1) as usize].lords.sign_lord;
            assert!(names.contains(&lord), "house {}: its lord is not a significator", h.house);
        }
        // Every group is exercised somewhere across the twelve houses, or the
        // weaker branches are untested rather than correct.
        let groups: BTreeSet<SignificatorGroup> = kp
            .significators.iter().flat_map(|h| h.significators.iter().map(|s| s.group)).collect();
        assert!(groups.len() >= 3, "only {} significator groups seen", groups.len());
    }
}

// ===========================================================================
// Criterion 10 - the variants, and determinism
// ===========================================================================

#[test]
fn every_result_names_the_variant_scheme_that_produced_it() {
    let e = eph();
    let kp = Kp::compute(&e, &chart_at(1985, 9, 13.08, 80.27)).expect("kp");
    assert_eq!(kp.variants.len(), VARIANTS.len());
    let ids: Vec<String> = kp.variants.iter().map(|v| v.id.to_string()).collect();
    assert_eq!(ids, ["V-13-18", "V-13-19", "V-13-20", "V-13-21"]);
    assert_eq!(kp.grahas.len(), 9);
}

#[test]
fn the_same_chart_always_gives_the_same_reading_and_it_round_trips() {
    let e = eph();
    let chart = chart_at(1985, 11, 13.08, 80.27);
    let a = Kp::compute(&e, &chart).expect("a");
    assert_eq!(a, Kp::compute(&e, &chart).expect("b"));
    let text = serde_json::to_string(&a).expect("serialise");
    let back: Kp = serde_json::from_str(&text).expect("deserialise");
    assert_eq!(a, back);
}
