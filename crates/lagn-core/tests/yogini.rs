//! Phase 13D QA suite: Yogini dasha.
//!
//! Written against `docs/phase13/YOGINI.md`, criteria 1 to 8 and 10.
//! The N-version check (criterion 9) lives in `scripts/qa_yogini_oracle.py`.

use std::collections::BTreeSet;

use lagn_core::yogini::{starting_yogini, Yogini, YoginiDasha, CYCLE_YEARS, HORIZON_YEARS, VARIANTS};
use lagn_core::*;

/// A second, in days. Boundaries closer than this are the same boundary.
const SECOND: f64 = 1.0 / 86400.0;

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

fn charts_with(n: usize, seed: u64, year_length: YearLength) -> Vec<Chart> {
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
        let settings = ChartSettings { year_length, ..ChartSettings::default() };
        if let Ok(c) = Chart::compute(b, settings) {
            out.push(c);
        }
    }
    out
}

fn random_charts(n: usize, seed: u64) -> Vec<Chart> {
    charts_with(n, seed, YearLength::default())
}

// ===========================================================================
// Criterion 1 - the table checks itself
// ===========================================================================

#[test]
fn the_eight_periods_are_the_consecutive_integers_one_to_eight_and_sum_to_thirty_six() {
    // This is the whole reason Yogini could be built from a specification
    // while the rest of 13D could not. The periods are not eight arbitrary
    // numbers to transcribe: they are 1 through 8, and they sum to the cycle
    // length. A wrong one breaks the run and the total together.
    let years: Vec<f64> = Yogini::ALL.iter().map(|y| y.years()).collect();
    assert_eq!(years, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    assert_eq!(years.iter().sum::<f64>(), CYCLE_YEARS);
    assert_eq!(CYCLE_YEARS, 36.0);

    // And each yogini's period is its own ordinal, which is what makes the
    // run consecutive by construction rather than by coincidence.
    for (i, y) in Yogini::ALL.iter().enumerate() {
        assert_eq!(y.number(), i as u8 + 1);
        assert_eq!(y.years(), y.number() as f64);
    }

    // The eight lords are eight distinct grahas, so no yogini is a duplicate
    // of another under a different name.
    let lords: BTreeSet<Graha> = Yogini::ALL.iter().map(|y| y.lord()).collect();
    assert_eq!(lords.len(), 8);
    // Sankata's lord is Rahu (V-13-36), which is the one disputed entry.
    assert_eq!(Yogini::Sankata.lord(), Graha::Rahu);
}

#[test]
fn nth_from_walks_the_cycle_and_wraps() {
    for y in Yogini::ALL {
        assert_eq!(y.nth_from(1), y, "the first from a yogini is itself");
        assert_eq!(y.nth_from(9), y, "the ninth wraps to itself");
        let seen: BTreeSet<Yogini> = (1..=8).map(|n| y.nth_from(n)).collect();
        assert_eq!(seen.len(), 8, "eight steps from {:?} do not cover the cycle", y);
    }
    assert_eq!(Yogini::Sankata.nth_from(2), Yogini::Mangala, "the cycle wraps round");
}

// ===========================================================================
// Criteria 2 and 3 - the sequence and where it starts
// ===========================================================================

#[test]
fn the_starting_yogini_follows_the_rule_for_all_twenty_seven_nakshatras() {
    // Criterion 3, over the whole nakshatra range rather than a sample.
    for i in 0..27u8 {
        let nak = Nakshatra::from_index(i as i32);
        let serial = i as u32 + 1;
        let r = (serial + 3) % 8;
        let want = Yogini::ALL[(if r == 0 { 8 } else { r }) as usize - 1];
        assert_eq!(starting_yogini(nak), want, "nakshatra {serial}");
    }
    // The two ends and a zero-remainder case, pinned by name so the rule's
    // effect is legible and not only its arithmetic.
    assert_eq!(starting_yogini(Nakshatra::from_index(0)), Yogini::Bhramari, "Ashwini");
    assert_eq!(starting_yogini(Nakshatra::from_index(4)), Yogini::Sankata, "Mrigashira, remainder 0");
    assert_eq!(starting_yogini(Nakshatra::from_index(26)), Yogini::Ulka, "Revati");
    // Every yogini is reachable as a starting point across the 27.
    let starts: BTreeSet<Yogini> = (0..27)
        .map(|i| starting_yogini(Nakshatra::from_index(i)))
        .collect();
    assert_eq!(starts.len(), 8, "only {} yoginis can ever start a chart", starts.len());
}

#[test]
fn each_cycle_visits_all_eight_yoginis_once_in_order() {
    for chart in random_charts(200, 0x409141) {
        let d = YoginiDasha::compute(&chart);
        assert_eq!(d.birth_yogini, starting_yogini(d.janma_nakshatra));
        assert_eq!(d.periods[0].yogini, d.birth_yogini, "the list opens on the birth yogini");

        // Consecutive mahadashas step one place in the cycle.
        for w in d.periods.windows(2) {
            assert_eq!(w[1].yogini, w[0].yogini.nth_from(2), "the sequence skipped");
            assert_eq!(w[0].lord, w[0].yogini.lord());
        }
        // The first eight are the eight, once each.
        let first_eight: BTreeSet<Yogini> = d.periods.iter().take(8).map(|p| p.yogini).collect();
        assert_eq!(first_eight.len(), 8);
    }
}

// ===========================================================================
// Criteria 4 and 5 - the balance, and the timeline
// ===========================================================================

#[test]
fn the_balance_is_proportional_to_the_untraversed_nakshatra() {
    let mut saw_early = false;
    let mut saw_late = false;
    for chart in random_charts(400, 0xBA1) {
        let d = YoginiDasha::compute(&chart);
        let f = chart.janma_nakshatra().fraction_traversed;
        let want = d.birth_yogini.years() * (1.0 - f);
        assert!((d.balance_years - want).abs() < 1e-9, "balance");
        assert!(d.balance_years >= 0.0 && d.balance_years <= d.birth_yogini.years() + 1e-9);

        // Criterion 4's two ends: nearly all of the period at the start of a
        // nakshatra, nearly none at the end.
        if f < 0.01 {
            saw_early = true;
            assert!(d.balance_years > d.birth_yogini.years() * 0.98);
        }
        if f > 0.99 {
            saw_late = true;
            assert!(d.balance_years < d.birth_yogini.years() * 0.02);
        }
    }
    assert!(saw_early, "the sweep never began a nakshatra");
    assert!(saw_late, "the sweep never ended one");
}

#[test]
fn the_first_period_opens_before_birth_and_the_second_after_it() {
    // Criterion 5's other half. The running yogini started before the native
    // did, and keeping that opening portion is what makes the antardasha
    // boundaries inside it correct - the same reason dasha.rs keeps the
    // Vimshottari birth mahadasha whole.
    for chart in random_charts(120, 0x0BE4) {
        let d = YoginiDasha::compute(&chart);
        assert!(d.periods[0].start_jd <= chart.jd_ut + SECOND, "the first period starts after birth");
        assert!(d.periods[0].end_jd > chart.jd_ut, "the first period ends before birth");
        assert!(d.periods[1].start_jd >= chart.jd_ut - SECOND, "the second period starts before birth");

        // And the balance is exactly what remains of it.
        let days = chart.settings.year_length.days();
        let remaining = (d.periods[0].end_jd - chart.jd_ut) / days;
        assert!((remaining - d.balance_years).abs() < 1e-6, "balance disagrees with the first period");
    }
}

#[test]
fn periods_tile_the_timeline_bitwise_with_no_gap_or_overlap() {
    for chart in random_charts(200, 0x7111E) {
        let d = YoginiDasha::compute(&chart);
        for w in d.periods.windows(2) {
            assert!(w[0].end_jd > w[0].start_jd, "{:?} has no duration", w[0].yogini);
            assert_eq!(
                w[0].end_jd.to_bits(), w[1].start_jd.to_bits(),
                "{:?} ends at {} but {:?} starts at {}",
                w[0].yogini, w[0].end_jd, w[1].yogini, w[1].start_jd,
            );
        }
        for p in &d.periods {
            assert_eq!(p.level, 1);
            let days = chart.settings.year_length.days();
            let years = p.duration_days() / days;
            assert!((years - p.yogini.years()).abs() < 1e-6, "{:?} runs {years} years", p.yogini);
        }
    }
}

// ===========================================================================
// Criteria 6 and 7 - antardashas, and the horizon
// ===========================================================================

#[test]
fn eight_proportional_antardashas_fill_each_mahadasha() {
    for chart in random_charts(120, 0xA27A4) {
        let d = YoginiDasha::compute(&chart);
        for p in &d.periods {
            assert_eq!(p.children.len(), 8, "{:?} has {} antardashas", p.yogini, p.children.len());
            // Opens on the mahadasha's own yogini and steps through the cycle.
            for (i, c) in p.children.iter().enumerate() {
                assert_eq!(c.level, 2);
                assert_eq!(c.yogini, p.yogini.nth_from(i as u8 + 1));
                assert_eq!(c.lord, c.yogini.lord());
            }
            // Proportional to the sub-yogini's own years out of 36.
            for c in &p.children {
                let want = p.duration_days() * c.yogini.years() / CYCLE_YEARS;
                assert!((c.duration_days() - want).abs() < 1e-6, "{:?}/{:?}", p.yogini, c.yogini);
            }
            // Seams exact, and the last closes on the parent.
            assert_eq!(p.children[0].start_jd.to_bits(), p.start_jd.to_bits());
            assert_eq!(p.children[7].end_jd.to_bits(), p.end_jd.to_bits());
            for w in p.children.windows(2) {
                assert_eq!(w[0].end_jd.to_bits(), w[1].start_jd.to_bits(), "an antardasha seam is not exact");
            }
        }
    }
}

#[test]
fn the_periods_reach_past_the_horizon() {
    // A 36-year cycle would run out well inside a long life, so the engine
    // has to repeat it. This is the property that catches an engine emitting
    // exactly eight periods.
    for chart in random_charts(200, 0x401A4) {
        let d = YoginiDasha::compute(&chart);
        let days = chart.settings.year_length.days();
        let covered = (d.periods.last().unwrap().end_jd - chart.jd_ut) / days;
        assert!(covered >= HORIZON_YEARS, "only {covered:.1} years covered");
        assert!(d.periods.len() > 8, "one cycle is not enough to reach the horizon");
        // Cycle numbering starts at 1 and never skips.
        let cycles: BTreeSet<u32> = d.periods.iter().map(|p| p.cycle).collect();
        assert_eq!(cycles.len() as u32, *cycles.iter().max().unwrap());
    }
}

#[test]
fn the_chain_at_an_instant_matches_the_period_containing_it() {
    for chart in random_charts(40, 0xC4A1E) {
        let d = YoginiDasha::compute(&chart);
        let days = chart.settings.year_length.days();
        for age in [0.0, 2.5, 19.0, 41.3, 77.0, 103.5] {
            let jd = chart.jd_ut + age * days;
            let chain = d.at(jd).expect("in range");
            let maha = d.periods.iter().find(|p| p.contains(jd)).unwrap();
            assert_eq!(chain.maha, maha.yogini);
            assert_eq!(chain.antar, maha.children.iter().find(|c| c.contains(jd)).unwrap().yogini);
        }
    }
}

// ===========================================================================
// Criterion 8 - the year length is the chart's
// ===========================================================================

#[test]
fn boundaries_scale_with_the_charts_own_year_length() {
    let julian = charts_with(25, 0x4EA4, YearLength::Julian);
    let savana = charts_with(25, 0x4EA4, YearLength::Savana);
    let ratio = YearLength::Savana.days() / YearLength::Julian.days();

    for (j, s) in julian.iter().zip(&savana) {
        let (dj, ds) = (YoginiDasha::compute(j), YoginiDasha::compute(s));
        assert_eq!(dj.birth_yogini, ds.birth_yogini, "the same chart must start in the same yogini");
        assert!((dj.balance_years - ds.balance_years).abs() < 1e-9, "the balance is in years, not days");
        for (pj, ps) in dj.periods.iter().zip(&ds.periods).take(10) {
            let (ej, es) = (pj.end_jd - dj.birth_jd, ps.end_jd - ds.birth_jd);
            assert!((es - ej * ratio).abs() < SECOND, "{:?}: {es} is not {ratio} times {ej}", pj.yogini);
        }
    }
}

// ===========================================================================
// Criterion 10 - the variants, and determinism
// ===========================================================================

#[test]
fn every_result_names_the_variant_scheme_that_produced_it() {
    let chart = random_charts(1, 0x7A71F).remove(0);
    let d = YoginiDasha::compute(&chart);
    assert_eq!(d.variants.len(), VARIANTS.len());
    let ids: Vec<String> = d.variants.iter().map(|v| v.id.to_string()).collect();
    assert_eq!(ids, ["V-13-33", "V-13-34", "V-13-35", "V-13-36", "V-13-37"]);
    for v in &d.variants {
        assert!(!v.question.is_empty() && !v.chosen.is_empty(), "{} is blank", v.id);
    }
}

#[test]
fn the_same_chart_always_gives_the_same_dasha_and_it_round_trips() {
    for chart in random_charts(30, 0xD37E) {
        assert_eq!(YoginiDasha::compute(&chart), YoginiDasha::compute(&chart));
    }
    let chart = random_charts(1, 0x1504).remove(0);
    let d = YoginiDasha::compute(&chart);
    let text = serde_json::to_string(&d).expect("serialise");
    let back: YoginiDasha = serde_json::from_str(&text).expect("deserialise");
    assert_eq!(d, back);
}
