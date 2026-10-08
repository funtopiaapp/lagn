//! Reviewed content other than rules: the topic catalogue, bhava
//! significations. Specification:
//! `docs/phase6/DESIGN.md` sections 2 and 5.

use serde::{Deserialize, Serialize};

use crate::model::Review;

/// One readable topic, e.g. "career".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopicMeta {
    pub id: String,
    pub title: String,
    pub summary: String,
    /// Shown with every reading of this topic. Required for health and vitality.
    #[serde(default)]
    pub disclaimer: Option<String>,
    /// A dosha readers come to this topic to ask about. The write-up states
    /// its verdict either way, so the summary need not promise one that may
    /// never appear.
    #[serde(default)]
    pub dosha: Option<DoshaNote>,
    /// Default age range for timing windows.
    pub ages: [f64; 2],
    /// What the write-up explains (DESIGN section 4a).
    #[serde(default)]
    pub focus: Focus,
    pub review: Review,
}

/// A dosha the topic reports on whether or not it is present. Its rules are
/// found by the prefix they share, so the corpus decides which rules judge it
/// and no topic name is written into the engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DoshaNote {
    /// As a reader knows it, e.g. "Chevvai dosha".
    pub name: String,
    /// Shared prefix of the rules that judge it, e.g. "marriage.kuja.".
    pub rule_prefix: String,
}

/// The bhavas, karakas and divisional charts a topic's write-up explains.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Focus {
    #[serde(default)]
    pub houses: Vec<u8>,
    #[serde(default)]
    pub karakas: Vec<Karaka>,
    #[serde(default)]
    pub vargas: Vec<VargaFocus>,
    /// Add the Ketu-Rahu section: what is carried, against what is asked now.
    #[serde(default)]
    pub karmic_axis: bool,
    /// Add the section linking these houses to the native's other readings.
    #[serde(default)]
    pub bridge: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Karaka {
    pub graha: lagn_core::Graha,
    /// What it signifies here, completing "the natural significator of ...".
    pub role: String,
    /// Only for natives of this sex (e.g. Jupiter as a woman's husband-karaka).
    #[serde(default)]
    pub sex: Option<crate::model::Sex>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VargaFocus {
    pub varga: lagn_core::Varga,
    /// The rasi-chart house whose lord is examined in the varga.
    pub house: u8,
}

/// The matters people consult about, each mapped to the reading that governs
/// it (`questions.json`).
///
/// A fixed catalogue, deliberately. A reader types and the app suggests from
/// this list; the answer is derived from the question they chose, never from
/// the text they typed. That keeps the reading deterministic and keeps every
/// birth detail on the device, which free-text interpretation could not.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Questions {
    pub questions: Vec<Question>,
    pub source: crate::model::Source,
    pub review: Review,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub id: String,
    /// As it is shown, ending in a question mark.
    pub question: String,
    /// The topic whose houses and karakas govern the matter.
    pub topic: String,
    /// Match terms only. Never shown to a reader.
    pub keywords: Vec<String>,
}

/// What each porutham measures, and what a match or a mismatch is read to
/// mean (`porutham.explain.json`). Content, so it goes through the review gate
/// like every other sentence a reader sees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoruthamExplainer {
    pub poruthams: Vec<PoruthamExplain>,
    pub source: crate::model::Source,
    pub review: Review,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoruthamExplain {
    pub kind: crate::porutham::PoruthamKind,
    pub name: String,
    /// Completes "it tests ...".
    pub measures: String,
    /// Completes "Matching here means ...".
    pub when_matching: String,
    /// Completes "Not matching here means ...".
    pub when_not: String,
    /// The area of married life this one speaks to, for grouping.
    pub touches: String,
}

impl PoruthamExplainer {
    pub fn get(&self, k: crate::porutham::PoruthamKind) -> Option<&PoruthamExplain> {
        self.poruthams.iter().find(|p| p.kind == k)
    }
}

/// What each bhava signifies, for explaining periods.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bhavas {
    pub houses: Vec<BhavaMeaning>,
    pub source: crate::model::Source,
    pub review: Review,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BhavaMeaning {
    pub house: u8,
    pub name: String,
    pub significations: Vec<String>,
}

/// `corpus/karma.json`: what the tradition reads from Ketu's house and sign
/// for the life carried forward (phase 9 design section 3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Karma {
    pub ketu_house: Vec<KetuHouse>,
    pub ketu_sign: Vec<KetuSign>,
    pub dispositor: Dispositor,
    pub source: crate::model::Source,
    pub review: Review,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KetuHouse {
    pub house: u8,
    /// The station that life took, completing "a life ...".
    pub station: String,
    /// The kinds of life this placement is traditionally associated with, as
    /// possibilities rather than a record. A chart cannot encode an occupation
    /// any more than it can encode a name, so these are always several, always
    /// framed as what the tradition associates, and never asserted of the
    /// native. Completes "lives of ...".
    #[serde(default)]
    pub callings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KetuSign {
    /// 1 = Mesha .. 12 = Meena.
    pub sign: u8,
    pub name: String,
    pub temperament: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dispositor {
    pub strong: String,
    pub weak: String,
    /// What each graha, as the dispositor, lends to the callings above: the
    /// craft, where the house gives the sphere. Keyed by the graha's own name.
    #[serde(default)]
    pub craft: Vec<DispositorCraft>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispositorCraft {
    /// "sun", "moon", ... as `Graha` serialises.
    pub graha: lagn_core::Graha,
    /// Completes "work of the kind that ...". A phrase, not a sentence.
    pub craft: String,
}

impl Karma {
    /// The callings for a Ketu house, if the catalogue carries any.
    pub fn callings(&self, house: u8) -> &[String] {
        self.ketu_house
            .iter()
            .find(|x| x.house == house)
            .map(|x| x.callings.as_slice())
            .unwrap_or(&[])
    }

    /// What a dispositor lends to those callings.
    pub fn craft(&self, g: lagn_core::Graha) -> Option<&str> {
        self.dispositor.craft.iter().find(|c| c.graha == g).map(|c| c.craft.as_str())
    }

    pub fn station(&self, house: u8) -> Option<&str> {
        self.ketu_house.iter().find(|x| x.house == house).map(|x| x.station.as_str())
    }
    pub fn temperament(&self, sign_index: u8) -> Option<&KetuSign> {
        self.ketu_sign.iter().find(|x| x.sign == sign_index)
    }
}
