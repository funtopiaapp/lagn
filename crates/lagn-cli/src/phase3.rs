//! Phase 3 output: rule validation, topic reports, porutham matching and the
//! astrologer review sheet.

use std::path::{Path, PathBuf};

use lagn_core::{jd_to_civil, BirthData, BirthMoment, Chart, ChartSettings};
use lagn_rules::corpus::{summary, Corpus};
use lagn_rules::porutham::{match_stars, PoruthamResult, StarPos, Verdict};
pub use lagn_rules::report::{match_report, review_sheet, MatchReport};
use lagn_rules::resolve::{Label, Mode, TopicReport};
use lagn_rules::{FactBase, ReviewStatus, Truth};
use serde::Serialize;

/// Locate the corpus: explicit flag, CWD, then the repo root baked in at build.
pub fn resolve_corpus(explicit: Option<&str>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let candidates: Vec<PathBuf> = match explicit {
        Some(p) => vec![p.into()],
        None => vec![
            "corpus".into(),
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus"),
        ],
    };
    for c in &candidates {
        if c.is_dir() {
            return Ok(c.canonicalize()?);
        }
    }
    Err(format!(
        "no corpus directory found (tried {}); pass --corpus",
        candidates.iter().map(|c| c.display().to_string()).collect::<Vec<_>>().join(", ")
    )
    .into())
}

pub fn load(explicit: Option<&str>) -> Result<Corpus, Box<dyn std::error::Error>> {
    Ok(lagn_rules::load(&resolve_corpus(explicit)?)?)
}

pub fn print_validation(c: &Corpus) {
    let s = summary(c);
    println!("corpus valid: {} rules", s.rules);
    for (t, n) in &s.by_topic {
        println!("  topic {t}: {n}");
    }
    for (st, n) in &s.by_status {
        println!("  status {st}: {n}");
    }
    if let Some(p) = &c.porutham {
        let approved = p.values().filter(|r| r.status == ReviewStatus::Approved).count();
        println!("  poruthams: {approved}/10 approved");
    }
}

// ---------------------------------------------------------------------------
// topic
// ---------------------------------------------------------------------------

