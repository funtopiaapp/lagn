//! Phase 2 QA suite: derived facts, invariants and boundaries.
//!
//! Written against `docs/phase2/DESIGN.md`, acceptance criteria 1, 3 and 4.
//! The N-version check (criterion 2) lives in `scripts/qa_phase2_oracle.py`.

use lagn_core::analysis::DerivationSettings;
use lagn_core::ashtakavarga::{benefic_houses, BAV_TOTALS, SAV_TOTAL};
use lagn_core::condition::{self, BaladiAvastha};
use lagn_core::dignity::{self, Dignity};
use lagn_core::drishti;
use lagn_core::relationship::SEVEN;
use lagn_core::*;

const ARCSEC: f64 = 1.0 / 3600.0;

fn init() {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(d).unwrap();
}

/// Dense sweep that avoids exact cell edges.
fn sweep(step: f64) -> impl Iterator<Item = f64> {
    let n = (360.0 / step) as usize;
    (0..n).map(move |i| i as f64 * step + step * 0.37)
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
        let year = rng.int(1300, 2390) as i32;
        let month = rng.int(1, 12) as u32;
        let day = rng.int(1, 28) as u32;
        let b = BirthData {
            moment: BirthMoment {
                year, month, day,
                hour: rng.int(0, 23) as u32, minute: rng.int(0, 59) as u32,
                second: rng.unit() * 59.0, utc_offset_hours: 5.5,
            },
            latitude: rng.unit() * 170.0 - 85.0,
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
// Section 3 - derived facts about the vargas
// ===========================================================================

#[test]
fn d7_d9_d27_equal_the_continuous_formula() {
    for n in [7u32, 9, 27] {
        let v = Varga::from_number(n).unwrap();
        for lon in sweep(0.001) {
            let continuous = Rasi::from_index((lon / (30.0 / n as f64)).floor() as i32);
            assert_eq!(v.sign_of(lon), continuous, "D-{n} at {lon}");
        }
    }
}

#[test]
fn uniform_vargas_give_every_sign_exactly_n_parts() {
    for n in [3u32, 4, 7, 9, 10, 12, 16, 20, 24, 27, 60] {
        let v = Varga::from_number(n).unwrap();
        let mut counts = [0u32; 12];
        // Sample the midpoint of every part in the zodiac.
        for cell in 0..(12 * n) {
            let lon = (cell as f64 + 0.5) * (30.0 / n as f64);
            counts[v.sign_of(lon).index() as usize] += 1;
        }
        assert_eq!(counts, [n; 12], "D-{n} distribution {counts:?}");
    }
}

#[test]
fn hora_only_ever_yields_simha_or_karka() {
    for lon in sweep(0.01) {
        let s = Varga::D2.sign_of(lon);
        assert!(s == Rasi::Simha || s == Rasi::Karka, "D-2 gave {s:?} at {lon}");
    }
}

#[test]
fn hora_follows_the_odd_even_rule() {
    // Odd (Mesha): first half Simha, second half Karka.
    assert_eq!(Varga::D2.sign_of(7.0), Rasi::Simha);
    assert_eq!(Varga::D2.sign_of(22.0), Rasi::Karka);
    // Even (Vrishabha): first half Karka, second half Simha.
    assert_eq!(Varga::D2.sign_of(37.0), Rasi::Karka);
    assert_eq!(Varga::D2.sign_of(52.0), Rasi::Simha);
}

#[test]
fn trimsamsa_never_yields_a_luminary_sign_and_uses_five_signs_per_parity() {
    let mut odd = std::collections::BTreeSet::new();
    let mut even = std::collections::BTreeSet::new();
    for lon in sweep(0.001) {
        let s = Varga::D30.sign_of(lon);
        assert!(s != Rasi::Simha && s != Rasi::Karka, "D-30 gave {s:?} at {lon}");
        if Rasi::from_longitude(lon).is_odd() { odd.insert(s); } else { even.insert(s); }
    }
    assert_eq!(odd.len(), 5);
    assert_eq!(even.len(), 5);
    assert!(odd.is_disjoint(&even), "odd and even signs share a trimsamsa sign");
}

#[test]
fn trimsamsa_table_matches_the_specification_bands() {
    // (degree inside band, odd-sign result, even-sign result)
    let bands = [
        (2.5, Rasi::Mesha, Rasi::Vrishabha),
        (7.0, Rasi::Kumbha, Rasi::Kanya),
        (11.0, Rasi::Dhanus, Rasi::Kanya),
        (15.0, Rasi::Dhanus, Rasi::Meena),
        (19.0, Rasi::Mithuna, Rasi::Meena),
        (22.0, Rasi::Mithuna, Rasi::Makara),
        (27.0, Rasi::Tula, Rasi::Vrischika),
    ];
    for (deg, odd, even) in bands {
        for sign in 0..12 {
            let lon = sign as f64 * 30.0 + deg;
            let want = if sign % 2 == 0 { odd } else { even };
            assert_eq!(Varga::D30.sign_of(lon), want, "sign {sign} at {deg} deg");
        }
    }
}

#[test]
fn drekkana_and_chaturthamsa_step_by_the_stated_houses() {
    for s in 0..12 {
        let base = s as f64 * 30.0;
        let sign = Rasi::from_index(s);
        // D-3: 1st, 5th, 9th from the sign.
        for (k, h) in [(0, 1), (1, 5), (2, 9)] {
            let got = Varga::D3.sign_of(base + k as f64 * 10.0 + 5.0);
            assert_eq!(sign.houses_to(got), h, "D-3 sign {s} part {k}");
        }
        // D-4: 1st, 4th, 7th, 10th.
        for (k, h) in [(0, 1), (1, 4), (2, 7), (3, 10)] {
            let got = Varga::D4.sign_of(base + k as f64 * 7.5 + 3.0);
            assert_eq!(sign.houses_to(got), h, "D-4 sign {s} part {k}");
        }
    }
}

#[test]
fn every_equal_part_varga_starts_where_the_specification_says() {
    // First part of each sign, stated as sign names straight from the table.
    use Rasi::*;
    let first_part = |v: Varga, s: i32| v.sign_of(s as f64 * 30.0 + 1e-6);
    for s in 0..12 {
        let sign = Rasi::from_index(s);
        let odd = sign.is_odd();
        let mob = sign.mobility();
        use lagn_core::Mobility::*;
        assert_eq!(first_part(Varga::D7, s), if odd { sign } else { Rasi::from_index(s + 6) }, "D-7 {s}");
        assert_eq!(first_part(Varga::D10, s), if odd { sign } else { Rasi::from_index(s + 8) }, "D-10 {s}");
        assert_eq!(first_part(Varga::D12, s), sign, "D-12 {s}");
        assert_eq!(first_part(Varga::D16, s), match mob { Chara => Mesha, Sthira => Simha, Dvisvabhava => Dhanus }, "D-16 {s}");
        assert_eq!(first_part(Varga::D20, s), match mob { Chara => Mesha, Sthira => Dhanus, Dvisvabhava => Simha }, "D-20 {s}");
        assert_eq!(first_part(Varga::D24, s), if odd { Simha } else { Karka }, "D-24 {s}");
        assert_eq!(first_part(Varga::D27, s), match sign.element() {
            Element::Agni => Mesha, Element::Prithvi => Karka, Element::Vayu => Tula, Element::Jala => Makara,
        }, "D-27 {s}");
        assert_eq!(first_part(Varga::D40, s), if odd { Mesha } else { Tula }, "D-40 {s}");
        assert_eq!(first_part(Varga::D45, s), match mob { Chara => Mesha, Sthira => Simha, Dvisvabhava => Dhanus }, "D-45 {s}");
        assert_eq!(first_part(Varga::D60, s), sign, "D-60 {s}");
    }
}

#[test]
fn equal_part_vargas_advance_one_sign_per_part() {
    // Every varga except D-2, D-3, D-4, D-30 steps forward one sign per part.
    for v in [Varga::D7, Varga::D9, Varga::D10, Varga::D12, Varga::D16, Varga::D20,
              Varga::D24, Varga::D27, Varga::D40, Varga::D45, Varga::D60] {
        let n = v.parts_per_sign();
        let span = 30.0 / n as f64;
        for s in 0..12 {
            let first = v.sign_of(s as f64 * 30.0 + span * 0.5);
            for k in 1..n {
                let got = v.sign_of(s as f64 * 30.0 + (k as f64 + 0.5) * span);
                assert_eq!(got, Rasi::from_index(first.index() as i32 + k as i32), "{} sign {s} part {k}", v.label());
            }
        }
    }
}

#[test]
fn d1_varga_is_exactly_the_rasi() {
    for lon in sweep(0.01) {
        assert_eq!(Varga::D1.sign_of(lon), Rasi::from_longitude(lon));
    }
}

// ===========================================================================
// Section 3/4 boundaries: every varga edge +/- 1 arcsec
// ===========================================================================

#[test]
fn every_equal_part_varga_changes_sign_exactly_where_the_rules_imply() {
    // Inside a sign, every part edge changes the varga sign, in every varga.
    //
    // At a sign boundary the start rules can make the last part of one sign
    // and the first part of the next coincide. Derived independently from the
    // rules by scripts/qa_phase2_oracle.py, the complete set is:
    //   D-2        at every sign boundary (odd ends Karka, even starts Karka;
    //              even ends Simha, odd starts Simha)
    //   D-10, D-24 on entering every even sign (last part of the odd sign and
    //              first part of the even sign land on the same sign)
    // and nowhere else. The QA assumption that no boundary ever repeats was
    // wrong for D-10; the kernel was right.
    for v in Varga::ALL {
        if v == Varga::D30 || v == Varga::D1 {
            continue;
        }
        let n = v.parts_per_sign();
        let span = 30.0 / n as f64;
        for cell in 0..(12 * n) {
            let edge = cell as f64 * span;
            let before = v.sign_of(lagn_ephem::norm360(edge - ARCSEC));
            let after = v.sign_of(edge + ARCSEC);
            assert_eq!(after, v.sign_of(edge + span * 0.5), "{} changed inside cell {cell}", v.label());

            let entering = (cell / n) as i32;
            let at_sign_boundary = cell % n == 0;
            let expect_repeat = at_sign_boundary
                && match v {
                    Varga::D2 => true,
                    Varga::D10 | Varga::D24 => entering % 2 == 1,
                    _ => false,
                };
            if expect_repeat {
                assert_eq!(before, after, "{} should repeat entering sign {entering}", v.label());
            } else {
                assert_ne!(before, after, "{} did not change at cell {cell} ({edge})", v.label());
            }
        }
    }
}

#[test]
fn trimsamsa_boundaries_are_half_open() {
    for (edge, odd_before, odd_after, even_before, even_after) in [
        (5.0, Rasi::Mesha, Rasi::Kumbha, Rasi::Vrishabha, Rasi::Kanya),
        (10.0, Rasi::Kumbha, Rasi::Dhanus, Rasi::Kanya, Rasi::Kanya),
        (12.0, Rasi::Dhanus, Rasi::Dhanus, Rasi::Kanya, Rasi::Meena),
        (18.0, Rasi::Dhanus, Rasi::Mithuna, Rasi::Meena, Rasi::Meena),
        (20.0, Rasi::Mithuna, Rasi::Mithuna, Rasi::Meena, Rasi::Makara),
        (25.0, Rasi::Mithuna, Rasi::Tula, Rasi::Makara, Rasi::Vrischika),
    ] {
        // Mesha (odd) and Vrishabha (even).
        assert_eq!(Varga::D30.sign_of(edge - ARCSEC), odd_before, "odd before {edge}");
        assert_eq!(Varga::D30.sign_of(edge), odd_after, "odd at {edge}");
        assert_eq!(Varga::D30.sign_of(30.0 + edge - ARCSEC), even_before, "even before {edge}");
        assert_eq!(Varga::D30.sign_of(30.0 + edge), even_after, "even at {edge}");
    }
}

#[test]
fn moolatrikona_and_exaltation_zone_boundaries() {
    let z = |g, lon| dignity::rasi_zone(g, lon);
    // Moon: exalted [0,3) Vrishabha, MT [3,30).
    assert_eq!(z(Graha::Moon, 33.0 - ARCSEC), Some(Dignity::Exalted));
    assert_eq!(z(Graha::Moon, 33.0 + ARCSEC), Some(Dignity::Moolatrikona));
    // Mercury: exalted [0,15), MT [15,20), own [20,30) in Kanya.
    assert_eq!(z(Graha::Mercury, 165.0 - ARCSEC), Some(Dignity::Exalted));
    assert_eq!(z(Graha::Mercury, 165.0 + ARCSEC), Some(Dignity::Moolatrikona));
    assert_eq!(z(Graha::Mercury, 170.0 - ARCSEC), Some(Dignity::Moolatrikona));
    assert_eq!(z(Graha::Mercury, 170.0 + ARCSEC), Some(Dignity::OwnSign));
    // Sun: MT [0,20), own [20,30) in Simha.
    assert_eq!(z(Graha::Sun, 140.0 - ARCSEC), Some(Dignity::Moolatrikona));
    assert_eq!(z(Graha::Sun, 140.0 + ARCSEC), Some(Dignity::OwnSign));
    // Sign edges: leaving the exaltation sign.
    assert_eq!(z(Graha::Sun, 30.0 - ARCSEC), Some(Dignity::Exalted));
    assert_eq!(z(Graha::Sun, 30.0 + ARCSEC), None);
}

#[test]
fn baladi_boundaries_every_six_degrees() {
    use BaladiAvastha::*;
    let odd = [Bala, Kumara, Yuva, Vriddha, Mrita];
    for sign in 0..12 {
        for band in 0..5 {
            let lon = sign as f64 * 30.0 + band as f64 * 6.0;
            let want = if sign % 2 == 0 { odd[band] } else { odd[4 - band] };
            assert_eq!(condition::baladi(lon + ARCSEC), want, "sign {sign} band {band}");
            if band > 0 {
                let prev = if sign % 2 == 0 { odd[band - 1] } else { odd[5 - band] };
                assert_eq!(condition::baladi(lon - ARCSEC), prev, "sign {sign} below band {band}");
            }
        }
    }
}

#[test]
fn combustion_orb_boundaries_for_every_graha() {
    let sun = 100.0;
    for (g, retro, orb) in [
        (Graha::Moon, false, 12.0), (Graha::Mars, false, 17.0),
        (Graha::Mercury, false, 14.0), (Graha::Mercury, true, 12.0),
        (Graha::Jupiter, false, 11.0), (Graha::Venus, false, 10.0),
        (Graha::Venus, true, 8.0), (Graha::Saturn, false, 15.0),
    ] {
        for side in [1.0, -1.0] {
            assert!(condition::is_combust(g, sun + side * (orb - ARCSEC), retro, sun), "{} inside", g.name());
            assert!(condition::is_combust(g, sun + side * orb, retro, sun), "{} exactly at orb", g.name());
            assert!(!condition::is_combust(g, sun + side * (orb + ARCSEC), retro, sun), "{} outside", g.name());
        }
    }
}

// ===========================================================================
// Section 4 - dignity invariants over real charts
// ===========================================================================

#[test]
fn dignity_is_total_for_the_seven_and_absent_for_the_nodes() {
    let s = DerivationSettings::default();
    for c in random_charts(300, 0x0D16_0001) {
        for v in Varga::ALL {
            for g in Graha::ALL {
                let d = c.dignity(g, v, &s);
                assert_eq!(d.is_none(), g.is_chhaya(), "{} in {}", g.name(), v.label());
            }
        }
    }
}

#[test]
fn varga_dignity_agrees_with_the_occupied_sign() {
    let s = DerivationSettings::default();
    for c in random_charts(300, 0x0D16_0002) {
        for v in Varga::ALL {
            let vc = c.varga(v);
            for g in SEVEN {
                let sign = vc.sign_of(g);
                let d = c.dignity(g, v, &s).unwrap();
                // Exalted and debilitated can only occur in the matching sign.
                if d == Dignity::Debilitated {
                    assert_eq!(Some(sign), dignity::debilitation_sign(g), "{} {}", g.name(), v.label());
                }
                if d == Dignity::Exalted {
                    assert_eq!(Some(sign), dignity::exaltation_sign(g), "{} {}", g.name(), v.label());
                }
                // In its own sign a graha is never judged by relationship.
                if dignity::own_signs(g).contains(&sign) {
                    assert!(
                        matches!(d, Dignity::OwnSign | Dignity::Moolatrikona | Dignity::Exalted),
                        "{} in own sign {:?} got {d:?} in {}", g.name(), sign, v.label()
                    );
                }
            }
        }
    }
}

#[test]
fn switching_a_variant_only_changes_what_it_governs() {
    let base = DerivationSettings::default();
    let mt_own = DerivationSettings { varga_moolatrikona: VargaMoolatrikona::AsOwnSign, ..base };
    for c in random_charts(200, 0x0D16_0003) {
        for v in Varga::ALL {
            for g in SEVEN {
                let a = c.dignity(g, v, &base).unwrap();
                let b = c.dignity(g, v, &mt_own).unwrap();
                if a != b {
                    // V-4 may only turn Moolatrikona into OwnSign, only in vargas.
                    assert_ne!(v, Varga::D1, "V-4 changed a D-1 dignity");
                    assert_eq!((a, b), (Dignity::Moolatrikona, Dignity::OwnSign));
                }
            }
        }
        // V-5 never affects D-1.
        let same = DerivationSettings { temporary_source: TemporarySource::SameVarga, ..base };
        for g in SEVEN {
            assert_eq!(c.dignity(g, Varga::D1, &base), c.dignity(g, Varga::D1, &same));
        }
    }
}

// ===========================================================================
// Section 5 - drishti over real charts
// ===========================================================================

#[test]
fn grahas_aspecting_is_the_inverse_of_aspected_signs() {
    for nodes in [NodeAspects::None, NodeAspects::Seventh, NodeAspects::FiveSevenNine] {
        for c in random_charts(100, 0x0D16_0004) {
            for target in Rasi::ALL {
                let aspecting = c.grahas_aspecting(target, nodes);
                for g in Graha::ALL {
                    let forward = c.aspected_signs(g, nodes).contains(&target);
                    assert_eq!(aspecting.contains(&g), forward, "{} -> {:?}", g.name(), target);
                }
            }
        }
    }
}

#[test]
fn aspect_counts_per_graha_are_fixed() {
    for c in random_charts(100, 0x0D16_0005) {
        for g in Graha::ALL {
            let n = c.aspected_signs(g, NodeAspects::FiveSevenNine).len();
            let want = match g {
                Graha::Mars | Graha::Jupiter | Graha::Saturn | Graha::Rahu | Graha::Ketu => 3,
                _ => 1,
            };
            assert_eq!(n, want, "{}", g.name());
        }
    }
    assert!(drishti::rasi_aspected_by(Rasi::Mesha).iter().all(|r| r.mobility() == Mobility::Sthira));
}

// ===========================================================================
// Section 6 - conditions over real charts
// ===========================================================================

#[test]
fn analysis_conditions_are_internally_consistent() {
    let s = DerivationSettings::default();
    for c in random_charts(300, 0x0D16_0006) {
        let a = c.analyse(&s);
        assert_eq!(a.conditions.len(), 9);
        assert_eq!(a.relations.len(), 42);
        for k in &a.conditions {
            assert_eq!(k.dignity, c.dignity(k.graha, Varga::D1, &s));
            assert_eq!(k.jagradadi.is_some(), !k.graha.is_chhaya());
            if matches!(k.graha, Graha::Sun | Graha::Rahu | Graha::Ketu) {
                assert!(!k.combust, "{} can never be combust", k.graha.name());
            }
            // War is symmetric.
            for other in &k.at_war_with {
                let o = a.conditions.iter().find(|x| x.graha == *other).unwrap();
                assert!(o.at_war_with.contains(&k.graha));
            }
        }
        for (x, y, d) in &a.wars {
            assert!(*d <= condition::YUDDHA_ORB);
            assert!(condition::WAR_CAPABLE.contains(x) && condition::WAR_CAPABLE.contains(y));
        }
    }
}

// ===========================================================================
// Section 7 - Ashtakavarga invariants over real charts
// ===========================================================================

#[test]
fn ashtakavarga_totals_are_chart_independent() {
    for c in random_charts(500, 0x0D16_0007) {
        let av = Ashtakavarga::compute(&c);
        for (p, g) in SEVEN.iter().enumerate() {
            let total: u32 = av.bav[p].iter().map(|&x| x as u32).sum();
            assert_eq!(total, BAV_TOTALS[p], "{} BAV total", g.name());
        }
        assert_eq!(av.sav.iter().map(|&x| x as u32).sum::<u32>(), SAV_TOTAL);
        for s in 0..12 {
            let col: u8 = (0..7).map(|p| av.bav[p][s]).sum();
            assert_eq!(av.sav[s], col, "SAV column {s}");
            for p in 0..7 {
                assert!(av.bav[p][s] <= 8);
                assert_eq!(av.bav[p][s] as u32, av.prastara[p][s].count_ones());
            }
        }
    }
}

#[test]
fn prastara_bits_follow_the_table_exactly() {
    for c in random_charts(200, 0x0D16_0008) {
        let av = Ashtakavarga::compute(&c);
        for g in SEVEN {
            for contrib in Contributor::ALL {
                let from = match contrib {
                    Contributor::Lagna => c.lagna.rasi,
                    Contributor::Sun => c.placement(Graha::Sun).rasi,
                    Contributor::Moon => c.placement(Graha::Moon).rasi,
                    Contributor::Mars => c.placement(Graha::Mars).rasi,
                    Contributor::Mercury => c.placement(Graha::Mercury).rasi,
                    Contributor::Jupiter => c.placement(Graha::Jupiter).rasi,
                    Contributor::Venus => c.placement(Graha::Venus).rasi,
                    Contributor::Saturn => c.placement(Graha::Saturn).rasi,
                };
                let houses = benefic_houses(g, contrib).unwrap();
                for sign in Rasi::ALL {
                    let gave = av.contributors(g, sign).unwrap().contains(&contrib);
                    assert_eq!(gave, houses.contains(&from.houses_to(sign)), "{} {:?} {:?}", g.name(), contrib, sign);
                }
            }
        }
    }
}

#[test]
fn sav_by_house_is_sav_by_sign_rotated_to_the_lagna() {
    for c in random_charts(100, 0x0D16_0009) {
        let av = Ashtakavarga::compute(&c);
        for h in 1..=12u8 {
            assert_eq!(av.sav_in_house(h), av.sav[c.rasi_of_house(h).index() as usize]);
        }
        let by_house: u32 = (1..=12).map(|h| av.sav_in_house(h) as u32).sum();
        assert_eq!(by_house, SAV_TOTAL);
    }
}

// ===========================================================================
// Determinism and serialisation
// ===========================================================================

#[test]
fn phase2_outputs_are_deterministic_and_round_trip_through_json() {
    let s = DerivationSettings::default();
    for c in random_charts(50, 0x0D16_000A) {
        let a1 = c.analyse(&s);
        let av1 = Ashtakavarga::compute(&c);
        assert_eq!(c.analyse(&s), a1);
        assert_eq!(Ashtakavarga::compute(&c), av1);
        let a2: Analysis = serde_json::from_str(&serde_json::to_string(&a1).unwrap()).unwrap();
        let av2: Ashtakavarga = serde_json::from_str(&serde_json::to_string(&av1).unwrap()).unwrap();
        assert_eq!(a2, a1);
        assert_eq!(av2, av1);
    }
}

// ===========================================================================
// Phase 6A: transits
// ===========================================================================

#[test]
fn sign_stays_tile_the_range_and_step_one_sign_at_a_time() {
    init();
    let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
    let start = julian_day_ut(1950, 1, 1, 0.0, Calendar::Gregorian).unwrap();
    let end = start + 60.0 * 365.25;
    for g in lagn_core::transit::TRANSIT_GRAHAS {
        let stays = lagn_core::transit::sign_stays(&e, g, start, end).unwrap();
        assert_eq!(stays.first().unwrap().start_jd, start);
        assert_eq!(stays.last().unwrap().end_jd, end);
        for w in stays.windows(2) {
            assert_eq!(w[0].end_jd, w[1].start_jd, "{} gap", g.name());
            let d = (w[1].sign.index() as i32 - w[0].sign.index() as i32).rem_euclid(12);
            assert!(d == 1 || d == 11, "{} jumped {} signs", g.name(), d);
        }
    }
}

#[test]
fn gochara_windows_agree_with_their_house_from_the_moon() {
    for c in random_charts(20, 0x6A_0001) {
        let ws = c.gochara(c.jd_ut, c.jd_ut + 40.0 * 365.25).unwrap();
        for w in &ws {
            assert_eq!(w.house_from_moon, c.janma_rasi().houses_to(w.sign));
            assert_eq!(Some(w.kind), lagn_core::transit::classify(w.graha, w.house_from_moon));
            assert!(w.start_jd < w.end_jd);
            if matches!(w.graha, Graha::Saturn | Graha::Jupiter) {
                let b = Ashtakavarga::compute(&c).bindus(w.graha, w.sign).unwrap();
                assert_eq!(w.bindus, Some(b));
                assert_eq!(w.bav_supports, Some(b >= 4));
            } else {
                assert_eq!(w.bindus, None);
            }
        }
        // Sade sati phases appear in order 12 -> 1 -> 2 (retrograde can revisit).
        let sade: Vec<u8> = ws.iter().filter(|w| w.graha == Graha::Saturn && [12, 1, 2].contains(&w.house_from_moon)).map(|w| w.house_from_moon).collect();
        for pair in sade.windows(2) {
            assert!(matches!((pair[0], pair[1]), (12, 1) | (1, 2) | (1, 12) | (2, 1) | (12, 12) | (1, 1) | (2, 2)) || pair[0] == 2 && pair[1] == 12, "{pair:?}");
        }
    }
}

#[test]
fn neecha_bhanga_matches_each_condition_and_every_condition_is_exercised_alone() {
    // Added after mutation testing: removing the navamsa condition passed every
    // other test, because it is rarely the *only* cancellation. This test
    // recomputes all four conditions independently and requires each to be
    // seen as the sole reason at least once.
    let s = DerivationSettings::default();
    let mut sole = [0usize; 4];
    let mut debilitated = 0;
    for c in random_charts(4000, 0x6A_0002) {
        let moon = c.placement(Graha::Moon).rasi;
        let kendra = |g: Graha| {
            let r = c.placement(g).rasi;
            [1, 4, 7, 10].contains(&c.lagna.rasi.houses_to(r)) || [1, 4, 7, 10].contains(&moon.houses_to(r))
        };
        for g in SEVEN {
            let got = c.neecha_bhanga(g, &s).unwrap();
            if c.dignity(g, Varga::D1, &s) != Some(Dignity::Debilitated) {
                assert!(!got, "{} not debilitated but reported cancelled", g.name());
                continue;
            }
            debilitated += 1;
            let deb = dignity::debilitation_sign(g).unwrap();
            let ex = dignity::exaltation_sign(g).unwrap();
            let p = c.placement(g);
            let conds = [
                kendra(deb.lord()),
                kendra(ex.lord()),
                deb.lord() != g && {
                    let ls = c.placement(deb.lord()).rasi;
                    ls == p.rasi || drishti::graha_aspects(deb.lord(), ls, p.rasi, s.node_aspects)
                },
                Varga::D9.sign_of(p.longitude) == ex,
            ];
            assert_eq!(got, conds.iter().any(|&x| x), "{} at {}", g.name(), c.jd_ut);
            if conds.iter().filter(|&&x| x).count() == 1 {
                sole[conds.iter().position(|&x| x).unwrap()] += 1;
            }
        }
    }
    assert!(debilitated > 1000, "{debilitated}");
    for (i, n) in sole.iter().enumerate() {
        assert!(*n > 0, "condition {} never occurred as the only cancellation: sample too small", i + 1);
    }
}
