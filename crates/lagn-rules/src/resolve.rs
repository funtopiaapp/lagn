//! The review gate, cancellation, and topic results.
//! Specification: `docs/phase3/DESIGN.md` sections 5 and 6.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::eval::{eval, TraceEntry, Truth};
use crate::facts::FactBase;
use crate::model::{ReviewStatus, Rule, Scope};
use crate::timing::{windows, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Approved content only. The default.
    #[default]
    Production,
    /// Approved and draft content; drafts are marked.
    Review,
}

impl Mode {
    /// Does the gate admit content with this status?
    pub fn admits(self, s: ReviewStatus) -> bool {
        matches!((self, s), (_, ReviewStatus::Approved) | (Mode::Review, ReviewStatus::Draft))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Label {
    Supportive,
    Afflicted,
    Mixed,
    Neutral,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleResult {
    pub id: String,
    pub title: String,
    pub status: ReviewStatus,
    /// Who reviewed the rule, so every result can state its provenance.
    pub reviewer: Option<String>,
    pub polarity: i8,
    pub outcome: Truth,
    /// Fired and not cancelled by any effective rule.
    pub effective: bool,
    /// The effective rule that cancelled this one, if any.
    pub cancelled_by: Option<String>,
    pub text: String,
    /// What this means for the reader, in plain words (DESIGN section 4b).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,
    pub trace: Vec<TraceEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// The rule's subject resolved for this chart (and period).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<lagn_core::Graha>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopicReport {
    pub topic: String,
    pub mode: Mode,
    /// Every rule the gate admitted, in id order.
    pub results: Vec<RuleResult>,
    /// Rules held back by the gate, by status.
    pub withheld: BTreeMap<String, usize>,
    pub supporting: Vec<String>,
    pub afflicting: Vec<String>,
    pub cancelled: Vec<(String, String)>,
    /// Rules that could not be evaluated for want of a fact.
    pub unknown: Vec<String>,
    pub score: i32,
    pub label: Label,
    pub timing: Vec<Window>,
}

/// Evaluate one topic.
///
/// `rules` must already be validated (see `corpus::validate`); in particular
/// the override graph must be acyclic.
pub fn evaluate_topic(
    topic: &str,
    rules: &[Rule],
    facts: &FactBase,
    mode: Mode,
    age_range: (f64, f64),
) -> TopicReport {
    let mut withheld: BTreeMap<String, usize> = BTreeMap::new();
    let mut admitted: Vec<&Rule> = Vec::new();
    for r in rules.iter().filter(|r| r.topic == topic && r.scope == Scope::Natal) {
        if mode.admits(r.review.status) {
            admitted.push(r);
        } else {
            *withheld.entry(format!("{:?}", r.review.status).to_lowercase()).or_default() += 1;
        }
    }
    admitted.sort_by(|a, b| a.id.cmp(&b.id));

    // Evaluate every admitted rule.
    let mut outcome: HashMap<&str, Truth> = HashMap::new();
    let mut traces: HashMap<&str, Vec<TraceEntry>> = HashMap::new();
    for r in &admitted {
        let mut t = Vec::new();
        let v = eval(&r.when, facts, &mut t);
        outcome.insert(r.id.as_str(), v);
        traces.insert(r.id.as_str(), t);
    }

    // Who cancels whom, among admitted rules only.
    let mut cancellers: HashMap<&str, Vec<&str>> = HashMap::new();
    for r in &admitted {
        for target in &r.overrides {
            cancellers.entry(target.as_str()).or_default().push(r.id.as_str());
        }
    }
    for v in cancellers.values_mut() {
        v.sort();
    }

    // effective(r) = fired(r) && no effective canceller. Memoised recursion;
    // terminates because the validator guarantees an acyclic graph.
    let mut memo: HashMap<&str, (bool, Option<String>)> = HashMap::new();
    fn effective<'a>(
        id: &'a str,
        outcome: &HashMap<&'a str, Truth>,
        cancellers: &HashMap<&'a str, Vec<&'a str>>,
        memo: &mut HashMap<&'a str, (bool, Option<String>)>,
    ) -> (bool, Option<String>) {
        if let Some(v) = memo.get(id) {
            return v.clone();
        }
        let fired = outcome.get(id) == Some(&Truth::True);
        let mut by = None;
        if fired {
            if let Some(cs) = cancellers.get(id) {
                for &c in cs {
                    if effective(c, outcome, cancellers, memo).0 {
                        by = Some(c.to_string());
                        break;
                    }
                }
            }
        }
        let v = (fired && by.is_none(), by);
        memo.insert(id, v.clone());
        v
    }

    let mut results = Vec::new();
    let (mut supporting, mut afflicting, mut cancelled, mut unknown) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut score: i32 = 0;
    let (mut pos, mut neg) = (0i32, 0i32);
    let mut timed: Vec<&Rule> = Vec::new();

    for r in &admitted {
        let (eff, by) = effective(r.id.as_str(), &outcome, &cancellers, &mut memo);
        let o = outcome[r.id.as_str()];
        if eff {
            score += r.polarity as i32;
            if r.polarity > 0 {
                supporting.push(r.id.clone());
                pos += r.polarity as i32;
            } else if r.polarity < 0 {
                afflicting.push(r.id.clone());
                neg += r.polarity as i32;
            }
            if !r.timing.is_empty() {
                timed.push(r);
            }
        }
        if let Some(b) = &by {
            cancelled.push((r.id.clone(), b.clone()));
        }
        if o == Truth::Unknown {
            unknown.push(r.id.clone());
        }
        results.push(RuleResult {
            id: r.id.clone(),
            title: r.title.clone(),
            status: r.review.status,
            reviewer: r.review.reviewer.clone(),
            polarity: r.polarity,
            outcome: o,
            effective: eff,
            cancelled_by: by,
            text: r.text.en.clone(),
            impact: r.text.impact.clone(),
            trace: traces.remove(r.id.as_str()).unwrap_or_default(),
            tags: r.tags.clone(),
            subject: r.subject.and_then(|g| facts.resolve(g)),
        });
    }

    let label = match (pos > 0, neg < 0) {
        (true, true) => Label::Mixed,
        (true, false) => Label::Supportive,
        (false, true) => Label::Afflicted,
        (false, false) => Label::Neutral,
    };

    let timing = timed.iter().flat_map(|r| windows(r, facts, age_range)).collect();

    TopicReport {
        topic: topic.to_string(),
        mode,
        results,
        withheld,
        supporting,
        afflicting,
        cancelled,
        unknown,
        score,
        label,
        timing,
    }
}

// ---------------------------------------------------------------------------
// Period rules (DESIGN 6, section 3.4)
// ---------------------------------------------------------------------------

/// Net score at or below which a window is reported as sensitive. Variant P-9.
pub const SENSITIVE_AT: i32 = -2;

/// One antardasha, with the period rules that fired in it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeriodWindow {
    pub maha: lagn_core::Graha,
    pub antar: lagn_core::Graha,
    pub start_jd: f64,
    pub end_jd: f64,
    pub score: i32,
    pub sensitive: bool,
    /// Effective rules with negative polarity: what amplifies difficulty.
    pub amplifiers: Vec<RuleResult>,
    /// Effective rules with positive polarity: what negates or supports.
    pub negators: Vec<RuleResult>,
    /// Effective rules with zero polarity: context.
    pub noted: Vec<RuleResult>,
    /// Rules cancelled in this window, with their canceller.
    pub cancelled: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeriodReport {
    pub mode: Mode,
    pub withheld: BTreeMap<String, usize>,
    pub windows: Vec<PeriodWindow>,
}

/// Evaluate every admitted period rule once per antardasha within the ages.
///
/// Each window is an independent evaluation - cancellation included - with the
/// period-lord references bound to that window's lords. Ages use the chart's
/// dasha year length; windows are clipped to the range.
pub fn evaluate_periods(rules: &[Rule], facts: &mut FactBase, mode: Mode, (from_age, to_age): (f64, f64)) -> PeriodReport {
    let period_rules: Vec<Rule> = rules.iter().filter(|r| r.scope == Scope::Period).cloned().collect();
    let mut withheld: BTreeMap<String, usize> = BTreeMap::new();
    for r in &period_rules {
        if !mode.admits(r.review.status) {
            *withheld.entry(format!("{:?}", r.review.status).to_lowercase()).or_default() += 1;
        }
    }
    // Reuse the natal machinery: present the period rules as natal ones under a
    // private topic, with the period context bound.
    let as_natal: Vec<Rule> = period_rules
        .iter()
        .map(|r| Rule { scope: Scope::Natal, topic: "__period".into(), ..r.clone() })
        .collect();

    let dpy = facts.chart.settings.year_length.days();
    let lo = facts.dasha.birth_jd + from_age * dpy;
    let hi = facts.dasha.birth_jd + to_age * dpy;
    let spans: Vec<(lagn_core::Graha, lagn_core::Graha, f64, f64)> = facts
        .dasha
        .mahadashas
        .iter()
        .flat_map(|m| m.children.iter().map(move |a| (m.lord, a.lord, a.start_jd, a.end_jd)))
        .filter(|&(_, _, s, e)| e > lo && s < hi)
        .map(|(m, a, s, e)| (m, a, s.max(lo), e.min(hi)))
        .collect();

    let mut windows = Vec::new();
    for (maha, antar, start, end) in spans {
        facts.period = Some((maha, antar));
        let rep = evaluate_topic("__period", &as_natal, facts, mode, (0.0, 0.0));
        let pick = |pred: &dyn Fn(i8) -> bool| -> Vec<RuleResult> {
            rep.results.iter().filter(|r| r.effective && pred(r.polarity)).cloned().collect()
        };
        windows.push(PeriodWindow {
            maha,
            antar,
            start_jd: start,
            end_jd: end,
            score: rep.score,
            sensitive: rep.score <= SENSITIVE_AT,
            amplifiers: pick(&|p| p < 0),
            negators: pick(&|p| p > 0),
            noted: pick(&|p| p == 0),
            cancelled: rep.cancelled.clone(),
        });
    }
    facts.period = None;
    PeriodReport { mode, withheld, windows }
}