fn date(jd: f64, tz: f64) -> String {
    let d = jd_to_civil(jd, tz);
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

pub fn print_topic(r: &TopicReport, facts: &FactBase) {
    let tz = facts.chart.birth.moment.utc_offset_hours;
    println!("\n  Topic: {}   mode: {:?}", r.topic, r.mode);
    if !r.withheld.is_empty() {
        let parts: Vec<String> = r.withheld.iter().map(|(k, v)| format!("{v} {k}")).collect();
        println!("  Withheld by the review gate: {}", parts.join(", "));
        if r.mode == Mode::Production && r.results.is_empty() {
            println!("  No approved rules yet. Nothing is shown until an astrologer approves rules.");
            println!("  Use --review to see draft rules.\n");
            return;
        }
    }
    let by_id = |id: &str| r.results.iter().find(|x| x.id == id).unwrap();
    let draft = |id: &str| if by_id(id).status == ReviewStatus::Draft { " [DRAFT]" } else { "" };

    let section = |title: &str, ids: &[String]| {
        if ids.is_empty() {
            return;
        }
        println!("\n  {title}");
        for id in ids {
            let x = by_id(id);
            println!("    {:+} {}{}", x.polarity, x.title, draft(id));
            println!("       {}", x.text);
            for t in &x.trace {
                println!("       - {} -> {:?}  ({})", t.condition, t.value, t.facts);
            }
        }
    };
    section("Supporting", &r.supporting);
    section("Afflicting", &r.afflicting);

    let info: Vec<String> = r
        .results
        .iter()
        .filter(|x| x.effective && x.polarity == 0 && !r.timing.iter().any(|w| w.rule == x.id))
        .map(|x| x.id.clone())
        .collect();
    section("Noted (not scored)", &info);

    if !r.cancelled.is_empty() {
        println!("\n  Cancelled");
        for (id, by) in &r.cancelled {
            println!("    {}{}\n       cancelled by: {}", by_id(id).title, draft(id), by_id(by).title);
        }
    }
    if !r.unknown.is_empty() {
        println!("\n  Could not evaluate (missing facts)");
        for id in &r.unknown {
            let x = by_id(id);
            let why: Vec<&str> = x.trace.iter().filter(|t| t.value == Truth::Unknown).map(|t| t.facts.as_str()).collect();
            println!("    {} - {}", x.title, why.join("; "));
        }
    }
    let label = match r.label {
        Label::Supportive => "supportive",
        Label::Afflicted => "afflicted",
        Label::Mixed => "mixed: supporting and afflicting factors are both present",
        Label::Neutral => "neutral",
    };
    println!("\n  Score {:+}  ({label})", r.score);

    if !r.timing.is_empty() {
        println!("\n  Periods to examine");
        for w in &r.timing {
            let m: Vec<&str> = w.matched.iter().map(|g| g.name()).collect();
            println!(
                "    {} -> {}   {} / {}   (via {})",
                date(w.start_jd, tz), date(w.end_jd, tz), w.maha.name(), w.antar.name(), m.join(", ")
            );
        }
    }
    println!();
}

// ---------------------------------------------------------------------------
// match
// ---------------------------------------------------------------------------

/// Parse "YYYY-MM-DD HH:MM[:SS] LAT LON TZ".
pub fn parse_person(spec: &str) -> Result<Chart, Box<dyn std::error::Error>> {
    let p: Vec<&str> = spec.split_whitespace().collect();
    if p.len() != 5 {
        return Err(format!("expected \"YYYY-MM-DD HH:MM[:SS] LAT LON TZ\", got {spec:?}").into());
    }
    let d: Vec<&str> = p[0].split('-').collect();
    let t: Vec<&str> = p[1].split(':').collect();
    if d.len() != 3 || !(2..=3).contains(&t.len()) {
        return Err(format!("bad date or time in {spec:?}").into());
    }
    let birth = BirthData {
        moment: BirthMoment {
            year: d[0].parse()?, month: d[1].parse()?, day: d[2].parse()?,
            hour: t[0].parse()?, minute: t[1].parse()?,
            second: if t.len() == 3 { t[2].parse()? } else { 0.0 },
            utc_offset_hours: p[4].parse()?,
        },
        latitude: p[2].parse()?,
        longitude: p[3].parse()?,
        place_name: String::new(),
    };
    Ok(Chart::compute(birth, ChartSettings::default())?)
}

pub fn star_of(c: &Chart) -> StarPos {
    StarPos { nakshatra: c.janma_nakshatra().nakshatra, rasi: c.janma_rasi() }
}

pub fn print_match(m: &MatchReport) {
    println!("\n  Porutham   mode: {:?}", m.mode);
    println!("  Bride: {} ({}), {}", m.bride.nakshatra.name(), m.bride.nakshatra.tamil_name(), m.bride.rasi.name());
    println!("  Groom: {} ({}), {}", m.groom.nakshatra.name(), m.groom.nakshatra.tamil_name(), m.groom.rasi.name());
    if m.withheld > 0 {
        println!("  Withheld by the review gate: {} poruthams", m.withheld);
    }
    if m.results.is_empty() {
        println!("  No approved poruthams yet. Use --review to see drafts.\n");
        return;
    }
    println!();
    for (r, st) in &m.results {
        let v = match r.verdict {
            Verdict::Matching => "matching",
            Verdict::NotMatching => if r.critical { "NOT MATCHING (critical)" } else { "not matching" },
            Verdict::NotEvaluated => "not evaluated",
        };
        let d = if *st == ReviewStatus::Draft { " [DRAFT]" } else { "" };
        println!("    {:<14} {:<24} {}{}", r.kind.name(), v, r.detail, d);
    }
    println!("\n  {}/{} evaluated poruthams match", m.matched, m.evaluated);
    if !m.critical_failures.is_empty() {
        let names: Vec<&str> = m.critical_failures.iter().map(|k| k.name()).collect();
        println!("  Critical: {}", names.join(", "));
    }
    println!("  This is the porutham procedure only; the decision rests with the family's astrologer.\n");
}

/// Raw procedure output for all 11,664 pada pairs, ungated, for the QA oracle.
pub fn match_table_json() -> String {
    #[derive(Serialize)]
    struct Row {
        bride: StarPos,
        groom: StarPos,
        results: Vec<PoruthamResult>,
    }
    let padas = StarPos::all_padas();
    let rows: Vec<Row> = padas
        .iter()
        .flat_map(|&b| padas.iter().map(move |&g| Row { bride: b, groom: g, results: match_stars(b, g) }))
        .collect();
    serde_json::to_string(&rows).unwrap()
}


pub fn print_write_up(w: &lagn_rules::writeup::WriteUp) {
    println!();
    for p in &w.summary {
        println!("  {p}\n");
    }
    for s in &w.sections {
        println!("  == {} ==\n", s.heading);
        for p in &s.paragraphs {
            println!("  {p}");
        }
        for p in &s.points {
            println!("   * {} ({:+})\n     {}", p.title, p.polarity, p.text);
            if let Some(m) = &p.meaning {
                println!("     What this means for you: {m}");
            }
            println!("     Because: {}", p.because.join("; "));
        }
        println!();
    }
}
