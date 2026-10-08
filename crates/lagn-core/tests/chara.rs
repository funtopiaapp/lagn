//! Phase 13B QA suite: Chara dasha.
//!
//! Written against `docs/phase13/CHARA-DASHA.md`, acceptance criteria 1 to 6
//! and 8. The N-version check (criterion 7) lives in
//! `scripts/qa_chara_oracle.py`.

use std::collections::BTreeSet;

use lagn_core::chara::{length, CharaDasha, Direction, HORIZON_YEARS, VARIANTS};
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
// Criterion 1 - the sequence
// ===========================================================================

#[test]
fn each_cycle_visits_all_twelve_signs_once_in_the_lagnas_direction() {
    for chart in random_charts(300, 0xC4A2A) {
        let d = CharaDasha::compute(&chart);

        // The direction is the lagna's parity, and nothing else (V-13-9).
        let expected = if chart.lagna.rasi.is_odd() { Direction::Direct } else { Direction::Reverse };
        assert_eq!(d.direction, expected, "lagna {} got the wrong direction", d.lagna.name());

        let signs: Vec<Rasi> = d.lengths.iter().map(|l| l.rasi).collect();
        assert_eq!(signs.len(), 12);
        assert_eq!(signs.iter().collect::<BTreeSet<_>>().len(), 12, "a sign appears twice");
        assert_eq!(signs[0], chart.lagna.rasi, "the sequence must open on the lagna");

        // Consecutive signs step one place in the sequence's direction.
        for w in signs.windows(2) {
            let step = w[1].index() as i32 - w[0].index() as i32;
            let want: i32 = if d.direction == Direction::Direct { 1 } else { -1 };
            assert_eq!(step.rem_euclid(12), want.rem_euclid(12), "{} to {} is not one step", w[0].name(), w[1].name());
        }
    }
}

#[test]
fn counting_runs_the_way_each_signs_own_parity_says() {
    // The trap this pins: the sequence's direction is the lagna's, but the
    // count that measures a sign's length is that sign's own (V-13-10). A
    // chart with an odd lagna still counts its even signs backwards.
    for chart in random_charts(60, 0xD12) {
        for rasi in Rasi::ALL {
            let l = length(&chart, rasi);
            assert_eq!(l.direction, Direction::of(rasi), "{} counted the wrong way", rasi.name());
            assert_eq!(l.direction == Direction::Direct, rasi.is_odd());
            assert_eq!(l.count, l.direction.count(rasi, l.lord_rasi));
        }
    }
}

#[test]
fn direction_helpers_agree_with_each_other() {
    // nth_from and count are inverses, in both directions, for all 144 pairs.
    for d in [Direction::Direct, Direction::Reverse] {
        for start in Rasi::ALL {
            for n in 1..=12u8 {
                let target = d.nth_from(start, n);
                assert_eq!(d.count(start, target), n, "{d:?} {} step {n}", start.name());
            }
        }
    }
    // And the two directions disagree except on the sign itself and the 7th.
    assert_eq!(Direction::Direct.nth_from(Rasi::Mesha, 2), Rasi::Vrishabha);
    assert_eq!(Direction::Reverse.nth_from(Rasi::Mesha, 2), Rasi::Meena);
    assert_eq!(Direction::Direct.nth_from(Rasi::Mesha, 7), Rasi::Tula);
    assert_eq!(Direction::Reverse.nth_from(Rasi::Mesha, 7), Rasi::Tula);
}

// ===========================================================================
// Criterion 2 - lengths
// ===========================================================================

#[test]
fn a_dasha_is_one_to_eleven_years_or_twelve_when_the_lord_is_at_home() {
    let mut saw_home = false;
    let mut saw_short = false;
    for chart in random_charts(300, 0x1E0Au64) {
        let d = CharaDasha::compute(&chart);
        for l in &d.lengths {
            assert_eq!(l.lord, l.rasi.lord(), "{} has the wrong lord", l.rasi.name());
            assert_eq!(l.lord_rasi, chart.placement(l.lord).rasi);
            assert!((1..=12).contains(&l.count), "count {} out of range", l.count);

            if l.lord_at_home {
                saw_home = true;
                assert_eq!(l.count, 1);
                assert_eq!(l.years, 12.0, "{} has its lord at home and must get 12", l.rasi.name());
                assert_eq!(l.lord_rasi, l.rasi);
            } else {
                assert_eq!(l.years, l.count as f64 - 1.0);
                assert!((1.0..=11.0).contains(&l.years), "{} years out of range", l.years);
                saw_short = true;
            }
        }
        // A cycle therefore runs between twelve and 144 years.
        let total = d.cycle_years();
        assert!((12.0..=144.0).contains(&total), "cycle of {total} years");
    }
    assert!(saw_home, "the sweep never put a lord in its own sign");
    assert!(saw_short, "the sweep produced only at-home lords");
}

