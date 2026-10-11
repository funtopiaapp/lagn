//! No reviewed interpretation may rest on an unverified number.
//!
//! `docs/phase13/DESIGN.md` section 6, constraint 3, and
//! `docs/phase13/SHADBALA.md` criterion 8.
//!
//! Shadbala ships labelled: every value depends on variant choices no
//! astrologer has signed off, and one of its six components is not computed at
//! all. That standing is honest only while it is visible. A corpus rule that
//! conditioned on a bala would launder the caveat away - the reader would see
//! a reviewed, approved finding with a reviewer's name on it, and no sign that
//! an unverified quantity produced it.
//!
//! So this is enforced rather than trusted. The rule language has no strength
//! term today, and this test fails the moment one appears in the corpus.

use std::path::Path;

/// Vocabulary that would mean a rule is reading a strength. Blunt on purpose:
/// if a legitimate rule ever needs one of these words, that is a conversation,
/// not a list to widen quietly.
const STRENGTH_TERMS: [&str; 18] = [
    "shadbala", "shad_bala", "sthana_bala", "sthanabala",
    "dig_bala", "digbala", "kala_bala", "kalabala",
    "cheshta", "naisargika", "drik_bala", "drikbala",
    "saptavargaja", "uchcha", "ojayugma", "drekkana_bala",
    "virupa", "rupa_strength",
];

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
fn no_corpus_rule_conditions_on_a_shadbala_value() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let mut files = Vec::new();
    json_files(&dir, &mut files);
    files.sort();
    assert!(files.len() > 10, "only found {} corpus files", files.len());

    let mut problems = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).expect("read").to_lowercase();
        for term in STRENGTH_TERMS {
            if text.contains(term) {
                problems.push(format!("{}: contains {term:?}", f.display()));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "a corpus rule appears to read an unverified strength:\n{}\n\n\
         Shadbala is Pro-only and labelled because its variants are unsigned and \
         one component is not computed. A reviewed rule resting on it would hide \
         that from the reader. See docs/phase13/SHADBALA.md criterion 8.",
        problems.join("\n"),
    );
}

#[test]
fn the_rule_language_offers_no_way_to_read_a_strength() {
    // The other half: even if nobody has written such a rule, the *language*
    // must not make one expressible, or the bar is a convention rather than a
    // constraint. Conditions are a closed enum, so this reads the source and
    // fails if a strength term ever becomes a variant of it.
    let model = include_str!("../src/model.rs");
    let lower = model.to_lowercase();
    for term in STRENGTH_TERMS {
        assert!(
            !lower.contains(term),
            "the rule language has gained a {term:?} term; \
             a reviewed rule could now rest on an unverified number",
        );
    }
}
