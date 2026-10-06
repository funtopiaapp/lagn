//! Reports shared by every front end: the porutham report and the astrologer
//! review sheet. Kept here, not in a front end, so the CLI and the web server
//! can't drift apart.

use lagn_core::{BirthData, BirthMoment, Chart, ChartSettings, DerivationSettings, Graha};
use serde::{Deserialize, Serialize};

use crate::corpus::Corpus;
use crate::explain;
use crate::porutham::{
    compare_papa, dasa_sandhi, match_stars, DasaSandhi, PapaComparison, PoruthamKind,
    PoruthamResult, StarPos, Verdict,
};
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
    /// What each porutham shown measures, and what its verdict is read to
    /// mean. Empty when the explainer is not in the corpus or not reviewed.
    pub explained: Vec<PoruthamExplained>,
    /// The ten results gathered into the areas of married life they speak to,
    /// so a reader sees what the count is actually about.
    pub areas: Vec<MatchArea>,
    /// How to read the whole thing, in the order a reader needs it.
    pub reading: Vec<String>,
    /// Papasamyam: the papa count of each chart, compared. Present only when
    /// the match was run from whole charts rather than two stars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub papa: Option<PapaComparison>,
    /// Mahadasha changes in the two charts that fall close together.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dasa_sandhi: Vec<DasaSandhi>,
    /// Chevvai dosha in each chart, from that chart's own marriage rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chevvai: Option<ChevvaiBoth>,
}

/// Chevvai dosha on both sides, which is how a matching report states it: the
/// tradition reads it as cancelled when both carry it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChevvaiBoth {
    pub bride: bool,
    pub groom: bool,
    /// The counts that applied, per side, as the rules named them.
    pub bride_counts: Vec<String>,
    pub groom_counts: Vec<String>,
    /// True when both carry it, which classical practice treats as the two
    /// cancelling each other.
    pub mutual: bool,
}

/// One porutham with its explanation attached.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoruthamExplained {
    pub kind: PoruthamKind,
    pub name: String,
    pub verdict: Verdict,
    pub critical: bool,
    /// The engine's own detail line: the stars, counts or groups compared.
    pub detail: String,
    /// Completes "it tests ...".
    pub measures: String,
    /// What this verdict is read to mean, matching or not.
    pub means: String,
    pub touches: String,
}

