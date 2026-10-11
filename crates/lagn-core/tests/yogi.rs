//! Phase 13D QA suite: Yogi, Avayogi and the Yoga sphuta.
//!
//! Written against `docs/phase13/YOGI.md`, criteria 1 to 4 and 6.
//! The N-version check (criterion 5) lives in `scripts/qa_yogi_oracle.py`.

use lagn_core::dasha::CYCLE;
use lagn_core::yogi::{sphuta, Yogi, AVAYOGI_COUNT, SPHUTA_OFFSET, VARIANTS};
use lagn_core::*;

const TOL: f64 = 1e-9;

fn init() {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(d).unwrap();
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

fn random_charts(n: usize, seed: u64) -> Vec<Chart> {
    init();
    let mut rng = Rng(seed | 1);
    let mut out = Vec::new();
    while out.len() < n {
        let b = BirthData {
            moment: BirthMoment {
                year: rng.int(1400, 2300) as i32,
                month: rng.int(1, 12) as u32,
                day: rng.int(1, 28) as u32,
                hour: rng.int(0, 23) as u32,
                minute: rng.int(0, 59) as u32,
                second: rng.unit() * 59.0,
                utc_offset_hours: 5.5,
            },
            latitude: rng.unit() * 120.0 - 60.0,
            longitude: rng.unit() * 360.0 - 180.0,
            place_name: String::new(),
        };
        if let Ok(c) = Chart::compute(b, ChartSettings::default()) {
            out.push(c);
        }
    }
    out
}

/// Where a graha sits in the nine-lord Vimshottari cycle.
fn cycle_index(g: Graha) -> usize {
    CYCLE.iter().position(|&x| x == g).expect("a Vimshottari lord")
}

// ===========================================================================
// Criterion 1 - the offset is seven nakshatras, not a decimal to mistype
// ===========================================================================

#[test]
fn the_sphuta_offset_is_exactly_seven_nakshatras() {
    // 7 x 13 deg 20 min = 93 deg 20 min. Writing the constant as seven spans
    // and checking it against the degrees-and-minutes value is what stops a
    // mistyped decimal shifting every chart quietly: 93.33 instead of
    // 93.3333 would move the sphuta by twelve arcseconds and nothing else
    // would notice.
    assert!((SPHUTA_OFFSET - (93.0 + 20.0 / 60.0)).abs() < TOL);
    assert!((SPHUTA_OFFSET - 7.0 * lagn_core::nakshatra::NAKSHATRA_SPAN).abs() < TOL);
    assert_eq!(AVAYOGI_COUNT, 6);
}

#[test]
fn the_sphuta_nakshatra_is_seven_past_the_one_sun_plus_moon_falls_in() {
    // Criterion 2, over a dense sweep of both luminaries rather than real
    // charts, so the whole circle is covered.
    for i in 0..360 {
        for j in (0..360).step_by(7) {
            let (sun, moon) = (i as f64 + 0.37, j as f64 + 0.11);
            let base = NakshatraPosition::from_longitude(lagn_ephem::norm360(sun + moon));
            let s = sphuta(sun, moon);
            let want = Nakshatra::from_index(base.nakshatra.index() as i32 + 7);
            assert_eq!(s.nakshatra, want, "sun {sun} moon {moon}");
            // And the point itself is where it claims to be.
            assert_eq!(s.rasi, Rasi::from_longitude(s.longitude));
            assert!((0.0..30.0).contains(&s.degrees_in_rasi));
            assert!((1..=4).contains(&s.pada));
        }
    }
}

// ===========================================================================
// Criteria 3 and 4 - the invariant between the two lords
// ===========================================================================

#[test]
fn the_avayogi_lord_is_always_five_places_past_the_yogi_in_the_cycle() {
    // This is the strongest check available without reference output, and it
    // needs none: the Avayogi is the sixth nakshatra from the Yogi's, which
    // is five on, and the Vimshottari lords repeat every nine nakshatras - so
    // the shift is five places for every chart that has ever existed.
    for chart in random_charts(300, 0x40611) {
        let y = Yogi::compute(&chart);
        let shift = (cycle_index(y.avayogi) + 9 - cycle_index(y.yogi)) % 9;
        assert_eq!(shift, 5, "Yogi {:?} to Avayogi {:?} shifted {shift}", y.yogi, y.avayogi);

        // Criterion 4: both are Vimshottari lords, and five places on in a
        // nine-cycle can never come back to the start, so they always differ.
        assert!(CYCLE.contains(&y.yogi));
        assert!(CYCLE.contains(&y.avayogi));
        assert_ne!(y.yogi, y.avayogi, "the Yogi cannot also be the Avayogi");

        // The Avayogi's nakshatra is the sixth from the Yogi's, inclusive.
        let count = (y.avayogi_nakshatra.index() as i32 - y.sphuta.nakshatra.index() as i32)
            .rem_euclid(27) + 1;
        assert_eq!(count, 6, "the Avayogi nakshatra is not the sixth from the Yogi's");
        assert_eq!(y.yogi, y.sphuta.nakshatra.lord());
        assert_eq!(y.avayogi, y.avayogi_nakshatra.lord());
    }
}

#[test]
fn the_shift_holds_for_every_nakshatra_the_yogi_could_land_in() {
    // The same invariant, exhaustively: whichever of the 27 the sphuta falls
    // in, the shift is five.
    for i in 0..27 {
        let yogi_nak = Nakshatra::from_index(i);
        let avy_nak = Nakshatra::from_index(i + AVAYOGI_COUNT as i32 - 1);
        let shift = (cycle_index(avy_nak.lord()) + 9 - cycle_index(yogi_nak.lord())) % 9;
        assert_eq!(shift, 5, "nakshatra {}", i + 1);
    }
}

// ===========================================================================
// Criterion 6 - the variants, including the held one
// ===========================================================================

#[test]
fn every_result_names_its_variants_and_says_the_duplicate_yogi_is_held() {
    let chart = random_charts(1, 0x7A720).remove(0);
    let y = Yogi::compute(&chart);
    assert_eq!(y.variants.len(), VARIANTS.len());
    let ids: Vec<String> = y.variants.iter().map(|v| v.id.to_string()).collect();
    assert_eq!(ids, ["V-13-38", "V-13-39", "V-13-40"]);
    let dup = y.variants.iter().find(|v| v.id == "V-13-40").unwrap();
    assert!(dup.chosen.contains("not computed"), "V-13-40 must say the Duplicate Yogi is held");
}

#[test]
fn the_same_chart_always_gives_the_same_points_and_they_round_trip() {
    for chart in random_charts(30, 0xD37F) {
        assert_eq!(Yogi::compute(&chart), Yogi::compute(&chart));
    }
    let chart = random_charts(1, 0x1505).remove(0);
    let y = Yogi::compute(&chart);
    let text = serde_json::to_string(&y).expect("serialise");
    let back: Yogi = serde_json::from_str(&text).expect("deserialise");
    assert_eq!(y, back);
}
