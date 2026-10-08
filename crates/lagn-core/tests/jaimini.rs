//! Phase 13A QA suite: chara karakas, arudha padas, argala.
//!
//! Written against `docs/phase13/DESIGN.md`, acceptance criteria 1 to 5.
//! The N-version check (criterion 6) lives in `scripts/qa_jaimini_oracle.py`.

use std::collections::{BTreeMap, BTreeSet};

use lagn_core::jaimini::{
    advancement, argala, arudha_pada, arudha_padas, ArgalaKind, ArgalaVerdict, CharaKarakas,
    Jaimini, Karaka, CANDIDATES, VARIANTS,
};
use lagn_core::*;

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

/// Charts spread over eleven centuries and the whole globe, so no property
/// below can be an artifact of one era or one latitude.
fn random_charts(n: usize, seed: u64) -> Vec<Chart> {
    init();
    let mut rng = Rng(seed | 1);
    let mut out = Vec::new();
    while out.len() < n {
        let b = BirthData {
            moment: BirthMoment {
                year: rng.int(1300, 2390) as i32,
                month: rng.int(1, 12) as u32,
                day: rng.int(1, 28) as u32,
                hour: rng.int(0, 23) as u32,
                minute: rng.int(0, 59) as u32,
                second: rng.unit() * 59.0,
                utc_offset_hours: 5.5,
            },
            latitude: rng.unit() * 140.0 - 70.0,
            longitude: rng.unit() * 360.0 - 180.0,
            place_name: String::new(),
        };
        if let Ok(c) = Chart::compute(b, ChartSettings::default()) {
            out.push(c);
        }
    }
    out
}

// ===========================================================================
// Criterion 1 - the karaka ranking is a total order
// ===========================================================================

#[test]
fn the_eight_karakas_are_eight_distinct_grahas_with_every_rank_filled_once() {
    for chart in random_charts(400, 0xC0FFEE) {
        let k = CharaKarakas::compute(&chart);
        assert_eq!(k.assigned.len(), 8, "expected eight karakas");

        let grahas: BTreeSet<_> = k.assigned.iter().map(|a| a.graha).collect();
        assert_eq!(grahas.len(), 8, "a graha holds two karakas");
        let ranks: BTreeSet<_> = k.assigned.iter().map(|a| a.karaka).collect();
        assert_eq!(ranks.len(), 8, "a karaka is assigned twice");

        // Ketu is never a candidate (V-13-3).
        assert!(!grahas.contains(&Graha::Ketu), "Ketu was ranked as a karaka");
        for g in CANDIDATES {
            assert!(grahas.contains(&g), "{} was not ranked", g.name());
        }

        // Descending advancement, which is what "rank" means here.
        for w in k.assigned.windows(2) {
            assert!(
                w[0].advancement >= w[1].advancement,
                "{} ({}) ranked above {} ({})",
                w[0].graha.name(), w[0].advancement, w[1].graha.name(), w[1].advancement,
            );
        }
        // And the ranks come out in the order the design lists them.
        let order: Vec<Karaka> = k.assigned.iter().map(|a| a.karaka).collect();
        assert_eq!(order, Karaka::ALL.to_vec());

        // Lookups both ways agree with the list.
        for a in &k.assigned {
            assert_eq!(k.of(a.karaka).graha, a.graha);
            assert_eq!(k.karaka_of(a.graha), Some(a.karaka));
        }
        assert_eq!(k.karaka_of(Graha::Ketu), None);
    }
}

#[test]
fn every_karaka_sits_in_the_sign_its_graha_occupies() {
    // The karaka is a label on a graha, not a position of its own. If these
    // ever diverged the Jaimini view would show a karaka in the wrong house.
    for chart in random_charts(150, 0x5EED) {
        for a in &CharaKarakas::compute(&chart).assigned {
            assert_eq!(a.rasi, chart.placement(a.graha).rasi, "{} misplaced", a.karaka.name());
        }
    }
}

// ===========================================================================
// Criterion 2 - Rahu's reversal is observable
// ===========================================================================

