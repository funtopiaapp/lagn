//! Phase 3 QA suite. Specification: docs/phase3/DESIGN.md, acceptance
//! criteria 1, 2, 5, 6 and 7. The N-version evaluator (criterion 3) and the
//! exhaustive porutham check (criterion 4) live in scripts/.

use lagn_core::*;
use lagn_rules::model::*;
use lagn_rules::resolve::{evaluate_topic, Label, Mode};
use lagn_rules::{validate, Corpus, FactBase, NativeInfo, Truth};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn init() {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ephe").canonicalize().unwrap();
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

fn facts(n: usize, seed: u64, sex: Option<Sex>) -> Vec<FactBase> {
    init();
    let mut rng = Rng(seed | 1);
    let mut out = Vec::new();
    while out.len() < n {
        let b = BirthData {
            moment: BirthMoment {
                year: rng.int(1900, 2030) as i32, month: rng.int(1, 12) as u32, day: rng.int(1, 28) as u32,
                hour: rng.int(0, 23) as u32, minute: rng.int(0, 59) as u32, second: 0.0,
                utc_offset_hours: 5.5,
            },
            latitude: rng.unit() * 120.0 - 60.0,
            longitude: rng.unit() * 360.0 - 180.0,
            place_name: String::new(),
        };
        if let Ok(c) = Chart::compute(b, ChartSettings::default()) {
            out.push(FactBase::new(c, DerivationSettings::default(), NativeInfo { sex }));
        }
    }
    out
}

fn rule(id: &str, when: Condition, polarity: i8, overrides: &[&str], status: ReviewStatus) -> Rule {
    Rule {
        id: id.into(),
        topic: id.split('.').next().unwrap().into(),
        title: id.into(),
        tradition: Tradition::Parashari,
        when,
        scope: Scope::Natal,
        polarity,
        overrides: overrides.iter().map(|s| s.to_string()).collect(),
        timing: vec![],
        tags: vec![],
        subject: None,
        text: Text { en: "t".into(), impact: Some("what it means for you".into()), ta: None },
        source: Source { reference: None, note: "n".into() },
        review: Review {
            status,
            reviewer: (status == ReviewStatus::Approved).then(|| "QA".into()),
            date: (status == ReviewStatus::Approved).then(|| "2026-09-21".into()),
            comment: None,
        },
    }
}

const T: Condition = Condition::Always {};
fn f_() -> Condition { Condition::Not(Box::new(Condition::Always {})) }
fn u_() -> Condition { Condition::Native { sex: Sex::Female } }  // Unknown when sex is None

fn eval(c: &Condition, fb: &FactBase) -> Truth {
    let mut t = Vec::new();
    lagn_rules::eval(c, fb, &mut t)
}

// ===========================================================================
// Criterion 1: Kleene logic, exhaustively
// ===========================================================================

const ALL3: [Truth; 3] = [Truth::True, Truth::False, Truth::Unknown];

fn kleene_and(a: Truth, b: Truth) -> Truth {
    use Truth::*;
    match (a, b) { (False, _) | (_, False) => False, (True, True) => True, _ => Unknown }
}
fn kleene_or(a: Truth, b: Truth) -> Truth {
    use Truth::*;
    match (a, b) { (True, _) | (_, True) => True, (False, False) => False, _ => Unknown }
}

#[test]
fn kleene_and_or_match_the_truth_tables_for_every_sequence_up_to_length_four() {
    // Every sequence of length 0 to 4, built layer by layer so none repeats.
    let mut layer: Vec<Vec<Truth>> = vec![vec![]];
    let mut seqs = layer.clone();
    for _ in 0..4 {
        layer = layer.iter().flat_map(|s| ALL3.iter().map(move |&t| {
            let mut v = s.clone(); v.push(t); v
        })).collect();
        seqs.extend(layer.iter().cloned());
    }
    assert_eq!(seqs.len(), 1 + 3 + 9 + 27 + 81);
    for s in &seqs {
        let want_all = s.iter().fold(Truth::True, |acc, &t| kleene_and(acc, t));
        let want_any = s.iter().fold(Truth::False, |acc, &t| kleene_or(acc, t));
        assert_eq!(Truth::all(s), want_all, "all{s:?}");
        assert_eq!(Truth::any(s), want_any, "any{s:?}");
    }
}

#[test]
fn kleene_not_and_de_morgan() {
    assert_eq!(!Truth::True, Truth::False);
    assert_eq!(!Truth::False, Truth::True);
    assert_eq!(!Truth::Unknown, Truth::Unknown);
    for a in ALL3 {
        assert_eq!(!!a, a);
        for b in ALL3 {
            assert_eq!(!Truth::all(&[a, b]), Truth::any(&[!a, !b]), "De Morgan AND {a:?} {b:?}");
            assert_eq!(!Truth::any(&[a, b]), Truth::all(&[!a, !b]), "De Morgan OR {a:?} {b:?}");
        }
    }
}

#[test]
fn evaluator_combinators_follow_kleene_on_real_charts() {
    let fb = &facts(1, 1, None)[0];
    let lit = |t: Truth| match t { Truth::True => T, Truth::False => f_(), Truth::Unknown => u_() };
    for a in ALL3 {
        assert_eq!(eval(&lit(a), fb), a, "literal {a:?}");
        assert_eq!(eval(&Condition::Not(Box::new(lit(a))), fb), !a);
        for b in ALL3 {
            assert_eq!(eval(&Condition::All(vec![lit(a), lit(b)]), fb), Truth::all(&[a, b]));
            assert_eq!(eval(&Condition::Any(vec![lit(a), lit(b)]), fb), Truth::any(&[a, b]));
        }
    }
}

#[test]
fn every_child_is_traced_even_when_the_result_is_already_decided() {
    let fb = &facts(1, 2, None)[0];
    let mut t = Vec::new();
    let c = Condition::All(vec![f_(), T, u_(), T]);
    assert_eq!(lagn_rules::eval(&c, fb, &mut t), Truth::False);
    assert_eq!(t.len(), 4, "no short-circuit: all four atoms must appear in the trace");
}

// ===========================================================================
// Criterion 2: every atomic condition against lagn-core computed directly
// ===========================================================================

#[test]
fn atomic_conditions_agree_with_lagn_core() {
    let settings = DerivationSettings::default();
    for fb in facts(150, 0xA70_0001, Some(Sex::Male)) {
        let c = &fb.chart;
        let av = Ashtakavarga::compute(c);
        for g in Graha::ALL {
            let p = c.placement(g);
            let r = GrahaRef::Named(g);
            for h in 1..=12u8 {
                assert_eq!(eval(&Condition::GrahaInHouse { graha: r, houses: vec![h], varga: Varga::D1 }, &fb),
                           Truth::from_bool(p.house == h));
            }
            for v in [Varga::D9, Varga::D10, Varga::D30, Varga::D60] {
                let vc = c.varga(v);
                assert_eq!(eval(&Condition::GrahaInHouse { graha: r, houses: vec![vc.house_of(g)], varga: v }, &fb), Truth::True);
                assert_eq!(eval(&Condition::GrahaInSign { graha: r, signs: vec![vc.sign_of(g)], varga: v }, &fb), Truth::True);
                let d = c.dignity(g, v, &settings);
                let cond = Condition::Dignity { graha: r, any_of: vec![Dignity::Exalted], varga: v };
                assert_eq!(eval(&cond, &fb), match d { None => Truth::Unknown, Some(x) => Truth::from_bool(x == Dignity::Exalted) });
            }
            assert_eq!(eval(&Condition::Retrograde { graha: r }, &fb), Truth::from_bool(p.retrograde));
            let sun = c.placement(Graha::Sun).longitude;
            assert_eq!(eval(&Condition::Combust { graha: r }, &fb),
                       Truth::from_bool(lagn_core::condition::is_combust(g, p.longitude, p.retrograde, sun)));
            // graha_from the Moon.
            let moon = c.placement(Graha::Moon).rasi;
            let h = moon.houses_to(p.rasi);
            assert_eq!(eval(&Condition::GrahaFrom { graha: r, from: FromRef::Graha(GrahaRef::Named(Graha::Moon)), houses: vec![h] }, &fb), Truth::True);
            // aspects_graha, against the Phase 2 aspect list.
            for t in Graha::ALL {
                let want = c.aspected_signs(g, settings.node_aspects).contains(&c.placement(t).rasi);
                assert_eq!(eval(&Condition::AspectsGraha { from: r, to: GrahaRef::Named(t) }, &fb), Truth::from_bool(want));
            }
        }
        // lord_of resolution, D-1 and D-9.
        for h in 1..=12u8 {
            assert_eq!(fb.resolve(GrahaRef::LordOf { house: h, varga: Varga::D1 }), Some(c.lord_of_house(h)));
            let d9 = c.varga(Varga::D9);
            let want = Rasi::from_index(d9.lagna.index() as i32 + h as i32 - 1).lord();
            assert_eq!(fb.resolve(GrahaRef::LordOf { house: h, varga: Varga::D9 }), Some(want));
            assert_eq!(eval(&Condition::Sav { house: h, min: Some(av.sav_in_house(h)), max: Some(av.sav_in_house(h)) }, &fb), Truth::True);
            assert_eq!(eval(&Condition::HouseOccupied { house: h, by: GrahaSet::Named(SetName::Any), min: 1, varga: Varga::D1 }, &fb),
                       Truth::from_bool(!c.grahas_in_house(h).is_empty()));
            let target = c.rasi_of_house(h);
            assert_eq!(eval(&Condition::HouseAspected { house: h, by: GrahaSet::Named(SetName::Any), min: 1 }, &fb),
                       Truth::from_bool(!c.grahas_aspecting(target, settings.node_aspects).is_empty()));
        }
        assert_eq!(eval(&Condition::Native { sex: Sex::Male }, &fb), Truth::True);
        assert_eq!(eval(&Condition::Native { sex: Sex::Female }, &fb), Truth::False);
    }
}

#[test]
fn benefic_classification_partitions_the_nine_and_tracks_the_moons_phase() {
    for fb in facts(200, 0xA70_0002, None) {
        let b = fb.members(&GrahaSet::Named(SetName::Benefics));
        let m = fb.members(&GrahaSet::Named(SetName::Malefics));
        assert_eq!(b.len() + m.len(), 9);
        assert!(b.iter().all(|g| !m.contains(g)));
        let elong = (fb.chart.placement(Graha::Moon).longitude - fb.chart.placement(Graha::Sun).longitude).rem_euclid(360.0);
        assert_eq!(b.contains(&Graha::Moon), elong < 180.0);
        for g in [Graha::Jupiter, Graha::Venus, Graha::Mercury] { assert!(b.contains(&g)); }
        for g in [Graha::Sun, Graha::Mars, Graha::Saturn, Graha::Rahu, Graha::Ketu] { assert!(m.contains(&g)); }
    }
}

#[test]
fn conjunct_with_itself_is_false_even_via_a_lord_reference() {
    for fb in facts(80, 0xA70_0003, None) {
        let l7 = GrahaRef::LordOf { house: 7, varga: Varga::D1 };
        let same = GrahaRef::Named(fb.resolve(l7).unwrap());
        assert_eq!(eval(&Condition::Conjunct { a: l7, b: same }, &fb), Truth::False);
    }
}

#[test]
fn missing_native_sex_is_unknown_never_false() {
    let fb = &facts(1, 4, None)[0];
    assert_eq!(eval(&Condition::Native { sex: Sex::Female }, fb), Truth::Unknown);
    assert_eq!(eval(&Condition::Native { sex: Sex::Male }, fb), Truth::Unknown);
    // Nodes have no dignity: Unknown, not False.
    let c = Condition::Dignity { graha: GrahaRef::Named(Graha::Rahu), any_of: vec![Dignity::Exalted], varga: Varga::D1 };
    assert_eq!(eval(&c, fb), Truth::Unknown);
}

// ===========================================================================
// Criteria 5 and 6: gate, cancellation, order independence
// ===========================================================================

fn report(rules: &[Rule], mode: Mode) -> lagn_rules::TopicReport {
    let fb = &facts(1, 5, None)[0];
    evaluate_topic("t", rules, fb, mode, (0.0, 100.0))
}

#[test]
fn production_mode_never_admits_draft_or_rejected_content() {
    use ReviewStatus::*;
    let mut rng = Rng(77);
    for _ in 0..300 {
        let statuses = [Draft, Approved, Rejected];
        let rules: Vec<Rule> = (0..12).map(|i| {
            rule(&format!("t.r{i:02}"), T, rng.int(-3, 3) as i8, &[], statuses[rng.int(0, 2) as usize])
        }).collect();
        let prod = report(&rules, Mode::Production);
        assert!(prod.results.iter().all(|r| r.status == Approved), "gate leak");
        let rev = report(&rules, Mode::Review);
        assert!(rev.results.iter().all(|r| r.status != Rejected), "rejected rule evaluated");
        let n = |s| rules.iter().filter(|r| r.review.status == s).count();
        assert_eq!(prod.results.len(), n(Approved));
        assert_eq!(rev.results.len(), n(Approved) + n(Draft));
        assert_eq!(prod.withheld.values().sum::<usize>(), n(Draft) + n(Rejected));
    }
}

#[test]
fn cancellation_semantics() {
    use ReviewStatus::Approved as A;
    // a fires; b cancels a; c cancels b -> a is effective again.
    let rules = vec![
        rule("t.a", T, -2, &[], A),
        rule("t.b", T, 0, &["t.a"], A),
        rule("t.c", T, 0, &["t.b"], A),
    ];
    let r = report(&rules, Mode::Production);
    let get = |id: &str| r.results.iter().find(|x| x.id == id).unwrap().clone();
    assert!(get("t.a").effective, "cancel-of-cancel should restore t.a");
    assert!(!get("t.b").effective);
    assert_eq!(get("t.b").cancelled_by.as_deref(), Some("t.c"));
    assert_eq!(r.score, -2);

    // A canceller that does not fire cancels nothing.
    let rules = vec![rule("t.a", T, -2, &[], A), rule("t.b", f_(), 0, &["t.a"], A)];
    let r = report(&rules, Mode::Production);
    assert!(r.results[0].effective);

    // Two effective cancellers: the smaller id is reported (DESIGN 6, rev 2).
    let rules = vec![rule("t.a", T, -2, &[], A), rule("t.z", T, 0, &["t.a"], A), rule("t.m", T, 0, &["t.a"], A)];
    let r = report(&rules, Mode::Production);
    assert_eq!(r.results.iter().find(|x| x.id == "t.a").unwrap().cancelled_by.as_deref(), Some("t.m"));

    // A draft canceller is invisible in production, so the target stands.
    let rules = vec![rule("t.a", T, -2, &[], A), rule("t.b", T, 0, &["t.a"], ReviewStatus::Draft)];
    assert!(report(&rules, Mode::Production).results[0].effective);
    assert!(!report(&rules, Mode::Review).results.iter().find(|x| x.id == "t.a").unwrap().effective);
}

#[test]
fn labels_follow_the_polarity_signs() {
    use ReviewStatus::Approved as A;
    let lbl = |ps: &[i8]| {
        let rules: Vec<Rule> = ps.iter().enumerate().map(|(i, &p)| rule(&format!("t.r{i}"), T, p, &[], A)).collect();
        report(&rules, Mode::Production).label
    };
    assert_eq!(lbl(&[2, 1]), Label::Supportive);
    assert_eq!(lbl(&[-2]), Label::Afflicted);
    assert_eq!(lbl(&[3, -3]), Label::Mixed, "a zero score with both sides present is mixed, not neutral");
    assert_eq!(lbl(&[0, 0]), Label::Neutral);
    assert_eq!(lbl(&[]), Label::Neutral);
}

#[test]
fn results_do_not_depend_on_rule_order() {
    use ReviewStatus::*;
    let mut rng = Rng(99);
    for _ in 0..100 {
        let mut rules: Vec<Rule> = (0..10).map(|i| {
            let overrides: Vec<String> = (0..i).filter(|_| rng.int(0, 4) == 0).map(|j| format!("t.r{j:02}")).collect();
            let when = if rng.int(0, 2) == 0 { f_() } else { T };
            let mut r = rule(&format!("t.r{i:02}"), when, rng.int(-3, 3) as i8, &[], [Draft, Approved][rng.int(0, 1) as usize]);
            r.overrides = overrides;
            r
        }).collect();
        let base = report(&rules, Mode::Review);
        for _ in 0..5 {
            // Fisher-Yates with the deterministic RNG.
            for i in (1..rules.len()).rev() {
                let j = rng.int(0, i as i64) as usize;
                rules.swap(i, j);
            }
            assert_eq!(report(&rules, Mode::Review), base, "report changed when rules were reordered");
        }
    }
}

// ===========================================================================
// Criterion 7: validator negative tests, one per validation rule
// ===========================================================================

fn problems(rules: Vec<Rule>) -> Vec<String> {
    validate(&Corpus { rules, ..Default::default() })
}

fn assert_rejects(rules: Vec<Rule>, needle: &str) {
    let p = problems(rules);
    assert!(p.iter().any(|x| x.contains(needle)), "expected a problem containing {needle:?}, got {p:?}");
}

#[test]
fn a_valid_corpus_has_no_problems() {
    assert!(problems(vec![rule("t.a", T, 1, &[], ReviewStatus::Draft)]).is_empty());
}

#[test]
fn validator_rejects_each_class_of_error() {
    use ReviewStatus::*;
    assert_rejects(vec![rule("t.a", T, 1, &[], Draft), rule("t.a", T, 1, &[], Draft)], "duplicate rule id");
    let mut r = rule("t.a", T, 1, &[], Draft); r.topic = "other".into();
    assert_rejects(vec![r], "must start with its topic");
    assert_rejects(vec![rule("t.a", T, 4, &[], Draft)], "polarity");
    assert_rejects(vec![rule("t.a", T, -4, &[], Draft)], "polarity");
    let mut r = rule("t.a", T, 1, &[], Draft); r.text.en = " ".into();
    assert_rejects(vec![r], "title and text.en");
    let mut r = rule("t.a", T, 1, &[], Draft); r.source.note = "".into();
    assert_rejects(vec![r], "source.note");
    // Every rule must say what it means for the reader (DESIGN section 4b).
    let mut r = rule("t.a", T, 1, &[], Draft); r.text.impact = None;
    assert_rejects(vec![r], "text.impact is required");
    let mut r = rule("t.a", T, 1, &[], Draft); r.text.impact = Some("too short".into());
    assert_rejects(vec![r], "too short");
    let mut r = rule("t.a", T, 1, &[], Draft);
    r.text.en = "the same sentence repeated in both fields".into();
    r.text.impact = Some(r.text.en.clone());
    assert_rejects(vec![r], "must say something the technical text does not");
    let mut r = rule("t.a", T, 1, &[], Approved); r.review.reviewer = None;
    assert_rejects(vec![r], "without a reviewer");
    let mut r = rule("t.a", T, 1, &[], Approved); r.review.date = None;
    assert_rejects(vec![r], "without a date");
    let mut r = rule("t.a", T, 1, &[], Approved); r.review.date = Some("21/09/2026".into());
    assert_rejects(vec![r], "not YYYY-MM-DD");
    assert_rejects(vec![rule("t.a", Condition::All(vec![]), 1, &[], Draft)], "empty all/any");
    let gi = |hs: Vec<u8>| Condition::GrahaInHouse { graha: GrahaRef::Named(Graha::Sun), houses: hs, varga: Varga::D1 };
    assert_rejects(vec![rule("t.a", gi(vec![]), 1, &[], Draft)], "empty house list");
    assert_rejects(vec![rule("t.a", gi(vec![13]), 1, &[], Draft)], "not 1-12");
    assert_rejects(vec![rule("t.a", gi(vec![0]), 1, &[], Draft)], "not 1-12");
    assert_rejects(vec![rule("t.a", gi(vec![7, 7]), 1, &[], Draft)], "twice");
    let lord13 = Condition::Combust { graha: GrahaRef::LordOf { house: 13, varga: Varga::D1 } };
    assert_rejects(vec![rule("t.a", lord13, 1, &[], Draft)], "lord_of house 13");
    let occ = |min| Condition::HouseOccupied { house: 7, by: GrahaSet::Named(SetName::Any), min, varga: Varga::D1 };
    assert_rejects(vec![rule("t.a", occ(0), 1, &[], Draft)], "min 0");
    assert_rejects(vec![rule("t.a", occ(10), 1, &[], Draft)], "min 10");
    let empty_set = Condition::HouseAspected { house: 7, by: GrahaSet::Explicit(ExplicitSet { grahas: vec![] }), min: 1 };
    assert_rejects(vec![rule("t.a", empty_set, 1, &[], Draft)], "empty graha set");
    assert_rejects(vec![rule("t.a", Condition::Sav { house: 7, min: None, max: None }, 1, &[], Draft)], "needs min or max");
    assert_rejects(vec![rule("t.a", Condition::Sav { house: 7, min: Some(30), max: Some(20) }, 1, &[], Draft)], "min 30 > max 20");
    assert_rejects(vec![rule("t.a", Condition::Sav { house: 7, min: Some(57), max: None }, 1, &[], Draft)], "exceeds 56");
    assert_rejects(vec![rule("t.a", Condition::GrahaInSign { graha: GrahaRef::Named(Graha::Sun), signs: vec![], varga: Varga::D1 }, 1, &[], Draft)], "empty sign list");
    assert_rejects(vec![rule("t.a", Condition::Dignity { graha: GrahaRef::Named(Graha::Sun), any_of: vec![], varga: Varga::D1 }, 1, &[], Draft)], "dignity has an empty list");
    // Overrides.
    assert_rejects(vec![rule("t.a", T, 1, &["t.nope"], Draft)], "unknown rule");
    assert_rejects(vec![rule("t.a", T, 1, &["t.a"], Draft)], "overrides itself");
    let other = { let mut r = rule("u.b", T, 1, &[], Draft); r.topic = "u".into(); r };
    assert_rejects(vec![rule("t.a", T, 1, &["u.b"], Draft), other], "different topic");
    assert_rejects(vec![rule("t.a", T, 1, &["t.b"], Draft), rule("t.b", T, 1, &["t.c"], Draft), rule("t.c", T, 1, &["t.a"], Draft)], "override cycle");
}

#[test]
fn corpus_json_typos_are_load_errors_not_ignored_fields() {
    let bad = r#"[{"id":"t.a","topic":"t","title":"x","tradition":"parashari",
        "when":{"graha_in_house":{"graha":"sun","house":[7]}},
        "polarity":1,"text":{"en":"x"},"source":{"reference":null,"note":"n"},
        "review":{"status":"draft"}}]"#;
    assert!(serde_json::from_str::<Vec<Rule>>(bad).is_err(), "misspelt 'house' was accepted");
    let bad_graha = bad.replace(r#""house":[7]"#, r#""houses":[7]"#).replace(r#""sun""#, r#""pluto""#);
    assert!(serde_json::from_str::<Vec<Rule>>(&bad_graha).is_err(), "unknown graha accepted");
    let good = bad.replace(r#""house":[7]"#, r#""houses":[7]"#);
    assert!(serde_json::from_str::<Vec<Rule>>(&good).is_ok());
}

#[test]
fn the_shipped_corpus_has_a_complete_review_record() {
    // Every rule has been reviewed (by the AI review the product owner
    // authorised on 2026-09-21; see docs/phase3/REVIEW-DECISIONS.md). No draft
    // may remain unreviewed in the shipped corpus, and every decision must
    // carry a reviewer, a date and its reasoning, so it can be re-reviewed.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let c = lagn_rules::load(&dir).expect("shipped corpus must validate");
    assert!(!c.rules.is_empty());
    for r in &c.rules {
        assert_ne!(r.review.status, ReviewStatus::Draft, "{} was never reviewed", r.id);
        assert!(r.review.reviewer.as_deref().is_some_and(|x| !x.trim().is_empty()), "{}: no reviewer", r.id);
        assert!(r.review.date.is_some(), "{}: no review date", r.id);
        assert!(r.review.comment.as_deref().is_some_and(|x| x.len() > 10), "{}: no review reasoning", r.id);
        // Still true: no citation has been checked against a text.
        assert!(r.source.reference.is_none(), "{}: a citation may only be added once verified", r.id);
    }
    let p = c.porutham.expect("porutham review file");
    assert_eq!(p.len(), 10);
    for (k, r) in &p {
        assert_ne!(r.status, ReviewStatus::Draft, "{k:?} never reviewed");
        assert!(r.comment.as_deref().is_some_and(|x| !x.is_empty()), "{k:?}: no review reasoning");
    }
    // Vasya has no table, so approving it would show a verdict the engine cannot compute.
    assert_ne!(p[&lagn_rules::PoruthamKind::Vasya].status, ReviewStatus::Approved);
}

#[test]
fn timing_windows_stay_inside_the_range_and_match_their_lords() {
    use ReviewStatus::Approved as A;
    let mut r = rule("t.a", T, 0, &[], A);
    r.timing = vec![GrahaRef::Named(Graha::Venus), GrahaRef::LordOf { house: 7, varga: Varga::D1 }];
    for fb in facts(40, 0xA70_0004, None) {
        let rep = evaluate_topic("t", std::slice::from_ref(&r), &fb, Mode::Production, (20.0, 40.0));
        let dpy = fb.chart.settings.year_length.days();
        let (lo, hi) = (fb.dasha.birth_jd + 20.0 * dpy, fb.dasha.birth_jd + 40.0 * dpy);
        let lords = [Graha::Venus, fb.resolve(GrahaRef::LordOf { house: 7, varga: Varga::D1 }).unwrap()];
        assert!(!rep.timing.is_empty());
        for w in &rep.timing {
            assert!(w.start_jd >= lo && w.end_jd <= hi && w.start_jd < w.end_jd);
            assert!(lords.contains(&w.maha) || lords.contains(&w.antar));
            assert!(!w.matched.is_empty());
        }
        for pair in rep.timing.windows(2) {
            assert!(pair[0].end_jd <= pair[1].start_jd + 1e-9, "windows overlap or are out of order");
        }
    }
}

// ===========================================================================
// Phase 6A
// ===========================================================================

#[test]
fn period_refs_are_only_valid_in_period_rules() {
    use ReviewStatus::Draft;
    let period_ref = Condition::Combust { graha: GrahaRef::Period(PeriodLevel::Maha) };
    let r = rule("t.a", period_ref.clone(), -1, &[], Draft);
    assert!(problems(vec![r]).iter().any(|p| p.contains("not a period rule")));
    let mut ok = rule("t.b", period_ref, -1, &[], Draft);
    ok.scope = Scope::Period;
    assert!(problems(vec![ok.clone()]).is_empty());
    let mut timed = ok;
    timed.timing = vec![GrahaRef::Named(Graha::Sun)];
    assert!(problems(vec![timed]).iter().any(|p| p.contains("already timed")));
}

#[test]
fn period_rules_never_run_in_natal_topics_and_natal_rules_never_run_per_period() {
    use ReviewStatus::Approved as A;
    let mut p = rule("t.p", Condition::Always {}, -1, &[], A);
    p.scope = Scope::Period;
    let n = rule("t.n", Condition::Always {}, 1, &[], A);
    let mut fb = facts(1, 11, None).pop().unwrap();
    let topic = evaluate_topic("t", &[p.clone(), n.clone()], &fb, Mode::Production, (0.0, 100.0));
    assert_eq!(topic.results.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["t.n"]);
    let per = lagn_rules::evaluate_periods(&[p, n], &mut fb, Mode::Production, (20.0, 30.0));
    assert!(!per.windows.is_empty());
    for w in &per.windows {
        assert_eq!(w.amplifiers.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["t.p"]);
        assert!(w.negators.is_empty(), "a natal rule ran per period");
    }
    assert!(fb.period.is_none(), "period context must be cleared after evaluation");
}

#[test]
fn period_lords_are_bound_to_each_window() {
    use lagn_core::FunctionalNature::*;
    use ReviewStatus::Approved as A;
    // Every graha except the nodes has a functional nature, so this fires in
    // exactly the windows whose antardasha lord is not Rahu or Ketu.
    let mut r = rule("t.antar_has_nature", Condition::Always {}, -1, &[], A);
    r.scope = Scope::Period;
    r.when = Condition::Functional { graha: GrahaRef::Period(PeriodLevel::Antar), is: vec![Yogakaraka, Benefic, Malefic, Neutral] };
    let mut fb = facts(1, 12, None).pop().unwrap();
    let per = lagn_rules::evaluate_periods(&[r], &mut fb, Mode::Production, (0.0, 60.0));
    assert!(per.windows.len() > 20);
    for w in &per.windows {
        assert_eq!(!w.amplifiers.is_empty(), !w.antar.is_chhaya(), "{:?}/{:?}", w.maha, w.antar);
    }
}

#[test]
fn functional_nature_exhaustively_matches_an_independent_derivation() {
    use lagn_core::FunctionalNature::*;
    // Independent: count each graha's houses by walking the zodiac from the lagna.
    let lords = [Graha::Mars, Graha::Venus, Graha::Mercury, Graha::Moon, Graha::Sun, Graha::Mercury,
                 Graha::Venus, Graha::Mars, Graha::Jupiter, Graha::Saturn, Graha::Saturn, Graha::Jupiter];
    for l in 0..12usize {
        for g in [Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury, Graha::Jupiter, Graha::Venus, Graha::Saturn] {
            let hs: Vec<usize> = (1..=12).filter(|h| lords[(l + h - 1) % 12] == g).collect();
            let want = if hs.iter().any(|h| [4, 7, 10].contains(h)) && hs.iter().any(|h| [5, 9].contains(h)) { Yogakaraka }
                else if hs.iter().any(|h| [1, 5, 9].contains(h)) { Benefic }
                else if hs.iter().any(|h| [3, 6, 8, 11, 12].contains(h)) { Malefic }
                else { Neutral };
            assert_eq!(lagn_core::functional::functional_nature(Rasi::from_index(l as i32), g), Some(want), "lagna {l}, {}", g.name());
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 6 content guardrails (docs/phase6/DESIGN.md section 2)
// ---------------------------------------------------------------------------

/// Words that must never reach a user: lifespan or death predictions, gemstone
/// prescriptions, payment, fatalistic statements about children, and claims of
/// certainty or cure. Matched as whole words or phrases, case-insensitively.
const BANNED: &[&str] = &[
    "death", "die", "dies", "died", "dying", "demise", "fatal", "lifespan", "life span",
    "life expectancy", "years to live", "widow", "widower",
    "gem", "gems", "gemstone", "gemstones", "ruby", "emerald", "sapphire", "pearl", "coral",
    "hessonite", "cat's eye",
    "childless", "barren", "infertile", "infertility", "sterile", "no children",
    "denial of children", "denied children",
    "cure", "cures", "guarantee", "guaranteed", "certainly will",
    "danger", "dangerous", "loss", "losses", "disaster", "accident", "accidents",
    "fee", "fees", "payment", "pay", "rupees", "price", "₹",
];

fn banned_in(text: &str) -> Vec<&'static str> {
    let t = text.to_lowercase();
    let is_word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric());
    BANNED
        .iter()
        .copied()
        .filter(|w| {
            // A symbol such as the rupee sign is banned wherever it appears.
            let symbol = !w.chars().any(char::is_alphabetic);
            t.match_indices(w).any(|(i, _)| {
                symbol || (!is_word(t[..i].chars().next_back()) && !is_word(t[i + w.len()..].chars().next()))
            })
        })
        .collect()
}

#[test]
fn the_banned_word_matcher_matches_whole_words_only() {
    assert_eq!(banned_in("A risk of Death here"), vec!["death"]);
    assert_eq!(banned_in("wear a ruby"), vec!["ruby"]);
    assert!(banned_in("studied diet and payable items; gemini; pearly; accurate; secure").is_empty());
    assert_eq!(banned_in("costs ₹500"), vec!["₹"]);
}

#[test]
fn no_user_visible_corpus_text_uses_banned_wording() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let c = lagn_rules::load(&dir).expect("shipped corpus must validate");
    // Every field a user can see. Provenance notes and review reasoning are for
    // reviewers and may name what the content avoids.
    let mut visible: Vec<(String, String)> = Vec::new();
    for r in &c.rules {
        visible.push((format!("{} title", r.id), r.title.clone()));
        visible.push((format!("{} text", r.id), r.text.en.clone()));
    }
    for t in &c.topics {
        visible.push((format!("topic {} title", t.id), t.title.clone()));
        visible.push((format!("topic {} summary", t.id), t.summary.clone()));
    }
    for b in &c.bhavas.as_ref().expect("bhava.json").houses {
        visible.push((format!("bhava {}", b.house), format!("{} {}", b.name, b.significations.join("; "))));
    }
    assert!(!c.pariharams.is_empty());
    for p in &c.pariharams {
        let mut all = vec![p.title.clone()];
        all.extend(p.deity.clone());
        all.extend(p.day.clone());
        all.extend(p.practices.clone());
        all.extend(p.places.clone());
        all.extend(p.charity.clone());
        visible.push((format!("pariharam {}", p.id), all.join(" | ")));
    }
    let bad: Vec<String> = visible
        .iter()
        .filter_map(|(k, v)| {
            let b = banned_in(v);
            (!b.is_empty()).then(|| format!("{k}: {b:?} in {v:?}"))
        })
        .collect();
    assert!(bad.is_empty(), "banned wording:\n{}", bad.join("\n"));
}

#[test]
fn user_facing_rule_text_has_no_internal_jargon() {
    // Notes for reviewers belong in review comments and provenance, not in
    // what a reader sees.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let c = lagn_rules::load(&dir).unwrap();
    let jargon = ["review", "reviewer", "variant", "corpus", "engine", "threshold", "draft", "polarity"];
    for r in &c.rules {
        let t = format!("{} {}", r.title, r.text.en).to_lowercase();
        for j in jargon {
            assert!(!t.split(|ch: char| !ch.is_alphanumeric()).any(|w| w == j), "{}: internal word {j:?} in user text", r.id);
        }
    }
}

#[test]
fn health_and_vitality_carry_the_required_disclaimers() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let c = lagn_rules::load(&dir).unwrap();
    for id in ["health", "vitality"] {
        let t = c.topics.iter().find(|t| t.id == id).unwrap_or_else(|| panic!("topic {id} missing"));
        let d = t.disclaimer.as_deref().unwrap_or_else(|| panic!("{id}: no disclaimer")).to_lowercase();
        assert!(d.contains("doctor"), "{id}: disclaimer must point to a doctor");
        assert!(d.contains("not a medical"), "{id}: disclaimer must say it is not medical");
    }
    let v = c.topics.iter().find(|t| t.id == "vitality").unwrap();
    assert!(v.disclaimer.as_deref().unwrap().contains("never estimates a lifespan"));
    // The vitality topic has no rule that could be read as a time of death:
    // its timing rules are framed as care periods.
    for r in c.rules.iter().filter(|r| r.topic == "vitality" && !r.timing.is_empty()) {
        assert!(r.text.en.to_lowercase().contains("care"), "{}: timing must be framed as care", r.id);
    }
}