// ===========================================================================
// Criteria 3, 4 and 5 - the timeline
// ===========================================================================

#[test]
fn mahadashas_tile_the_timeline_from_birth_with_no_gap_or_overlap() {
    for chart in random_charts(200, 0x7111) {
        let d = CharaDasha::compute(&chart);
        assert!(!d.periods.is_empty());
        assert!(
            (d.periods[0].start_jd - chart.jd_ut).abs() < SECOND,
            "the first mahadasha must begin at birth, not {} days off",
            d.periods[0].start_jd - chart.jd_ut,
        );

        for w in d.periods.windows(2) {
            assert!(w[0].end_jd > w[0].start_jd, "{} has no duration", w[0].rasi.name());
            assert!(
                (w[1].start_jd - w[0].end_jd).abs() < SECOND,
                "gap of {} days between {} and {}",
                w[1].start_jd - w[0].end_jd, w[0].rasi.name(), w[1].rasi.name(),
            );
        }
        for p in &d.periods {
            assert_eq!(p.level, 1);
            assert!(p.cycle >= 1);
        }
    }
}

#[test]
fn twelve_equal_antardashas_fill_each_mahadasha() {
    for chart in random_charts(120, 0xA27A2) {
        let d = CharaDasha::compute(&chart);
        for p in &d.periods {
            assert_eq!(p.children.len(), 12, "{} has {} antardashas", p.rasi.name(), p.children.len());
            // Opens on the mahadasha's own sign, and steps in the sequence's
            // direction (V-13-11).
            assert_eq!(p.children[0].rasi, p.rasi);
            for (i, c) in p.children.iter().enumerate() {
                assert_eq!(c.level, 2);
                assert_eq!(c.rasi, d.direction.nth_from(p.rasi, i as u8 + 1));
            }
            // Equal twelfths, and they close exactly on the parent's end.
            let twelfth = p.duration_days() / 12.0;
            for c in &p.children {
                assert!((c.duration_days() - twelfth).abs() < SECOND, "uneven antardasha");
            }
            assert!((p.children[0].start_jd - p.start_jd).abs() < SECOND);
            assert!(
                (p.children[11].end_jd - p.end_jd).abs() < 1e-9,
                "the last antardasha must close on the mahadasha, not {} days off",
                p.children[11].end_jd - p.end_jd,
            );
            let signs: BTreeSet<_> = p.children.iter().map(|c| c.rasi).collect();
            assert_eq!(signs.len(), 12, "an antardasha sign repeats");
        }
    }
}

#[test]
fn the_periods_reach_past_the_horizon_however_short_the_cycle() {
    // A cycle can be twelve years, so one pass is not enough. This is the
    // property that catches an engine that emits exactly twelve periods.
    for chart in random_charts(200, 0x401A0) {
        let d = CharaDasha::compute(&chart);
        let days = chart.settings.year_length.days();
        let covered = (d.periods.last().unwrap().end_jd - chart.jd_ut) / days;
        assert!(
            covered >= HORIZON_YEARS,
            "only {covered:.1} years covered by a {:.0}-year cycle",
            d.cycle_years(),
        );
        // Cycles are numbered from 1 and never skip.
        let cycles: BTreeSet<u32> = d.periods.iter().map(|p| p.cycle).collect();
        let max = *cycles.iter().max().unwrap();
        assert_eq!(cycles.len() as u32, max, "cycle numbering has a hole");
    }
}

#[test]
fn the_chain_at_an_instant_matches_the_period_containing_it() {
    for chart in random_charts(40, 0xC4A1) {
        let d = CharaDasha::compute(&chart);
        let days = chart.settings.year_length.days();
        for age in [0.0, 1.5, 17.0, 33.3, 61.0, 97.7] {
            let jd = chart.jd_ut + age * days;
            let chain = d.at(jd).expect("in range");
            let maha = d.periods.iter().find(|p| p.contains(jd)).unwrap();
            assert_eq!(chain.maha, maha.rasi);
            assert_eq!(chain.antar, maha.children.iter().find(|c| c.contains(jd)).unwrap().rasi);
        }
        // Before birth there is nothing to report.
        assert!(d.at(chart.jd_ut - 1.0).is_none());
    }
}

// ===========================================================================
// Criterion 6 - the year length is the chart's
// ===========================================================================