#[test]
fn rahu_is_ranked_on_the_part_of_the_sign_it_has_left_behind() {
    // The pure rule first: Rahu moves anti-zodiacally, so one degree into a
    // sign means twenty-nine degrees of it travelled (V-13-2).
    assert_eq!(advancement(Graha::Rahu, 1.0), 29.0);
    assert_eq!(advancement(Graha::Rahu, 29.0), 1.0);
    assert_eq!(advancement(Graha::Rahu, 15.0), 15.0);
    // Everyone else is read forwards.
    for g in CANDIDATES.iter().filter(|&&g| g != Graha::Rahu) {
        assert_eq!(advancement(*g, 1.0), 1.0, "{} should not be reversed", g.name());
    }

    // And the consequence, on real charts: Rahu early in its sign must rank
    // high, Rahu late must rank low. Without the reversal both would be the
    // other way round, so this pins the sign of the effect, not just the
    // arithmetic.
    let (mut saw_early, mut saw_late) = (false, false);
    for chart in random_charts(300, 0x4A4Fu64) {
        let deg = chart.placement(Graha::Rahu).degrees_in_rasi;
        let rank = CharaKarakas::compute(&chart)
            .assigned.iter().position(|a| a.graha == Graha::Rahu).unwrap();
        if deg < 2.0 {
            saw_early = true;
            assert!(rank <= 2, "Rahu at {deg:.2} deg ranked {rank}, too low for a reversed reading");
        }
        if deg > 28.0 {
            saw_late = true;
            assert!(rank >= 5, "Rahu at {deg:.2} deg ranked {rank}, too high for a reversed reading");
        }
    }
    assert!(saw_early, "the sweep never put Rahu early in a sign");
    assert!(saw_late, "the sweep never put Rahu late in a sign");
}

// ===========================================================================
// Criteria 3 and 4 - the arudha padas
// ===========================================================================

#[test]
fn no_pada_sits_on_its_own_bhava_or_opposite_it() {
    // This is the property the exception in section 5.1 step 3 exists to
    // guarantee, checked on every pada of every chart rather than argued.
    for chart in random_charts(400, 0xA12DA) {
        for p in arudha_padas(&chart) {
            let from_bhava = p.bhava_rasi.houses_to(p.rasi);
            assert!(
                from_bhava != 1 && from_bhava != 7,
                "A{} landed {} from its own bhava ({} -> {})",
                p.bhava, from_bhava, p.bhava_rasi.name(), p.rasi.name(),
            );
        }
    }
}

#[test]
fn a_pada_is_the_count_to_the_lord_taken_again_from_the_lord() {
    for chart in random_charts(200, 0xDEF) {
        for p in arudha_padas(&chart) {
            assert_eq!(p.bhava_rasi, chart.rasi_of_house(p.bhava));
            assert_eq!(p.lord, p.bhava_rasi.lord());
            assert_eq!(p.lord_rasi, chart.placement(p.lord).rasi);
            assert_eq!(p.count, p.bhava_rasi.houses_to(p.lord_rasi));
            assert!((1..=12).contains(&p.count));

            // raw is the count taken again from the lord's sign.
            assert_eq!(p.raw, Rasi::from_index(p.lord_rasi.index() as i32 + p.count as i32 - 1));
            // and the pada is raw, or the tenth from it when raw was refused.
            if p.adjusted {
                assert_eq!(p.rasi, Rasi::from_index(p.raw.index() as i32 + 9));
            } else {
                assert_eq!(p.rasi, p.raw);
            }
        }
    }
}

#[test]
fn the_exception_fires_exactly_on_counts_of_one_four_seven_and_ten() {
    // Those four counts are the ones that land the pada on the bhava or its
    // seventh. Every count must be seen by the sweep, and the adjustment must
    // follow the count exactly - never fire on another count, never fail to
    // fire on these.
    let mut seen: BTreeMap<u8, usize> = BTreeMap::new();
    for chart in random_charts(400, 0x1A7) {
        for p in arudha_padas(&chart) {
            *seen.entry(p.count).or_default() += 1;
            let should = matches!(p.count, 1 | 4 | 7 | 10);
            assert_eq!(
                p.adjusted, should,
                "count {} adjusted={} on bhava {}", p.count, p.adjusted, p.bhava,
            );
        }
    }
    for c in [1u8, 4, 7, 10] {
        assert!(seen.contains_key(&c), "the sweep never produced a count of {c}");
    }
    // And the ordinary counts occurred too, so the test is not only exercising
    // the exception.
    for c in [2u8, 3, 5, 6, 8, 9, 11, 12] {
        assert!(seen.contains_key(&c), "the sweep never produced a count of {c}");
    }
}

#[test]
fn the_twelve_padas_are_labelled_the_way_practice_names_them() {
    let chart = random_charts(1, 0xBEEF).remove(0);
    let padas = arudha_padas(&chart);
    assert_eq!(padas.len(), 12);
    assert_eq!(padas[0].label(), "AL", "A1 is the Arudha Lagna");
    assert_eq!(padas[11].label(), "UL", "A12 is the Upapada");
    assert_eq!(padas[6].label(), "A7");
    for (i, p) in padas.iter().enumerate() {
        assert_eq!(p.bhava, i as u8 + 1);
    }
}