#[test]
fn every_pariharam_is_reachable_and_every_graha_has_one() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let c = lagn_rules::load(&dir).unwrap();
    for g in lagn_core::Graha::ALL {
        let tag = format!("graha:{}", serde_json::to_value(g).unwrap().as_str().unwrap());
        assert!(c.pariharams.iter().any(|p| p.triggers.contains(&tag)), "no pariharam for {tag}");
    }
    // A dosha pariharam must be triggerable by some shipped rule's tag.
    for p in &c.pariharams {
        for t in p.triggers.iter().filter(|t| t.starts_with("dosha:")) {
            assert!(c.rules.iter().any(|r| r.tags.contains(t)), "{}: no rule carries {t}", p.id);
        }
    }
    // Every negative, effective-capable rule names what it afflicts, so a
    // pariharam can be suggested for it, unless it is a timing or house rule.
    let unnamed: Vec<&str> = c
        .rules
        .iter()
        .filter(|r| r.polarity < 0 && r.review.status == ReviewStatus::Approved && r.subject.is_none() && !r.tags.iter().any(|t| t.starts_with("dosha:")))
        .map(|r| r.id.as_str())
        .collect();
    // These describe a house (any of several grahas may occupy it) or the
    // relation between two period lords, so no single graha's pariharam fits.
    // A new negative rule must name its subject or be added here deliberately.
    let expected = [
        "education.h4.malefic", "education.h5.malefic", "health.h1.malefic", "later_life.h12.expenses",
        "marriage.h7.malefic_occupant", "marriage.sav.h7_weak", "parents.mother.malefic",
        "parents.father.malefic", "past_life.h5.malefic", "past_life.h12.unfinished",
        "periods.shashtashtaka", "vitality.h8.malefic",
    ];
    assert_eq!(unnamed, expected, "negative rules without a subject changed");
}