#[test]
fn boundaries_scale_with_the_charts_own_year_length() {
    // Chara dasha must not carry its own idea of how long a year is: a chart
    // computed in savana years has to give savana Chara boundaries, or one
    // chart would hold two calendars.
    let julian = charts_with(25, 0x4EA2, YearLength::Julian);
    let savana = charts_with(25, 0x4EA2, YearLength::Savana);
    let ratio = YearLength::Savana.days() / YearLength::Julian.days();

    for (j, s) in julian.iter().zip(&savana) {
        let (dj, ds) = (CharaDasha::compute(j), CharaDasha::compute(s));
        assert_eq!(dj.year_length, YearLength::Julian);
        assert_eq!(ds.year_length, YearLength::Savana);
        // Same chart, same sequence and same year counts; only the days differ.
        assert_eq!(
            dj.lengths.iter().map(|l| l.years).collect::<Vec<_>>(),
            ds.lengths.iter().map(|l| l.years).collect::<Vec<_>>(),
        );
        for (pj, ps) in dj.periods.iter().zip(&ds.periods).take(12) {
            let (ej, es) = (pj.end_jd - dj.birth_jd, ps.end_jd - ds.birth_jd);
            assert!(
                (es - ej * ratio).abs() < SECOND,
                "{}: {es} is not {ratio} times {ej}", pj.rasi.name(),
            );
        }
    }
}

// ===========================================================================
// Criterion 8 - the variants, and determinism
// ===========================================================================

#[test]
fn every_result_names_the_variant_scheme_that_produced_it() {
    let chart = random_charts(1, 0x7A71B).remove(0);
    let d = CharaDasha::compute(&chart);
    assert_eq!(d.variants.len(), VARIANTS.len());
    let ids: Vec<String> = d.variants.iter().map(|v| v.id.to_string()).collect();
    assert_eq!(ids, ["V-13-9", "V-13-10", "V-13-11", "V-13-12", "V-13-13"]);
    for v in &d.variants {
        assert!(!v.question.is_empty() && !v.chosen.is_empty(), "{} is blank", v.id);
    }
}

#[test]
fn the_same_chart_always_gives_the_same_chara_dasha() {
    for chart in random_charts(30, 0xD37A) {
        assert_eq!(CharaDasha::compute(&chart), CharaDasha::compute(&chart));
    }
}

#[test]
fn a_chara_dasha_survives_a_round_trip_through_json() {
    let chart = random_charts(1, 0x1502).remove(0);
    let d = CharaDasha::compute(&chart);
    let text = serde_json::to_string(&d).expect("serialise");
    let back: CharaDasha = serde_json::from_str(&text).expect("deserialise");
    assert_eq!(d, back);
}

// ===========================================================================
// Regression - found by the oracle, not by this suite
// ===========================================================================

#[test]
fn a_cycle_that_divides_the_horizon_exactly_does_not_run_an_extra_cycle() {
    // The oracle caught this. Boundaries were accumulated in Julian days, so
    // by the twenty-fourth period the running total was an ulp short of the
    // 120-year horizon and the loop started a third cycle that was not
    // needed. This chart has a 60-year cycle, which divides 120 exactly, and
    // is the shape that exposes it.
    init();
    let chart = Chart::compute(
        BirthData {
            moment: BirthMoment {
                year: 1985, month: 10, day: 28,
                hour: 3, minute: 34, second: 2.0, utc_offset_hours: 5.5,
            },
            latitude: 8.9481,
            longitude: 151.4803,
            place_name: String::new(),
        },
        ChartSettings { year_length: YearLength::Tropical, ..ChartSettings::default() },
    )
    .expect("chart");

    let d = CharaDasha::compute(&chart);
    assert_eq!(d.cycle_years(), 60.0, "this fixture is only useful at 60 years");
    assert_eq!(d.periods.len(), 24, "two cycles cover 120 years exactly; a third is waste");
    assert_eq!(d.periods.iter().map(|p| p.cycle).max(), Some(2));
}

#[test]
fn boundaries_are_exact_rather_than_merely_close() {
    // The other half of the same fix: a period's end and the next one's start
    // are computed from the identical expression, so they are bitwise equal.
    // Accumulating days made them differ by an ulp that grew with distance
    // from birth, which is how the drift above arose.
    for chart in random_charts(60, 0xE4AC7) {
        let d = CharaDasha::compute(&chart);
        for w in d.periods.windows(2) {
            assert_eq!(
                w[0].end_jd.to_bits(), w[1].start_jd.to_bits(),
                "{} ends at {} but {} starts at {}",
                w[0].rasi.name(), w[0].end_jd, w[1].rasi.name(), w[1].start_jd,
            );
        }
        for p in &d.periods {
            assert_eq!(p.children[0].start_jd.to_bits(), p.start_jd.to_bits());
            assert_eq!(p.children[11].end_jd.to_bits(), p.end_jd.to_bits());
            for w in p.children.windows(2) {
                assert_eq!(w[0].end_jd.to_bits(), w[1].start_jd.to_bits(), "antardasha seam is not exact");
            }
        }
    }
}
