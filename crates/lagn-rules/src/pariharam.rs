//! Selecting pariharams for what a reading found.
//! Specification: `docs/phase6/DESIGN.md` sections 2 and 5.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::catalogue::Pariharam;
use crate::corpus::Corpus;
use crate::resolve::{Mode, RuleResult};

/// A pariharam, with the findings that brought it up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Suggested {
    pub pariharam: Pariharam,
    /// Titles of the rules or transits that triggered it.
    pub because: Vec<String>,
}

/// The tags an effective afflicting result contributes: its own tags plus
/// `graha:<subject>`.
fn tags_of(r: &RuleResult) -> Vec<String> {
    let mut t = r.tags.clone();
    if let Some(g) = r.subject {
        t.push(format!("graha:{}", serde_json::to_value(g).unwrap().as_str().unwrap()));
    }
    t
}

/// Pariharams for the afflicting results and challenging transits given,
/// gated by review like everything else. Deterministic: catalogue order.
pub fn suggest<'a>(
    corpus: &Corpus,
    afflicting: impl IntoIterator<Item = &'a RuleResult>,
    transits: &[(String, &'static str)],
    mode: Mode,
) -> Vec<Suggested> {
    let mut reasons: Vec<(String, String)> = Vec::new();
    for r in afflicting {
        if r.effective && r.polarity < 0 {
            for t in tags_of(r) {
                reasons.push((t, r.title.clone()));
            }
        }
    }
    for (title, tag) in transits {
        reasons.push((tag.to_string(), title.clone()));
    }
    corpus
        .pariharams
        .iter()
        .filter(|p| mode.admits(p.review.status))
        .filter_map(|p| {
            let because: BTreeSet<String> = reasons
                .iter()
                .filter(|(t, _)| p.triggers.contains(t))
                .map(|(_, why)| why.clone())
                .collect();
            (!because.is_empty()).then(|| Suggested { pariharam: p.clone(), because: because.into_iter().collect() })
        })
        .collect()
}