#[test]
fn generated_period_explanations_obey_the_same_wording_guardrails() {
    // The explanation templates and transit labels reach users too.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).unwrap();
    let mut lines = 0;
    let mut sensitive = 0;
    for mut f in facts(25, 0x6_0005, None) {
        let t = lagn_rules::reading::transits(&f.chart, 0.0, 90.0).unwrap();
        let r = lagn_rules::reading::sensitive_periods(&corpus, &mut f, Mode::Review, (0.0, 90.0), &t.windows);
        for w in &r.windows {
            sensitive += w.window.sensitive as usize;
            let mut all = w.explanation.clone();
            all.extend(w.pressures.iter().chain(&w.supports).map(|p| p.label.clone()));
            all.extend(w.focus.iter().flat_map(|h| h.significations.clone()));
            for line in &all {
                lines += 1;
                assert!(banned_in(line).is_empty(), "banned wording {:?} in {line:?}", banned_in(line));
            }
            // A sensitive window always says why.
            if w.window.sensitive {
                assert!(w.explanation.iter().any(|l| l.starts_with("This is a sensitive period")));
                assert!(w.explanation.iter().any(|l| l.starts_with("Calls for care:")));
            }
        }
    }
    assert!(lines > 1000 && sensitive > 0, "too little exercised: {lines} lines, {sensitive} sensitive windows");
}

