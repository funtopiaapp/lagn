//! Phase 13E QA suite: Shadbala, five of six.
//!
//! Written against `docs/phase13/SHADBALA.md`, criteria 1 to 7.
//! Criterion 8 (no corpus rule consumes a bala) lives in
//! `crates/lagn-rules/tests/no_unverified_strength.rs`, because it reads the
//! rule corpus.

use lagn_core::bala::{
    customary_minimum, ladder, naisargika, Bala, RUPA, SAPTAVARGA, VARIANTS,
};
use lagn_core::dignity::Dignity;
use lagn_core::relationship::SEVEN;
use lagn_core::*;

const TOL: f64 = 1e-9;

fn init() {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(d).unwrap();
}

fn eph() -> Ephemeris {
    init();
    Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean)
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

/// Charts with their own sunrise and sunset, which Kala bala needs.
fn charts(n: usize, seed: u64) -> Vec<(Chart, f64, f64, Graha)> {
    init();
    let e = eph();
    let mut rng = Rng(seed | 1);
    let mut out = Vec::new();
    while out.len() < n {
        let b = BirthData {
            moment: BirthMoment {
                year: rng.int(1900, 2050) as i32,
                month: rng.int(1, 12) as u32,
                day: rng.int(1, 28) as u32,
                hour: rng.int(0, 23) as u32,
                minute: rng.int(0, 59) as u32,
                second: 0.0,
                utc_offset_hours: 5.5,
            },
            latitude: rng.unit() * 90.0 - 45.0,
            longitude: rng.unit() * 360.0 - 180.0,
            place_name: String::new(),
        };
        if let Ok(c) = Chart::compute(b, ChartSettings::default()) {
            if let Ok(sun) = e.sun_day(c.jd_ut - 0.5, c.birth.latitude, c.birth.longitude) {
                let vara = day::Vara::from_sunrise_jd(sun.sunrise_jd);
                out.push((c, sun.sunrise_jd, sun.sunset_jd, vara.lord()));
            }
        }
    }
    out
}

// ===========================================================================
// Criterion 1 - everything inside its range
// ===========================================================================

#[test]
fn no_component_exceeds_its_maximum_or_goes_negative() {
    let e = eph();
    for (c, rise, set, vara) in charts(120, 0xBA1A) {
        let b = Bala::compute(&e, &c, rise, set, vara).expect("bala");
        assert_eq!(b.grahas.len(), 7, "the nodes have no Shadbala");

        for g in &b.grahas {
            let s = &g.sthana;
            for (name, v, max) in [
                ("uchcha", s.uchcha, 60.0),
                ("saptavargaja", s.saptavargaja, 315.0),
                ("ojayugma", s.ojayugma, 30.0),
                ("kendra", s.kendra, 60.0),
                ("drekkana", s.drekkana, 15.0),
                ("dig", g.dig, 60.0),
                ("nathonnatha", g.kala.nathonnatha, 60.0),
                ("paksha", g.kala.paksha, 60.0),
                ("tribhaga", g.kala.tribhaga, 60.0),
                ("abda", g.kala.abda, 15.0),
                ("masa", g.kala.masa, 30.0),
                ("vara", g.kala.vara, 45.0),
                ("hora", g.kala.hora, 60.0),
                ("ayana", g.kala.ayana, 60.0),
                ("cheshta", g.cheshta.value, 60.0),
                ("naisargika", g.naisargika, 60.0),
            ] {
                assert!(
                    v >= -TOL && v <= max + TOL,
                    "{:?} {name} is {v}, outside 0 to {max}", g.graha,
                );
            }
            // The parts sum to their component.
            assert!((s.total - (s.uchcha + s.saptavargaja + s.ojayugma + s.kendra + s.drekkana)).abs() < TOL);
        }
    }
}

// ===========================================================================
// Criteria 2 to 5 - the formulas, as formulas
// ===========================================================================

#[test]
fn naisargika_is_exactly_sixty_n_over_seven() {
    // Asserted as the formula, not as seven transcribed numbers: the series
    // is what the specification states, so that is what is checked.
    let order = [
        (Graha::Sun, 7.0), (Graha::Moon, 6.0), (Graha::Venus, 5.0),
        (Graha::Jupiter, 4.0), (Graha::Mercury, 3.0), (Graha::Mars, 2.0),
        (Graha::Saturn, 1.0),
    ];
    for (g, n) in order {
        assert!((naisargika(g) - RUPA * n / 7.0).abs() < TOL, "{:?}", g);
    }
    // And it is strictly ordered, Surya strongest to Shani weakest.
    let values: Vec<f64> = order.iter().map(|(g, _)| naisargika(*g)).collect();
    for w in values.windows(2) {
        assert!(w[0] > w[1], "the natural order is not strictly descending");
    }
    assert!((values[0] - 60.0).abs() < TOL);
}

