//! Reports shared by every front end: the porutham report and the astrologer
//! review sheet. Kept here, not in a front end, so the CLI and the web server
//! can't drift apart.

use lagn_core::{BirthData, BirthMoment, Chart, ChartSettings, DerivationSettings, Graha};
use serde::{Deserialize, Serialize};

use crate::corpus::Corpus;
use crate::explain;
use crate::porutham::{match_stars, PoruthamKind, PoruthamResult, StarPos, Verdict};
use crate::resolve::{evaluate_topic, Mode, TopicReport};
use crate::model::{Rule, Scope};
use crate::{FactBase, NativeInfo, ReviewStatus, Truth};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchReport {
    pub mode: Mode,
    pub bride: StarPos,
    pub groom: StarPos,
    /// Poruthams the gate admitted.
    pub results: Vec<(PoruthamResult, ReviewStatus)>,
    pub withheld: usize,
    /// Distinct reviewers of the poruthams shown.
    pub reviewers: Vec<String>,
    pub matched: usize,
    pub evaluated: usize,
    pub critical_failures: Vec<PoruthamKind>,
}

pub fn match_report(corpus: &Corpus, bride: StarPos, groom: StarPos, mode: Mode) -> MatchReport {
    let review = |k: PoruthamKind| corpus.porutham.as_ref().and_then(|m| m.get(&k));
    let status = |k: PoruthamKind| review(k).map(|r| r.status).unwrap_or(ReviewStatus::Draft);
    let mut reviewers: Vec<String> = Vec::new();
    let mut results = Vec::new();
    let mut withheld = 0;
    for res in match_stars(bride, groom) {
        let st = status(res.kind);
        if mode.admits(st) {
            if let Some(r) = review(res.kind).and_then(|r| r.reviewer.clone()) {
                if !reviewers.contains(&r) {
                    reviewers.push(r);
                }
            }
            results.push((res, st));
        } else {
            withheld += 1;
        }
    }
    let evaluated = results.iter().filter(|(r, _)| r.verdict != Verdict::NotEvaluated).count();
    let matched = results.iter().filter(|(r, _)| r.verdict == Verdict::Matching).count();
    let critical_failures = results.iter().filter(|(r, _)| r.critical).map(|(r, _)| r.kind).collect();
    MatchReport { mode, bride, groom, results, withheld, reviewers, matched, evaluated, critical_failures }
}


/// Deterministic sample of South Indian births for firing-rate statistics.
fn sample_facts(n: usize) -> Vec<FactBase> {
    let mut x: u64 = 0x5EED_2026_0921_0003;
    let mut next = || {
        x ^= x >> 12; x ^= x << 25; x ^= x >> 27;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let r = |v: u64, lo: u64, hi: u64| lo + v % (hi - lo + 1);
        let birth = BirthData {
            moment: BirthMoment {
                year: r(next(), 1920, 2025) as i32, month: r(next(), 1, 12) as u32,
                day: r(next(), 1, 28) as u32, hour: r(next(), 0, 23) as u32,
                minute: r(next(), 0, 59) as u32, second: 0.0, utc_offset_hours: 5.5,
            },
            latitude: 8.0 + (next() % 1200) as f64 / 100.0,
            longitude: 74.0 + (next() % 1000) as f64 / 100.0,
            place_name: String::new(),
        };
        if let Ok(c) = Chart::compute(birth, ChartSettings::default()) {
            out.push(FactBase::new(c, DerivationSettings::default(), NativeInfo::default()));
        }
    }
    out
}