#[test]
fn period_readings_match_independent_expectations() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).unwrap();
    // Sign lords by sign index, written out rather than taken from lagn-core.
    const LORD: [Graha; 12] = [
        Graha::Mars, Graha::Venus, Graha::Mercury, Graha::Moon, Graha::Sun, Graha::Mercury,
        Graha::Venus, Graha::Mars, Graha::Jupiter, Graha::Saturn, Graha::Saturn, Graha::Jupiter,
    ];
    let (mut node_windows, mut shani_pressure) = (0, 0);
    for mut f in facts(30, 0x6_0006, None) {
        let t = lagn_rules::reading::transits(&f.chart, 0.0, 90.0).unwrap();
        let r = lagn_rules::reading::sensitive_periods(&corpus, &mut f, Mode::Production, (0.0, 90.0), &t.windows);
        let lagna = f.chart.lagna.rasi.index() as usize;
        let house_of_sign = |s: usize| ((s + 12 - lagna) % 12 + 1) as u8;
        for w in &r.windows {
            // Focus: occupied house plus houses ruled (for nodes, by the dispositor).
            let g = w.window.antar;
            let sign = f.chart.placement(g).rasi.index() as usize;
            let ruler = if matches!(g, Graha::Rahu | Graha::Ketu) { node_windows += 1; LORD[sign] } else { g };
            let mut want: Vec<u8> = (0..12).filter(|&s| LORD[s] == ruler).map(house_of_sign).collect();
            want.push(house_of_sign(sign));
            want.sort();
            want.dedup();
            let got: Vec<u8> = w.focus.iter().map(|h| h.house).collect();
            assert_eq!(got, want, "focus of {g:?}");

            // No explanation line is repeated.
            let mut lines = w.explanation.clone();
            lines.sort();
            let n = lines.len();
            lines.dedup();
            assert_eq!(lines.len(), n, "repeated explanation line in {:?}", w.explanation);

            // A challenging Saturn transit in the window brings up the Saturn-transit pariharam.
            let shani = w.pressures.iter().any(|p| lagn_rules::catalogue::transit_tag(p.transit.kind).is_some());
            let has = w.pariharams.iter().any(|p| p.pariharam.id == "transit.shani");
            assert_eq!(has, shani, "transit pariharam mismatch in {:?}/{:?}", w.window.maha, w.window.antar);
            shani_pressure += shani as usize;
        }
    }
    assert!(node_windows > 0 && shani_pressure > 0);
}