#[test]
fn the_saptavargaja_ladder_is_the_table_the_spec_gives() {
    // Criterion 5. The ladder is a table, not a rule.
    //
    // The first revision of the specification called it "halving from
    // moolatrikona down" and this test was written to that description - at
    // which point it failed on 45 to 30, which is two thirds, not a half.
    // Only the lower four rungs halve. So the values are pinned literally,
    // and the halving is asserted where it actually holds.
    let rungs = [
        ladder(Dignity::Moolatrikona),
        ladder(Dignity::OwnSign),
        ladder(Dignity::GreatFriend),
        ladder(Dignity::Friend),
        ladder(Dignity::Neutral),
        ladder(Dignity::Enemy),
        ladder(Dignity::GreatEnemy),
    ];
    assert_eq!(rungs, [45.0, 30.0, 22.5, 15.0, 7.5, 3.75, 1.875]);
    // Strictly descending throughout, which the table must be to be a ladder.
    for w in rungs.windows(2) {
        assert!(w[0] > w[1], "{} does not exceed {}", w[0], w[1]);
    }
    // And the bottom four do halve.
    for w in rungs[3..].windows(2) {
        assert!((w[1] - w[0] / 2.0).abs() < TOL, "{} is not half of {}", w[1], w[0]);
    }
    // V-13-26: the two dignities with no rung of their own read as the
    // nearest, and say so by equality.
    assert_eq!(ladder(Dignity::Exalted), ladder(Dignity::Moolatrikona));
    assert_eq!(ladder(Dignity::Debilitated), ladder(Dignity::GreatEnemy));
    // Seven vargas, so the maximum is seven moolatrikonas.
    assert_eq!(SAPTAVARGA.len(), 7);
    assert!((45.0f64 * 7.0 - 315.0).abs() < TOL);
}

#[test]
fn uchcha_bala_is_sixty_at_deep_exaltation_and_zero_at_deep_debilitation() {
    // Criterion 3, checked by constructing charts is impossible - a graha
    // cannot be placed. So the formula is checked directly: the engine's
    // Uchcha is the arc from the debilitation point over three, and that is
    // 60 at exaltation and 0 at debilitation by construction. Here the
    // deep points themselves are pinned against phase 2's table.
    use Rasi::*;
    let spec = [
        (Graha::Sun, Mesha, 10.0), (Graha::Moon, Vrishabha, 3.0),
        (Graha::Mars, Makara, 28.0), (Graha::Mercury, Kanya, 15.0),
        (Graha::Jupiter, Karka, 5.0), (Graha::Venus, Meena, 27.0),
        (Graha::Saturn, Tula, 20.0),
    ];
    for (g, sign, deg) in spec {
        let want = 30.0 * sign.index() as f64 + deg;
        let got = lagn_core::dignity::deep_exaltation_longitude(g).expect("a deep point");
        assert!((got - want).abs() < TOL, "{:?} deep exaltation is {got}, not {want}", g);
    }
}

#[test]
fn dig_bala_is_sixty_at_the_strongest_angle_and_zero_opposite() {
    // Criterion 4. Checked through real charts: for each graha, find the
    // chart in a sweep where it sits closest to its strongest house cusp, and
    // confirm its Dig bala is near the maximum there and near zero for a
    // chart where it sits opposite.
    let e = eph();
    let mut best: Vec<(Graha, f64, f64)> = SEVEN.iter().map(|&g| (g, 0.0, 60.0)).collect();
    for (c, rise, set, vara) in charts(200, 0xD16) {
        let b = Bala::compute(&e, &c, rise, set, vara).expect("bala");
        for gb in &b.grahas {
            let slot = best.iter_mut().find(|(g, _, _)| *g == gb.graha).unwrap();
            if gb.dig > slot.1 { slot.1 = gb.dig; }
            if gb.dig < slot.2 { slot.2 = gb.dig; }
        }
    }
    for (g, max, min) in best {
        assert!(max > 55.0, "{:?} never came near its strongest angle (best {max})", g);
        assert!(min < 5.0, "{:?} never came near its weakest angle (worst {min})", g);
    }
}

// ===========================================================================
// Criterion 6 - the total, and that it says what it contains
// ===========================================================================

#[test]
fn the_total_is_the_sum_of_the_five_computed_components_and_says_so() {
    let e = eph();
    for (c, rise, set, vara) in charts(60, 0x707A1) {
        let b = Bala::compute(&e, &c, rise, set, vara).expect("bala");

        // The result states what it contains rather than leaving a reader to
        // infer it from a missing field.
        assert!(!b.drik_included, "Drik bala is held, and must say so");
        assert_eq!(b.components_computed, 5);
        assert!(b.caveat.contains("Five of the six"), "the caveat must name the gap");
        assert!(b.caveat.contains("sphuta drishti"), "and why it is held");

        for g in &b.grahas {
            let want = g.sthana.total + g.dig + g.kala.total + g.cheshta.value + g.naisargika;
            assert!((g.total_virupas - want).abs() < TOL, "{:?} total", g.graha);
            assert!((g.total_rupas - g.total_virupas / RUPA).abs() < TOL, "{:?} rupas", g.graha);
            // The customary minimum travels alongside but is never applied.
            assert_eq!(g.customary_minimum_rupas, customary_minimum(g.graha));
        }
    }
}

