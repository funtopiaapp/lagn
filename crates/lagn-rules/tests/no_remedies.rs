//! The corpus recommends no remedial practice, anywhere.
//!
//! The product owner's direction: pariharams are not recommended to anyone.
//! The catalogue that held them is gone, and so is the code that suggested
//! them - but the part that could quietly come back is the *text*. A rule
//! that says "light a lamp on Saturdays" is a pariharam whether or not there
//! is a struct called Pariharam, so this reads every shipped string a reader
//! can see and fails on the vocabulary of prescribed practice.
//!
//! This is deliberately a blunt instrument. If a legitimate reading ever
//! needs one of these words, the right move is to discuss it, not to widen
//! the list quietly.

use std::collections::BTreeSet;
use std::path::Path;

/// Words that mean "perform this practice". Matched case-insensitively on
/// word boundaries where a bare substring would be ambiguous.
/// "ayilyam" is deliberately absent: it is the Tamil name of the nakshatra
/// Ashlesha as well as of a puja, and banning it would fail a legitimate
/// reading that names the star. The practice is caught by "puja" anyway.
const BANNED: [&str; 25] = [
    "pariharam", "parihara", "parihar",
    "tarpanam", "puja", "pooja", "abhishekam", "archana", "homam", "havan",
    "pradosham", "navagraha", "japa",
    "propitiate", "propitiation",
    "remedy", "remedies", "remedial",
    "observance", "observances",
    "vratam", "upavasam",
    "amulet", "talisman", "gemstone",
];

/// Phrases that prescribe rather than describe. These words have innocent
/// uses ("a temple town", "the lamp of knowledge"), so they are only banned
/// in the company of an instruction.
const BANNED_PAIRS: [(&str, &str); 6] = [
    ("light", "lamp"),
    ("offer", "water"),
    ("worship", "on"),
    ("recite", "times"),
    ("donate", "to"),
    ("visit", "temple"),
];

/// Every string a *reader* can see.
///
/// `review` and `source` are skipped deliberately. They are provenance for
/// reviewers - who approved a rule, what was changed and why - and none of it
/// is sent to the app: a RuleResult carries the reviewer's name and nothing
/// else from either block. A change note that says "the prescribed worship
/// was removed" must not itself trip a test looking for prescribed worship.
fn reader_strings(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::String(s) => out.push(s.clone()),
        serde_json::Value::Array(a) => a.iter().for_each(|x| reader_strings(x, out)),
        serde_json::Value::Object(o) => {
            for (k, val) in o {
                if k == "review" || k == "source" {
                    continue;
                }
                reader_strings(val, out);
            }
        }
        _ => {}
    }
}

fn offences(text: &str) -> BTreeSet<String> {
    let lower = text.to_lowercase();
    let mut found = BTreeSet::new();
    for w in BANNED {
        if lower.contains(w) {
            found.insert(w.to_string());
        }
    }
    for (a, b) in BANNED_PAIRS {
        if lower.contains(a) && lower.contains(b) {
            found.insert(format!("{a} ... {b}"));
        }
    }
    found
}

fn json_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).expect("read corpus").flatten() {
        let p = e.path();
        if p.is_dir() {
            json_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "json") {
            out.push(p);
        }
    }
}

#[test]
fn no_shipped_corpus_text_prescribes_a_remedial_practice() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let mut files = Vec::new();
    json_files(&dir, &mut files);
    files.sort();
    assert!(files.len() > 10, "only found {} corpus files", files.len());

    let mut problems = Vec::new();
    for f in &files {
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(f).expect("read")).expect("parse");
        let mut texts = Vec::new();
        reader_strings(&v, &mut texts);
        for t in texts {
            let bad = offences(&t);
            if !bad.is_empty() {
                problems.push(format!("{}: {:?} in {:?}", f.display(), bad, t));
            }
        }
    }
    assert!(problems.is_empty(), "remedial language in the corpus:\n{}", problems.join("\n"));
}