// ---------------------------------------------------------------------------
// Phase 6C: family readings
// ---------------------------------------------------------------------------

#[test]
fn family_agreement_truth_table() {
    use lagn_rules::family::{agreement, Agreement::*, Lean::*};
    let all = [None, Some(Favourable), Some(Balanced), Some(CallsForCare)];
    for a in all {
        for b in all {
            let want = match (a, b) {
                (Some(Favourable), Some(Favourable)) | (Some(CallsForCare), Some(CallsForCare)) => Agree,
                (Some(Favourable), Some(CallsForCare)) | (Some(CallsForCare), Some(Favourable)) => Differ,
                _ => Inconclusive,
            };
            assert_eq!(agreement(a, b), want, "{a:?} vs {b:?}");
            assert_eq!(agreement(a, b), agreement(b, a), "agreement must be symmetric");
        }
    }
}

#[test]
fn family_readings_follow_their_definitions() {
    use lagn_rules::family::{family_reading, Lean, Relation};
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).unwrap();
    let people = facts(40, 0x6_0007, Some(Sex::Female));
    let lean = |s: i32| if s > 0 { Lean::Favourable } else if s < 0 { Lean::CallsForCare } else { Lean::Balanced };
    let mut seen = std::collections::BTreeSet::new();
    for pair in people.chunks(2) {
        let (native, member) = (&pair[0], &pair[1]);
        for rel in [Relation::Spouse, Relation::Child, Relation::Mother, Relation::Father] {
            let f = family_reading(&corpus, native, member, rel, Mode::Production);
            let (topic, tag, house) = match rel {
                Relation::Spouse => ("marriage", None, 7),
                Relation::Child => ("progeny", None, 5),
                Relation::Mother => ("parents", Some("parent:mother"), 4),
                Relation::Father => ("parents", Some("parent:father"), 9),
            };
            assert_eq!((f.relational.topic.as_str(), f.relational.house), (topic, house));
            // Recompute the relational score from the native's full topic report.
            let t = corpus.topics.iter().find(|t| t.id == topic).unwrap();
            let rep = evaluate_topic(topic, &corpus.rules, native, Mode::Production, (t.ages[0], t.ages[1]));
            let mine: Vec<_> = rep.results.iter().filter(|r| tag.is_none_or(|g| r.tags.iter().any(|x| x == g))).collect();
            assert!(!mine.is_empty());
            let score: i32 = mine.iter().filter(|r| r.effective).map(|r| r.polarity as i32).sum();
            assert_eq!(f.relational.score, score, "{rel:?}");
            assert_eq!(f.relational.lean, Some(lean(score)));
            if tag.is_none() {
                assert_eq!(score, rep.score, "{rel:?}: an untagged relation reads the whole topic");
            }
            for r in f.relational.supporting.iter().chain(&f.relational.afflicting) {
                assert!(r.effective && r.polarity != 0);
                if let Some(g) = tag {
                    assert!(r.tags.iter().any(|x| x == g), "{} is not about the {rel:?}", r.id);
                }
            }
            // The member's own readings are their own chart's topic reports.
            let want_topics: &[&str] = match rel {
                Relation::Spouse => &["marriage", "health"],
                Relation::Child => &["health", "education", "marriage"],
                _ => &["health", "later_life"],
            };
            let got: Vec<&str> = f.own.iter().map(|o| o.report.topic.as_str()).collect();
            assert_eq!(got, want_topics);
            for o in &f.own {
                let m = corpus.topics.iter().find(|t| t.id == o.report.topic).unwrap();
                assert_eq!(o.report, evaluate_topic(&o.report.topic, &corpus.rules, member, Mode::Production, (m.ages[0], m.ages[1])));
            }
            assert_eq!(f.compared_with, want_topics[0]);
            assert_eq!(f.agreement, lagn_rules::family::agreement(f.relational.lean, f.own[0].lean));
            seen.insert(format!("{:?}", f.agreement));
        }
    }
    assert!(seen.len() >= 2, "agreement outcomes too uniform to test: {seen:?}");
}

// ---------------------------------------------------------------------------
// Phase 6E: write-ups (DESIGN section 4a)
// ---------------------------------------------------------------------------

