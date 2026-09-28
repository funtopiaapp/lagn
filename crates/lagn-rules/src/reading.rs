//! Assembling a sensitive-periods reading: each dasha window with the houses
//! it brings into focus, the transits running through it, the pariharams its
//! findings call for, and a plain-language explanation. Every sentence is a
//! fixed template filled from computed facts and reviewed corpus text.
//! Specification: `docs/phase6/DESIGN.md` section 3.6.

use std::collections::BTreeMap;

use lagn_core::transit::{ingresses, TRANSIT_GRAHAS};
use lagn_core::{Chart, Ephemeris, Graha, Ingress, Rasi, TransitWindow};
use serde::{Deserialize, Serialize};

use crate::catalogue::{transit_tag, Bhavas};
use crate::corpus::Corpus;
use crate::facts::FactBase;
use crate::pariharam::{suggest, Suggested};
use crate::resolve::{evaluate_periods, Mode, PeriodWindow};

/// Why a house is in focus during a graha's period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusVia {
    /// The graha occupies the house.
    Occupies,
    /// The graha rules the house (from the lagna).
    Rules,
    /// Rahu and Ketu rule no sign; classically they give the results of the
    /// lord of the sign they occupy (their dispositor), which rules this house.
    Dispositor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FocusHouse {
    pub house: u8,
    pub name: String,
    pub significations: Vec<String>,
    pub via: Vec<FocusVia>,
}

/// A transit overlapping a period window, clipped to the window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitNote {
    pub transit: TransitWindow,
    pub label: String,
    pub start_jd: f64,
    pub end_jd: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeriodDetail {
    #[serde(flatten)]
    pub window: PeriodWindow,
    /// Houses the antardasha lord brings into focus, in house order.
    pub focus: Vec<FocusHouse>,
    /// Challenging transits overlapping the window.
    pub pressures: Vec<TransitNote>,
    /// Supportive transits overlapping the window.
    pub supports: Vec<TransitNote>,
    pub pariharams: Vec<Suggested>,
    pub explanation: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensitivePeriods {
    pub mode: Mode,
    pub withheld: BTreeMap<String, usize>,
    pub windows: Vec<PeriodDetail>,
}

/// Houses a graha's period brings into focus: the house it occupies and the
/// houses it rules, or for Rahu and Ketu the houses their dispositor rules.
pub fn focus_houses(chart: &Chart, g: Graha, bhavas: Option<&Bhavas>) -> Vec<FocusHouse> {
    let lagna = chart.lagna.rasi;
    let sign = chart.placement(g).rasi;
    let mut via: BTreeMap<u8, Vec<FocusVia>> = BTreeMap::new();
    via.entry(lagna.houses_to(sign)).or_default().push(FocusVia::Occupies);
    let (ruler, how) = match g {
        Graha::Rahu | Graha::Ketu => (sign.lord(), FocusVia::Dispositor),
        _ => (g, FocusVia::Rules),
    };
    for h in lagn_core::functional::houses_ruled(lagna, ruler) {
        via.entry(h).or_default().push(how);
    }
    via.into_iter()
        .map(|(house, via)| {
            let meaning = bhavas.and_then(|b| b.houses.iter().find(|m| m.house == house));
            FocusHouse {
                house,
                name: meaning.map(|m| m.name.clone()).unwrap_or_default(),
                significations: meaning.map(|m| m.significations.clone()).unwrap_or_default(),
                via,
            }
        })
        .collect()
}

fn overlapping(transits: &[TransitWindow], start: f64, end: f64, supportive: bool) -> Vec<TransitNote> {
    transits
        .iter()
        .filter(|t| t.supportive == supportive && t.end_jd > start && t.start_jd < end)
        .map(|t| TransitNote { transit: *t, label: t.kind.label().to_string(), start_jd: t.start_jd.max(start), end_jd: t.end_jd.min(end) })
        .collect()
}

fn join_and(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        // Items may contain "and" themselves ("debts, disputes and
        // competition"), so separate them with semicolons.
        [init @ .., last] if items.iter().any(|i| i.contains(" and ") || i.contains(',')) => {
            format!("{}; and {}", init.join("; "), last)
        }
        [init @ .., last] => format!("{} and {}", init.join(", "), last),
    }
}