#[test]
fn the_pariharam_catalogue_is_gone_and_the_corpus_still_loads() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    assert!(!dir.join("pariharam.json").exists(), "corpus/pariharam.json is back");

    // And the corpus is still valid without it, so removing it did not leave
    // a dangling trigger or a rule pointing at something that no longer ships.
    let c = lagn_rules::load(&dir).expect("the corpus must still load");
    assert!(!c.rules.is_empty());
    let problems = lagn_rules::corpus::validate(&c);
    assert!(problems.is_empty(), "corpus no longer validates:\n{problems:#?}");
}

#[test]
fn no_rule_promises_that_something_can_be_settled_by_practice() {
    // The debts in past_life used to end by naming an observance. They now
    // describe conduct ("keep your word", "care given to your mother") or
    // nothing at all. What must never return is a pointer to a list that no
    // longer exists - "listed below", "the remedies below" - which is both a
    // recommendation and a dangling reference.
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let c = lagn_rules::load(&dir).expect("corpus");
    for r in &c.rules {
        for t in [Some(&r.text.en), r.text.impact.as_ref()].into_iter().flatten() {
            let lower = t.to_lowercase();
            for phrase in ["listed below", "remedies below", "observances below", "shown below"] {
                assert!(!lower.contains(phrase), "{}: still points at {phrase:?}", r.id);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The generated prose, not just the corpus
// ---------------------------------------------------------------------------
//
// The corpus sweep above missed a real one: the write-up generator holds its
// own template sentences in Rust, and the debts section ended with "The
// remedies below are the customary ones" - a recommendation, and a pointer to
// a list that had just been deleted. A browser test caught it. This closes the
// gap at the source, by generating the prose and reading it.

use lagn_core::{BirthData, BirthMoment, Chart, ChartSettings, DerivationSettings, Ephemeris};
use lagn_rules::resolve::{evaluate_topic, Mode};
use lagn_rules::{FactBase, NativeInfo};

fn init_ephemeris() {
    let d = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().expect("ephe");
    Ephemeris::set_ephemeris_path(d).expect("set ephemeris path");
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

#[test]
fn no_generated_write_up_recommends_a_remedial_practice() {
    init_ephemeris();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let corpus = lagn_rules::load(&dir).expect("corpus");

    let mut rng = Rng(0x9ED_1E5);
    let mut charts = Vec::new();
    // Enough charts that every topic's sections, including the debts that only
    // some charts carry, are actually generated.
    while charts.len() < 60 {
        let b = BirthData {
            moment: BirthMoment {
                year: rng.int(1900, 2030) as i32,
                month: rng.int(1, 12) as u32,
                day: rng.int(1, 28) as u32,
                hour: rng.int(0, 23) as u32,
                minute: rng.int(0, 59) as u32,
                second: 0.0,
                utc_offset_hours: 5.5,
            },
            latitude: rng.unit() * 120.0 - 60.0,
            longitude: rng.unit() * 360.0 - 180.0,
            place_name: String::new(),
        };
        if let Ok(c) = Chart::compute(b, ChartSettings::default()) {
            charts.push(FactBase::new(c, DerivationSettings::default(), NativeInfo::default()));
        }
    }

    let mut sections = 0;
    let mut problems = Vec::new();
    for f in &charts {
        for t in &corpus.topics {
            let ages = (t.ages[0], t.ages[1]);
            let report = evaluate_topic(&t.id, &corpus.rules, f, Mode::Production, ages);
            let w = lagn_rules::writeup::write_up(&corpus, t, &t.focus, &report, f, ages, |_| true);
            let mut texts: Vec<&str> = w.summary.iter().map(String::as_str).collect();
            for s in &w.sections {
                sections += 1;
                texts.push(&s.heading);
                texts.extend(s.paragraphs.iter().map(String::as_str));
                for p in s.points.iter() {
                    texts.push(&p.title);
                    texts.push(&p.text);
                    if let Some(m) = &p.meaning {
                        texts.push(m);
                    }
                    texts.extend(p.because.iter().map(String::as_str));
                }
            }
            for text in texts {
                let bad = offences(text);
                if !bad.is_empty() {
                    problems.push(format!("{}: {:?} in {:?}", t.id, bad, text));
                }
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(sections > 200, "only {sections} sections generated; the sweep is too thin");
    assert!(
        problems.is_empty(),
        "{} generated passages recommend a practice:\n{}",
        problems.len(),
        problems.join("\n"),
    );
}