#[test]
fn write_ups_are_correct_justified_and_clean() {
    use lagn_rules::writeup::{graha_name, ordinal, topic_write_up};
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).unwrap();
    // Sign lords by sign index, written out independently of lagn-core.
    const LORD: [Graha; 12] = [
        Graha::Mars, Graha::Venus, Graha::Mercury, Graha::Moon, Graha::Sun, Graha::Mercury,
        Graha::Venus, Graha::Mars, Graha::Jupiter, Graha::Saturn, Graha::Saturn, Graha::Jupiter,
    ];
    let jargon = ["reviewer", "variant", "corpus", "polarity", "draft"];
    let (mut points, mut sections) = (0, 0);
    for (i, f) in facts(60, 0x6_000E, None).into_iter().enumerate() {
        let mut f = f;
        f.native.sex = [None, Some(Sex::Female), Some(Sex::Male)][i % 3];
        let lagna = f.chart.lagna.rasi.index() as usize;
        for meta in &corpus.topics {
            let rep = evaluate_topic(&meta.id, &corpus.rules, &f, Mode::Production, (meta.ages[0], meta.ages[1]));
            let w = topic_write_up(&corpus, &rep, &f, (meta.ages[0], meta.ages[1])).unwrap();
            // (d) determinism
            assert_eq!(w, topic_write_up(&corpus, &rep, &f, (meta.ages[0], meta.ages[1])).unwrap());

            // (a) every focus house names the right lord and the lord's house.
            for &h in &meta.focus.houses {
                let s = w.sections.iter().find(|s| s.heading.starts_with(&format!("The {} house", ordinal(h))))
                    .unwrap_or_else(|| panic!("{}: no section for house {h}", meta.id));
                sections += 1;
                let text = s.paragraphs.join(" ");
                let lord = LORD[(lagna + h as usize - 1) % 12];
                assert!(text.contains(&format!("so its lord is {}.", graha_name(lord))), "{}: wrong lord for house {h}: {text}", meta.id);
                let lsign = f.chart.placement(lord).rasi.index() as usize;
                let lh = ((lsign + 12 - lagna) % 12 + 1) as u8;
                if lh == h {
                    assert!(text.contains("sits in this very house"), "{text}");
                } else {
                    assert!(text.contains(&format!("is in your {} house", ordinal(lh))), "{}: wrong lord house: {text}", meta.id);
                }
                assert!(text.contains(&format!("holds {} bindus", f.ashtakavarga.sav_in_house(h))));
            }
            // Karakas: shown exactly when applicable to the native's sex.
            let karaka_text = w.sections.iter().find(|s| s.heading.starts_with("The significators")).map(|s| s.paragraphs.join(" ")).unwrap_or_default();
            for k in &meta.focus.karakas {
                let shown = karaka_text.contains(&format!("the natural significator of {}", k.role));
                assert_eq!(shown, k.sex.is_none_or(|s| f.native.sex == Some(s)), "{}: karaka {:?}", meta.id, k.graha);
            }

            // (c) points match effective results exactly, with sound justification.
            let effective: Vec<&str> = rep.results.iter()
                .filter(|r| r.effective && r.polarity != 0)
                .map(|r| r.id.as_str()).collect();
            let mut pointed: Vec<&str> = Vec::new();
            for s in &w.sections {
                for p in &s.points {
                    points += 1;
                    let r = rep.results.iter().find(|r| r.id == p.rule).expect("point for an unknown rule");
                    assert!(r.effective, "{} shown but not effective", p.rule);
                    match s.heading.as_str() {
                        "What supports" => assert!(r.polarity > 0),
                        "What calls for care" => assert!(r.polarity < 0),
                        "Also noted" => assert!(r.polarity == 0),
                        // Debts (rina) are difficulties gathered into their own
                        // section; they still count as scored findings.
                        "Debts carried forward" => {
                            assert!(r.polarity < 0);
                            assert!(r.tags.iter().any(|t| t.starts_with("rina:")));
                        }
                        h => panic!("points in unexpected section {h}"),
                    }
                    if r.polarity != 0 { pointed.push(r.id.as_str()); }
                    let facts: Vec<&str> = r.trace.iter().map(|t| t.facts.as_str()).collect();
                    let is_always = matches!(corpus.rules.iter().find(|x| x.id == r.id).unwrap().when, Condition::Always {});
                    assert!(is_always || !p.because.is_empty(), "{}: no justification", r.id);
                    for b in &p.because {
                        assert!(facts.contains(&b.as_str()), "{}: because {b:?} is not a fact the rule read", r.id);
                    }
                    // Minimal: never longer than the trace, and without repeats.
                    assert!(p.because.len() <= r.trace.len());
                }
            }
            // A cancellation rule never appears on its own as a "noted" point.
            for s in w.sections.iter().filter(|s| s.heading == "Also noted") {
                for p in &s.points {
                    assert!(corpus.rules.iter().find(|x| x.id == p.rule).unwrap().overrides.is_empty(), "{} announced relief from nothing", p.rule);
                }
            }
            // The conclusion follows the weights.
            let pos: i32 = rep.results.iter().filter(|r| r.effective && r.polarity > 0).map(|r| r.polarity as i32).sum();
            let neg: i32 = rep.results.iter().filter(|r| r.effective && r.polarity < 0).map(|r| -(r.polarity as i32)).sum();
            let lead = &w.summary[0];
            let expect = if rep.results.is_empty() { "No reviewed interpretations" }
                else if pos == 0 && neg == 0 { "applies strongly either way" }
                else if neg == 0 { "clearly supportive" }
                else if pos == 0 { "call for care and effort" }
                else if pos > neg { "are supportive" }
                else if pos < neg { "calls for care and effort" }
                else { "evenly balanced" };
            assert!(lead.contains(expect), "{}: +{pos}/-{neg} but the conclusion says {lead:?}", meta.id);
            pointed.sort();
            let mut want = effective.clone();
            want.sort();
            assert_eq!(pointed, want, "{}: scored points must be exactly the effective scored results", meta.id);

            // (b) wording guardrails over all generated text.
            let mut all: Vec<String> = w.summary.clone();
            for s in &w.sections {
                all.push(s.heading.clone());
                all.extend(s.paragraphs.clone());
                for p in &s.points { all.push(p.title.clone()); all.push(p.text.clone()); all.extend(p.because.clone()); }
            }
            // The disclaimer is a policy statement ("never estimates a
            // lifespan"), checked by its own test.
            for line in all.iter().filter(|l| Some(*l) != meta.disclaimer.as_ref()) {
                assert!(banned_in(line).is_empty(), "{}: banned {:?} in {line:?}", meta.id, banned_in(line));
                let lower = line.to_lowercase();
                for j in jargon {
                    assert!(!lower.split(|c: char| !c.is_alphanumeric()).any(|x| x == j), "{}: jargon {j:?} in {line:?}", meta.id);
                }
            }
            if let Some(d) = &meta.disclaimer {
                assert!(w.summary.contains(d), "{}: the write-up must carry the disclaimer", meta.id);
            }
        }
    }
    assert!(points > 500 && sections > 500, "too little exercised: {points} points, {sections} sections");
}

#[test]
fn justification_is_minimal_for_any_and_complete_for_all() {
    // The yogakaraka-style `any` of `all`s: only the deciding branch is cited.
    let f = &facts(1, 0x6_000F, None)[0];
    let g = |x: Graha| GrahaRef::Named(x);
    let branch = |x: Graha| Condition::All(vec![
        Condition::GrahaInHouse { graha: g(x), houses: (1..=12).collect(), varga: Varga::D1 },
        Condition::GrahaInHouse { graha: g(x), houses: vec![f.house_of(x, Varga::D1)], varga: Varga::D1 },
    ]);
    let never = Condition::GrahaInHouse { graha: g(Graha::Sun), houses: vec![], varga: Varga::D1 };
    let c = Condition::Any(vec![Condition::All(vec![never.clone(), branch(Graha::Moon)]), branch(Graha::Mars), branch(Graha::Venus)]);
    let j = lagn_rules::justify(&c, f);
    let mars = format!("{} in house {}", Graha::Mars.name(), f.house_of(Graha::Mars, Varga::D1));
    assert_eq!(j, vec![mars], "only the first true branch, deduplicated: {j:?}");
    assert!(lagn_rules::justify(&Condition::Always {}, f).is_empty());
}

// ---------------------------------------------------------------------------
// Phase 6F: plain-language impact (DESIGN section 4b)
// ---------------------------------------------------------------------------

/// Words that would state a fixed outcome rather than a tendency.
const ABSOLUTES: &[&str] = &[
    "will", "always", "certainly", "definitely", "guaranteed", "inevitable", "inevitably",
    "must", "cannot", "impossible", "doomed", "destined", "fated",
];
/// Marks of a constructive close: what helps, or what the tradition softens it with.
const CONSTRUCTIVE: &[&str] = &[
    "help", "helps", "helping", "eases", "eased", "softens", "cancelled", "patience", "patient",
    "care", "attention", "attentive", "routine", "routines", "regular", "plan", "planning",
    "worth", "matters", "serve", "serves", "support", "practical", "answer", "habit", "habits",
    "effort", "remedies", "check-ups", "difference", "prevents", "record", "space", "budgeting",
    "consistency", "steady", "tending", "conversation", "contact", "presence",
    "persistence", "perseverance", "suit", "suits", "resolve", "turn", "pace", "pacing", "build",
    "settles", "settle", "settled", "observance", "tarpanam", "worship", "keep", "finish", "reliable",
];

fn words(s: &str) -> Vec<String> {
    s.to_lowercase().split(|c: char| !c.is_alphanumeric() && c != '-').filter(|w| !w.is_empty()).map(str::to_string).collect()
}

#[test]
fn every_rule_says_what_it_means_for_the_reader() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let c = lagn_rules::load(&dir).expect("shipped corpus must validate");
    let mut negative = 0;
    for r in &c.rules {
        let m = r.text.impact.as_deref().unwrap_or_else(|| panic!("{}: no impact line", r.id));
        let w = words(m);

        // Plain words: no technical vocabulary, no house numbers as jargon.
        for jargon in ["kendra", "trikona", "dusthana", "lagna", "moolatrikona", "debilitated",
                       "exalted", "combust", "navamsa", "varga", "drishti", "sav", "bindus",
                       "ashtakavarga", "antardasha", "mahadasha", "polarity", "reviewer"] {
            assert!(!w.iter().any(|x| x == jargon), "{}: technical word {jargon:?} in the plain-language line: {m:?}", r.id);
        }
        // Tendency, not fate.
        for a in ABSOLUTES {
            assert!(!w.iter().any(|x| x == a), "{}: {a:?} states an outcome: {m:?}", r.id);
        }
        // Never frightening, and never internally inconsistent with the guardrails.
        assert!(banned_in(m).is_empty(), "{}: banned wording {:?} in {m:?}", r.id, banned_in(m));
        // Written prose, distinct from the technical line.
        assert!(m.len() >= 40 && m.len() <= 500, "{}: impact line is {} characters", r.id, m.len());
        assert!(m.trim_end().ends_with('.'), "{}: impact line must end in a full stop", r.id);
        assert!(m.chars().next().is_some_and(char::is_uppercase), "{}: impact line must start with a capital", r.id);
        assert_ne!(m.trim(), r.text.en.trim());
        // A difficult finding never ends on the difficulty alone.
        if r.polarity < 0 {
            negative += 1;
            assert!(w.iter().any(|x| CONSTRUCTIVE.contains(&x.as_str())), "{}: a difficult finding must say what helps: {m:?}", r.id);
        }
    }
    assert!(negative >= 40, "only {negative} negative rules checked");
}