pub fn review_sheet(corpus: &Corpus, topic: &str, samples: usize) -> String {
    let mut facts = sample_facts(samples);
    let mut rules: Vec<_> = corpus.rules.iter().filter(|r| r.topic == topic).collect();
    // Period rules are evaluated for every (mahadasha, antardasha) lord pair,
    // presented as natal rules with the period context bound - the same
    // reduction evaluate_periods uses.
    let period = rules.iter().any(|r| r.scope == Scope::Period);
    let reports: Vec<TopicReport> = if period {
        let as_natal: Vec<Rule> =
            rules.iter().map(|r| Rule { scope: Scope::Natal, ..(*r).clone() }).collect();
        let mut out = Vec::new();
        for f in facts.iter_mut() {
            for maha in Graha::ALL {
                for antar in Graha::ALL {
                    f.period = Some((maha, antar));
                    out.push(evaluate_topic(topic, &as_natal, f, Mode::Review, (0.0, 0.0)));
                }
            }
            f.period = None;
        }
        out
    } else {
        facts.iter().map(|f| evaluate_topic(topic, &corpus.rules, f, Mode::Review, (18.0, 45.0))).collect()
    };
    let denominator = reports.len();
    rules.sort_by(|a, b| a.id.cmp(&b.id));

    let mut out = String::new();
    out.push_str(&format!("# Review sheet: {topic}\n\n"));
    out.push_str("For the reviewing astrologer. Each rule below is exactly what the engine evaluates:\n");
    out.push_str("the \"Condition\" line is generated from the rule itself, not written separately.\n\n");
    out.push_str("To approve a rule, set `review.status` to `approved` and fill in `reviewer` and\n");
    out.push_str("`date` in the corpus file. To reject, set `rejected`. Please correct anything wrong:\n");
    out.push_str("the condition, the polarity (-3 to +3), the cancellations, or the wording.\n\n");
    out.push_str(&format!(
        "Firing rates come from {samples} sample births in South India, 1920-2025, evaluated in review mode{}.\n\n",
        if period { ", each under all 81 mahadasha/antardasha lord pairs" } else { "" }
    ));
    out.push_str(&format!("| Rules | {} |\n|---|---|\n", rules.len()));
    // Count each status: "not approved" is not the same as "draft".
    let count = |s: ReviewStatus| rules.iter().filter(|r| r.review.status == s).count();
    out.push_str(&format!(
        "| Approved | {} |\n| Draft | {} |\n| Rejected | {} |\n\n",
        count(ReviewStatus::Approved), count(ReviewStatus::Draft), count(ReviewStatus::Rejected)
    ));

    for r in rules {
        let fired = reports.iter().filter(|rep| rep.results.iter().any(|x| x.id == r.id && x.outcome == Truth::True)).count();
        let eff = reports.iter().filter(|rep| rep.results.iter().any(|x| x.id == r.id && x.effective)).count();
        let unk = reports.iter().filter(|rep| rep.results.iter().any(|x| x.id == r.id && x.outcome == Truth::Unknown)).count();
        let pct = |n: usize| 100.0 * n as f64 / denominator as f64;

        out.push_str(&format!("## {}\n\n", r.title));
        out.push_str(&format!("- **Id:** `{}`\n", r.id));
        out.push_str(&format!("- **Status:** {:?}\n", r.review.status));
        out.push_str(&format!("- **Tradition:** {:?}\n", r.tradition));
        out.push_str(&format!("- **Polarity:** {:+}\n", r.polarity));
        out.push_str(&format!("- **Condition:** {}\n", explain::condition(&r.when)));
        if !r.overrides.is_empty() {
            out.push_str(&format!("- **Cancels:** {}\n", r.overrides.iter().map(|o| format!("`{o}`")).collect::<Vec<_>>().join(", ")));
        }
        if !r.timing.is_empty() {
            let t: Vec<String> = r.timing.iter().map(explain::graha).collect();
            out.push_str(&format!("- **Timing lords:** {}\n", t.join(", ")));
        }
        out.push_str(&format!("- **Text shown to users:** {}\n", r.text.en));
        out.push_str(&format!("- **Source:** {}\n", r.source.reference.as_deref().unwrap_or("none supplied")));
        out.push_str(&format!("- **Provenance note:** {}\n", r.source.note));
        out.push_str(&format!(
            "- **Fires in:** {:.1}% of samples ({:.1}% after cancellations){}\n\n",
            pct(fired), pct(eff),
            if unk > 0 { format!("; could not evaluate in {:.1}%", pct(unk)) } else { String::new() }
        ));
    }

    if topic == "marriage" {
        out.push_str("## Poruthams\n\n");
        out.push_str("The ten-porutham procedure is specified in `docs/phase3/DESIGN.md` section 8,\n");
        out.push_str("with its tables and variant choices (P-1 to P-8). Approve each porutham in\n");
        out.push_str("`corpus/marriage/porutham.review.json`. Match rates are over all 11,664\n");
        out.push_str("bride and groom pada combinations.\n\n");
        let padas = StarPos::all_padas();
        let total = (padas.len() * padas.len()) as f64;
        out.push_str("| Porutham | Status | Matching |\n|---|---|---|\n");
        for k in PoruthamKind::ALL {
            let n = padas.iter().flat_map(|&b| padas.iter().map(move |&g| match_stars(b, g)))
                .filter(|rs| rs.iter().any(|r| r.kind == k && r.verdict == Verdict::Matching))
                .count();
            let st = corpus.porutham.as_ref().and_then(|m| m.get(&k)).map(|r| format!("{:?}", r.status)).unwrap_or("-".into());
            let rate = if k == PoruthamKind::Vasya { "blocked: table to be supplied".to_string() } else { format!("{:.1}%", 100.0 * n as f64 / total) };
            out.push_str(&format!("| {} | {} | {} |\n", k.name(), st, rate));
        }
        out.push('\n');
    }
    out
}