#[test]
#[should_panic(expected = "bhava 13 out of range")]
fn a_bhava_outside_one_to_twelve_is_refused() {
    let chart = random_charts(1, 0xB00).remove(0);
    arudha_pada(&chart, 13);
}

// ===========================================================================
// Criterion 5 - argala
// ===========================================================================

#[test]
fn argala_uses_exactly_the_three_pairs_in_the_table() {
    for chart in random_charts(100, 0xA69A1A) {
        for sign in Rasi::ALL {
            let a = argala(&chart, sign);
            assert_eq!(a.rasi, sign);
            assert_eq!(a.pairs.len(), 3, "there are three argala pairs, not {}", a.pairs.len());

            let kinds: Vec<ArgalaKind> = a.pairs.iter().map(|p| p.kind).collect();
            assert_eq!(kinds, ArgalaKind::ALL.to_vec());

            for p in &a.pairs {
                let (ah, ch) = p.kind.houses();
                assert_eq!(p.argala_rasi, Rasi::from_index(sign.index() as i32 + ah as i32 - 1));
                assert_eq!(p.counter_rasi, Rasi::from_index(sign.index() as i32 + ch as i32 - 1));

                // The verdict is a function of the two counts, and nothing else.
                let (na, nc) = (p.argala_grahas.len(), p.counter_grahas.len());
                let expected = if na == 0 {
                    ArgalaVerdict::None
                } else if na > nc {
                    ArgalaVerdict::Stands
                } else if na == nc {
                    ArgalaVerdict::Neutralised
                } else {
                    ArgalaVerdict::Overcome
                };
                assert_eq!(p.verdict, expected, "{} argala on {}", p.kind.name(), sign.name());
            }
        }
    }
}

#[test]
fn the_pairs_are_the_houses_the_design_names() {
    assert_eq!(ArgalaKind::Wealth.houses(), (2, 12));
    assert_eq!(ArgalaKind::Home.houses(), (4, 10));
    assert_eq!(ArgalaKind::Gain.houses(), (11, 3));
}

#[test]
fn occupancy_counts_all_nine_grahas_including_ketu() {
    // V-13-5: a node in a sign is an occupant, as it is everywhere else in
    // this engine. Summing every sign's occupants over one pair must therefore
    // come to nine, not seven.
    for chart in random_charts(60, 0x9E70) {
        let total: usize = Rasi::ALL
            .iter()
            .map(|&r| argala(&chart, r).pairs[0].argala_grahas.len())
            .sum();
        assert_eq!(total, 9, "every graha should be counted exactly once across the twelve signs");

        let mut ketu_seen = false;
        for r in Rasi::ALL {
            if argala(&chart, r).pairs[0].argala_grahas.contains(&Graha::Ketu) { ketu_seen = true; }
        }
        assert!(ketu_seen, "Ketu was not counted as an occupant anywhere");
    }
}

// ===========================================================================
// Criterion 7 - the variants are on the output
// ===========================================================================

#[test]
fn every_result_names_the_variant_scheme_that_produced_it() {
    let chart = random_charts(1, 0x7A71A).remove(0);
    let j = Jaimini::compute(&chart);

    assert_eq!(j.variants.len(), VARIANTS.len());
    assert_eq!(j.variants.len(), 8, "section 7 of the design registers eight variants");
    let ids: Vec<String> = j.variants.iter().map(|v| v.id.to_string()).collect();
    assert_eq!(
        ids,
        ["V-13-1", "V-13-2", "V-13-3", "V-13-4", "V-13-5", "V-13-6", "V-13-7", "V-13-8"],
    );
    for v in &j.variants {
        assert!(!v.question.is_empty(), "{} has no question", v.id);
        assert!(!v.chosen.is_empty(), "{} records no choice", v.id);
    }

    assert_eq!(j.padas.len(), 12);
    assert_eq!(j.argala.len(), 12, "one entry per sign");
    assert_eq!(j.karakas.assigned.len(), 8);
}

#[test]
fn the_same_chart_always_gives_the_same_jaimini() {
    // Determinism is the whole claim of this engine, and the karaka ranking
    // is the one place here that sorts floating-point values.
    for chart in random_charts(40, 0xD37) {
        assert_eq!(Jaimini::compute(&chart), Jaimini::compute(&chart));
    }
}

#[test]
fn a_jaimini_result_survives_a_round_trip_through_json() {
    let chart = random_charts(1, 0x1501).remove(0);
    let j = Jaimini::compute(&chart);
    let text = serde_json::to_string(&j).expect("serialise");
    let back: Jaimini = serde_json::from_str(&text).expect("deserialise");
    assert_eq!(j, back);
}
