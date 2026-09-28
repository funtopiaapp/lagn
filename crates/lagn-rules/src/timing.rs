//! Dasha timing windows. Specification: `docs/phase3/DESIGN.md` section 7.

use lagn_core::Graha;
use serde::{Deserialize, Serialize};

use crate::facts::FactBase;
use crate::model::Rule;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Window {
    pub rule: String,
    pub start_jd: f64,
    pub end_jd: f64,
    pub maha: Graha,
    pub antar: Graha,
    /// Which of the rule's timing lords this period activates.
    pub matched: Vec<Graha>,
}

/// Antardashas within `[from_age, to_age)` whose maha or antar lord is one of
/// the rule's timing lords. Ages use the chart's dasha year length. Periods
/// are clipped to the range.
pub fn windows(rule: &Rule, f: &FactBase, (from_age, to_age): (f64, f64)) -> Vec<Window> {
    let mut lords: Vec<Graha> = rule.timing.iter().filter_map(|&r| f.resolve(r)).collect();
    lords.sort();
    lords.dedup();

    let dpy = f.chart.settings.year_length.days();
    let lo = f.dasha.birth_jd + from_age * dpy;
    let hi = f.dasha.birth_jd + to_age * dpy;

    let mut out = Vec::new();
    for maha in &f.dasha.mahadashas {
        for antar in &maha.children {
            if antar.end_jd <= lo || antar.start_jd >= hi {
                continue;
            }
            let matched: Vec<Graha> = lords
                .iter()
                .copied()
                .filter(|&g| g == maha.lord || g == antar.lord)
                .collect();
            if matched.is_empty() {
                continue;
            }
            out.push(Window {
                rule: rule.id.clone(),
                start_jd: antar.start_jd.max(lo),
                end_jd: antar.end_jd.min(hi),
                maha: maha.lord,
                antar: antar.lord,
                matched,
            });
        }
    }
    out
}
