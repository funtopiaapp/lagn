//! Reviewed content other than rules: the topic catalogue, bhava
//! significations and the pariharam catalogue. Specification:
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
    /// Default age range for timing windows.
    pub ages: [f64; 2],
    /// What the write-up explains (DESIGN section 4a).
    #[serde(default)]
    pub focus: Focus,
    pub review: Review,
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

/// A traditional remedy, shown when a rule or transit carrying one of its
/// trigger tags is in effect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pariharam {
    pub id: String,
    pub title: String,
    /// Tags on effective rules (e.g. `graha:saturn`, `dosha:kuja`) or
    /// transit tags (e.g. `transit:sade_sati`) that bring this pariharam up.
    pub triggers: Vec<String>,
    pub deity: Option<String>,
    pub day: Option<String>,
    pub practices: Vec<String>,
    pub places: Vec<String>,
    pub charity: Option<String>,
    pub source: crate::model::Source,
    pub review: Review,
}

/// Transit tags the engine emits (see `transit_tags`).
pub const TRANSIT_TAGS: [&str; 4] = ["transit:sade_sati", "transit:ashtama_shani", "transit:ardhashtama_shani", "transit:kantaka_shani"];

/// The pariharam tag for a challenging transit, if any.
pub fn transit_tag(kind: lagn_core::TransitKind) -> Option<&'static str> {
    use lagn_core::TransitKind::*;
    match kind {
        SadeSatiRising | SadeSatiPeak | SadeSatiSetting => Some("transit:sade_sati"),
        Ashtama => Some("transit:ashtama_shani"),
        Ardhashtama => Some("transit:ardhashtama_shani"),
        Kantaka => Some("transit:kantaka_shani"),
        GuruFavourable | NodeFavourable => None,
    }
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
}

impl Karma {
    pub fn station(&self, house: u8) -> Option<&str> {
        self.ketu_house.iter().find(|x| x.house == house).map(|x| x.station.as_str())
    }
    pub fn temperament(&self, sign_index: u8) -> Option<&KetuSign> {
        self.ketu_sign.iter().find(|x| x.sign == sign_index)
    }
}
