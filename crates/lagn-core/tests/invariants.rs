//! Property-based invariants.
//!
//! Boundary tests check the places we already suspect. These check properties
//! that must hold for *every* chart, over a large randomised sample.
//!
//! The generator is a seeded xorshift rather than a `rand` dependency, so a
//! failure is reproducible from the seed alone and the suite stays hermetic -
//! which matters more than usual for a project whose product is determinism.

use lagn_core::*;

/// Deterministic xorshift64*. Fixed seed: any failure reproduces exactly.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed | 1)
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform in `[0, 1)`.
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.unit() * (hi - lo)
    }
    fn int(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next_u64() % ((hi - lo + 1) as u64)) as i64
    }
}

fn init() {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(d).unwrap();
}

/// A random but always-valid birth.
fn random_birth(rng: &mut Rng) -> BirthData {
    let year = rng.int(1300, 2390) as i32;
    let month = rng.int(1, 12) as u32;
    let cal = Calendar::for_date(year, month, 1);
    let day = rng.int(1, days_in_month(year, month, cal) as i64) as u32;
    BirthData {
        moment: BirthMoment {
            year, month, day,
            hour: rng.int(0, 23) as u32,
            minute: rng.int(0, 59) as u32,
            second: rng.range(0.0, 59.999),
            // Real-world offsets, including the Indian historical ones.
            utc_offset_hours: [-11.0, -5.0, 0.0, 4.85, 5.3514, 5.5, 6.5, 9.0, 12.0]
                [(rng.next_u64() % 9) as usize],
        },
        // Avoid the exact poles, which are legitimately refused.
        latitude: rng.range(-89.0, 89.0),
        longitude: rng.range(-180.0, 180.0),
        place_name: String::new(),
    }
}

const SEED: u64 = 0x1A6E_5EED_C0FF_EE01;
const SAMPLES: usize = 600;

