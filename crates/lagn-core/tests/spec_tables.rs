//! The kernel's hand-transcribed tables, checked against the specification
//! text itself.
//!
//! Added after Phase 2 mutation testing: moving one house number inside an
//! Ashtakavarga row while keeping the row total unchanged passed every other
//! Rust test, because they all read the same transcription they were meant to
//! check. Only the Python oracle, which parses DESIGN.md, caught it. These
//! tests bring that guard into `cargo test`.

use lagn_core::ashtakavarga::benefic_houses;
use lagn_core::relationship::{natural, NaturalRelation, SEVEN};
use lagn_core::{Contributor, Graha, Rasi, Varga};

fn design() -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/phase2/DESIGN.md");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

fn graha(name: &str) -> Option<Graha> {
    SEVEN.into_iter().find(|g| {
        let n = serde_json::to_value(g).unwrap();
        n.as_str().unwrap().eq_ignore_ascii_case(name)
    })
}

fn rasi(name: &str) -> Rasi {
    Rasi::ALL.into_iter().find(|r| r.name() == name).unwrap_or_else(|| panic!("unknown sign {name}"))
}

/// Pipe-table rows between two markers, split into trimmed cells.
fn rows(text: &str, from: &str, to: &str) -> Vec<Vec<String>> {
    let start = text.find(from).unwrap_or_else(|| panic!("marker {from:?} missing from DESIGN.md"));
    let end = start + text[start..].find(to).unwrap_or_else(|| panic!("marker {to:?} missing"));
    text[start..end]
        .lines()
        .filter(|l| l.trim_start().starts_with('|'))
        .map(|l| l.trim().trim_matches('|').split('|').map(|c| c.trim().to_string()).collect())
        .collect()
}

#[test]
fn ashtakavarga_table_matches_the_specification_text() {
    let text = design();
    let cols = [
        Contributor::Sun, Contributor::Moon, Contributor::Mars, Contributor::Mercury,
        Contributor::Jupiter, Contributor::Venus, Contributor::Saturn, Contributor::Lagna,
    ];
    let mut seen = 0;
    for row in rows(&text, "| BAV of |", "**Chart-independent invariants.**") {
        let Some(g) = graha(&row[0]) else { continue };
        assert_eq!(row.len(), 10, "malformed spec row for {}", g.name());
        for (i, c) in cols.iter().enumerate() {
            let spec: Vec<u8> = row[i + 1].split_whitespace().map(|x| x.parse().unwrap()).collect();
            assert_eq!(
                benefic_houses(g, *c).unwrap(), spec.as_slice(),
                "{}'s BAV from {:?} differs from DESIGN.md", g.name(), c
            );
        }
        seen += 1;
    }
    assert_eq!(seen, 7, "expected seven BAV rows in the spec");
}

#[test]
fn natural_relationships_match_the_specification_text() {
    let text = design();
    let mut seen = 0;
    for row in rows(&text, "**Natural (naisargika)", "**Temporary (tatkalika)") {
        let Some(of) = graha(&row[0]) else { continue };
        for (rel, cell) in [NaturalRelation::Friend, NaturalRelation::Neutral, NaturalRelation::Enemy]
            .into_iter()
            .zip(&row[1..4])
        {
            if cell.eq_ignore_ascii_case("none") {
                continue;
            }
            for name in cell.split(',') {
                let toward = graha(name.trim()).unwrap();
                assert_eq!(natural(of, toward), Some(rel), "{} -> {}", of.name(), toward.name());
            }
        }
        seen += 1;
    }
    assert_eq!(seen, 7);
}

#[test]
fn trimsamsa_table_matches_the_specification_text() {
    let text = design();
    let mut seen = 0;
    for row in rows(&text, "**D-30 (Parashari)", "**Derived facts QA must verify") {
        // | 0-5 | Mesha | Mars | 0-5 | Vrishabha | Venus |
        // Data rows only: skip the header and the |---| separator.
        if row.len() != 6 || !row[0].starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        for (range, sign, base) in [(&row[0], &row[1], 0.0), (&row[3], &row[4], 30.0)] {
            let (a, b) = range.split_once('-').unwrap();
            let (a, b): (f64, f64) = (a.parse().unwrap(), b.parse().unwrap());
            let want = rasi(sign);
            // Start of band, middle, and just inside the end.
            for deg in [a, (a + b) / 2.0, b - 1e-6] {
                assert_eq!(Varga::D30.sign_of(base + deg), want, "D-30 at {} + {deg}", base);
            }
        }
        seen += 1;
    }
    assert_eq!(seen, 5, "expected five D-30 bands in the spec");
}