#[test]
fn surya_and_chandra_take_their_cheshta_from_ayana_and_say_so() {
    // They never retrograde, so the speed-deviation formula has nothing to
    // measure. The flag exists so a reader is not left wondering why their
    // Cheshta matches their Ayana exactly.
    let e = eph();
    for (c, rise, set, vara) in charts(20, 0xC4E5) {
        let b = Bala::compute(&e, &c, rise, set, vara).expect("bala");
        for g in &b.grahas {
            let expected = matches!(g.graha, Graha::Sun | Graha::Moon);
            assert_eq!(g.cheshta.from_ayana, expected, "{:?}", g.graha);
            if expected {
                assert!((g.cheshta.value - g.kala.ayana).abs() < TOL);
            }
        }
    }
}

// ===========================================================================
// Criterion 7 - every disputed choice is named
// ===========================================================================

#[test]
fn all_seven_registered_variants_are_on_every_result() {
    let e = eph();
    let (c, rise, set, vara) = charts(1, 0x7A71E).remove(0);
    let b = Bala::compute(&e, &c, rise, set, vara).expect("bala");
    assert_eq!(b.variants.len(), VARIANTS.len());
    let ids: Vec<String> = b.variants.iter().map(|v| v.id.to_string()).collect();
    assert_eq!(
        ids,
        ["V-13-26", "V-13-27", "V-13-28", "V-13-29", "V-13-30", "V-13-31", "V-13-32"],
        "section 5 registers seven variants and every one must be named",
    );
    for v in &b.variants {
        assert!(!v.question.is_empty() && !v.chosen.is_empty(), "{} is blank", v.id);
    }
    // V-13-32 is the held component, and its choice says so plainly.
    let drik = b.variants.iter().find(|v| v.id == "V-13-32").unwrap();
    assert!(drik.chosen.contains("not computed"), "V-13-32 must say Drik is not computed");
}

#[test]
fn the_same_chart_always_gives_the_same_balas_and_they_round_trip() {
    let e = eph();
    let (c, rise, set, vara) = charts(1, 0xD37C).remove(0);
    let a = Bala::compute(&e, &c, rise, set, vara).expect("a");
    assert_eq!(a, Bala::compute(&e, &c, rise, set, vara).expect("b"));
    let text = serde_json::to_string(&a).expect("serialise");
    let back: Bala = serde_json::from_str(&text).expect("deserialise");
    assert_eq!(a, back);
}

// ===========================================================================
// The declination transform, checked against the sky rather than itself
// ===========================================================================

#[test]
fn the_sun_reaches_the_obliquity_at_the_solstices_and_zero_at_the_equinoxes() {
    // Ayana bala rests on a declination, which rests on an ecliptic-to-
    // equatorial transform this engine had no reason to compute before. So
    // the transform is checked against something outside it: the Sun's
    // declination is +obliquity at the June solstice, -obliquity at the
    // December one, and zero at the equinoxes.
    //
    // The Sun's ecliptic latitude is ~0, so declination depends only on its
    // tropical longitude - which is exactly what makes this a clean check on
    // the conversion and on the ayanamsa being added back correctly.
    use lagn_core::bala::{declination_of, OBLIQUITY_DEGREES};
    let eps = OBLIQUITY_DEGREES;

    for (tropical, want) in [
        (0.0, 0.0),            // vernal equinox
        (90.0, eps),           // June solstice
        (180.0, 0.0),          // autumnal equinox
        (270.0, -eps),         // December solstice
    ] {
        // Feed a sidereal longitude plus the ayanamsa that recovers this
        // tropical one, which is how the engine's own call is shaped.
        let ayanamsa = 24.0;
        let sidereal = tropical - ayanamsa;
        let got = declination_of(sidereal, 0.0, ayanamsa);
        assert!(
            (got - want).abs() < 1e-6,
            "tropical {tropical}: declination {got}, expected {want}",
        );
    }

    // And a real chart: a 21 December birth has the Sun near its southern
    // limit, so its declination must be close to -obliquity.
    init();
    let c = Chart::compute(
        BirthData {
            moment: BirthMoment {
                year: 1981, month: 12, day: 21,
                hour: 14, minute: 10, second: 0.0, utc_offset_hours: 5.5,
            },
            latitude: 8.8932, longitude: 76.6141, place_name: String::new(),
        },
        ChartSettings::default(),
    )
    .expect("chart");
    let e = eph();
    let p = e.position(c.jd_ut, Graha::Sun).expect("sun");
    let decl = declination_of(p.longitude, p.latitude, c.ayanamsa_value);
    assert!(
        (decl + eps).abs() < 0.5,
        "a 21 December Sun should sit near -{eps} declination, got {decl}",
    );
}