fn charts(n: usize) -> Vec<Chart> {
    init();
    let mut rng = Rng::new(SEED);
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let b = random_birth(&mut rng);
        if let Ok(c) = Chart::compute(b, ChartSettings::default()) {
            out.push(c);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Chart structure
// ---------------------------------------------------------------------------

#[test]
fn every_chart_has_all_nine_grahas_exactly_once() {
    for c in charts(SAMPLES) {
        let mut seen: Vec<Graha> = c.placements.iter().map(|p| p.graha).collect();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 9, "chart at JD {} lost a graha", c.jd_ut);
        for g in Graha::ALL {
            assert!(seen.contains(&g), "{} missing", g.name());
        }
    }
}

#[test]
fn every_derived_field_agrees_with_the_longitude_it_came_from() {
    for c in charts(SAMPLES) {
        for p in &c.placements {
            assert!(
                (0.0..360.0).contains(&p.longitude),
                "{} longitude {} out of range", p.graha.name(), p.longitude
            );
            assert_eq!(p.rasi, Rasi::from_longitude(p.longitude), "{} rasi", p.graha.name());
            assert_eq!(p.navamsa, navamsa_sign(p.longitude), "{} navamsa", p.graha.name());
            assert_eq!(
                p.nakshatra, NakshatraPosition::from_longitude(p.longitude),
                "{} nakshatra", p.graha.name()
            );
            assert!(
                (p.degrees_in_rasi - Rasi::degrees_within(p.longitude)).abs() < 1e-12,
                "{} degrees_in_rasi", p.graha.name()
            );
            assert!((0.0..30.0).contains(&p.degrees_in_rasi));
            assert!((1..=12).contains(&p.house), "{} house {}", p.graha.name(), p.house);
        }
    }
}

#[test]
fn house_number_is_always_the_count_from_the_lagna() {
    for c in charts(SAMPLES) {
        for p in &c.placements {
            assert_eq!(
                p.house, c.lagna.rasi.houses_to(p.rasi),
                "{} house disagrees with lagna-relative count", p.graha.name()
            );
            // And the inverse mapping must round-trip.
            assert_eq!(c.rasi_of_house(p.house), p.rasi, "{} rasi_of_house", p.graha.name());
        }
        // The lagna is always in the first house.
        assert_eq!(c.rasi_of_house(1), c.lagna.rasi);
    }
}

#[test]
fn the_twelve_houses_partition_the_nine_grahas() {
    for c in charts(SAMPLES) {
        let total: usize = (1..=12).map(|h| c.grahas_in_house(h).len()).sum();
        assert_eq!(total, 9, "houses hold {total} grahas, expected 9");
        // And the twelve houses cover all twelve rasis exactly once.
        let mut signs: Vec<Rasi> = (1..=12).map(|h| c.rasi_of_house(h)).collect();
        signs.sort();
        signs.dedup();
        assert_eq!(signs.len(), 12);
    }
}

#[test]
fn ketu_is_always_the_seventh_from_rahu() {
    for c in charts(SAMPLES) {
        let r = c.placement(Graha::Rahu);
        let k = c.placement(Graha::Ketu);
        assert_eq!(
            r.rasi.houses_to(k.rasi), 7,
            "Rahu in {:?}, Ketu in {:?} - not opposite", r.rasi, k.rasi
        );
        let sep = lagn_ephem::norm360(k.longitude - r.longitude);
        assert!((sep - 180.0).abs() < 1e-9, "separation {sep}");
        // Same degree within their respective signs.
        assert!((r.degrees_in_rasi - k.degrees_in_rasi).abs() < 1e-9);
    }
}

#[test]
fn janma_rasi_and_nakshatra_always_come_from_the_moon() {
    for c in charts(SAMPLES) {
        let moon = c.placement(Graha::Moon);
        assert_eq!(c.janma_rasi(), moon.rasi);
        assert_eq!(c.janma_nakshatra(), moon.nakshatra);
    }
}

#[test]
fn bhava_lords_follow_the_classical_rulership_table() {
    for c in charts(SAMPLES) {
        for h in 1..=12u8 {
            assert_eq!(c.lord_of_house(h), c.rasi_of_house(h).lord(), "house {h} lord");
        }
        // Across all twelve houses the seven lords appear with the right counts.
        let mut counts = std::collections::HashMap::new();
        for h in 1..=12u8 {
            *counts.entry(c.lord_of_house(h)).or_insert(0) += 1;
        }
        assert_eq!(counts[&Graha::Sun], 1);
        assert_eq!(counts[&Graha::Moon], 1);
        for g in [Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Venus, Graha::Saturn] {
            assert_eq!(counts[&g], 2, "{} should rule two houses", g.name());
        }
        assert!(!counts.contains_key(&Graha::Rahu), "Rahu rules no sign");
        assert!(!counts.contains_key(&Graha::Ketu), "Ketu rules no sign");
    }
}

// ---------------------------------------------------------------------------
// Varga
// ---------------------------------------------------------------------------

#[test]
fn varga_chart_is_consistent_with_the_rasi_chart_it_came_from() {
    for c in charts(SAMPLES) {
        let d1 = c.varga(Varga::D1);
        // D-1 must reproduce the rasi chart exactly.
        assert_eq!(d1.lagna, c.lagna.rasi);
        for p in &c.placements {
            assert_eq!(d1.sign_of(p.graha), p.rasi, "{} D-1 sign", p.graha.name());
            assert_eq!(d1.house_of(p.graha), p.house, "{} D-1 house", p.graha.name());
        }

        let d9 = c.varga(Varga::D9);
        assert_eq!(d9.lagna, c.lagna.navamsa);
        for p in &c.placements {
            assert_eq!(d9.sign_of(p.graha), p.navamsa, "{} D-9 sign", p.graha.name());
            assert_eq!(
                d9.house_of(p.graha), d9.lagna.houses_to(p.navamsa),
                "{} D-9 house", p.graha.name()
            );
        }
        // Houses partition the grahas in the varga too.
        let total: usize = (1..=12).map(|h| d9.grahas_in_house(h).len()).sum();
        assert_eq!(total, 9);
    }
}

// ---------------------------------------------------------------------------
// Dasha
// ---------------------------------------------------------------------------

#[test]
fn vimshottari_tiles_its_whole_span_with_no_gaps_or_overlaps() {
    for c in charts(200) {
        let v = Vimshottari::compute(&c);
        for w in v.mahadashas.windows(2) {
            assert_eq!(w[0].end_jd, w[1].start_jd, "mahadasha gap");
        }
        for m in &v.mahadashas {
            assert_eq!(m.children[0].start_jd, m.start_jd);
            assert_eq!(m.children[8].end_jd, m.end_jd);
            for w in m.children.windows(2) {
                assert_eq!(w[0].end_jd, w[1].start_jd, "antardasha gap");
            }
            for a in &m.children {
                assert_eq!(a.children[0].start_jd, a.start_jd);
                assert_eq!(a.children[8].end_jd, a.end_jd);
                for w in a.children.windows(2) {
                    assert_eq!(w[0].end_jd, w[1].start_jd, "pratyantardasha gap");
                }
            }
        }
    }
}

#[test]
fn sub_period_durations_sum_exactly_to_their_parent() {
    for c in charts(100) {
        let v = Vimshottari::compute(&c);
        for m in &v.mahadashas {
            let s: f64 = m.children.iter().map(|x| x.duration_days()).sum();
            assert!(
                (s - m.duration_days()).abs() < 1e-9,
                "antardashas sum to {s}, mahadasha is {}", m.duration_days()
            );
            for a in &m.children {
                let s2: f64 = a.children.iter().map(|x| x.duration_days()).sum();
                assert!(
                    (s2 - a.duration_days()).abs() < 1e-9,
                    "pratyantardashas sum to {s2}, antardasha is {}", a.duration_days()
                );
            }
        }
    }
}

#[test]
fn the_birth_dasha_lord_is_always_the_janma_nakshatra_lord() {
    for c in charts(SAMPLES) {
        let v = Vimshottari::compute(&c);
        assert_eq!(v.birth_lord, c.janma_nakshatra().nakshatra.lord());
        assert_eq!(v.janma_nakshatra, c.janma_nakshatra().nakshatra);
        let chain = v.at_birth().expect("a chain must be running at birth");
        assert_eq!(chain.maha, v.birth_lord);
    }
}

#[test]
fn dasha_balance_never_exceeds_the_lords_full_term() {
    for c in charts(SAMPLES) {
        let v = Vimshottari::compute(&c);
        let full = lagn_core::dasha::dasha_years(v.birth_lord);
        assert!(
            v.balance_years > 0.0 && v.balance_years <= full + 1e-9,
            "balance {} outside (0, {full}] for {}", v.balance_years, v.birth_lord.name()
        );
    }
}

#[test]
fn every_instant_in_the_tree_resolves_to_a_three_level_chain() {
    for c in charts(60) {
        let v = Vimshottari::compute(&c);
        let first = v.mahadashas.first().unwrap().start_jd;
        let last = v.mahadashas.last().unwrap().end_jd;
        let mut rng = Rng::new(0xD00D_5EED);
        // Random instants, plus every pratyantardasha edge.
        for _ in 0..500 {
            let jd = rng.range(first, last);
            assert!(v.at(jd).is_some(), "no chain at JD {jd}");
        }
        for m in &v.mahadashas {
            for a in &m.children {
                for p in &a.children {
                    assert!(v.at(p.start_jd).is_some());
                    assert!(v.at(next_down(p.end_jd)).is_some());
                }
            }
        }
        assert!(v.at(first - 1.0).is_none(), "resolved before the tree begins");
        assert!(v.at(last).is_none(), "resolved at/after the tree ends");
    }
}

fn next_down(x: f64) -> f64 {
    f64::from_bits(x.to_bits() - 1)
}

// ---------------------------------------------------------------------------
// Determinism - the project's central promise
// ---------------------------------------------------------------------------

#[test]
fn recomputation_is_bit_identical() {
    init();
    let mut rng = Rng::new(SEED ^ 0xABCD);
    for _ in 0..80 {
        let b = random_birth(&mut rng);
        let Ok(first) = Chart::compute(b.clone(), ChartSettings::default()) else { continue };
        let v1 = Vimshottari::compute(&first);
        for _ in 0..5 {
            let again = Chart::compute(b.clone(), ChartSettings::default()).unwrap();
            assert_eq!(again, first, "chart differed on recomputation");
            assert_eq!(Vimshottari::compute(&again), v1, "dasha differed");
        }
    }
}

#[test]
fn settings_are_honoured_and_actually_change_the_result() {
    init();
    let b = BirthData {
        moment: BirthMoment {
            year: 1985, month: 6, day: 21, hour: 14, minute: 30, second: 0.0,
            utc_offset_hours: 5.5,
        },
        latitude: 13.0827, longitude: 80.2707, place_name: String::new(),
    };
    let lahiri = Chart::compute(b.clone(), ChartSettings::default()).unwrap();
    for aya in [Ayanamsa::Raman, Ayanamsa::Krishnamurti, Ayanamsa::TrueChitra, Ayanamsa::Yukteshwar] {
        let other = Chart::compute(
            b.clone(),
            ChartSettings { ayanamsa: aya, ..Default::default() },
        ).unwrap();
        assert_ne!(
            other.ayanamsa_value, lahiri.ayanamsa_value,
            "{aya:?} produced the same ayanamsa as Lahiri"
        );
        assert_eq!(other.settings.ayanamsa, aya, "settings not recorded on the chart");
    }
    // Node type must be recorded and must move Rahu.
    let true_node = Chart::compute(
        b.clone(),
        ChartSettings { node_type: NodeType::True, ..Default::default() },
    ).unwrap();
    assert_ne!(
        true_node.placement(Graha::Rahu).longitude,
        lahiri.placement(Graha::Rahu).longitude
    );
    // Everything except the nodes must be untouched by the node setting.
    for g in [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury,
              Graha::Jupiter, Graha::Venus, Graha::Saturn] {
        assert_eq!(
            true_node.placement(g).longitude, lahiri.placement(g).longitude,
            "{} moved when only the node type changed", g.name()
        );
    }
}