/// The explanation for one window. Fixed templates only.
fn explain(w: &PeriodWindow, focus: &[FocusHouse], pressures: &[TransitNote], supports: &[TransitNote]) -> Vec<String> {
    let mut out = Vec::new();
    let areas: Vec<String> = focus
        .iter()
        .filter(|f| !f.significations.is_empty())
        .map(|f| format!("{} (house {})", join_and(&f.significations[..f.significations.len().min(3)]), f.house))
        .collect();
    let lead = format!("{} mahadasha, {} bhukti.", w.maha.name(), w.antar.name());
    if areas.is_empty() {
        out.push(lead);
    } else {
        out.push(format!("{lead} The matters of this period are {}.", join_and(&areas)));
    }
    if w.sensitive {
        out.push("This is a sensitive period: the factors below that call for care outweigh those that support.".into());
    }
    // The plain-language line is what a reader needs; the technical one follows.
    let line = |r: &crate::resolve::RuleResult| match &r.impact {
        Some(i) => format!("{i} ({})", r.text.trim_end_matches('.')),
        None => r.text.clone(),
    };
    for r in &w.amplifiers {
        out.push(format!("Calls for care: {}", line(r)));
    }
    for r in &w.negators {
        out.push(format!("Supports: {}", line(r)));
    }
    for (a, b) in &w.cancelled {
        out.push(format!("Eased: the rule {a} is cancelled by {b}."));
    }
    // A transit that re-enters its sign during retrogression appears as more
    // than one stretch; the explanation names each kind once.
    let mut seen = std::collections::BTreeSet::new();
    for t in pressures {
        if seen.insert(&t.label) {
            let extra = if t.transit.bav_supports == Some(true) { " Ashtakavarga support makes it milder." } else { "" };
            out.push(format!("Transit that adds pressure: {}.{extra}", t.label));
        }
    }
    for t in supports {
        if seen.insert(&t.label) {
            out.push(format!("Transit that eases: {}.", t.label));
        }
    }
    out
}

/// The sensitive-periods reading over an age range. `transits` are the
/// gochara windows over (at least) the same range.
pub fn sensitive_periods(
    corpus: &Corpus,
    facts: &mut FactBase,
    mode: Mode,
    ages: (f64, f64),
    transits: &[TransitWindow],
) -> SensitivePeriods {
    let report = evaluate_periods(&corpus.rules, facts, mode, ages);
    let windows = report
        .windows
        .into_iter()
        .map(|w| {
            let focus = focus_houses(&facts.chart, w.antar, corpus.bhavas.as_ref());
            let pressures = overlapping(transits, w.start_jd, w.end_jd, false);
            let supports = overlapping(transits, w.start_jd, w.end_jd, true);
            let tags: Vec<(String, &'static str)> = pressures
                .iter()
                .filter_map(|t| transit_tag(t.transit.kind).map(|tag| (t.label.clone(), tag)))
                .collect();
            let pariharams = suggest(corpus, &w.amplifiers, &tags, mode);
            let explanation = explain(&w, &focus, &pressures, &supports);
            PeriodDetail { window: w, focus, pressures, supports, pariharams, explanation }
        })
        .collect();
    SensitivePeriods { mode: report.mode, withheld: report.withheld, windows }
}

/// Saturn, Jupiter and node transits from the natal Moon over an age range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitReport {
    pub moon_sign: Rasi,
    pub start_jd: f64,
    pub end_jd: f64,
    pub ingresses: Vec<Ingress>,
    pub windows: Vec<TransitWindow>,
}

pub fn transits(c: &Chart, from_age: f64, to_age: f64) -> Result<TransitReport, lagn_core::EphemError> {
    let dpy = c.settings.year_length.days();
    let (start, end) = (c.jd_ut + from_age * dpy, c.jd_ut + to_age * dpy);
    let eph = Ephemeris::new(c.settings.ayanamsa, c.settings.node_type);
    let mut all = Vec::new();
    for g in TRANSIT_GRAHAS {
        all.extend(ingresses(&eph, g, start, end)?);
    }
    all.sort_by(|a, b| a.jd_ut.total_cmp(&b.jd_ut));
    Ok(TransitReport { moon_sign: c.janma_rasi(), start_jd: start, end_jd: end, ingresses: all, windows: c.gochara(start, end)? })
}