#[test]
fn readings_carry_the_plain_language_line_wherever_a_rule_is_shown() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).unwrap();
    let (mut points, mut period_lines) = (0, 0);
    for mut f in facts(12, 0x6_0010, Some(Sex::Female)) {
        for meta in &corpus.topics {
            let rep = evaluate_topic(&meta.id, &corpus.rules, &f, Mode::Production, (meta.ages[0], meta.ages[1]));
            for r in rep.results.iter().filter(|r| r.effective) {
                let want = corpus.rules.iter().find(|x| x.id == r.id).unwrap().text.impact.clone();
                assert_eq!(r.impact, want, "{}: result must carry the impact line", r.id);
            }
            let w = lagn_rules::writeup::topic_write_up(&corpus, &rep, &f, (meta.ages[0], meta.ages[1])).unwrap();
            for s in &w.sections {
                for p in &s.points {
                    points += 1;
                    assert!(p.meaning.is_some(), "{}: write-up point without a plain-language line", p.rule);
                }
            }
        }
        // Period readings lead with the plain-language line.
        let t = lagn_rules::reading::transits(&f.chart, 20.0, 45.0).unwrap();
        let r = lagn_rules::reading::sensitive_periods(&corpus, &mut f, Mode::Production, (20.0, 45.0), &t.windows);
        for win in &r.windows {
            for line in win.explanation.iter().filter(|l| l.starts_with("Calls for care:") || l.starts_with("Supports:")) {
                period_lines += 1;
                let rule = win.window.amplifiers.iter().chain(&win.window.negators)
                    .find(|x| x.impact.as_ref().is_some_and(|i| line.contains(i.as_str())))
                    .unwrap_or_else(|| panic!("no plain-language line in {line:?}"));
                assert!(line.contains(rule.text.trim_end_matches('.')), "the technical line must still be there: {line:?}");
            }
        }
    }
    assert!(points > 200 && period_lines > 200, "too little exercised: {points} points, {period_lines} period lines");
}

// ---------------------------------------------------------------------------
// Phase 7: the "Together" comparison (docs/phase7/DESIGN.md section 4)
// ---------------------------------------------------------------------------

#[test]
fn the_together_text_states_both_readings_without_merging_them() {
    use lagn_rules::family::{family_reading, Agreement, Lean, Relation, NAME_TOKEN};
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).unwrap();
    let people = facts(24, 0x7_0001, Some(Sex::Female));
    let mut seen: std::collections::BTreeSet<String> = Default::default();
    for pair in people.chunks(2) {
        let (native, member) = (&pair[0], &pair[1]);
        for rel in [Relation::Spouse, Relation::Child, Relation::Mother, Relation::Father] {
            let f = family_reading(&corpus, native, member, rel, Mode::Production);
            let c = &f.comparison;
            assert_eq!(*c, family_reading(&corpus, native, member, rel, Mode::Production).comparison, "not deterministic");
            assert!(c.len() >= 4, "{rel:?}: too thin: {c:?}");
            let all = c.join(" ");

            // Names never leave the device: the text carries the placeholder.
            assert!(all.contains(NAME_TOKEN), "{rel:?}: no name placeholder");
            assert!(!all.replace(NAME_TOKEN, "").contains('{'), "{rel:?}: stray placeholder in {all:?}");

            // Both readings are stated, in plain words, neither merged away.
            let rel_lean = f.relational.lean.map(Lean::plain).unwrap_or("no reviewed indication");
            let own = f.own.iter().find(|o| o.report.topic == f.compared_with);
            let own_lean = own.and_then(|o| o.lean).map(Lean::plain).unwrap_or("no reviewed indication");
            assert!(c[1].contains(rel_lean), "{rel:?}: the native's reading is not stated: {:?}", c[1]);
            assert!(c[2].contains(own_lean), "{rel:?}: the member's reading is not stated: {:?}", c[2]);
            assert!(c[1].contains("Your chart"));
            assert!(c[2].starts_with(NAME_TOKEN));
            // No averaging: it never claims a single combined verdict.
            for word in ["average", "averaged", "overall verdict", "combined"] {
                assert!(!all.to_lowercase().contains(word), "{rel:?}: {word:?} in {all:?}");
            }
            // The verdict sentence matches the computed agreement.
            let expect = match f.agreement {
                Agreement::Agree => "point the same way",
                Agreement::Differ => "point different ways",
                Agreement::Inconclusive => "do not give a clear comparison",
            };
            assert!(c[3].contains(expect), "{:?} does not match {:?}", c[3], f.agreement);
            seen.insert(format!("{:?}", f.agreement));

            // Phase 6 wording guardrails apply here too.
            for line in c {
                let plain = line.replace(NAME_TOKEN, "Meena");
                assert!(banned_in(&plain).is_empty(), "banned {:?} in {plain:?}", banned_in(&plain));
                for j in ["kendra", "dusthana", "polarity", "corpus", "reviewer", "varga"] {
                    assert!(!plain.to_lowercase().split(|c: char| !c.is_alphanumeric()).any(|w| w == j), "jargon {j:?} in {plain:?}");
                }
            }
        }
    }
    assert!(seen.len() >= 2, "agreement outcomes too uniform: {seen:?}");
}

#[test]
fn write_up_sections_are_typed_for_the_interface() {
    use lagn_rules::writeup::{topic_write_up, SectionKind};
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).unwrap();
    let mut kinds: std::collections::BTreeSet<String> = Default::default();
    for f in facts(20, 0x7_0002, Some(Sex::Male)) {
        for meta in &corpus.topics {
            let rep = evaluate_topic(&meta.id, &corpus.rules, &f, Mode::Review, (meta.ages[0], meta.ages[1]));
            let w = topic_write_up(&corpus, &rep, &f, (meta.ages[0], meta.ages[1])).unwrap();
            for s in &w.sections {
                kinds.insert(format!("{:?}", s.kind));
                // The kind always matches what the section holds.
                match s.kind {
                    SectionKind::Supporting => assert!(s.points.iter().all(|p| p.polarity > 0)),
                    SectionKind::Care => assert!(s.points.iter().all(|p| p.polarity < 0)),
                    SectionKind::Noted => assert!(s.points.iter().all(|p| p.polarity == 0)),
                    SectionKind::Debts => assert!(s.points.iter().all(|p| p.polarity < 0) && !s.points.is_empty()),
                    SectionKind::House => assert!(s.heading.contains("house") && s.points.is_empty()),
                    _ => assert!(s.points.is_empty(), "{:?} must not carry findings", s.kind),
                }
            }
        }
    }
    for want in ["House", "Karakas", "Varga", "Supporting", "Care", "Noted", "Eased", "Timing"] {
        assert!(kinds.contains(want), "{want} never produced; kinds seen: {kinds:?}");
    }
}

// ---------------------------------------------------------------------------
// Phase 8: the past-life reading (docs/phase8/DESIGN.md)
// ---------------------------------------------------------------------------

/// The vocabulary of biography and of blame. Neither belongs in this topic.
const PAST_LIFE_BANNED: &[&str] = &[
    // Biography: a chart cannot encode a name, a year or a place, so none is claimed.
    "reincarnation", "reincarnated", "previous birth", "former life", "past incarnation",
    "you were", "were born as", "in that life", "kingdom", "dynasty", "century", "medieval",
    "ancient", "bc", "ad", "era",
    // Blame. The debt vocabulary itself (rina, debt, shapa, curse) is required
    // content since phase 9, but nothing is framed as punishment or as earned.
    "sin", "sins", "sinful", "punishment", "punished", "punishes", "deserved", "retribution",
    "doomed", "wrongdoing", "guilty",
];

fn past_life_banned_in(text: &str) -> Vec<&'static str> {
    let t = text.to_lowercase();
    PAST_LIFE_BANNED
        .iter()
        .copied()
        .filter(|w| {
            if w.contains(' ') {
                t.contains(w)
            } else {
                t.split(|c: char| !c.is_alphanumeric()).any(|x| x == *w)
            }
        })
        .collect()
}

#[test]
fn the_past_life_topic_never_claims_a_biography_or_blames_the_native() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).unwrap();
    let meta = corpus.topics.iter().find(|t| t.id == "past_life").expect("the topic must be in the catalogue");
    // The frame is declared plainly, every time.
    let d = meta.disclaimer.as_deref().expect("past_life needs a disclaimer").to_lowercase();
    assert!(d.contains("not a claim about events"));
    assert!(d.contains("never names a past identity"));

    let rules: Vec<_> = corpus.rules.iter().filter(|r| r.topic == "past_life").collect();
    assert!(rules.len() >= 15, "only {} rules", rules.len());
    for r in &rules {
        for field in [&r.title, &r.text.en, r.text.impact.as_ref().unwrap()] {
            assert!(past_life_banned_in(field).is_empty(), "{}: {:?} in {field:?}", r.id, past_life_banned_in(field));
            assert!(banned_in(field).is_empty(), "{}: {:?} in {field:?}", r.id, banned_in(field));
        }
    }

    // And over generated text, for many charts.
    let mut axis = 0;
    for f in facts(30, 0x8_0001, None) {
        let rep = evaluate_topic("past_life", &corpus.rules, &f, Mode::Review, (meta.ages[0], meta.ages[1]));
        let w = lagn_rules::writeup::topic_write_up(&corpus, &rep, &f, (meta.ages[0], meta.ages[1])).unwrap();
        assert!(w.summary.contains(meta.disclaimer.as_ref().unwrap()));
        let mut all = w.summary.clone();
        for s in &w.sections {
            all.push(s.heading.clone());
            all.extend(s.paragraphs.clone());
            for p in &s.points {
                all.push(p.title.clone());
                all.push(p.text.clone());
                all.extend(p.meaning.clone());
            }
            if s.kind == lagn_rules::writeup::SectionKind::KarmicAxis { axis += 1; }
        }
        for line in all.iter().filter(|l| Some(*l) != meta.disclaimer.as_ref()) {
            assert!(past_life_banned_in(line).is_empty(), "{:?} in {line:?}", past_life_banned_in(line));
            assert!(banned_in(line).is_empty(), "{:?} in {line:?}", banned_in(line));
        }
    }
    assert_eq!(axis, 30, "every reading needs the karmic axis");
}