/// One area of married life, with the poruthams that speak to it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArea {
    pub touches: String,
    pub matching: Vec<String>,
    pub not_matching: Vec<String>,
    /// True when any porutham in this area is one the texts treat as serious.
    pub critical: bool,
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
    let critical_failures: Vec<PoruthamKind> =
        results.iter().filter(|(r, _)| r.critical).map(|(r, _)| r.kind).collect();

    // The explanation of each result, from the reviewed catalogue. Without it
    // the report is a score sheet; with it a reader learns what was measured.
    let explained: Vec<PoruthamExplained> = corpus
        .porutham_explain
        .as_ref()
        .filter(|e| mode.admits(e.review.status))
        .map(|e| {
            results
                .iter()
                .filter_map(|(r, _)| {
                    let x = e.get(r.kind)?;
                    Some(PoruthamExplained {
                        kind: r.kind,
                        name: x.name.clone(),
                        verdict: r.verdict,
                        critical: r.critical,
                        detail: r.detail.clone(),
                        measures: x.measures.clone(),
                        means: match r.verdict {
                            Verdict::Matching => x.when_matching.clone(),
                            Verdict::NotMatching => x.when_not.clone(),
                            Verdict::NotEvaluated => String::new(),
                        },
                        touches: x.touches.clone(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Gathered by area, in first-appearance order so the output is stable.
    let mut areas: Vec<MatchArea> = Vec::new();
    for x in &explained {
        if x.verdict == Verdict::NotEvaluated {
            continue;
        }
        let a = match areas.iter_mut().find(|a| a.touches == x.touches) {
            Some(a) => a,
            None => {
                areas.push(MatchArea {
                    touches: x.touches.clone(),
                    matching: Vec::new(),
                    not_matching: Vec::new(),
                    critical: false,
                });
                areas.last_mut().expect("just pushed")
            }
        };
        if x.verdict == Verdict::Matching {
            a.matching.push(x.name.clone());
        } else {
            a.not_matching.push(x.name.clone());
        }
        a.critical |= x.critical;
    }

    let reading = match_reading(&areas, matched, evaluated, &critical_failures);

    MatchReport {
        mode, bride, groom, results, withheld, reviewers, matched, evaluated,
        critical_failures, explained, areas, reading,
        papa: None, dasa_sandhi: Vec::new(), chevvai: None,
    }
}

/// The whole matching report: the poruthams, plus the checks that need both
/// charts rather than both stars.
///
/// A commercial South Indian matching report prints papasamyam, dasa sandhi
/// and Chevvai dosha for both sides alongside the poruthams, so a report
/// without them is not the thing people are looking for.
pub fn match_report_full(
    corpus: &Corpus,
    bride: &lagn_core::Chart,
    groom: &lagn_core::Chart,
    mode: Mode,
) -> MatchReport {
    let star = |c: &lagn_core::Chart| StarPos {
        nakshatra: c.janma_nakshatra().nakshatra,
        rasi: c.janma_rasi(),
    };
    let mut r = match_report(corpus, star(bride), star(groom), mode);
    r.papa = Some(compare_papa(bride, groom));
    // Eighteen months is the window Tamil reports commonly use.
    r.dasa_sandhi = dasa_sandhi(bride, groom, 18.0);
    r.chevvai = Some(chevvai_both(corpus, bride, groom, mode));

    // Those three change what the reading has to say, so it is composed again
    // with them in hand.
    r.reading = match_reading(&r.areas, r.matched, r.evaluated, &r.critical_failures);
    if let Some(p) = &r.papa {
        r.reading.insert(
            r.reading.len() - 1,
            if p.balanced {
                format!(
                    "Papasamyam: {} papa placements on the groom's side against {} on the bride's. \
                     The groom's count is not the lower of the two, which is what the check asks \
                     for.",
                    p.groom.total, p.bride.total
                )
            } else {
                format!(
                    "Papasamyam: {} papa placements on the groom's side against {} on the \
                     bride's. The check asks that the groom's count not be the lower, and here it \
                     is lower by {}. Weighted variants of this count exist and differ between \
                     sources, so take this one to an astrologer rather than as settled.",
                    p.groom.total, p.bride.total, -p.difference
                )
            },
        );
    }
    if let Some(c) = &r.chevvai {
        let line = match (c.bride, c.groom) {
            (false, false) => "Chevvai dosha: neither chart carries it.".to_string(),
            (true, true) => "Chevvai dosha: both charts carry it, which classical practice reads \
                             as the two cancelling each other."
                .to_string(),
            (true, false) => "Chevvai dosha: the bride's chart carries it and the groom's does \
                              not, so it is not cancelled between them. This is one to put to an \
                              astrologer."
                .to_string(),
            (false, true) => "Chevvai dosha: the groom's chart carries it and the bride's does \
                              not, so it is not cancelled between them. This is one to put to an \
                              astrologer."
                .to_string(),
        };
        r.reading.insert(r.reading.len() - 1, line);
    }
    if !r.dasa_sandhi.is_empty() {
        r.reading.insert(
            r.reading.len() - 1,
            format!(
                "Dasa sandhi: {} point{} where both charts change major period within eighteen \
                 months of each other. The texts read two turnings at once as a strain on a new \
                 household, and the dates are listed below so they can be weighed against when a \
                 marriage is actually planned.",
                r.dasa_sandhi.len(),
                if r.dasa_sandhi.len() == 1 { "" } else { "s" },
            ),
        );
    }
    r
}

/// Chevvai dosha on each side, read from each chart's own marriage rules so
/// the match and the individual reading can never disagree.
fn chevvai_both(
    corpus: &Corpus,
    bride: &lagn_core::Chart,
    groom: &lagn_core::Chart,
    mode: Mode,
) -> ChevvaiBoth {
    let counts = |c: &lagn_core::Chart| -> Vec<String> {
        let f = FactBase::new(c.clone(), Default::default(), NativeInfo { sex: None });
        let rep = evaluate_topic("marriage", &corpus.rules, &f, mode, (18.0, 45.0));
        rep.results
            .iter()
            .filter(|r| r.effective && r.polarity < 0 && r.id.starts_with("marriage.kuja."))
            .map(|r| r.title.clone())
            .collect()
    };
    let (b, g) = (counts(bride), counts(groom));
    ChevvaiBoth {
        bride: !b.is_empty(),
        groom: !g.is_empty(),
        mutual: !b.is_empty() && !g.is_empty(),
        bride_counts: b,
        groom_counts: g,
    }
}

/// How to read the whole match, in the order a reader needs it: what the count
/// is, then what the areas say, then what the tradition does about the two it
/// treats seriously - and always that the decision is not the app's.
fn match_reading(
    areas: &[MatchArea],
    matched: usize,
    evaluated: usize,
    critical: &[PoruthamKind],
) -> Vec<String> {
    if evaluated == 0 {
        return Vec::new();
    }
    let mut out = vec![format!(
        "{matched} of the {evaluated} poruthams examined match. The count is the start of the \
         reading, not the whole of it: which ones matched matters more than how many, and the \
         areas below say which.",
    )];

    let clear: Vec<&MatchArea> = areas.iter().filter(|a| a.not_matching.is_empty()).collect();
    if !clear.is_empty() {
        let names: Vec<String> = clear.iter().map(|a| a.touches.clone()).collect();
        out.push(format!("Nothing is flagged in {}.", join_and(&names)));
    }
    for a in areas.iter().filter(|a| !a.not_matching.is_empty()) {
        let mut t = format!(
            "On {}: {} {} not match",
            a.touches,
            join_and(&a.not_matching),
            if a.not_matching.len() == 1 { "does" } else { "do" },
        );
        if !a.matching.is_empty() {
            t.push_str(&format!(", while {} {} match", join_and(&a.matching),
                if a.matching.len() == 1 { "does" } else { "do" }));
        }
        t.push('.');
        out.push(t);
    }

    if !critical.is_empty() {
        let names: Vec<String> = critical.iter().map(|k| k.name().to_string()).collect();
        out.push(format!(
            "{} {} among the poruthams the texts treat most seriously. Where one of those does \
             not match, the tradition's own course is to put the match to an astrologer rather \
             than to weigh it against the count of the others.",
            join_and(&names),
            if critical.len() == 1 { "is" } else { "are" },
        ));
    }
    out.push(
        "This is the porutham procedure and nothing more. It does not read either chart's own \
         marriage indications, and the decision rests with the family's astrologer."
            .to_string(),
    );
    out
}

/// "a", "a and b", "a, b and c".
fn join_and(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
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