#[test]
fn the_karmic_axis_and_bridge_are_computed_from_the_chart() {
    use lagn_rules::writeup::{ordinal, topic_write_up, SectionKind};
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).unwrap();
    let meta = corpus.topics.iter().find(|t| t.id == "past_life").unwrap();
    for f in facts(30, 0x8_0002, None) {
        let rep = evaluate_topic("past_life", &corpus.rules, &f, Mode::Production, (meta.ages[0], meta.ages[1]));
        let w = topic_write_up(&corpus, &rep, &f, (meta.ages[0], meta.ages[1])).unwrap();
        // Houses derived independently: 1 + signs between the lagna and the graha.
        let lagna = f.chart.lagna.rasi.index() as i32;
        let house = |g: Graha| (((f.chart.placement(g).rasi.index() as i32 - lagna) + 12) % 12 + 1) as u8;
        let (kh, rh) = (house(Graha::Ketu), house(Graha::Rahu));
        assert_eq!((kh + 6 - 1) % 12 + 1, rh, "Ketu and Rahu must be opposite");

        let axis = w.sections.iter().find(|s| s.kind == SectionKind::KarmicAxis).expect("no axis section");
        let text = axis.paragraphs.join(" ");
        assert!(text.contains(&format!("Ketu sits in your {} house", ordinal(kh))), "{text}");
        assert!(text.contains(&format!("Rahu stands opposite, in your {} house", ordinal(rh))), "{text}");
        // Ketu acts through the lord of its sign.
        let disp = f.chart.placement(Graha::Ketu).rasi.lord();
        assert!(text.contains(&lagn_rules::writeup::graha_name(disp)), "the dispositor {disp:?} is not named: {text}");

        // The bridge lists exactly the topics whose focus covers those houses.
        let bridge = w.sections.iter().find(|s| s.kind == SectionKind::Bridge);
        let joined = bridge.map(|s| s.paragraphs.join(" ")).unwrap_or_default();
        for (house, _) in [(kh, ()), (rh, ()), (5u8, ())] {
            for t in corpus.topics.iter().filter(|t| t.id != "past_life" && t.focus.houses.contains(&house)) {
                assert!(joined.to_lowercase().contains(&t.title.to_lowercase()), "{} missing from the bridge for house {house}: {joined}", t.title);
                // Its verdict is that topic's own score, not a new judgement.
                let other = evaluate_topic(&t.id, &corpus.rules, &f, Mode::Production, (t.ages[0], t.ages[1]));
                let want = match other.score.signum() {
                    _ if other.results.is_empty() => "has no reviewed indication".to_string(),
                    1 => format!("comes out favourable ({:+})", other.score),
                    -1 => format!("calls for care ({:+})", other.score),
                    _ => "comes out evenly balanced".to_string(),
                };
                assert!(joined.contains(&want), "{}: expected {want:?} in {joined}", t.id);
            }
        }
        // It never reports on itself.
        assert!(!joined.to_lowercase().contains("your past life reading"));
    }
}

// ---------------------------------------------------------------------------
// Phase 9: the life carried forward, bonds, and debts (docs/phase9/DESIGN.md)
// ---------------------------------------------------------------------------

#[test]
fn the_life_carried_forward_is_the_chart_s_own_ketu() {
    use lagn_rules::writeup::{ordinal, topic_write_up, SectionKind};
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).unwrap();
    let karma = corpus.karma.as_ref().expect("karma.json must be loaded");
    let meta = corpus.topics.iter().find(|t| t.id == "past_life").unwrap();
    // Sign names written out independently of lagn-core, in zodiac order.
    const SIGNS: [&str; 12] = ["Mesha", "Rishabha", "Mithuna", "Karka", "Simha", "Kanya",
                               "Tula", "Vrischika", "Dhanus", "Makara", "Kumbha", "Meena"];
    let (mut stations, mut temperaments) = (std::collections::BTreeSet::new(), std::collections::BTreeSet::new());
    for f in facts(40, 0x9_0001, None) {
        let rep = evaluate_topic("past_life", &corpus.rules, &f, Mode::Production, (meta.ages[0], meta.ages[1]));
        let w = topic_write_up(&corpus, &rep, &f, (meta.ages[0], meta.ages[1])).unwrap();
        let life = w.sections.iter().find(|s| s.kind == SectionKind::LifeCarried).expect("no life section");
        let text = life.paragraphs.join(" ");

        let lagna = f.chart.lagna.rasi.index() as i32;
        let ki = f.chart.placement(Graha::Ketu).rasi.index() as i32;
        let house = ((ki - lagna + 12) % 12 + 1) as u8;
        assert!(text.contains(&format!("Ketu sits in your {} house", ordinal(house))), "{text}");
        assert!(text.contains(karma.station(house).unwrap()), "wrong station for house {house}");
        assert!(text.contains(SIGNS[ki as usize]), "wrong sign name: {text}");
        assert!(text.contains(&karma.temperament(ki as u8 + 1).unwrap().temperament), "wrong temperament");
        // Ketu acts through the lord of its sign, and that judgement is stated.
        let disp = f.chart.placement(Graha::Ketu).rasi.lord();
        assert!(text.contains(&lagn_rules::writeup::graha_name(disp)));
        // "How that life went" must be the right one of the two, not either.
        let dh = ((f.chart.placement(disp).rasi.index() as i32 - lagna + 12) % 12 + 1) as u8;
        let friendly = matches!(
            f.dignity(disp, Varga::D1),
            Some(Dignity::Exalted) | Some(Dignity::Moolatrikona) | Some(Dignity::OwnSign)
                | Some(Dignity::GreatFriend) | Some(Dignity::Friend)
        );
        let well_placed = !matches!(dh, 6 | 8 | 12);
        let (want, other) = if friendly && well_placed {
            (&karma.dispositor.strong, &karma.dispositor.weak)
        } else {
            (&karma.dispositor.weak, &karma.dispositor.strong)
        };
        assert!(text.contains(want.as_str()), "wrong verdict on how that life went: {text}");
        assert!(!text.contains(other.as_str()));
        // The limit is stated in the reading itself, not only in the docs.
        assert!(text.contains("it cannot hold a name, a year or a place"));
        stations.insert(house);
        temperaments.insert(ki);

        // Bonds name both derived houses and say why they are read.
        let bonds = w.sections.iter().find(|s| s.kind == SectionKind::Bonds).expect("no bonds section");
        let b = bonds.paragraphs.join(" ");
        assert!(b.contains("your 6th house") && b.contains("12th from the 7th"), "{b}");
        assert!(b.contains("your 4th house") && b.contains("12th from the 5th"), "{b}");
        assert!(b.contains("they take precedence"), "the present-life readings must take precedence");
    }
    assert!(stations.len() >= 8 && temperaments.len() >= 8, "too few placements exercised");
}

#[test]
fn every_debt_names_its_settlement_and_reaches_its_remedy() {
    use lagn_rules::writeup::{topic_write_up, SectionKind};
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).unwrap();
    let meta = corpus.topics.iter().find(|t| t.id == "past_life").unwrap();

    // Corpus level: a debt rule states what settles it, and carries a dosha or
    // graha tag so the pariharam machinery can answer it.
    let debts: Vec<_> = corpus.rules.iter().filter(|r| r.tags.iter().any(|t| t.starts_with("rina:"))).collect();
    assert!(debts.len() >= 5, "only {} debt rules", debts.len());
    for r in &debts {
        let m = r.text.impact.as_ref().unwrap().to_lowercase();
        assert!(m.contains("settles it") || m.contains("customary") || m.contains("support") || m.contains("settle"),
                "{}: a debt must say what settles it: {m}", r.id);
        let has_remedy_tag = r.tags.iter().any(|t| t.starts_with("dosha:")) || r.subject.is_some();
        assert!(has_remedy_tag, "{}: no tag or subject to hang a remedy on", r.id);
        for t in r.tags.iter().filter(|t| t.starts_with("dosha:")) {
            assert!(corpus.pariharams.iter().any(|p| p.triggers.contains(t)), "{}: no pariharam answers {t}", r.id);
        }
    }

    // Chart level: when a debt fires, its remedy is among those suggested.
    let mut fired = 0;
    for f in facts(60, 0x9_0002, None) {
        let rep = evaluate_topic("past_life", &corpus.rules, &f, Mode::Production, (meta.ages[0], meta.ages[1]));
        let suggested = lagn_rules::pariharam::suggest(&corpus, &rep.results, &[], Mode::Production);
        let w = topic_write_up(&corpus, &rep, &f, (meta.ages[0], meta.ages[1])).unwrap();
        let section = w.sections.iter().find(|s| s.kind == SectionKind::Debts);

        let effective: Vec<&str> = rep.results.iter()
            .filter(|r| r.effective && r.tags.iter().any(|t| t.starts_with("rina:")))
            .map(|r| r.id.as_str()).collect();
        match section {
            None => assert!(effective.is_empty(), "debts fired but no section: {effective:?}"),
            Some(s) => {
                let shown: Vec<&str> = s.points.iter().map(|p| p.rule.as_str()).collect();
                assert_eq!(shown, effective, "the debts section must hold exactly the debts that fired");
                assert!(s.paragraphs[0].contains("rather than faults to be answered for"));
                fired += shown.len();
                for id in &shown {
                    let rule = corpus.rules.iter().find(|r| &r.id == id).unwrap();
                    for t in rule.tags.iter().filter(|t| t.starts_with("dosha:")) {
                        assert!(suggested.iter().any(|p| p.pariharam.triggers.contains(t)),
                                "{id} fired but its remedy for {t} was not suggested");
                    }
                }
                // Debts are not repeated among the ordinary difficulties.
                for other in w.sections.iter().filter(|x| x.kind == SectionKind::Care || x.kind == SectionKind::Noted) {
                    assert!(other.points.iter().all(|p| !shown.contains(&p.rule.as_str())));
                }
            }
        }
    }
    assert!(fired > 20, "only {fired} debts exercised");
}
